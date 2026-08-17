//! فهرس العمليات والأقسام.
//!
//! ## مصدرٌ واحد، وكل شيء آخر مشتقّ منه
//!
//! `ALL` أدناه هو الفهرس. منه تُشتقّ قائمةُ الشاشة، وعددُ عمليات كل قسم،
//! وترتيبُ العرض، ونتائجُ البحث، وحالةُ الإتاحة على هذا الجهاز. لا يوجد في
//! المنتج رقمٌ مكتوبٌ بيدٍ يقول «ثماني عمليات»، ولا قائمةٌ ثانية في الواجهة
//! تتقادم حين يتغيّر هذا الملف.
//!
//! ## الترتيب معلَنٌ لا عرَضيّ
//!
//! العمليات تُرتَّب بـ (ترتيب القسم، ترتيب العملية، المعرّف). الحدّ الأخير
//! ليس تجميلًا: بدونه يصير ترتيبُ متساويَين في `sort_order` هو ترتيبَ سطريهما
//! في المصفوفة أدناه — أي شيءٌ يتغيّر بالسهو عند كل إضافة، ويظهر للمستخدم
//! قفزةً في مواضع البطاقات بين إصدارين.

use crate::categories::{self, CategorySummary, Kind};
use crate::error::{CoreError, Result};
use crate::ops;
use crate::policy::Policy;
use crate::spec::{Availability, Category, OperationSpec, OperationSummary, Visibility};

/// كل العمليات المُترجَمة، ظاهرةً كانت أو داخلية.
static ALL: &[&OperationSpec] = &[
    // ── الملفات والمجلدات ───────────────────────────────────────────────
    &ops::files_copy::SPEC,
    &ops::files_move::SPEC,
    &ops::files_mkdir::SPEC,
    &ops::files_find_large::SPEC,
    &ops::files_find_stale::SPEC,
    &ops::files_find_name::SPEC,
    &ops::files_tree_size::SPEC,
    &ops::files_open::SPEC,
    &ops::files_list::SPEC,
    &ops::files_identify::SPEC,
    // ── الضغط وفكّ الضغط ────────────────────────────────────────────────
    &ops::compress_ditto::SPEC,
    &ops::compress_zip_list::SPEC,
    &ops::compress_zip_extract::SPEC,
    &ops::compress_zip_test::SPEC,
    &ops::compress_tar_create::SPEC,
    &ops::compress_tar_extract::SPEC,
    &ops::compress_tar_list::SPEC,
    // ── الصور ───────────────────────────────────────────────────────────
    &ops::image_convert::SPEC,
    &ops::image_resize::SPEC,
    &ops::image_rotate::SPEC,
    &ops::image_info::SPEC,
    // ── النصوص والمستندات ───────────────────────────────────────────────
    &ops::text_merge::SPEC,
    &ops::text_split::SPEC,
    &ops::text_encoding::SPEC,
    &ops::text_diff::SPEC,
    &ops::text_search::SPEC,
    // ── الأقراص ومساحة التخزين ──────────────────────────────────────────
    &ops::disk_free::SPEC,
    &ops::disk_hash::SPEC,
    &ops::disk_compare::SPEC,
    &ops::disk_compare_bytes::SPEC,
    &ops::disk_list::SPEC,
    // ── الشبكة والاتصال ─────────────────────────────────────────────────
    &ops::net_ping::SPEC,
    &ops::net_dns::SPEC,
    &ops::net_ports::SPEC,
    &ops::net_download::SPEC,
    &ops::net_headers::SPEC,
    // ── الأمان والصلاحيات ───────────────────────────────────────────────
    &ops::security_permissions::SPEC,
    &ops::security_xattr::SPEC,
    &ops::security_gatekeeper::SPEC,
    &ops::security_codesign::SPEC,
    &ops::security_codesign_verify::SPEC,
    // ── Git ومستودعات الشفرة ────────────────────────────────────────────
    &ops::git_init::SPEC,
    &ops::git_status::SPEC,
    &ops::git_commit::SPEC,
    &ops::git_diff::SPEC,
    &ops::git_branches::SPEC,
    &ops::git_archive::SPEC,
    &ops::git_log::SPEC,
    &ops::git_diff_commits::SPEC,
    &ops::git_show_file::SPEC,
    &ops::git_blame::SPEC,
    &ops::git_grep::SPEC,
    &ops::git_version::SPEC,
    // ── النظام والصيانة الدورية ─────────────────────────────────────────
    &ops::system_processes::SPEC,
    &ops::system_info::SPEC,
    &ops::system_architecture::SPEC,
    &ops::system_uptime::SPEC,
    &ops::system_dns_flush::SPEC,
    &ops::system_report::SPEC,
    // ── أدوات المطوّرين ──────────────────────────────────────────────────
    &ops::dev_npm_typecheck::SPEC,
    &ops::dev_npm_lint::SPEC,
    &ops::dev_npm_test::SPEC,
    &ops::dev_npm_install::SPEC,
    &ops::dev_npm_dev::SPEC,
    &ops::dev_tauri_dev::SPEC,
    &ops::dev_tauri_build::SPEC,
    &ops::dev_cargo_test::SPEC,
    &ops::dev_cargo_check::SPEC,
    &ops::dev_cargo_clippy::SPEC,
    &ops::dev_cargo_fmt_check::SPEC,
    &ops::dev_cargo_fmt::SPEC,
    &ops::dev_cargo_build_release::SPEC,
    &ops::dev_cargo_clean::SPEC,
    // ── داخليّة ─────────────────────────────────────────────────────────
    &ops::internal_echo::SPEC,
];

