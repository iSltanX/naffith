//! قراءة صلاحيات ملف أو مجلد كاملةً، باستخدام `ls`.
//!
//! ## لماذا `ls` لا `stat`
//!
//! `stat` تطبع بتات الوضع والمالك والمجموعة، ولا تعرف شيئًا عن قائمة التحكّم
//! بالوصول ولا عن السمات الممتدّة. ومن يسأل «لماذا لا أستطيع الكتابة في هذا
//! الملف وهو `rw-`؟» فجوابه في إحدى الطبقتين اللتين لا تراهما `stat` تحديدًا.
//! أمّا `ls` فتجمع الطبقات الثلاث في مخرجٍ واحد برايتين إضافيتين، فلا تحتاج
//! الشاشة ثلاثة أوامر ولا يحتاج المستخدم أن يجمع بينها بنفسه.
//!
//! ## الصيغة
//!
//! ```text
//! /bin/ls -l -e -@ -d -- <الهدف>
//! ```
//!
//! * `-l` — السطر الطويل: النوع، ووضع الصلاحيات، وعدد الروابط، والمالك
//!   والمجموعة، والحجم، وتاريخ آخر تعديل.
//! * `-e` — قائمة التحكّم بالوصول (ACL). هذه هي الطبقة التي لا يعرضها `chmod`
//!   ولا نافذة «عرض المعلومات» في Finder، وقاعدةٌ واحدة فيها تكفي لمنع الكتابة
//!   بينما يقول الوضع إنها مسموحة.
//! * `-@` — أسماء السمات الممتدّة وأحجامها بالبايت. الأسماء وحدها لا القيم:
//!   القيم في عملية `security.xattr`، والفصل مقصود لأن قيمة سمةٍ واحدة قد
//!   تكون كيلوبايتات من بيانات ثنائية لا مكان لها في سطر صلاحيات.
//! * `-d` — صِف العنصر نفسه لا محتواه. بدونها يسرد الأمر *ما داخل* المجلد،
//!   فيقرأ من سأل عن صلاحيات مجلده صلاحيات أبنائه. وهو خطأٌ صامت: المخرج يبدو
//!   معقولًا تمامًا، ولا شيء فيه يقول إنه يجيب عن سؤالٍ آخر.
//! * `--` — نهاية الرايات. `ls` على macOS تفهمه (قيس على هذا الجهاز باسمٍ يبدأ
//!   بشرطة)، والمسارات هنا مطلقة أصلًا فهو حزام أمانٍ فوق حزام.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تكتب شيئًا ولا تُنتج ملفًا: `Danger::Safe` و`Conflict::NoArtifact`
//! معلَنان في المواصفة، و`read_only()` هي التي تُنهي بناء الأمر — فلا سبيل من
//! هذا الملف إلى إعلان ناتج.
//!
//! ولا تفحص رابطًا رمزيًا بوصفه رابطًا: كل مسارٍ يصل إلى هنا مرّ بـ`paths.rs`
//! فصار محلولًا. أي أن سهم `->` الذي يعرضه `-l` لن يظهر في مخرج هذه العملية
//! أبدًا، وأن ما يُقرأ هو صلاحيات ما يشير إليه الرابط لا صلاحيات الرابط نفسه.
//! التحذير `warn.target.resolved` يقول ذلك حين يختلف ما اخترته عمّا سيُنفَّذ
//! عليه، بدل أن يُحلّ الرابط صامتًا.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "security.permissions";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.security.permissions.title",
    description_key: "op.security.permissions.description",
    category: Category::Security,
    // قراءةٌ محضة: لا تكتب بايتًا واحدًا في أي موضع.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::LS,
    // لا ناتج، فلا اسم نهائي يتضارب. اختبارٌ في `registry.rs` يربط هذا بـ`Safe`.
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("target", InputKind::ExistingPath)],
    sort_order: 10,
    search_terms: &[
        "ls",
        "صلاحيات",
        "permissions",
        "أذونات",
        "acl",
        "chmod",
        "مالك",
        "owner",
        "ملكية",
        "وصول",
    ],
    plan,
};

