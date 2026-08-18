//! قراءة توقيع تطبيق أو ملف تنفيذي، باستخدام `codesign`.
//!
//! ## سؤالٌ غير سؤال Gatekeeper
//!
//! `security.gatekeeper` تسأل: «أيسمح هذا النظام بفتح هذا؟» — وهو حكمٌ يتغيّر
//! بتغيّر سياسة الجهاز. وهذه تسأل: «ما التوقيع الملتصق بهذا الملف، ومن سلسلة
//! الجهات التي تشهد له؟» — وهي حقيقةٌ في الملف نفسه لا في الجهاز. من يريد أن
//! يعرف *لماذا* رُفض تطبيقٌ يحتاج الجواب الثاني بعد الأول، ولذلك عمليتان لا
//! واحدة.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/codesign -d -vv -- <الهدف>
//! ```
//!
//! * `-d` — اعرض (‏display) التوقيع القائم. هذه راية القراءة وحدها؛ رايات
//!   التوقيع (`-s`) والاستبدال القسري (`-f`) والإزالة ليست في هذا الأمر، ولا
//!   مدخل في هذه العملية يمكن أن يصير راية — `argv` تُبنى من قائمة ثابتة في
//!   شيفرة مترجَمة، واختبارٌ أدناه يثبّت ذلك على أسماء ملفاتٍ اختيرت لتبدو رايات.
//! * `-vv` — أسهب مرّتين. المستوى الأول يعطي المسار والمعرّف؛ والثاني هو ما
//!   يطبع `Authority=` مرّةً لكل حلقة في السلسلة (المطوّر، ثم جهة التصديق، ثم
//!   الجذر) ويطبع `TeamIdentifier=`. وبدون الحلقة الثانية يبقى «موقَّع» كلمةً
//!   لا تقول من وقّع.
//! * `--` — نهاية الرايات. `codesign` تفهمه (قيس على هذا الجهاز).
//!
//! ## التقرير يُكتب على قناة الخطأ لا على الخرج القياسي
//!
//! قيس: `codesign -d -vv -- /bin/ls 2>/dev/null` لا تطبع شيئًا — كل التقرير،
//! بما فيه حالة النجاح، يذهب إلى `stderr`. التطبيق يبثّ القناتين معًا فلا يضيع
//! منه حرف، لكن الشاشة تسم كل سطرٍ بقناته: فيرى المستخدم تقريرًا سليمًا كاملًا
//! موسومًا «خطأ». وهذا ليس عطبًا في التشغيل ولا في الوسم — إنما هو ما تفعله
//! الأداة — ويستحق أن يُقال قبل أن يُستنتَج خطأً.
//!
//! ## أمانة: «فشل» هنا قد يعني «لا توقيع على هذا الملف»
//!
//! `codesign` تخرج **برمزٍ غير صفري حين لا يكون الهدف موقَّعًا**: عقد النتائج
//! يترجم الواحد إلى `unsigned` والصفر إلى `signed`. كلاهما جوابٌ مكتمل، وما
//! عداه يبقى فشل تنفيذ.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "security.codesign";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.security.codesign.title",
    description_key: "op.security.codesign.description",
    category: Category::Security,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::CODESIGN,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("target", InputKind::ExistingPath)],
    sort_order: 40,
    search_terms: &[
        "codesign",
        "توقيع",
        "signature",
        "signing",
        "شهادة",
        "certificate",
        "مطور",
        "developer",
        "تطبيق",
        "موثوق",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let target = inputs.any_path("target")?;

    let mut argv = Argv::tool(tools::CODESIGN, "explain.codesign.tool")
        .flag("-d", "explain.codesign.display")
        .flag("-vv", "explain.codesign.verbose")
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
        let found = listed.iter().find(|o| o.id == ID).expect("security.codesign must be listed");
        assert_eq!(found.category, Category::Security);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_last_argument_is_the_target() {
        let s = Scratch::new("sign-argv").unwrap();
        let file = s.file("تطبيق", b"data");

        let cmd = plan_with(&file).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/codesign"));
        assert_eq!(args.len(), 4);
        assert_eq!(args[0], "-d");
        assert_eq!(args[1], "-vv");
        assert_eq!(args[2], "--");
        assert_eq!(Path::new(&args[3]), file.as_path());

        assert!(cmd.artifact.is_none());
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.cwd.is_none(), "every path is absolute, so nothing resolves against a cwd");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("sign-explain").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_flag_in_the_command_carries_its_own_explanation() {
        let s = Scratch::new("sign-keys").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role == TokenRole::Flag) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn the_result_contract_replaces_the_obsolete_unsigned_warning() {
        let s = Scratch::new("sign-warn").unwrap();

        for name in ["تطبيق", "مستند.txt", "-f"] {
            let f = s.file(name, b"x");
            let cmd = plan_with(&f).unwrap();
            assert!(cmd.warnings.is_empty(), "{name:?}: {:?}", cmd.warnings);
        }
    }

    /// `codesign` توقّع وتستبدل. لا طريق في هذه العملية إلى رايةٍ تفعل ذلك.
    #[test]
    fn no_signing_flag_can_ever_enter_this_command() {
        let s = Scratch::new("sign-readonly").unwrap();

        for name in ["-s", "-f", "--remove-signature", "--sign"] {
            let target = s.file(name, b"x");
            let cmd = plan_with(&target).unwrap();
            let args = args_of(&cmd);

            assert_eq!(args.len(), 4, "{name:?} added an argument");
            assert_eq!(args[0], "-d", "the first flag must stay the reading flag");
            assert_eq!(args[1], "-vv");
            assert_eq!(args[2], "--");
            assert_eq!(Path::new(&args[3]), target.as_path());
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("sign-dashes").unwrap();
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
        let s = Scratch::new("sign-shellish").unwrap();

        for name in ["ملف 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let target = s.file(name, b"x");
            let cmd = plan_with(&target).unwrap();
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
            assert_eq!(Path::new(cmd.args.last().unwrap()), target.as_path());
        }
    }

    #[test]
    fn a_symlinked_target_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("sign-symlink").unwrap();
        let real = s.file("الحقيقي", b"x");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(cmd.args.last().unwrap()), real.as_path());
        assert!(cmd.warnings.contains(&"warn.target.resolved"), "{:?}", cmd.warnings);
        assert!(!cmd.warnings.contains(&"warn.codesign.unsigned"), "{:?}", cmd.warnings);
    }

    #[test]
    fn a_target_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("sign-missing").unwrap();
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
        let s = Scratch::new("sign-dotdot").unwrap();
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
        let s = Scratch::new("sign-clean").unwrap();
        let f = s.file("ملف", b"x");

        for _ in 0..10 {
            plan_with(&f).unwrap();
        }

        assert_eq!(s.names(s.path()), vec!["ملف".to_owned()], "planning must create nothing");
        assert_eq!(std::fs::read(&f).unwrap(), b"x", "and must not touch the target");
    }
}
