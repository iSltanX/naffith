//! مقارنة ملفين نصّيين وعرض الفرق، باستخدام `diff`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/diff -u -- <الأيسر> <الأيمن>
//! ```
//!
//! `-u` تُخرج «الفرق الموحّد»: أسطرٌ مسبوقة بـ`-` حُذفت ومسبوقة بـ`+` أُضيفت،
//! مع ثلاثة أسطر من السياق حولها. اختيرت على الشكل الافتراضي (`1c1`) لأن
//! الأخير يقول «تغيّر السطر الأول» ولا يعرض ما كان وما صار، وعلى `-y`
//! (عمودين) لأن العمودين يقصّان الأسطر الطويلة على عرض الشاشة — وشاشة هذا
//! المنتج تبثّ السطور بثًّا لا تجدولها.
//!
//! `--` تفهمها `diff` على macOS، وهي حزام أمانٍ فوق كون المسارين مطلقين.
//!
//! ## لا تكتب شيئًا
//!
//! `diff` تقرأ وتطبع. لا ناتج، ولا اسم نهائي يتضارب، ولا ملف مؤقّت — ولذلك
//! `Danger::Safe` و`Conflict::NoArtifact`. و«إظهار في Finder» يفتح مجلد
//! الملف الأيسر، لأن سؤال «أين أنظر؟» يبقى له جواب حتى حين لا يُنتج شيء.
//!
//! ## حدٌّ معروف: `diff` تخرج بـ‎1‎ حين تجد فرقًا
//!
//! يقول `man diff`: `0` لا فرق، `1` وُجد فرق، وما فوق خطأ. أي أن **النجاح
//! التام** لهذه العملية — أن تجد فرقًا وتعرضه — يخرج بـ‎1‎.
//!
//! عقد النتائج يترجم الصفر إلى `no_differences` والواحد إلى `differences`؛
//! كلاهما جوابٌ مكتمل، وما فوق الواحد وحده فشل تنفيذ.

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "text.diff";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.text.diff.title",
    description_key: "op.text.diff.description",
    category: Category::Text,
    // قراءةٌ محضة: لا تكتب بايتًا واحدًا على القرص.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::DIFF,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("left", InputKind::ExistingFile),
        InputSpec::new("right", InputKind::ExistingFile),
    ],
    sort_order: 40,
    search_terms: &["diff", "فرق", "مقارنة", "compare", "قارن", "نص", "text", "تغييرات"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let left = inputs.file("left")?;
    let right = inputs.file("right")?;

    // المساران محلولا الروابط، فالمقارنة مقارنة موضعين: رابطٌ رمزي إلى الملف
    // نفسه يُلتقط هنا. ومقارنة الملف بنفسه لا تفشل في `diff` — تخرج بصفر بلا
    // سطرٍ واحد — لكنها شاشةٌ فارغة لا تقول للمستخدم إنه قارن شيئًا بنفسه.
    if left == right {
        return Err(CoreError::SamePath.on_input("right"));
    }

    // مجلد الملف الأيسر جواب «أين أنظر؟». وملفٌ قائمٌ مطلق له أبٌ دائمًا،
    // والبديل عند غيابه هو الملف نفسه لا مسارٌ نسبيّ يرفضه `reveal`.
    let folder = left.parent().unwrap_or(left);

    Argv::tool(tools::DIFF, "explain.diff.tool")
        .flag("-u", "explain.diff.unified")
        .end_of_flags()
        .explained_path(left, "explain.role.diff_left")
        .explained_path(right, "explain.role.diff_right")
        .warn_all(warnings_for(inputs, left, right))
        .reveal(folder)
        .read_only()
}

