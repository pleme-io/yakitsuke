use serde::{Deserialize, Serialize};

/// A partition to be flashed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionFlash {
    pub name: String,
    pub image_path: String,
    pub size: u64,
}

/// A complete flash plan for a device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlashPlan {
    pub device: String,
    pub partitions: Vec<PartitionFlash>,
    pub backup_first: bool,
}

/// Result of a single preflight check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreflightCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

const MIN_BATTERY_PCT: u8 = 50;
const MIN_STORAGE_FREE_MB: u64 = 2048;

/// Run preflight checks before flashing.
///
/// Validates battery level, bootloader unlock status, and available storage.
#[must_use]
pub fn preflight_checks(
    battery_pct: u8,
    bootloader_unlocked: bool,
    storage_free_mb: u64,
) -> Vec<PreflightCheck> {
    let mut checks = Vec::new();

    checks.push(PreflightCheck {
        name: "battery".to_string(),
        passed: battery_pct >= MIN_BATTERY_PCT,
        message: if battery_pct >= MIN_BATTERY_PCT {
            format!("Battery at {battery_pct}% (>= {MIN_BATTERY_PCT}%)")
        } else {
            format!("Battery at {battery_pct}% — must be >= {MIN_BATTERY_PCT}%")
        },
    });

    checks.push(PreflightCheck {
        name: "bootloader".to_string(),
        passed: bootloader_unlocked,
        message: if bootloader_unlocked {
            "Bootloader is unlocked".to_string()
        } else {
            "Bootloader is locked — unlock before flashing".to_string()
        },
    });

    checks.push(PreflightCheck {
        name: "storage".to_string(),
        passed: storage_free_mb >= MIN_STORAGE_FREE_MB,
        message: if storage_free_mb >= MIN_STORAGE_FREE_MB {
            format!("{storage_free_mb} MB free (>= {MIN_STORAGE_FREE_MB} MB)")
        } else {
            format!("{storage_free_mb} MB free — need >= {MIN_STORAGE_FREE_MB} MB")
        },
    });

    checks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_checks_pass() {
        let checks = preflight_checks(80, true, 4096);
        assert!(checks.iter().all(|c| c.passed));
        assert_eq!(checks.len(), 3);
    }

    #[test]
    fn low_battery_fails() {
        let checks = preflight_checks(20, true, 4096);
        let battery = checks.iter().find(|c| c.name == "battery").unwrap();
        assert!(!battery.passed);
        assert!(battery.message.contains("must be >= 50%"));
    }

    #[test]
    fn locked_bootloader_fails() {
        let checks = preflight_checks(80, false, 4096);
        let bl = checks.iter().find(|c| c.name == "bootloader").unwrap();
        assert!(!bl.passed);
        assert!(bl.message.contains("locked"));
    }

    #[test]
    fn low_storage_fails() {
        let checks = preflight_checks(80, true, 512);
        let storage = checks.iter().find(|c| c.name == "storage").unwrap();
        assert!(!storage.passed);
        assert!(storage.message.contains("need >= 2048 MB"));
    }

    #[test]
    fn flash_plan_serialization() {
        let plan = FlashPlan {
            device: "pixel8".to_string(),
            partitions: vec![
                PartitionFlash {
                    name: "boot".to_string(),
                    image_path: "boot.img".to_string(),
                    size: 67_108_864,
                },
                PartitionFlash {
                    name: "system".to_string(),
                    image_path: "system.img".to_string(),
                    size: 4_294_967_296,
                },
            ],
            backup_first: true,
        };

        let json = serde_json::to_string(&plan).unwrap();
        let deser: FlashPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, deser);
    }

    // --- Preflight boundary values ------------------------------------

    #[test]
    fn battery_passes_at_exact_minimum() {
        // Guard is `>= MIN_BATTERY_PCT`. If a future refactor flips
        // to `>`, flashing at exactly 50% would silently fail.
        let checks = preflight_checks(50, true, 4096);
        let battery = checks.iter().find(|c| c.name == "battery").unwrap();
        assert!(battery.passed);
        assert!(battery.message.contains("Battery at 50%"));
        assert!(battery.message.contains(">= 50%"));
    }

    #[test]
    fn battery_fails_one_below_minimum() {
        // 49 — the tight negative boundary. Together with the
        // passes-at-50 test this pins the `>= 50` comparison.
        let checks = preflight_checks(49, true, 4096);
        let battery = checks.iter().find(|c| c.name == "battery").unwrap();
        assert!(!battery.passed);
        assert!(battery.message.contains("Battery at 49%"));
    }

    #[test]
    fn battery_fails_at_zero() {
        let checks = preflight_checks(0, true, 4096);
        let battery = checks.iter().find(|c| c.name == "battery").unwrap();
        assert!(!battery.passed);
        assert!(battery.message.contains("Battery at 0%"));
    }

    #[test]
    fn storage_passes_at_exact_minimum() {
        // Same `>=` guard on storage. Flashing with exactly 2048 MB
        // free must pass.
        let checks = preflight_checks(80, true, 2048);
        let storage = checks.iter().find(|c| c.name == "storage").unwrap();
        assert!(storage.passed);
        assert!(storage.message.contains("2048 MB free"));
        assert!(storage.message.contains(">= 2048 MB"));
    }

    #[test]
    fn storage_fails_one_below_minimum() {
        let checks = preflight_checks(80, true, 2047);
        let storage = checks.iter().find(|c| c.name == "storage").unwrap();
        assert!(!storage.passed);
        assert!(storage.message.contains("2047 MB free"));
    }

    #[test]
    fn bootloader_pass_message_format() {
        // The pass-branch message had no assertion before. A regex
        // typo in either branch would silently ship.
        let checks = preflight_checks(80, true, 4096);
        let bl = checks.iter().find(|c| c.name == "bootloader").unwrap();
        assert!(bl.passed);
        assert_eq!(bl.message, "Bootloader is unlocked");
    }

    #[test]
    fn all_checks_can_fail_together() {
        // Integration: all three failure branches fire independently
        // and the returned Vec still has length 3 (never short-
        // circuits on the first failure).
        let checks = preflight_checks(10, false, 100);
        assert_eq!(checks.len(), 3);
        assert!(checks.iter().all(|c| !c.passed));
    }

    #[test]
    fn check_order_is_battery_bootloader_storage() {
        // UI renders in the order returned. If a future refactor
        // reorders the push()es, dashboards / log parsers would
        // break silently — pin the sequence.
        let checks = preflight_checks(80, true, 4096);
        assert_eq!(checks.len(), 3);
        assert_eq!(checks[0].name, "battery");
        assert_eq!(checks[1].name, "bootloader");
        assert_eq!(checks[2].name, "storage");
    }

    // --- Serde round-trips for the smaller types ----------------------

    #[test]
    fn partition_flash_round_trip() {
        // PartitionFlash is serialized independently (as part of
        // plan logs). Pin each field by name + value roundtrip so
        // a rename would fail loudly.
        let p = PartitionFlash {
            name: "vendor".into(),
            image_path: "/tmp/vendor.img".into(),
            size: u64::MAX,
        };
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"name\":\"vendor\""));
        assert!(json.contains("\"image_path\":\"/tmp/vendor.img\""));
        let back: PartitionFlash = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn preflight_check_round_trip() {
        // PreflightCheck is what external observers consume (JSON
        // on stdout or event bus). Pin its wire shape.
        let original = PreflightCheck {
            name: "battery".into(),
            passed: false,
            message: "Battery at 10% — must be >= 50%".into(),
        };
        let json = serde_json::to_string(&original).unwrap();
        assert!(json.contains("\"passed\":false"));
        let back: PreflightCheck = serde_json::from_str(&json).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn flash_plan_empty_partitions_roundtrip() {
        // A plan with zero partitions (pure backup-only case) must
        // still serialize cleanly and deserialize to an empty Vec,
        // not Option<Vec> collapsed to null.
        let plan = FlashPlan {
            device: "emulator".into(),
            partitions: vec![],
            backup_first: true,
        };
        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("\"partitions\":[]"));
        let back: FlashPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(back, plan);
    }
}
