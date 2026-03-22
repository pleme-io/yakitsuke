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
}