/// ترتيب العرض: القسم أولًا، ثم العملية داخله، ثم المعرّف لفكّ التعادل.
fn ordering_key(op: &OperationSpec) -> (u16, u16, &'static str) {
    let category_order = categories::find(op.category).map(|c| c.sort_order).unwrap_or(u16::MAX);
    (category_order, op.sort_order, op.id)
}

/// العمليات الظاهرة، مرتّبة. **لا تُسقط غير المتاح**: الإتاحة حقلٌ يُعرض.
///
/// إخفاءُ عمليةٍ لأن أداتها غائبة يجعل المستخدم يظنّ أن المنتج لا يفعل ذلك
/// أصلًا. عرضُها معطّلةً بسببٍ مسمّى يجعله يعرف أن أداةً واحدة تنقصه.
pub fn list(_policy: Policy) -> Vec<OperationSummary> {
    let mut visible: Vec<&'static OperationSpec> =
        ALL.iter().copied().filter(|op| op.visibility == Visibility::Production).collect();
    visible.sort_by_key(|op| ordering_key(op));
    visible.into_iter().map(OperationSummary::from).collect()
}

/// عمليات قسمٍ بعينه، مرتّبة.
pub fn in_category(category: Category, policy: Policy) -> Vec<OperationSummary> {
    list(policy).into_iter().filter(|o| o.category == category).collect()
}

