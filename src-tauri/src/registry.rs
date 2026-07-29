//! البحث في فهرس العمليات.

use crate::error::{CoreError, Result};
use crate::ops;
use crate::policy::Policy;
use crate::spec::{OperationSpec, OperationSummary, Visibility};

/// كل العمليات المُترجَمة، ظاهرةً كانت أو داخلية.
static ALL: &[&OperationSpec] = &[&ops::compress_ditto::SPEC, &ops::internal_echo::SPEC];

/// ما يُعرض في الفهرس. العمليات الداخلية لا تظهر هنا **في أي بناء** — حتى في
/// التطوير — كي لا تتسرّب إلى لقطة شاشة أو توثيق.
pub fn list(_policy: Policy) -> Vec<OperationSummary> {
    ALL.iter()
        .filter(|op| op.visibility == Visibility::Production)
        .map(|op| OperationSummary::from(*op))
        .collect()
}

/// يبحث عن عملية مع فرض السياسة.
///
/// عملية داخلية في وضع الإنتاج تُرفض بـ `OperationNotAvailable` لا
/// `UnknownOperation`: التمييز داخلي ومفيد في السجل، والواجهة تعرض للحالتين
/// نصًا واحدًا فلا تتسرّب معلومة عن وجودها.
pub fn find(id: &str, policy: Policy) -> Result<&'static OperationSpec> {
    let Some(op) = ALL.iter().find(|op| op.id == id) else {
        return Err(CoreError::UnknownOperation(id.to_owned()));
    };
    if op.visibility == Visibility::Internal && !policy.allow_internal_operations {
        return Err(CoreError::OperationNotAvailable(id.to_owned()));
    }
    Ok(op)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_operations_never_appear_in_the_catalogue() {
        for policy in [Policy::production(), Policy::dev()] {
            let listed = list(policy);
            assert!(
                !listed.iter().any(|o| o.id.starts_with("internal.")),
                "internal ops must not be listed under {policy:?}"
            );
        }
    }

    #[test]
    fn production_refuses_to_plan_an_internal_operation() {
        let r = find("internal.echo", Policy::production());
        assert!(matches!(r, Err(CoreError::OperationNotAvailable(_))), "got {r:?}");
    }

    #[test]
    fn dev_can_reach_the_internal_operation() {
        assert!(find("internal.echo", Policy::dev()).is_ok());
    }

    #[test]
    fn an_unknown_id_is_rejected_under_every_policy() {
        for policy in [Policy::production(), Policy::dev()] {
            assert!(matches!(
                find("no.such.operation", policy),
                Err(CoreError::UnknownOperation(_))
            ));
        }
    }

    #[test]
    fn every_operation_declares_an_absolute_tool_path() {
        for op in ALL {
            assert!(
                std::path::Path::new(op.tool.absolute).is_absolute(),
                "{} resolves its tool through PATH",
                op.id
            );
        }
    }

    #[test]
    fn operation_ids_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for op in ALL {
            assert!(seen.insert(op.id), "duplicate operation id: {}", op.id);
        }
    }

    #[test]
    fn every_operation_declares_arabic_title_and_description_keys() {
        for op in ALL {
            assert!(!op.title_key.is_empty(), "{} has no title key", op.id);
            assert!(!op.description_key.is_empty(), "{} has no description key", op.id);
            assert_ne!(op.title_key, op.description_key, "{} reuses one key twice", op.id);
        }
    }

    #[test]
    fn the_production_catalogue_holds_exactly_the_operations_m1_ships() {
        let ids: Vec<&str> = list(Policy::production()).iter().map(|o| o.id).collect();
        assert_eq!(
            ids,
            vec!["compress.folder.zip"],
            "M1 ships one operation. Adding another is a deliberate act — update this test."
        );
    }

    #[test]
    fn input_ids_are_unique_within_an_operation() {
        for op in ALL {
            let mut seen = std::collections::HashSet::new();
            for i in op.inputs {
                assert!(seen.insert(i.id), "{} declares `{}` twice", op.id, i.id);
            }
        }
    }
}