/// المجلد الذي يعيش فيه الهدف، وهو جواب «أين أنظر؟» بعد تشغيلٍ لا يُنتج ملفًا.
///
/// المجلد الحاوي لا الهدف نفسه: العنصر المفحوص قد يكون ملفًا وقد يكون مجلدًا،
/// والمجلد الحاوي هو الموضع الذي يقارن فيه المستخدم ما قرأه بجيران العنصر —
/// وهو جوابٌ واحد يصحّ في الحالين بدل جوابين يتبدّلان بنوع الهدف.
///
/// و`unwrap_or` ليست حالةً واقعية: `paths.rs` لا يمرّر إلا ما كان تحت المنزل
/// أو تحت `/Volumes`، فلكل هدفٍ أبٌ. لكنها لا تكلّف شيئًا ولا تكذب — ولا تجعل
/// جذر النظام يصل إلى `reveal` بوصفه أبًا لنفسه إلا في حالةٍ لا تقع.
fn containing_folder(target: &Path) -> &Path {
    target.parent().unwrap_or(target)
}

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let target = inputs.any_path("target")?;

    let mut argv = Argv::tool(tools::LS, "explain.ls.tool")
        .flag("-l", "explain.ls.long")
        .flag("-e", "explain.ls.acl")
        .flag("-@", "explain.ls.xattr")
        .flag("-d", "explain.ls.self")
        // بعد هذا الفاصل لا تُقرأ رايةٌ مهما بدا الاسم. المسار مطلق فلا يمكن
        // أن يبدأ بشرطة أصلًا، لكن الحارس لا يُترك للاستنتاج.
        .end_of_flags()
        .path(target)
        .reveal(containing_folder(target));

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
        let found =
            listed.iter().find(|o| o.id == ID).expect("security.permissions must be listed");
        assert_eq!(found.category, Category::Security);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_last_argument_is_the_target() {
        let s = Scratch::new("perm-argv").unwrap();
        let file = s.file("مستند.txt", b"data");

        let cmd = plan_with(&file).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/bin/ls"));
        assert_eq!(args.len(), 6);
        assert_eq!(args[0], "-l");
        assert_eq!(args[1], "-e");
        assert_eq!(args[2], "-@");
        assert_eq!(args[3], "-d");
        assert_eq!(args[4], "--");
        assert_eq!(Path::new(&args[5]), file.as_path());

        // قراءةٌ محضة: لا ناتج مؤقّت ولا توجيه للخرج.
        assert!(cmd.artifact.is_none());
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.cwd.is_none(), "every path is absolute, so nothing resolves against a cwd");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("perm-explain").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_flag_in_the_command_carries_its_own_explanation() {
        // أطروحة المنتج في سطر: رايةٌ بلا شرح تعني أن الشاشة تعرض رمزًا لا
        // يستطيع المستخدم أن يعرف ماذا يفعل بملفه.
        let s = Scratch::new("perm-keys").unwrap();
        let f = s.file("م", b"x");

        let cmd = plan_with(&f).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role == TokenRole::Flag) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn reveal_opens_the_folder_that_holds_the_target() {
        let s = Scratch::new("perm-reveal").unwrap();
        let f = s.file("ملف", b"x");
        assert_eq!(plan_with(&f).unwrap().reveal_target.as_deref(), Some(s.path()));

        // والجواب واحدٌ سواء كان الهدف ملفًا أو مجلدًا.
        let d = s.dir("مجلد");
        assert_eq!(plan_with(&d).unwrap().reveal_target.as_deref(), Some(s.path()));
    }

    #[test]
    fn a_folder_target_is_described_rather_than_listed() {
        // بلا `-d` كان الأمر يسرد الأبناء، فيقرأ من سأل عن مجلده صلاحيات
        // غيره — والمخرج يبدو معقولًا فلا شيء ينبّهه.
        let s = Scratch::new("perm-dir").unwrap();
        let dir = s.dir("مجلد");
        std::fs::write(dir.join("ابن"), b"x").unwrap();

        let args = args_of(&plan_with(&dir).unwrap());
        assert!(args.contains(&"-d".to_owned()), "{args:?}");
        assert_eq!(Path::new(&args[5]), dir.as_path());
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("perm-dashes").unwrap();
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
        let s = Scratch::new("perm-shellish").unwrap();

        for name in ["ملف 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let target = s.file(name, b"x");
            let cmd = plan_with(&target).unwrap();
            assert_eq!(cmd.args.len(), 6, "{name:?} must not add arguments");
            assert_eq!(Path::new(cmd.args.last().unwrap()), target.as_path());
        }
    }

    #[test]
    fn a_symlinked_target_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("perm-symlink").unwrap();
        let real = s.file("الحقيقي", b"x");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(cmd.args.last().unwrap()), real.as_path());
        assert!(cmd.warnings.contains(&"warn.target.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_target_raises_no_warning_at_all() {
        let s = Scratch::new("perm-quiet").unwrap();
        let f = s.file("ملف", b"x");
        assert_eq!(plan_with(&f).unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn a_target_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("perm-missing").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"))),
            ("err.path.missing", Some("target"))
        );
    }

    #[test]
    fn a_target_outside_the_allowed_roots_is_refused() {
        // ‏`/etc` موجود ومقروء على كل جهاز؛ رفضه سياسةٌ لا حادث.
        assert_eq!(refusal(plan_with(Path::new("/etc"))), ("err.path.outside", Some("target")));
    }

    #[test]
    fn a_target_that_climbs_out_with_dotdot_is_refused_before_the_disk_is_touched() {
        let s = Scratch::new("perm-dotdot").unwrap();
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
        let s = Scratch::new("perm-clean").unwrap();
        let f = s.file("ملف", b"x");

        for _ in 0..10 {
            plan_with(&f).unwrap();
        }

        assert_eq!(s.names(s.path()), vec!["ملف".to_owned()], "planning must create nothing");
        assert_eq!(std::fs::read(&f).unwrap(), b"x", "and must not touch the target");
    }
}
