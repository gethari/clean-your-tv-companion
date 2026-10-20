//! Bounded local image import. Paths enter only through Rust's native picker or
//! the native window drag/drop event bridge; no command accepts a frontend path.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use image::{DynamicImage, GenericImageView, ImageDecoder, ImageFormat, ImageReader, Limits};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

pub(crate) const MAX_FILE_BYTES: u64 = 20 * 1024 * 1024;
pub(crate) const MAX_IMAGE_DIMENSION: u32 = 16_384;
pub(crate) const MAX_IMAGE_PIXELS: u64 = 33_177_600;
pub(crate) const MAX_COLLECTION_ITEMS: usize = 200;
pub(crate) const MAX_COLLECTION_BYTES: u64 = 256 * 1024 * 1024;
const MAX_THUMBNAIL_BYTES: usize = 512 * 1024;
const MAX_DIRECTORY_ENTRIES: usize = 2_000;
const MAX_REJECTIONS: usize = 200;
const THUMBNAIL_WIDTH: u32 = 320;
const THUMBNAIL_HEIGHT: u32 = 180;
const TRANSFORM_WIDTH: u32 = 1_920;
const TRANSFORM_HEIGHT: u32 = 1_080;
static MEDIA_WORK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportedImage {
    pub id: String,
    pub sha256: String,
    pub mime: String,
    pub bytes: u64,
    pub width: u32,
    pub height: u32,
    pub thumbnail_data_url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportIssue {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct ImportBatch {
    pub accepted: Vec<ImportedImage>,
    pub rejected: Vec<ImportIssue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransformedImage {
    pub sha256: String,
    pub mime: String,
    pub bytes: u64,
    pub width: u32,
    pub height: u32,
    pub thumbnail_data_url: String,
}

#[derive(Debug, Clone, Copy)]
struct Format {
    image: ImageFormat,
    mime: &'static str,
}

fn supported_format(format: ImageFormat) -> Option<Format> {
    let mime = match format {
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Png => "image/png",
        ImageFormat::WebP => "image/webp",
        _ => return None,
    };
    Some(Format { image: format, mime })
}

fn hash_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn random_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| "Could not create an image handle.")?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn decode_image(bytes: &[u8]) -> Result<(DynamicImage, Format), &'static str> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("file-too-large");
    }
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| "corrupt-image")?;
    let detected = reader.format().ok_or("corrupt-image")?;
    let format = supported_format(detected).ok_or("unsupported-format")?;
    let mut decoder = reader.into_decoder().map_err(|_| "corrupt-image")?;
    let (width, height) = decoder.dimensions();
    if width == 0 || height == 0
        || width > MAX_IMAGE_DIMENSION
        || height > MAX_IMAGE_DIMENSION
        || (width as u64).saturating_mul(height as u64) > MAX_IMAGE_PIXELS
    {
        return Err("image-dimensions-exceeded");
    }

    // Dimensions are checked before decode. Width/height decoder limits add a
    // second bound, and max_alloc limits decoders that support that best-effort cap.
    let orientation = decoder.orientation().map_err(|_| "corrupt-image")?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(160 * 1024 * 1024);
    decoder.set_limits(limits).map_err(|_| "image-dimensions-exceeded")?;
    let mut decoded = DynamicImage::from_decoder(decoder).map_err(|_| "corrupt-image")?;
    decoded.apply_orientation(orientation);
    Ok((decoded, format))
}

fn encode(image: &DynamicImage, format: ImageFormat) -> Result<Vec<u8>, &'static str> {
    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, format).map_err(|_| "image-encode-failed")?;
    let bytes = cursor.into_inner();
    if bytes.is_empty() || bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("normalized-file-too-large");
    }
    Ok(bytes)
}

fn center_crop_dimensions(width: u32, height: u32) -> (u32, u32) {
    if (width as u64) * 9 > (height as u64) * 16 {
        ((height * 16 / 9).max(1), height)
    } else {
        (width, (width * 9 / 16).max(1))
    }
}

