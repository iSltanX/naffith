//! تعرّف نوع ملفٍّ أو مجلدٍ من محتواه، لا من امتداد اسمه، بـ`file -b`.
//!
//! ## سؤالٌ أسبق من سؤال `image.info`
//!
//! `image.info` تفترض أن الهدف صورةٌ بالفعل، وتسأل سؤالًا عميقًا عنها: كم
//! عرضها، وما دقّتها، وما فضاء ألوانها. هذه العملية تسأل السؤال الذي يسبق كل
//! ذلك: «ما هذا الملفّ أصلًا؟» — ولأيّ ملفٍّ لا لصورةٍ وحدها: مستند، أرشيف،
//! ملفٌّ تنفيذي، أو مجلد. امتداد الاسم لا يُصدَّق — ملفٌّ اسمه `صورة.jpg` قد
//! يكون نصًّا عاديًا أُعيدت تسميته — و`file` تقرأ البايتات الأولى فعلًا
//! (‏magic bytes) بدل أن تثق بما يقوله الاسم عن نفسه.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/file -b -- <الهدف>
//! ```
//!
//! * `-b` (‏brief) — اطبع وصف النوع وحده، بلا إعادة كتابة المسار قبله. بدون
//!   هذه الراية يطبع `file` سطرًا مثل `‎/المسار: ASCII text‎` — والمسار معروضٌ
//!   أصلًا في `argv` نفسه؛ تكراره في الخرج لا يضيف شيئًا ويزيد الشاشة ازدحامًا
//!   بلا فائدة.
//! * `--` — نهاية الرايات. `file` تفهمه (قيس على هذا الجهاز)، واحتياطٌ لملفٍّ
//!   اسمه يبدأ بشرطة.
//!
//! ## ملفٌ ومجلدٌ سواء
//!
//! `file` تحدّد نوع كليهما: لملفٍّ تطبع وصف صيغته (`ASCII text`، `JPEG image
//! data`، …)، ولمجلدٍ تطبع ببساطة `directory`. المدخل الواحد هنا
//! `InputKind::ExistingPath` لا `ExistingFile` تحديدًا: لا سبب لإجبار المستخدم
//! على معرفة نوع الهدف قبل أن يسأل عن نوعه.
//!
//! ## لا حكم في رمز الخروج
//!
//! خلافًا لـ`security.codesign` و`security.gatekeeper` — حيث الرمز غير
//! الصفري حكمٌ مكتمل («غير موقَّع»، «مرفوض») يترجَمه عقد النتائج — `file` لا
//! تحمل رمز خروجها حكمًا من هذا النوع؛ خروجها غير الصفري فشلٌ حقيقي في
//! التنفيذ (هدفٌ لا يُقرأ مثلًا)، لا جوابًا آخر مكتملًا. القراءة أبسط هنا لأنها
//! لا تسأل عن قرار سياسةٍ بل عن حقيقةٍ في الملف نفسه.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "files.identify";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.identify.title",
    description_key: "op.files.identify.description",
    category: Category::Files,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::FILE,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("target", InputKind::ExistingPath)],
    sort_order: 100,
    search_terms: &["file", "identify", "type", "magic", "نوع", "تعرّف", "تحديد", "ملفّ", "صيغة"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let target = inputs.any_path("target")?;

    let mut argv = Argv::tool(tools::FILE, "explain.file.tool")
        .flag("-b", "explain.file.brief")
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
        let found = listed.iter().find(|o| o.id == ID).expect("files.identify must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_last_argument_is_the_target() {
        let s = Scratch::new("identify-argv").unwrap();
        let file = s.file("مستند", b"data");

        let cmd = plan_with(&file).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/file"));
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "-b");
        assert_eq!(args[1], "--");
        assert_eq!(Path::new(&args[2]), file.as_path());

        assert!(cmd.artifact.is_none());
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.cwd.is_none(), "every path is absolute, so nothing resolves against a cwd");
    }

    #[test]
    fn a_directory_target_is_accepted_just_like_a_file() {
        let s = Scratch::new("identify-dir").unwrap();
        let d = s.dir("مجلد");

        let cmd = plan_with(&d).unwrap();
        let args = args_of(&cmd);

        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "-b");
        assert_eq!(args[1], "--");
        assert_eq!(Path::new(&args[2]), d.as_path());
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("identify-explain").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_flag_in_the_command_carries_its_own_explanation() {
        let s = Scratch::new("identify-keys").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role == TokenRole::Flag) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    /// لا تحذير خاصّ بهذه العملية؛ التحذير الوحيد الممكن هو الرابط الرمزي
    /// المحلول، وله اختبارٌ منفصل أدناه.
    #[test]
    fn plain_targets_carry_no_warning() {
        let s = Scratch::new("identify-warn").unwrap();

        for name in ["مستند.txt", "أرشيف.zip", "-f"] {
            let f = s.file(name, b"x");
            let cmd = plan_with(&f).unwrap();
            assert!(cmd.warnings.is_empty(), "{name:?}: {:?}", cmd.warnings);
        }

        let d = s.dir("مجلد");
        assert!(plan_with(&d).unwrap().warnings.is_empty());
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("identify-dashes").unwrap();
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
        let s = Scratch::new("identify-shellish").unwrap();

        for name in ["ملف 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let target = s.file(name, b"x");
            let cmd = plan_with(&target).unwrap();
            assert_eq!(cmd.args.len(), 3, "{name:?} must not add arguments");
            assert_eq!(Path::new(cmd.args.last().unwrap()), target.as_path());
        }
    }

    #[test]
    fn a_symlinked_target_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("identify-symlink").unwrap();
        let real = s.file("الحقيقي", b"x");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(cmd.args.last().unwrap()), real.as_path());
        assert!(cmd.warnings.contains(&"warn.target.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn a_target_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("identify-missing").unwrap();
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
        let s = Scratch::new("identify-dotdot").unwrap();
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
        let s = Scratch::new("identify-clean").unwrap();
        let f = s.file("ملف", b"x");

        for _ in 0..10 {
            plan_with(&f).unwrap();
        }

        assert_eq!(s.names(s.path()), vec!["ملف".to_owned()], "planning must create nothing");
        assert_eq!(std::fs::read(&f).unwrap(), b"x", "and must not touch the target");
    }
}