fn warnings_for(inputs: &Inputs, left: &Path, right: &Path) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    warnings.extend(warn_if_resolved(inputs, "left", left, "warn.source.resolved"));
    warnings.extend(warn_if_resolved(inputs, "right", right, "warn.source.resolved"));
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;

    fn raw(left: &Path, right: &Path) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("left".to_owned(), RawValue::Path(left.display().to_string())),
            ("right".to_owned(), RawValue::Path(right.display().to_string())),
        ])
    }

    fn plan_with(left: &Path, right: &Path) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(left, right))?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("text.diff must be listed");
        assert_eq!(found.category, Category::Text);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact, "a reader has no final name to protect");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_is_written() {
        let s = Scratch::new("diff-argv").unwrap();
        let left = s.file("قبل.txt", b"a\n");
        let right = s.file("بعد.txt", b"b\n");

        let cmd = plan_with(&left, &right).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/diff"));
        assert_eq!(args[0], "-u");
        assert_eq!(args[1], "--");
        assert_eq!(Path::new(&args[2]), left.as_path());
        assert_eq!(Path::new(&args[3]), right.as_path());
        assert_eq!(args.len(), 4);

        assert!(cmd.artifact.is_none(), "diff produces no file");
        assert!(cmd.stdout_to.is_none(), "the difference is streamed, not captured into a file");
        assert!(cmd.estimate.is_none());
        assert_eq!(cmd.reveal_target.as_deref(), Some(s.path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("diff-explain").unwrap();
        let left = s.file("أ", b"1\n");
        let right = s.file("ب", b"2\n");

        let cmd = plan_with(&left, &right).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("diff-dashes").unwrap();
        let left = s.file("-rf", b"1\n");
        let right = s.file("--version", b"2\n");

        let cmd = plan_with(&left, &right).unwrap();
        // `-u` و`--` رايتان معلَنتان؛ ما بعدهما بيانات.
        for arg in cmd.args.iter().skip(2) {
            assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag");
        }
    }

    #[test]
    fn comparing_a_file_with_itself_is_refused() {
        let s = Scratch::new("diff-same").unwrap();
        let one = s.file("واحد.txt", b"x\n");
        assert_eq!(refusal(plan_with(&one, &one)), ("err.path.same", Some("right")));
    }

    #[test]
    fn a_symlink_to_the_same_file_is_caught_because_paths_are_resolved_first() {
        let s = Scratch::new("diff-same-link").unwrap();
        let real = s.file("حقيقي.txt", b"x\n");
        let link = s.path().join("رابط.txt");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert_eq!(refusal(plan_with(&real, &link)), ("err.path.same", Some("right")));
    }

    #[test]
    fn a_missing_file_is_refused_with_the_field_that_names_it() {
        let s = Scratch::new("diff-missing").unwrap();
        let left = s.file("موجود.txt", b"x\n");
        let ghost = s.path().join("لا-وجود-له.txt");
        assert_eq!(refusal(plan_with(&left, &ghost)), ("err.path.missing", Some("right")));
    }

    #[test]
    fn the_result_contract_replaces_the_obsolete_exit_code_warning() {
        let s = Scratch::new("diff-warn").unwrap();
        let left = s.file("أ", b"1\n");
        let right = s.file("ب", b"2\n");

        let cmd = plan_with(&left, &right).unwrap();
        assert!(!cmd.warnings.contains(&"warn.diff.exit_code"), "{:?}", cmd.warnings);
    }

    #[test]
    fn shell_syntax_in_a_file_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("diff-shellish").unwrap();
        let right = s.file("ثابت.txt", b"0\n");

        for name in ["ملف 'اليوم'.txt", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let left = s.file(name, b"1\n");
            let cmd = plan_with(&left, &right).unwrap();
            assert_eq!(Path::new(&cmd.args[2]), left.as_path());
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_symlinked_file_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("diff-symlink").unwrap();
        let real = s.file("الحقيقي.txt", b"x\n");
        let link = s.path().join("رابط.txt");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let other = s.file("آخر.txt", b"y\n");

        let cmd = plan_with(&link, &other).unwrap();
        assert_eq!(Path::new(&cmd.args[2]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_writes_nothing_to_disk() {
        let s = Scratch::new("diff-clean").unwrap();
        let left = s.file("أ", b"1\n");
        let right = s.file("ب", b"2\n");

        let before = s.names(s.path());
        for _ in 0..10 {
            plan_with(&left, &right).unwrap();
        }
        assert_eq!(s.names(s.path()), before, "planning a read-only command must touch nothing");
    }
}