fn make_thumbnail(image: &DynamicImage) -> Result<String, &'static str> {
    let thumbnail = crop_to_frame(image, 0.5, 0.5, THUMBNAIL_WIDTH, THUMBNAIL_HEIGHT);
    let png = encode_thumbnail(&thumbnail)?;
    if png.len() > MAX_THUMBNAIL_BYTES {
        return Err("thumbnail-too-large");
    }
    Ok(format!("data:image/png;base64,{}", BASE64.encode(png)))
}

fn crop_to_frame(image: &DynamicImage, crop_x: f64, crop_y: f64, width: u32, height: u32) -> DynamicImage {
    let (source_width, source_height) = image.dimensions();
    let (crop_width, crop_height) = center_crop_dimensions(source_width, source_height);
    let left = ((source_width - crop_width) as f64 * crop_x).round() as u32;
    let top = ((source_height - crop_height) as f64 * crop_y).round() as u32;
    image.crop_imm(left, top, crop_width, crop_height)
        .resize_exact(width, height, image::imageops::FilterType::Lanczos3)
}

fn encode_thumbnail(image: &DynamicImage) -> Result<Vec<u8>, &'static str> {
    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::Png).map_err(|_| "thumbnail-failed")?;
    Ok(cursor.into_inner())
}

fn write_cache(root: &Path, hash: &str, bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_FILE_BYTES || hash_bytes(bytes) != hash {
        return Err("cache-unavailable");
    }
    fs::create_dir_all(root).map_err(|_| "cache-unavailable")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(root, fs::Permissions::from_mode(0o700)).map_err(|_| "cache-unavailable")?;
    }

    let final_path = root.join(hash);
    match fs::symlink_metadata(&final_path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            return match cached_bytes(root, hash) {
                Ok(existing) if existing == bytes => Ok(()),
                _ => Err("cache-unavailable"),
            };
        }
        Ok(_) => return Err("cache-unavailable"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
        Err(_) => return Err("cache-unavailable"),
    }

    let temp_path = root.join(format!(".{hash}.{}.tmp", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = match options.open(&temp_path) {
        Ok(file) => file,
        Err(_) => return Err("cache-unavailable"),
    };
    let write_result = file.write_all(bytes).and_then(|_| file.sync_all());
    drop(file);
    if write_result.is_err() {
        let _ = fs::remove_file(&temp_path);
        return Err("cache-unavailable");
    }
    // hard_link is a same-filesystem atomic create-if-absent operation. It
    // avoids replacing an existing cache object if another writer races us.
    match fs::hard_link(&temp_path, &final_path) {
        Ok(()) => { let _ = fs::remove_file(&temp_path); Ok(()) }
        Err(_) if fs::symlink_metadata(&final_path).is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
            && cached_bytes(root, hash).is_ok_and(|existing| existing == bytes) => {
            let _ = fs::remove_file(&temp_path);
            Ok(())
        }
        Err(_) => {
            let _ = fs::remove_file(&temp_path);
            Err("cache-unavailable")
        }
    }
}

fn issue(batch: &mut ImportBatch, path: &Path, code: &str) {
    if batch.rejected.len() >= MAX_REJECTIONS { return; }
    batch.rejected.push(ImportIssue {
        name: path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_else(|| "Selected item".into()),
        code: code.into(),
    });
}

fn collect_folder(folder: &Path, output: &mut Vec<PathBuf>, batch: &mut ImportBatch, examined: &mut usize) {
    let canonical_root = match fs::canonicalize(folder) {
        Ok(path) => path,
        Err(_) => { issue(batch, folder, "unreadable-folder"); return; }
    };
    let mut stack = vec![canonical_root.clone()];
    while let Some(directory) = stack.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(_) => { issue(batch, &directory, "unreadable-folder"); continue; }
        };
        for entry in entries {
            if *examined >= MAX_DIRECTORY_ENTRIES {
                issue(batch, &directory, "directory-entry-limit");
                return;
            }
            *examined += 1;
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => { issue(batch, &directory, "unreadable-item"); continue; }
            };
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(_) => { issue(batch, &path, "unreadable-item"); continue; }
            };
            if metadata.file_type().is_symlink() {
                issue(batch, &path, "symlink-not-allowed");
                continue;
            }
            if metadata.is_dir() {
                match fs::canonicalize(&path) {
                    Ok(canonical) if canonical.starts_with(&canonical_root) => stack.push(canonical),
                    _ => issue(batch, &path, "outside-selected-folder"),
                }
            } else if metadata.is_file() {
                match fs::canonicalize(&path) {
                    Ok(canonical) if canonical.starts_with(&canonical_root) => output.push(canonical),
                    _ => issue(batch, &path, "outside-selected-folder"),
                }
            }
        }
    }
}

