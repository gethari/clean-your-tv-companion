//! Pure, deny-by-default selection policy for optional package cleanup.
//!
//! This module only creates plans. It has no transport, executor, or package
//! mutation path. The built-in catalog remains empty until physical evidence
//! is reviewed and an exact firmware profile is approved.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TvIdentity {
    pub manufacturer: String,
    pub model: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedPackage {
    pub package_name: String,
    pub enabled: Option<bool>,
    pub is_system: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceObservation {
    pub identity: TvIdentity,
    pub packages: Vec<ObservedPackage>,
    /// Must be a successful, complete HOME observation.
    pub home_packages: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileStatus { Draft, EvidencePending, Approved }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationEvidence {
    pub device_fingerprint: String,
    pub firmware_version: String,
    pub test_run_id: String,
    pub checked_functions: Vec<ProtectedFeature>,
    pub reviewer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtectedFeature {
    StockHome, GooglePlay, Casting, Voice, Input, Accessibility, Network, Updates, Setup, Hdmi, Remote,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedPackageGroup {
    pub feature: ProtectedFeature,
    pub package_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupCandidate {
    pub package_name: String,
    pub effect: String,
    pub protected_dependencies: Vec<ProtectedFeature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareProfile {
    pub schema_version: u32,
    pub profile_id: String,
    pub manufacturer: String,
    pub model: String,
    pub fingerprint: String,
    pub firmware_version: String,
    pub provenance: String,
    pub status: ProfileStatus,
    pub evidence: Vec<VerificationEvidence>,
    pub candidates: Vec<CleanupCandidate>,
    pub protected_groups: Vec<ProtectedPackageGroup>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    NoApprovedProfile, InvalidProfile, MissingHomeObservation, EmptyHomeObservation,
    MissingPackageObservation, PackageStateUnknown, PackageNotAllowlisted, ProtectedPackage,
    InvalidPackageName, ConflictingPackageObservation, DuplicateSelection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedChange {
    pub package_name: String,
    pub before_enabled: bool,
    pub after_enabled: bool,
    pub effect: String,
}

/// The shipped catalog intentionally has no approved profiles. Test fixtures
/// must be injected into select_plan and are never shipped as approved data.
pub fn built_in_profiles() -> &'static [FirmwareProfile] { &[] }

fn nonempty(value: &str) -> bool { !value.trim().is_empty() }

fn valid_package_name(value: &str) -> bool {
    let mut parts = value.split('.');
    let valid_part = |part: &str| !part.is_empty()
        && part.as_bytes()[0].is_ascii_alphabetic()
        && part.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    parts.clone().count() >= 2 && parts.all(valid_part)
}

fn builtin_protected_ids() -> &'static [&'static str] {
    &[
        "com.google.android.tungsten.setupwraith", "com.google.android.apps.tv.launcherx",
        "com.google.android.tvlauncher", "com.google.android.leanbacklauncher",
        "com.google.android.gms", "com.android.vending", "com.google.android.apps.mediashell",
        "com.google.android.katniss", "com.google.android.tv.remote.service",
        "com.google.android.tv.dfuservice", "com.google.android.marvin.talkback",
        "com.google.android.inputmethod.latin", "com.tcl.tv", "com.tcl.tvinput",
        "com.tcl.providers.config", "com.tcl.initsetup", "com.tcl.factory.view",
    ]
}

fn required_features() -> [ProtectedFeature; 11] {
    [ProtectedFeature::StockHome, ProtectedFeature::GooglePlay, ProtectedFeature::Casting,
     ProtectedFeature::Voice, ProtectedFeature::Input, ProtectedFeature::Accessibility,
     ProtectedFeature::Network, ProtectedFeature::Updates, ProtectedFeature::Setup,
     ProtectedFeature::Hdmi, ProtectedFeature::Remote]
}

fn profile_is_valid(profile: &FirmwareProfile) -> bool {
    profile.schema_version == 1
        && nonempty(&profile.profile_id) && nonempty(&profile.manufacturer)
        && nonempty(&profile.model) && nonempty(&profile.fingerprint)
        && nonempty(&profile.firmware_version) && nonempty(&profile.provenance)
        && profile.status == ProfileStatus::Approved
        && !profile.candidates.is_empty() && !profile.evidence.is_empty()
        && required_features().iter().all(|feature| profile.protected_groups.iter().any(|group| {
            group.feature == *feature && !group.package_names.is_empty()
        }))
        && profile.evidence.iter().all(|item| {
            nonempty(&item.device_fingerprint)
                && item.device_fingerprint == profile.fingerprint
                && item.firmware_version == profile.firmware_version
                && nonempty(&item.test_run_id) && nonempty(&item.reviewer)
                && required_features().iter().all(|feature| item.checked_functions.contains(feature))
        })
        && profile.candidates.iter().all(|candidate| {
            valid_package_name(&candidate.package_name) && nonempty(&candidate.effect)
                && candidate.protected_dependencies.is_empty()
        })
        && profile.protected_groups.iter().all(|group| {
            !group.package_names.is_empty() && group.package_names.iter().all(|name| valid_package_name(name))
        })
}

/// Validate exact identity and approved profile evidence, then create
/// disable-only changes for explicit selections. Previously disabled items
/// produce no operation. No choice is inferred from the profile.
pub fn select_plan(
    observation: &DeviceObservation,
    catalog: &[FirmwareProfile],
    selected_packages: &[String],
) -> Result<Vec<PlannedChange>, PolicyError> {
    let home_packages = observation.home_packages.as_ref().ok_or(PolicyError::MissingHomeObservation)?;
    if home_packages.is_empty() { return Err(PolicyError::EmptyHomeObservation); }
    if observation.packages.iter().any(|item| !valid_package_name(&item.package_name))
        || home_packages.iter().any(|item| !valid_package_name(item))
        || selected_packages.iter().any(|item| !valid_package_name(item)) {
        return Err(PolicyError::InvalidPackageName);
    }
    for (index, package) in observation.packages.iter().enumerate() {
        if observation.packages[index + 1..].iter().any(|other| {
            other.package_name == package.package_name
                && (other.enabled != package.enabled || other.is_system != package.is_system)
        }) { return Err(PolicyError::ConflictingPackageObservation); }
    }
    let mut selected = std::collections::HashSet::new();
    if selected_packages.iter().any(|package| !selected.insert(package)) {
        return Err(PolicyError::DuplicateSelection);
    }
    let profile = catalog.iter().find(|profile| {
        profile.status == ProfileStatus::Approved
            && profile.manufacturer == observation.identity.manufacturer
            && profile.model == observation.identity.model
            && profile.fingerprint == observation.identity.fingerprint
    }).ok_or(PolicyError::NoApprovedProfile)?;
    if !profile_is_valid(profile) { return Err(PolicyError::InvalidProfile); }

    let mut changes = Vec::new();
    for package_name in selected_packages {
        let candidate = profile.candidates.iter().find(|item| item.package_name == *package_name)
            .ok_or(PolicyError::PackageNotAllowlisted)?;
        if home_packages.contains(package_name)
            || builtin_protected_ids().contains(&package_name.as_str())
            || profile.protected_groups.iter().any(|group| group.package_names.contains(package_name)) {
            return Err(PolicyError::ProtectedPackage);
        }
        let observed = observation.packages.iter().find(|item| item.package_name == *package_name)
            .ok_or(PolicyError::MissingPackageObservation)?;
        match observed.enabled {
            None => return Err(PolicyError::PackageStateUnknown),
            Some(false) => continue,
            Some(true) => changes.push(PlannedChange {
                package_name: package_name.clone(), before_enabled: true, after_enabled: false,
                effect: candidate.effect.clone(),
            }),
        }
    }
    Ok(changes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_test_profile(candidates: Vec<&str>, mut protected_groups: Vec<ProtectedPackageGroup>) -> FirmwareProfile {
        let fingerprint = "tcl/test/firmware:1.0/release-keys".to_string();
        for (index, feature) in required_features().iter().enumerate() {
            if !protected_groups.iter().any(|group| group.feature == *feature) {
                protected_groups.push(ProtectedPackageGroup {
                    feature: *feature,
                    package_names: vec![format!("com.fixture.protected.f{index}")],
                });
            }
        }
        FirmwareProfile {
            schema_version: 1, profile_id: "TEST-ONLY-NOT-SHIPPED".into(),
            manufacturer: "TCL".into(), model: "TEST-MODEL".into(), fingerprint: fingerprint.clone(),
            firmware_version: "1.0-test".into(), provenance: "Synthetic fixture; no physical evidence.".into(),
            status: ProfileStatus::Approved,
            evidence: vec![VerificationEvidence {
                device_fingerprint: fingerprint, firmware_version: "1.0-test".into(),
                test_run_id: "fixture-only".into(), checked_functions: required_features().to_vec(),
                reviewer: "unit-test-fixture".into(),
            }],
            candidates: candidates.into_iter().map(|package_name| CleanupCandidate {
                package_name: package_name.into(), effect: "Synthetic optional behavior".into(),
                protected_dependencies: vec![],
            }).collect(),
            protected_groups,
        }
    }

    fn observation() -> DeviceObservation {
        DeviceObservation {
            identity: TvIdentity { manufacturer: "TCL".into(), model: "TEST-MODEL".into(),
                fingerprint: "tcl/test/firmware:1.0/release-keys".into() },
            packages: vec![
                ObservedPackage { package_name: "com.test.enabled".into(), enabled: Some(true), is_system: false },
                ObservedPackage { package_name: "com.test.disabled".into(), enabled: Some(false), is_system: false },
                ObservedPackage { package_name: "com.test.castvoice".into(), enabled: Some(true), is_system: false },
            ],
            home_packages: Some(vec!["com.stock.home".into()]),
        }
    }

    #[test]
    fn shipped_catalog_is_empty_and_unknown_fingerprint_has_no_match() {
        assert!(built_in_profiles().is_empty());
        let profile = synthetic_test_profile(vec!["com.test.enabled"], vec![]);
        let mut observed = observation(); observed.identity.fingerprint.push_str("-other");
        assert_eq!(select_plan(&observed, &[profile], &["com.test.enabled".into()]), Err(PolicyError::NoApprovedProfile));
    }

    #[test]
    fn pre_disabled_and_unselected_items_produce_no_operations() {
        let profile = synthetic_test_profile(vec!["com.test.enabled", "com.test.disabled"], vec![]);
        assert!(select_plan(&observation(), &[profile], &["com.test.disabled".into()]).unwrap().is_empty());
        let profile = synthetic_test_profile(vec!["com.test.enabled"], vec![]);
        assert!(select_plan(&observation(), &[profile], &[]).unwrap().is_empty());
    }

    #[test]
    fn unknown_and_missing_home_state_are_denied() {
        let profile = synthetic_test_profile(vec!["com.test.enabled"], vec![]);
        let mut observed = observation(); observed.home_packages = None;
        assert_eq!(select_plan(&observed, &[profile.clone()], &[]), Err(PolicyError::MissingHomeObservation));
        observed.home_packages = Some(vec!["com.stock.home".into()]);
        observed.packages.push(ObservedPackage { package_name: "com.unknown.enabled".into(), enabled: Some(true), is_system: false });
        assert_eq!(select_plan(&observed, &[profile.clone()], &["com.unknown.enabled".into()]), Err(PolicyError::PackageNotAllowlisted));
        observed.packages.clear();
        assert_eq!(select_plan(&observed, &[profile], &["com.test.enabled".into()]), Err(PolicyError::MissingPackageObservation));
    }

    #[test]
    fn bundled_casting_and_voice_package_is_protected() {
        let profile = synthetic_test_profile(vec!["com.test.castvoice"], vec![
            ProtectedPackageGroup { feature: ProtectedFeature::Casting, package_names: vec!["com.test.castvoice".into()] },
            ProtectedPackageGroup { feature: ProtectedFeature::Voice, package_names: vec!["com.test.castvoice".into()] },
        ]);
        assert_eq!(select_plan(&observation(), &[profile], &["com.test.castvoice".into()]), Err(PolicyError::ProtectedPackage));
    }

    #[test]
    fn evidence_must_match_exact_profile_fingerprint_and_firmware() {
        let mut profile = synthetic_test_profile(vec!["com.test.enabled"], vec![]);
        profile.evidence[0].device_fingerprint.push_str("-other");
        assert_eq!(select_plan(&observation(), &[profile], &["com.test.enabled".into()]), Err(PolicyError::InvalidProfile));
    }

    #[test]
    fn builtin_protected_ids_remain_denied_even_if_profile_allowlists_them() {
        let package = "com.google.android.gms";
        let mut observed = observation();
        observed.packages.push(ObservedPackage { package_name: package.into(), enabled: Some(true), is_system: true });
        let profile = synthetic_test_profile(vec![package], vec![]);
        assert_eq!(select_plan(&observed, &[profile], &[package.into()]), Err(PolicyError::ProtectedPackage));
    }

    #[test]
    fn duplicate_selections_and_invalid_package_names_are_rejected() {
        let profile = synthetic_test_profile(vec!["com.test.enabled"], vec![]);
        assert_eq!(select_plan(&observation(), &[profile.clone()], &["com.test.enabled".into(), "com.test.enabled".into()]), Err(PolicyError::DuplicateSelection));
        assert_eq!(select_plan(&observation(), &[profile], &["not a package".into()]), Err(PolicyError::InvalidPackageName));
    }

    #[test]
    fn conflicting_duplicate_package_observations_are_rejected() {
        let profile = synthetic_test_profile(vec!["com.test.enabled"], vec![]);
        let mut observed = observation();
        observed.packages.push(ObservedPackage { package_name: "com.test.enabled".into(), enabled: Some(false), is_system: false });
        assert_eq!(select_plan(&observed, &[profile], &["com.test.enabled".into()]), Err(PolicyError::ConflictingPackageObservation));
    }

    #[test]
    fn allowlisted_package_with_unknown_enabled_state_is_rejected() {
        let profile = synthetic_test_profile(vec!["com.test.unknown"], vec![]);
        let mut observed = observation();
        observed.packages.push(ObservedPackage {
            package_name: "com.test.unknown".into(), enabled: None, is_system: false,
        });
        assert_eq!(select_plan(&observed, &[profile], &["com.test.unknown".into()]), Err(PolicyError::PackageStateUnknown));
    }

    #[test]
    fn explicitly_allowlisted_system_package_can_be_disabled_but_not_uninstalled() {
        let package = "com.vendor.optional";
        let mut observed = observation();
        observed.packages.push(ObservedPackage { package_name: package.into(), enabled: Some(true), is_system: true });
        let profile = synthetic_test_profile(vec![package], vec![]);
        let plan = select_plan(&observed, &[profile], &[package.into()]).unwrap();
        assert_eq!(plan.len(), 1);
        assert!(!plan[0].after_enabled);
    }

    #[test]
    fn empty_home_observation_is_not_accepted() {
        let profile = synthetic_test_profile(vec!["com.test.enabled"], vec![]);
        let mut observed = observation(); observed.home_packages = Some(vec![]);
        assert_eq!(select_plan(&observed, &[profile], &[]), Err(PolicyError::EmptyHomeObservation));
    }
}
