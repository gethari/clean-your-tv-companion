# Local photo collections

The Backgrounds tab imports JPEG, PNG, and WebP images from a file picker, a selected folder, or a desktop drop. The companion copies accepted files into its local application cache; it does not watch the source folder or require a cloud account. Folder imports skip links and items outside the selected folder.

Each source image is limited to 20 MiB, 16,384 pixels on either side, and 33,177,600 decoded pixels. A collection can contain up to 200 images and 256 MiB total. Unsupported formats, unreadable or corrupt images, and items beyond those limits appear with an item-specific explanation.

Images keep their source dimensions until a non-center crop is selected; adjusted crops are prepared as 1920 × 1080 TV backgrounds. The TV itself fills the 16:9 screen for the center crop. The collection editor supports sequential or shuffled order, a 1–1,440 minute rotation interval, and background dimming. The app previews newly imported photos locally. Images already stored on the TV are intentionally shown as “Preview unavailable” because the app does not download them for preview.

Importing and editing a draft does not change the TV. The user must review the changes and explicitly apply them. An interrupted apply remains a saved operation to reconcile; the UI only reports success when the journal verifies the TV state. Recovery baseline location is shown in the review. Demo mode uses only a sample image and fixture state and does not call native commands.

UI checks use mocked Tauri commands and do not connect to a TV. Physical-device behavior depends on the installed launcher supporting collection settings and the native media processor.