fn collect_candidates(paths: &[PathBuf], batch: &mut ImportBatch) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let mut examined = 0usize;
    for path in paths {
        if examined >= MAX_DIRECTORY_ENTRIES {
            issue(batch, path, "directory-entry-limit");
            break;
        }
        examined += 1;
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => { issue(batch, path, "unreadable-item"); continue; }
        };
        if metadata.file_type().is_symlink() {
            issue(batch, path, "symlink-not-allowed");
        } else if metadata.is_dir() {
            collect_folder(path, &mut candidates, batch, &mut examined);
        } else if metadata.is_file() {
            match fs::canonicalize(path) {
                Ok(canonical) => candidates.push(canonical),
                Err(_) => issue(batch, path, "unreadable-item"),
            }
        } else {
            issue(batch, path, "not-a-regular-file");
        }
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

/// Import native picker/drop paths into `<app_data_dir>/media/<sha256>`.
/// This remains crate-private so frontend IPC cannot submit arbitrary paths.
pub(crate) fn ingest_native_paths(app_data_dir: &Path, paths: &[PathBuf]) -> ImportBatch {
    let _guard = match MEDIA_WORK.lock() {
        Ok(guard) => guard,
        Err(_) => {
            let mut batch = ImportBatch::default();
            issue(&mut batch, Path::new("Selected item"), "image-processor-unavailable");
            return batch;
        }
    };
    ingest_native_paths_locked(app_data_dir, paths)
}

fn ingest_native_paths_locked(app_data_dir: &Path, paths: &[PathBuf]) -> ImportBatch {
    let media_root = app_data_dir.join("media");
    let mut batch = ImportBatch::default();
    let candidates = collect_candidates(paths, &mut batch);
    let mut seen_hashes = HashSet::new();
    let mut total_bytes = 0u64;

    for path in candidates {
        if batch.accepted.len() >= MAX_COLLECTION_ITEMS {
            issue(&mut batch, &path, "collection-item-limit");
            continue;
        }
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(_) => { issue(&mut batch, &path, "unreadable-item"); continue; }
        };
        let file_len = match file.metadata() {
            Ok(metadata) if metadata.is_file() => metadata.len(),
            _ => { issue(&mut batch, &path, "not-a-regular-file"); continue; }
        };
        if file_len == 0 || file_len > MAX_FILE_BYTES {
            issue(&mut batch, &path, "file-too-large");
            continue;
        }
        let mut raw = Vec::with_capacity(file_len as usize);
        if file.take(MAX_FILE_BYTES + 1).read_to_end(&mut raw).is_err() || raw.len() as u64 != file_len {
            issue(&mut batch, &path, "unreadable-item");
            continue;
        }
        let (decoded, format) = match decode_image(&raw) {
            Ok(result) => result,
            Err(code) => { issue(&mut batch, &path, code); continue; }
        };
        let (width, height) = decoded.dimensions();
        let thumbnail_data_url = match make_thumbnail(&decoded) {
            Ok(thumbnail) => thumbnail,
            Err(code) => { issue(&mut batch, &path, code); continue; }
        };
        let normalized = match encode(&decoded, format.image) {
            Ok(bytes) => bytes,
            Err(code) => { issue(&mut batch, &path, code); continue; }
        };
        let hash = hash_bytes(&normalized);
        if !seen_hashes.insert(hash.clone()) { continue; }
        let bytes = normalized.len() as u64;
        if total_bytes.saturating_add(bytes) > MAX_COLLECTION_BYTES {
            issue(&mut batch, &path, "collection-byte-limit");
            continue;
        }
        if write_cache(&media_root, &hash, &normalized).is_err() {
            issue(&mut batch, &path, "cache-unavailable");
            continue;
        }
        let id = match random_id() {
            Ok(id) => id,
            Err(_) => { issue(&mut batch, &path, "handle-unavailable"); continue; }
        };
        total_bytes += bytes;
        batch.accepted.push(ImportedImage {
            id,
            sha256: hash,
            mime: format.mime.into(),
            bytes,
            width,
            height,
            thumbnail_data_url,
        });
    }
    batch
}

