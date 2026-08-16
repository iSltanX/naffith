//! قراءة السمات الممتدّة لملف أو مجلد، باستخدام `xattr`.
//!
//! ## ما السمة الممتدّة أصلًا
//!
//! بيانات مرفقة بالملف **خارج محتواه**: مفتاحٌ نصّي وقيمةٌ بايتات. لا يعرضها
//! Finder، ولا يعرضها `ls` بلا `-@`، ولا تظهر في حجم الملف — لكنها تنتقل معه
//! حين يُنسخ (وهي أحد أسباب اختيار `ditto` في عمليات النسخ في هذا المنتج).
//!
//! واثنتان منها يقابلهما المستخدم فعلًا:
//!
//! * `com.apple.quarantine` — ما يضعه المتصفّح أو البريد على كل ما يُنزَّل.
//!   وهي سبب سؤال «هذا الملف نُزّل من الإنترنت، أتريد فتحه؟»، وسبب رفض
//!   Gatekeeper لتطبيقٍ لم يُفحص بعد. حذفها هو ما تفعله النصائح المتداولة
//!   على الشبكة — وهذه العملية لا تحذفها.
//! * `com.apple.metadata:kMDItemWhereFroms` — العنوان الذي جاء منه الملف.
//!   يبقى ملتصقًا به بعد أن يُنسخ ويُرسل، فمن شارك ملفًا نزّله من موضعٍ لا يريد
//!   ذكره فقد شارك ذلك الموضع معه. هذه العملية توجد كي يُرى ذلك قبل الإرسال.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/xattr -l -- <الهدف>
//! ```
//!
//! * `-l` — اسرد كل سمةٍ باسمها **وقيمتها**. وهذا الفرق بينها وبين `ls -@` في
//!   عملية `security.permissions`: تلك تعطي الأسماء والأحجام، وهذه تعطي ما
//!   بداخلها.
//! * `--` — نهاية الرايات. `xattr` على macOS تفهمه (قيس على هذا الجهاز على
//!   ملفٍ اسمه `-d`).
//!
//! ## هذه العملية تقرأ ولا تكتب — وليس ذلك وعدًا
//!
//! `xattr` تستطيع الحذف (`-d` لسمةٍ بعينها، `-c` لمسحها كلها) والكتابة (`-w`).
//! أيٌّ منها على ملفٍ موقَّع يكسر توقيعه، وحذف `com.apple.quarantine` يعطّل
//! فحصًا وُضع لسبب.
//!
//! وامتناع هذا التطبيق عنها ليس سياسةً تُنقض بتعديل: `argv` تُبنى هنا من قائمةٍ
//! ثابتة في شيفرة Rust مترجَمة، ولا يوجد في مدخلات هذه العملية — مسارٌ واحد
//! من نوع `ExistingPath` — ما يمكن أن يصير راية. `RawValue` لا تملك صيغةً
//! تعبّر عن «راية للأداة» أصلًا (انظر `value.rs`). واختبارٌ أدناه يُنشئ ملفات
//! أسماؤها `-d` و`-c` و`-w` ويثبت أن الأمر يبقى ثلاثة وسائط.
//!
//! ## حدّان يُقالان بدل أن يُكتشفا
//!
//! 1. **لا `-r`**: مجلدٌ يعرض سماته هو، لا سمات ما بداخله. من يفحص مجلد
//!    تنزيلات بحثًا عن ملفٍ محجور لن يجده هنا.
//! 2. **القيم الثنائية تظهر خامًا**: `kMDItemWhereFroms` قائمة خصائص ثنائية
//!    (‏`bplist00…`)، فيظهر منها ما يقرؤه الإنسان مبعثرًا بين بايتات لا تُقرأ.
//!    المعلومة موجودة، لكن شكلها ليس جدولًا.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "security.xattr";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.security.xattr.title",
    description_key: "op.security.xattr.description",
    category: Category::Security,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::XATTR,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("target", InputKind::ExistingPath)],
    sort_order: 20,
    search_terms: &[
        "xattr",
        "سمات",
        "attributes",
        "quarantine",
        "حجر",
        "metadata",
        "بيانات وصفية",
        "مصدر",
        "wherefroms",
        "تنزيل",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let target = inputs.any_path("target")?;

    // راية واحدة، وهي راية قراءة. لا فرع في هذه الدالة يضيف غيرها.
    let mut argv = Argv::tool(tools::XATTR, "explain.xattr.tool")
        .flag("-l", "explain.xattr.list")
        .end_of_flags()
        .path(target);

    // `xattr` تتبع الروابط الرمزية ما لم تُعطَ `-s`، فالموضع المقروء هو
    // الموضع المحلول في الحالين. والتحذير مع ذلك يُرفع: أن يتطابق ما يفعله
    // النظام مع ما نفعله لا يعني أن يبقى المستخدم غير عالمٍ بأيّهما قُرئ.
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
        let found = listed.iter().find(|o| o.id == ID).expect("security.xattr must be listed");
        assert_eq!(found.category, Category::Security);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_last_argument_is_the_target() {
        let s = Scratch::new("xattr-argv").unwrap();
        let file = s.file("مستند.txt", b"data");

        let cmd = plan_with(&file).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/xattr"));
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "-l");
        assert_eq!(args[1], "--");
        assert_eq!(Path::new(&args[2]), file.as_path());

        assert!(cmd.artifact.is_none());
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.cwd.is_none(), "every path is absolute, so nothing resolves against a cwd");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("xattr-explain").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_flag_in_the_command_carries_its_own_explanation() {
        let s = Scratch::new("xattr-keys").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role == TokenRole::Flag) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    /// الادّعاء المكتوب في الوثيقة أعلاه، مُختبَرًا لا موعودًا.
    ///
    /// أسماء الملفات هنا هي بالضبط رايات `xattr` المدمّرة. لو كان في الشيفرة
    /// طريقٌ واحد يحوّل نصًّا من المستخدم إلى راية لسقط هذا الاختبار — والأمر
    /// عندها كان سيمسح سمات ملفٍ بدل أن يقرأها.
    #[test]
    fn no_deleting_or_writing_flag_can_ever_enter_this_command() {
        let s = Scratch::new("xattr-readonly").unwrap();

        for name in ["-d", "-c", "-w", "-r", "-s"] {
            let target = s.file(name, b"x");
            let cmd = plan_with(&target).unwrap();
            let args = args_of(&cmd);

            assert_eq!(args.len(), 3, "{name:?} added an argument");
            assert_eq!(args[0], "-l", "the only flag must stay the reading flag");
            assert_eq!(args[1], "--");
            assert_eq!(Path::new(&args[2]), target.as_path());
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("xattr-dashes").unwrap();
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
        let s = Scratch::new("xattr-shellish").unwrap();

        for name in ["ملف 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let target = s.file(name, b"x");
            let cmd = plan_with(&target).unwrap();
            assert_eq!(cmd.args.len(), 3, "{name:?} must not add arguments");
            assert_eq!(Path::new(cmd.args.last().unwrap()), target.as_path());
        }
    }

    #[test]
    fn a_symlinked_target_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("xattr-symlink").unwrap();
        let real = s.file("الحقيقي", b"x");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(cmd.args.last().unwrap()), real.as_path());
        assert!(cmd.warnings.contains(&"warn.target.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_target_raises_no_warning_at_all() {
        let s = Scratch::new("xattr-quiet").unwrap();
        let f = s.file("ملف", b"x");
        assert_eq!(plan_with(&f).unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn a_target_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("xattr-missing").unwrap();
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
        let s = Scratch::new("xattr-dotdot").unwrap();
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
        let s = Scratch::new("xattr-clean").unwrap();
        let f = s.file("ملف", b"x");

        for _ in 0..10 {
            plan_with(&f).unwrap();
        }

        assert_eq!(s.names(s.path()), vec!["ملف".to_owned()], "planning must create nothing");
        assert_eq!(std::fs::read(&f).unwrap(), b"x", "and must not touch the target");
    }
}
