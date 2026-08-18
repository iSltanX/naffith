//! تقييم Gatekeeper لملف أو تطبيق، باستخدام `spctl`.
//!
//! ## لماذا `spctl` لا فحصٌ نكتبه
//!
//! السؤال المطروح ليس «أهذا التطبيق موقَّع؟» — ذاك سؤال `security.codesign` —
//! بل «أيسمح **هذا النظام** بفتحه؟». والفرق بينهما سياسةٌ محلّية: التوثيق
//! (‏notarization)، وحالة المطوّر، وما إذا كان المستخدم قد استثنى الملف من
//! قبل. أي فحصٍ نكتبه بأنفسنا كان سيجيب عن سؤالٍ يشبه السؤال ولا يساويه، وكان
//! سيصير خطأً صامتًا يوم تتغيّر السياسة في تحديثٍ للنظام. `spctl` تسأل المحرّك
//! نفسه الذي يسأله النظام عند النقر المزدوج.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/sbin/spctl -a -vv -- <الهدف>
//! ```
//!
//! * `-a` — قيّم الهدف بسياسة التنفيذ (‏assess).
//! * `-vv` — أسهب مرّتين. المستوى الأول يعطي الحكم وحده؛ والثاني يضيف
//!   `origin=` أي الجهة الموقِّعة — وهي المعلومة التي تفيد فعلًا، لأن «مرفوض»
//!   بلا مصدرٍ لا يقول للمستخدم إن كان الملف مزوَّرًا أم غير موثَّق فحسب.
//! * `--` — نهاية الرايات. `spctl` تفهمه (قيس على هذا الجهاز).
//!
//! ## أمانة: «فشل» هنا قد يعني «رُفض بحقّ»
//!
//! `spctl` تخرج **برمزٍ غير صفري حين ترفض**: عقد النتائج يترجم الرمز ٣ إلى
//! `rejected` والصفر إلى `accepted`. كلاهما حكمٌ مكتمل، وما عداه يفشل مغلقًا.
//!
//! وأمانةٌ ثانية: `spctl` تكتب حكمها على قناة الخطأ لا على الخرج القياسي
//! (قيس)، والتطبيق يبثّ القناتين معًا فلا يضيع شيء.
//!
//! ## ما لا تقيّمه
//!
//! السياسة لا تعرف إلا ما تفهمه: التطبيقات والمثبِّتات وصور الأقراص والملفات
//! التنفيذية. مستندٌ عادي يعود «مرفوضًا» لأنه ليس مما يُقيَّم أصلًا، لا لأن فيه
//! عيبًا — وقد قيس ذلك على `/bin/ls` نفسها: «‏the code is valid but does not
//! seem to be an app».

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "security.gatekeeper";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.security.gatekeeper.title",
    description_key: "op.security.gatekeeper.description",
    category: Category::Security,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::SPCTL,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("target", InputKind::ExistingPath)],
    sort_order: 30,
    search_terms: &[
        "spctl",
        "gatekeeper",
        "أمان",
        "security",
        "تقييم",
        "assess",
        "بوابة",
        "توثيق",
        "notarization",
        "تطبيق",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let target = inputs.any_path("target")?;

    let mut argv = Argv::tool(tools::SPCTL, "explain.spctl.tool")
        .flag("-a", "explain.spctl.assess")
        .flag("-vv", "explain.spctl.verbose")
        .end_of_flags()
        .path(target);

    if let Some(key) = warn_if_resolved(inputs, "target", target, "warn.target.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn plan_with(target: &Path) -> Result<PlannedCommand> {
        let raw =
            BTreeMap::from([("target".to_owned(), RawValue::Path(target.display().to_string()))]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    fn args_of(cmd: &PlannedCommand) -> Vec<String> {
        cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("security.gatekeeper must be listed");
        assert_eq!(found.category, Category::Security);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_last_argument_is_the_target() {
        let s = Scratch::new("gate-argv").unwrap();
        let file = s.file("برنامج", b"data");

        let cmd = plan_with(&file).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/sbin/spctl"));
        assert_eq!(args.len(), 4);
        assert_eq!(args[0], "-a");
        assert_eq!(args[1], "-vv");
        assert_eq!(args[2], "--");
        assert_eq!(Path::new(&args[3]), file.as_path());

        assert!(cmd.artifact.is_none());
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.cwd.is_none(), "every path is absolute, so nothing resolves against a cwd");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("gate-explain").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_flag_in_the_command_carries_its_own_explanation() {
        let s = Scratch::new("gate-keys").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role == TokenRole::Flag) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    /// التحذير الذي لا يجوز أن يغيب.
    ///
    #[test]
    fn the_result_contract_replaces_the_obsolete_exit_code_warning() {
        let s = Scratch::new("gate-warn").unwrap();

        for name in ["تطبيق", "مستند.txt", "-rf"] {
            let f = s.file(name, b"x");
            let cmd = plan_with(&f).unwrap();
            assert!(cmd.warnings.is_empty(), "{name:?}: {:?}", cmd.warnings);
        }

        let d = s.dir("مجلد");
        assert!(plan_with(&d).unwrap().warnings.is_empty());
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("gate-dashes").unwrap();
        let target = s.file("-rf", b"x");

        let cmd = plan_with(&target).unwrap();
        let args = args_of(&cmd);
        let separator = args
            .iter()
            .position(|a| a.as_str() == "--")
            .expect("the separator must be in the argv");
        for a in cmd.args.iter().skip(separator + 1) {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn shell_syntax_in_the_target_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("gate-shellish").unwrap();

        for name in ["ملف 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let target = s.file(name, b"x");
            let cmd = plan_with(&target).unwrap();
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
            assert_eq!(Path::new(cmd.args.last().unwrap()), target.as_path());
        }
    }

    #[test]
    fn a_symlinked_target_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("gate-symlink").unwrap();
        let real = s.file("الحقيقي", b"x");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(cmd.args.last().unwrap()), real.as_path());
        assert!(cmd.warnings.contains(&"warn.target.resolved"), "{:?}", cmd.warnings);
        assert!(!cmd.warnings.contains(&"warn.spctl.exit_code"), "{:?}", cmd.warnings);
    }

    #[test]
    fn a_target_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("gate-missing").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"))),
            ("err.path.missing", Some("target"))
        );
    }

    #[test]
    fn a_target_outside_the_allowed_roots_is_refused() {
        assert_eq!(refusal(plan_with(Path::new("/etc"))), ("err.path.outside", Some("target")));
    }

    #[test]
    fn a_target_that_climbs_out_with_dotdot_is_refused_before_the_disk_is_touched() {
        let s = Scratch::new("gate-dotdot").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("../خارج"))),
            ("err.path.traversal", Some("target"))
        );
    }

    #[test]
    fn a_relative_target_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("نسبي/هنا"))),
            ("err.path.relative", Some("target"))
        );
    }

    #[test]
    fn planning_writes_nothing_anywhere() {
        let s = Scratch::new("gate-clean").unwrap();
        let f = s.file("ملف", b"x");

        for _ in 0..10 {
            plan_with(&f).unwrap();
        }

        assert_eq!(s.names(s.path()), vec!["ملف".to_owned()], "planning must create nothing");
        assert_eq!(std::fs::read(&f).unwrap(), b"x", "and must not touch the target");
    }
}