fn cached_bytes(media_root: &Path, hash: &str) -> Result<Vec<u8>, String> {
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err("Invalid imported image handle.".into());
    }
    let path = media_root.join(hash);
    let metadata = fs::symlink_metadata(&path).map_err(|_| "The imported image is missing. Import it again.")?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() == 0 || metadata.len() > MAX_FILE_BYTES {
        return Err("The imported image is unavailable. Import it again.".into());
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(path).map_err(|_| "Could not read the imported image.")?
        .take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes).map_err(|_| "Could not read the imported image.")?;
    if bytes.len() as u64 != metadata.len() || hash_bytes(&bytes) != hash {
        return Err("The imported image failed verification. Import it again.".into());
    }
    Ok(bytes)
}

/// Crop a cached normalized image to the TV's 16:9 frame and return a
/// content-addressed manifest. Crop anchors are normalized to the range 0..1.
#[tauri::command]
pub(crate) async fn transform_image(
    app: tauri::AppHandle,
    sha256: String,
    crop_x: f64,
    crop_y: f64,
) -> Result<TransformedImage, String> {
    if !crop_x.is_finite() || !crop_y.is_finite() || !(0.0..=1.0).contains(&crop_x) || !(0.0..=1.0).contains(&crop_y) {
        return Err("Choose a valid image crop.".into());
    }
    let app_data_dir = app.path().app_data_dir().map_err(|_| "Could not locate imported media.")?;
    let root = app_data_dir.join("media");
    let expected_hash = sha256.clone();
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<TransformedImage, String> {
        let _guard = MEDIA_WORK.lock().map_err(|_| "Image processor is unavailable.")?;
        let original = cached_bytes(&root, &expected_hash)?;
        let (image, format) = decode_image(&original).map_err(|_| "The imported image could not be decoded.")?;
        let cropped = crop_to_frame(&image, crop_x, crop_y, TRANSFORM_WIDTH, TRANSFORM_HEIGHT);
        let normalized = encode(&cropped, format.image).map_err(|_| "The transformed image exceeds the storage limit.")?;
        let hash = hash_bytes(&normalized);
        write_cache(&root, &hash, &normalized).map_err(|_| "Could not save the transformed image.")?;
        let thumbnail_data_url = make_thumbnail(&cropped).map_err(|_| "Could not create the image preview.")?;
        Ok(TransformedImage { sha256: hash, mime: format.mime.into(), bytes: normalized.len() as u64, width: TRANSFORM_WIDTH, height: TRANSFORM_HEIGHT, thumbnail_data_url })
    }).await.map_err(|_| "Image processing was interrupted.")??;
    Ok(result)
}

