pub const LOW_RAM_GIB_THRESHOLD: u64 = 8;
pub const LOW_DISK_GIB_THRESHOLD: u64 = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl CheckStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckResult {
    pub id: &'static str,
    pub label: &'static str,
    pub status: CheckStatus,
    pub message: String,
    pub is_hard: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreflightContext {
    pub is_uefi: bool,
    pub has_disk: bool,
    pub online: bool,
    pub ram_gib: u64,
    pub disk_gib: Option<u64>,
}

pub fn evaluate_checks(ctx: &PreflightContext) -> Vec<CheckResult> {
    let mut checks = Vec::new();

    checks.push(CheckResult {
        id: "uefi_mode",
        label: "UEFI mode",
        status: if ctx.is_uefi {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        message: if ctx.is_uefi {
            "UEFI mode detected.".to_string()
        } else {
            "UEFI mode not detected. Boot installer in UEFI mode.".to_string()
        },
        is_hard: true,
    });

    checks.push(CheckResult {
        id: "disk_selected",
        label: "Target disk selected",
        status: if ctx.has_disk {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        message: if ctx.has_disk {
            "Target disk is selected.".to_string()
        } else {
            "No target disk selected.".to_string()
        },
        is_hard: true,
    });

    checks.push(CheckResult {
        id: "internet",
        label: "Internet reachable",
        status: if ctx.online {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        message: if ctx.online {
            "Network connectivity verified.".to_string()
        } else {
            "No internet connectivity detected.".to_string()
        },
        is_hard: true,
    });

    let low_ram = ctx.ram_gib < LOW_RAM_GIB_THRESHOLD;
    checks.push(CheckResult {
        id: "ram_capacity",
        label: "RAM capacity",
        status: if low_ram {
            CheckStatus::Warn
        } else {
            CheckStatus::Pass
        },
        message: if low_ram {
            format!(
                "Detected {} GiB RAM (< {} GiB recommended).",
                ctx.ram_gib, LOW_RAM_GIB_THRESHOLD
            )
        } else {
            format!("Detected {} GiB RAM.", ctx.ram_gib)
        },
        is_hard: false,
    });

    let (disk_status, disk_message) = if !ctx.has_disk {
        (
            CheckStatus::Pass,
            "Disk capacity check skipped until a target disk is selected.".to_string(),
        )
    } else if let Some(disk_gib) = ctx.disk_gib {
        if disk_gib < LOW_DISK_GIB_THRESHOLD {
            (
                CheckStatus::Warn,
                format!(
                    "Detected {} GiB disk (< {} GiB recommended).",
                    disk_gib, LOW_DISK_GIB_THRESHOLD
                ),
            )
        } else {
            (CheckStatus::Pass, format!("Detected {} GiB disk.", disk_gib))
        }
    } else {
        (
            CheckStatus::Warn,
            "Unable to determine selected disk capacity.".to_string(),
        )
    };
    checks.push(CheckResult {
        id: "disk_capacity",
        label: "Disk capacity",
        status: disk_status,
        message: disk_message,
        is_hard: false,
    });

    checks
}

pub fn has_hard_fail(results: &[CheckResult]) -> bool {
    results
        .iter()
        .any(|check| check.is_hard && check.status == CheckStatus::Fail)
}

#[cfg(test)]
mod tests {
    use super::{CheckStatus, LOW_DISK_GIB_THRESHOLD, LOW_RAM_GIB_THRESHOLD, PreflightContext, evaluate_checks, has_hard_fail};

    fn context() -> PreflightContext {
        PreflightContext {
            is_uefi: true,
            has_disk: true,
            online: true,
            ram_gib: 16,
            disk_gib: Some(512),
        }
    }

    #[test]
    fn hard_fail_blocks_progress_when_uefi_missing() {
        let mut ctx = context();
        ctx.is_uefi = false;
        let results = evaluate_checks(&ctx);
        assert!(has_hard_fail(&results));
    }

    #[test]
    fn soft_warnings_do_not_create_hard_fail() {
        let mut ctx = context();
        ctx.ram_gib = LOW_RAM_GIB_THRESHOLD - 1;
        ctx.disk_gib = Some(LOW_DISK_GIB_THRESHOLD - 1);
        let results = evaluate_checks(&ctx);
        assert!(!has_hard_fail(&results));
        assert_eq!(
            results
                .iter()
                .filter(|check| check.status == CheckStatus::Warn)
                .count(),
            2
        );
    }

    #[test]
    fn all_good_context_passes_hard_checks() {
        let results = evaluate_checks(&context());
        assert!(!has_hard_fail(&results));
        assert!(results.iter().all(|check| check.status != CheckStatus::Fail));
    }
}