/// الأقسام المعروضة، وعددُ عمليات كلٍّ منها **محسوبًا من الفهرس**.
///
/// القسم الفارغ يُسقَط: وعدٌ لا يُوفى أسوأ من الغياب. ويستثنى قسم السجلّ لأن
/// مصدره ليس الفهرس، وقسمُ الداخليّ لأنه لا يُعرض في أي بناء.
pub fn categories(_policy: Policy) -> Vec<CategorySummary> {
    let mut out: Vec<CategorySummary> = categories::ALL
        .iter()
        .filter(|c| c.kind != Kind::Hidden)
        .filter_map(|meta| {
            let ops_here: Vec<&'static OperationSpec> = ALL
                .iter()
                .copied()
                .filter(|op| op.visibility == Visibility::Production && op.category == meta.id)
                .collect();

            if meta.kind == Kind::Operations && ops_here.is_empty() {
                return None;
            }

            let available =
                ops_here.iter().filter(|op| Availability::of(op).is_available()).count();

            Some(CategorySummary {
                id: meta.id,
                title_key: meta.title_key,
                description_key: meta.description_key,
                icon: meta.icon,
                sort_order: meta.sort_order,
                kind: meta.kind,
                operation_count: ops_here.len(),
                available_count: available,
            })
        })
        .collect();
    out.sort_by_key(|c| c.sort_order);
    out
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
    use crate::spec::{Conflict, Danger, InputKind};
    use std::collections::HashSet;

    fn production() -> Vec<OperationSummary> {
        list(Policy::production())
    }

    #[test]
    fn internal_operations_never_appear_in_the_catalogue() {
        for policy in [Policy::production(), Policy::dev()] {
            let listed = list(policy);
            assert!(
                !listed.iter().any(|o| o.id.starts_with("internal.")),
                "internal ops must not be listed under {policy:?}"
            );
            assert!(!listed.iter().any(|o| o.category == Category::Internal));
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
        let mut seen = HashSet::new();
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

    /// مفتاحٌ يُشتقّ من المعرّف لا يُخترع.
    ///
    /// الحارس يمنع الحالتين اللتين تقعان فعلًا: عمليةٌ تنسخ مفاتيح أختها
    /// فتعرض الشاشة عنوانَ عمليةٍ أخرى، وعمليةٌ تُعاد تسميتها ويبقى مفتاحها
    /// على الاسم القديم فلا يجده أحد في `i18n.ts`.
    #[test]
    fn message_keys_are_derived_from_the_operation_id() {
        for op in ALL {
            assert_eq!(op.title_key, format!("op.{}.title", op.id), "{}", op.id);
            assert_eq!(op.description_key, format!("op.{}.description", op.id), "{}", op.id);
        }
    }

    #[test]
    fn input_ids_are_unique_within_an_operation() {
        for op in ALL {
            let mut seen = HashSet::new();
            for i in op.inputs {
                assert!(seen.insert(i.id), "{} declares `{}` twice", op.id, i.id);
            }
        }
    }

    #[test]
    fn every_operation_belongs_to_a_category_that_exists() {
        for op in ALL {
            assert!(
                categories::find(op.category).is_some(),
                "{} sits in a category with no metadata",
                op.id
            );
        }
    }

    #[test]
    fn only_the_internal_operation_lives_in_the_internal_category() {
        for op in ALL {
            assert_eq!(
                op.category == Category::Internal,
                op.visibility == Visibility::Internal,
                "{} disagrees about being internal",
                op.id
            );
        }
    }

    #[test]
    fn no_production_operation_sits_in_the_journal_category() {
        // قسم السجلّ مصدره السجلّ. عمليةٌ تُوضع فيه لا يعرضها شيء.
        for op in ALL {
            assert_ne!(op.category, Category::History, "{} would never be shown", op.id);
        }
    }

    #[test]
    fn the_catalogue_is_ordered_by_category_then_by_operation() {
        let listed = production();
        let keys: Vec<_> = listed
            .iter()
            .map(|o| {
                let cat = categories::find(o.category).unwrap().sort_order;
                (cat, o.sort_order, o.id)
            })
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "list() must return the declared order, not the array order");
    }

    #[test]
    fn sort_order_is_unique_within_each_category() {
        // تعادلٌ في الترتيب يجعل موضع البطاقتين رهنَ فكّ التعادل بالمعرّف — وهو
        // ثابتٌ لكنه ليس قرار منتج. الأصحّ أن يُعلن الترتيب صراحةً.
        let mut seen: HashSet<(Category, u16)> = HashSet::new();
        for op in ALL.iter().filter(|o| o.visibility == Visibility::Production) {
            assert!(
                seen.insert((op.category, op.sort_order)),
                "{} shares sort_order {} with a sibling",
                op.id,
                op.sort_order
            );
        }
    }

    #[test]
    fn every_operation_carries_search_terms_that_include_its_tool() {
        // البحث يجب أن يجد العملية باسم أداتها: من يعرف `ditto` يكتبها.
        for op in ALL.iter().filter(|o| o.visibility == Visibility::Production) {
            assert!(!op.search_terms.is_empty(), "{} is unsearchable", op.id);
            assert!(
                op.search_terms.contains(&op.tool.id),
                "{} does not answer a search for its own tool ({})",
                op.id,
                op.tool.id
            );
            for term in op.search_terms {
                assert!(!term.is_empty(), "{} declares an empty search term", op.id);
                assert_eq!(*term, term.trim(), "{} declares a padded search term", op.id);
                assert_eq!(
                    *term,
                    term.to_lowercase(),
                    "{}: search terms are matched lowercased, so declare them lowercased",
                    op.id
                );
            }
        }
    }

    #[test]
    fn an_operation_that_produces_nothing_declares_no_artifact_conflict() {
        // السياسة المعلَنة تُعرض للمستخدم قبل التنفيذ. عمليةُ قراءةٍ تعلن
        // `Refuse` تَعِد بحمايةِ اسمٍ لا تكتبه، وعمليةٌ تكتب وتعلن `NoArtifact`
        // تتجاوز فحص التضارب المركزي في `planner`.
        for op in ALL {
            if op.danger == Danger::Safe {
                assert_eq!(
                    op.conflict,
                    Conflict::NoArtifact,
                    "{} reads only, so it has no final name to protect",
                    op.id
                );
            }
        }
    }

    #[test]
    fn every_operation_declares_at_least_one_required_input_or_none_at_all() {
        // عمليةٌ بلا مدخلات مشروعة (‏`sw_vers`)، وعمليةٌ كل مدخلاتها اختيارية
        // ليست كذلك: الواجهة تعتبرها مكتملةً فورًا وتخطّطها قبل أن يفهم
        // المستخدم أنه أمام نموذج.
        for op in ALL {
            if !op.inputs.is_empty() {
                assert!(
                    op.inputs.iter().any(|i| i.required),
                    "{} has inputs but none of them is required",
                    op.id
                );
            }
        }
    }

    #[test]
    fn a_choice_input_declares_at_least_two_distinct_options() {
        for op in ALL {
            for input in op.inputs {
                if let InputKind::Choice { options } = input.kind {
                    assert!(
                        options.len() >= 2,
                        "{}.{} is a choice of {} — that is not a choice",
                        op.id,
                        input.id,
                        options.len()
                    );
                    let mut seen = HashSet::new();
                    for o in options {
                        assert!(
                            seen.insert(o.value),
                            "{}.{} lists {} twice",
                            op.id,
                            input.id,
                            o.value
                        );
                        assert!(
                            !o.value.starts_with('-'),
                            "{}.{}: option {:?} could be read as a flag",
                            op.id,
                            input.id,
                            o.value
                        );
                        assert!(!o.label_key.is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn a_number_input_declares_a_usable_range_and_a_default_inside_it() {
        for op in ALL {
            for input in op.inputs {
                if let InputKind::Number { min, max, default } = input.kind {
                    assert!(min < max, "{}.{} has an empty range", op.id, input.id);
                    assert!(
                        (min..=max).contains(&default),
                        "{}.{} defaults to {default}, outside {min}..={max}",
                        op.id,
                        input.id
                    );
                }
            }
        }
    }

    // ── الأقسام ────────────────────────────────────────────────────────

    #[test]
    fn category_counts_are_computed_from_the_catalogue_not_written_by_hand() {
        for c in categories(Policy::production()) {
            let actual = in_category(c.id, Policy::production()).len();
            assert_eq!(c.operation_count, actual, "{:?} miscounts its operations", c.id);
            assert!(c.available_count <= c.operation_count);
        }
    }

    #[test]
    fn the_counts_add_up_to_the_whole_catalogue() {
        let total: usize = categories(Policy::production()).iter().map(|c| c.operation_count).sum();
        assert_eq!(
            total,
            production().len(),
            "every listed operation must belong to exactly one shown category"
        );
    }

    #[test]
    fn no_empty_operations_category_is_shown() {
        for c in categories(Policy::production()) {
            if c.kind == Kind::Operations {
                assert!(c.operation_count > 0, "{:?} is an empty promise", c.id);
            }
        }
    }

    #[test]
    fn the_journal_category_is_shown_even_though_it_holds_no_operations() {
        let shown = categories(Policy::production());
        let history = shown.iter().find(|c| c.id == Category::History);
        let history = history.expect("the run journal is a section of the library, not a footnote");
        assert_eq!(history.kind, Kind::Journal);
        assert_eq!(history.operation_count, 0);
    }

    #[test]
    fn the_hidden_category_is_never_shown() {
        assert!(!categories(Policy::dev()).iter().any(|c| c.id == Category::Internal));
    }

    #[test]
    fn the_shown_categories_come_back_in_their_declared_order() {
        let shown = categories(Policy::production());
        let orders: Vec<u16> = shown.iter().map(|c| c.sort_order).collect();
        let mut sorted = orders.clone();
        sorted.sort();
        assert_eq!(orders, sorted);
    }

    #[test]
    fn the_catalogue_covers_every_functional_category() {
        // العدّ ليس رقمًا تجميليًا: هذا يثبت أن كل قسمٍ أُعلن قد امتلأ فعلًا.
        let shown: HashSet<Category> =
            categories(Policy::production()).iter().map(|c| c.id).collect();
        for expected in [
            Category::Files,
            Category::Compress,
            Category::Images,
            Category::Text,
            Category::Disk,
            Category::Network,
            Category::Security,
            Category::Git,
            Category::System,
            Category::History,
        ] {
            assert!(shown.contains(&expected), "{expected:?} is missing from the library");
        }
    }

    #[test]
    fn the_production_catalogue_is_large_enough_to_be_a_library() {
        // حارسٌ ضدّ الانكماش الصامت: عمليةٌ تُحذف بالسهو تسقط هنا.
        assert!(
            production().len() >= 24,
            "the library ships {} operations; the floor is 24",
            production().len()
        );
    }
}