fn selected_paths(app: &tauri::AppHandle, folder: bool) -> Result<Option<Vec<PathBuf>>, String> {
    let selected = if folder {
        app.dialog().file().blocking_pick_folder().map(|path| vec![path])
    } else {
        app.dialog().file().add_filter("Images", &["jpg", "jpeg", "png", "webp"]).blocking_pick_files()
    };
    selected.map(|paths| paths.into_iter().map(|path| {
        path.into_path().map_err(|_| "The selected item is not a readable local path.".to_owned())
    }).collect()).transpose()
}

async fn run_picker(app: tauri::AppHandle, folder: bool) -> Result<ImportBatch, String> {
    let picker_app = app.clone();
    let paths = tauri::async_runtime::spawn_blocking(move || selected_paths(&picker_app, folder))
        .await.map_err(|_| "The image picker was interrupted.")??;
    let Some(paths) = paths else { return Ok(ImportBatch::default()); };
    if paths.is_empty() { return Ok(ImportBatch::default()); }
    let app_data_dir = app.path().app_data_dir().map_err(|_| "Could not locate imported media.")?;
    tauri::async_runtime::spawn_blocking(move || Ok(ingest_native_paths(&app_data_dir, &paths)))
        .await.map_err(|_| "Image import was interrupted.")?
}

#[tauri::command]
pub(crate) async fn pick_photo_files(app: tauri::AppHandle) -> Result<ImportBatch, String> {
    run_picker(app, false).await
}

#[tauri::command]
pub(crate) async fn pick_photo_folder(app: tauri::AppHandle) -> Result<ImportBatch, String> {
    run_picker(app, true).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempTree(PathBuf);

    impl TempTree {
        fn new() -> Self {
            let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("cytv-media-{}-{unique}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> &Path { &self.0 }
    }

    impl Drop for TempTree {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    fn png_bytes(width: u32, height: u32) -> Vec<u8> {
        let image = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(width, height, image::Rgba([12, 34, 56, 255])));
        encode_thumbnail(&image).unwrap()
    }

    #[test]
    fn imports_by_content_and_detects_format_instead_of_trusting_extension() {
        let tree = TempTree::new();
        let input = tree.path().join("misleading.jpg");
        let bytes = png_bytes(16, 9);
        fs::write(&input, &bytes).unwrap();
        let output = tree.path().join("app-data");

        let batch = ingest_native_paths(&output, &[input]);
        assert_eq!(batch.accepted.len(), 1);
        assert!(batch.rejected.is_empty());
        let image = &batch.accepted[0];
        assert_eq!(image.mime, "image/png");
        assert_eq!((image.width, image.height), (16, 9));
        let cached = fs::read(output.join("media").join(&image.sha256)).unwrap();
        assert_eq!(hash_bytes(&cached), image.sha256);
        let (cached_image, cached_format) = decode_image(&cached).unwrap();
        assert_eq!(cached_format.mime, "image/png");
        assert_eq!(cached_image.dimensions(), (16, 9));
        assert!(image.thumbnail_data_url.starts_with("data:image/png;base64,"));
        assert_eq!(image.thumbnail_data_url.len() < MAX_THUMBNAIL_BYTES * 2, true);
    }

    #[test]
    fn corrupt_and_unsupported_files_are_reported_without_blocking_valid_files() {
        let tree = TempTree::new();
        let corrupt = tree.path().join("corrupt.png");
        let unsupported = tree.path().join("unsupported.gif");
        let valid = tree.path().join("valid.webp");
        fs::write(&corrupt, b"not an image").unwrap();
        fs::write(&unsupported, b"GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff\x21\xf9\x04\x00\x00\x00\x00\x00\x2c\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02\x44\x01\x00\x3b").unwrap();
        // Keep WebP bytes but use a misleading extension to exercise content sniffing.
        let webp_bytes = {
            let mut cursor = Cursor::new(Vec::new());
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(8, 8, image::Rgb([1, 2, 3])))
                .write_to(&mut cursor, ImageFormat::WebP).unwrap();
            cursor.into_inner()
        };
        fs::write(&valid, webp_bytes).unwrap();

        let batch = ingest_native_paths(&tree.path().join("app"), &[corrupt, unsupported, valid]);
        assert_eq!(batch.accepted.len(), 1);
        assert_eq!(batch.accepted[0].mime, "image/webp");
        assert!(batch.rejected.iter().any(|item| item.code == "corrupt-image"));
        assert!(batch.rejected.iter().any(|item| item.code == "unsupported-format"));
    }

    #[test]
    fn rejects_over_dimension_and_per_file_limits_before_decoding() {
        let over_file = vec![0u8; MAX_FILE_BYTES as usize + 1];
        assert_eq!(decode_image(&over_file).unwrap_err(), "file-too-large");

        let tree = TempTree::new();
        let path = tree.path().join("oversize.bin");
        File::create(&path).unwrap().set_len(MAX_FILE_BYTES + 1).unwrap();
        let batch = ingest_native_paths(&tree.path().join("app"), &[path]);
        assert_eq!(batch.accepted.len(), 0);
        assert_eq!(batch.rejected[0].code, "file-too-large");
        assert_eq!(decode_image(&png_bytes(16_385, 1)).unwrap_err(), "image-dimensions-exceeded");
    }

    #[cfg(unix)]
    #[test]
    fn selected_folder_does_not_follow_symlinks_outside_its_root() {
        use std::os::unix::fs::symlink;
        let tree = TempTree::new();
        let selected = tree.path().join("selected");
        let outside = tree.path().join("outside");
        fs::create_dir_all(&selected).unwrap();
        let external_image = outside.with_extension("png");
        fs::write(&external_image, png_bytes(8, 8)).unwrap();
        symlink(&external_image, selected.join("linked.png")).unwrap();

        let batch = ingest_native_paths(&tree.path().join("app"), &[selected]);
        assert!(batch.accepted.is_empty());
        assert!(batch.rejected.iter().any(|item| item.code == "symlink-not-allowed"));
    }

    #[test]
    fn transform_validates_crop_and_returns_content_addressed_16_by_9_output() {
        let tree = TempTree::new();
        let output = tree.path().join("app");
        let input = tree.path().join("wide.png");
        fs::write(&input, png_bytes(32, 16)).unwrap();
        let batch = ingest_native_paths(&output, &[input]);
        let imported = &batch.accepted[0];
        let bytes = cached_bytes(&output.join("media"), &imported.sha256).unwrap();
        let (decoded, format) = decode_image(&bytes).unwrap();
        assert_eq!(format.mime, "image/png");
        let (width, height) = decoded.dimensions();
        let (crop_width, crop_height) = center_crop_dimensions(width, height);
        assert_eq!((crop_width, crop_height), (28, 16));
        let transformed = crop_to_frame(&decoded, 1.0, 0.5, TRANSFORM_WIDTH, TRANSFORM_HEIGHT);
        assert_eq!(transformed.dimensions(), (1920, 1080));
        assert_eq!(TRANSFORM_WIDTH * 9, TRANSFORM_HEIGHT * 16);
        assert_eq!(MAX_COLLECTION_ITEMS, 200);
        assert_eq!(MAX_COLLECTION_BYTES, 256 * 1024 * 1024);
    }

    #[test]
    fn collection_item_limit_keeps_accepted_images_and_reports_excess() {
        let tree = TempTree::new();
        let inputs: Vec<PathBuf> = (0..=MAX_COLLECTION_ITEMS)
            .map(|index| {
                let path = tree.path().join(format!("{index:03}.png"));
                let color = image::Rgba([index as u8, (index >> 8) as u8, 0, 255]);
                let fixture = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(4, 4, color));
                fs::write(&path, encode_thumbnail(&fixture).unwrap()).unwrap();
                path
            })
            .collect();
        let batch = ingest_native_paths(&tree.path().join("app"), &inputs);
        assert_eq!(batch.accepted.len(), MAX_COLLECTION_ITEMS);
        assert!(batch.rejected.iter().any(|item| item.code == "collection-item-limit"));
    }
}
