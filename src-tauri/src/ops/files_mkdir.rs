//! إنشاء مجلد جديد داخل مجلدٍ قائم، باستخدام `mkdir`.
//!
//! ## الصيغة
//!
//! ```text
//! /bin/mkdir -- <المسار النهائي>
//! ```
//!
//! ## لماذا **بلا** `-p`
//!
//! `-p` تفعل شيئين، وكلاهما خطأ هنا:
//!
//! 1. **تُنشئ الآباء الناقصين.** ولا آباء ناقصين أصلًا: `parent` من نوع
//!    `TargetDir`، أي مجلدٌ قائم مُتحقَّق منه وقابل للكتابة فعلًا لا اسميًا،
//!    وقد مرّ بسياسة المسارات كاملة. و`-p` كانت ستجعل الأداة تُنشئ في طريقها
//!    مجلداتٍ لم تمرّ بشيء من ذلك.
//! 2. **تنجح صامتةً إن كان المجلد موجودًا.** وهذا بالضبط التضارب الذي تعلن
//!    هذه العملية أنها ترفضه. مع `-p` يخرج الأمر بصفر، وتقول الشاشة «تمّ»،
//!    ولا يعرف المستخدم أنّ ما يراه مجلدٌ قديم بمحتوياته لا مجلدٌ جديد فارغ.
//!
//! فبلا `-p` تصير `mkdir` نفسها الحارسَ الثاني: هي `mkdir(2)` مباشرة، ذرّيّة
//! على مستوى النواة، تفشل بـ`EEXIST` إن سبقنا أحدٌ إلى الاسم بين لحظة الفحص
//! ولحظة التنفيذ. والحارس الأول فحصُنا في `plan` — وهو ما يعطي المستخدم رسالةً
//! تُصلح، منسوبةً إلى الحقل الذي يصلحها، **قبل** أن يُعرض عليه أمرٌ ينفّذه.
//!
//! ## ولماذا بلا ناتج مؤقّت
//!
//! العمليات التي تبني شيئًا تكتبه باسمٍ عابر ثم تُرقّيه، لأن انقطاعها يترك
//! نصفَ ناتج. و`mkdir` لا تترك نصف مجلد: إمّا وُجدت المدخلة في الدليل وإمّا
//! لم توجد. مؤقّتٌ هنا كان طقسًا بلا معنًى — و`reveal` على المجلد الأب هو ما
//! يجيب «أين أنظر؟» بدلًا منه.

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;

pub const ID: &str = "files.mkdir";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.mkdir.title",
    description_key: "op.files.mkdir.description",
    category: Category::Files,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::MKDIR,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("parent", InputKind::TargetDir),
        InputSpec::new("folder_name", InputKind::NewDirName),
    ],
    sort_order: 30,
    search_terms: &["mkdir", "مجلد", "folder", "directory", "إنشاء مجلد", "new folder", "دليل"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let parent = inputs.target_dir("parent")?;
    let name = inputs.name("folder_name")?;

    let final_path = paths::new_dir_in(parent, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "folder_name" } else { "parent" };
        e.on_input(field)
    })?;

    // `symlink_metadata` لا `metadata`: رابطٌ رمزي معلَّق باسم المجلد المطلوب
    // اسمٌ مشغول، و`mkdir` كانت ستفشل عليه برسالةٍ من طبقة النظام بدل رسالةٍ
    // منسوبة إلى الحقل الذي يصلحها.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("folder_name"));
    }

    let mut argv = Argv::tool(tools::MKDIR, "explain.mkdir.tool")
        // `mkdir` تفهم `--`، وقد جُرّب. المسار النهائي مطلق فيبدأ بـ`/` ولا
        // يمكن أن يُقرأ راية؛ الفاصل حزام أمانٍ فوق ذلك لا بديلٌ عنه.
        .end_of_flags()
        .explained_path(&final_path, "explain.role.new_dir")
        // لا `Artifact` يجيب «أين أنظر؟»، فيُعلَن الجواب صراحة.
        .reveal(parent);

    if let Some(key) = warn_if_resolved(inputs, "parent", parent, "warn.destination.resolved") {
        argv = argv.warn(key);
    }

    // «بلا ناتجٍ مؤقّت يُرقّى» لا «بلا أثر»: الأثر معلَنٌ في `Danger::Creates`.
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

    fn raw(parent: &Path, name: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("parent".to_owned(), RawValue::Path(parent.display().to_string())),
            ("folder_name".to_owned(), RawValue::Text(name.to_owned())),
        ])
    }

    fn plan_with(parent: &Path, name: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(parent, name))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("files.mkdir must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_carries_no_dash_p() {
        let s = Scratch::new("mkdir-argv").unwrap();
        let parent = s.dir("الأب");

        let cmd = plan_with(&parent, "مجلد جديد").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/bin/mkdir"));
        assert_eq!(args[0], "--");
        assert_eq!(Path::new(&args[1]), parent.join("مجلد جديد"));
        assert_eq!(args.len(), 2);
        assert!(!args.iter().any(|a| a == "-p"), "-p would succeed silently on an existing folder");

        assert!(cmd.artifact.is_none(), "mkdir is atomic in itself; there is nothing to promote");
        assert_eq!(cmd.reveal_target.as_deref(), Some(parent.as_path()));
        assert!(!parent.join("مجلد جديد").exists(), "planning must create nothing");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("mkdir-explain").unwrap();
        let parent = s.dir("أ");

        let cmd = plan_with(&parent, "ج").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("mkdir-dashes").unwrap();
        let parent = s.dir("-rf");

        let cmd = plan_with(&parent, "-x").unwrap();
        for a in cmd.args.iter().skip(1) {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn a_taken_name_stops_the_plan_and_blames_the_field_that_fixes_it() {
        let s = Scratch::new("mkdir-exists").unwrap();
        let parent = s.dir("أ");
        s.dir("أ/موجود");

        assert_eq!(refusal(plan_with(&parent, "موجود")), ("err.dest.exists", Some("folder_name")));
    }

    #[test]
    fn a_file_holding_the_name_is_a_conflict_too_not_only_a_folder() {
        let s = Scratch::new("mkdir-file").unwrap();
        let parent = s.dir("أ");
        std::fs::write(parent.join("مأخوذ"), b"PRECIOUS").unwrap();

        assert_eq!(refusal(plan_with(&parent, "مأخوذ")), ("err.dest.exists", Some("folder_name")));
        assert_eq!(std::fs::read(parent.join("مأخوذ")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn a_dangling_symlink_at_the_name_still_counts_as_taken() {
        let s = Scratch::new("mkdir-dangling").unwrap();
        let parent = s.dir("أ");
        std::os::unix::fs::symlink(s.path().join("لا-وجود-له"), parent.join("رابط")).unwrap();

        assert_eq!(refusal(plan_with(&parent, "رابط")), ("err.dest.exists", Some("folder_name")));
    }

    #[test]
    fn a_name_that_would_break_the_path_is_blamed_on_the_name_not_the_parent() {
        let s = Scratch::new("mkdir-badname").unwrap();
        let parent = s.dir("أ");

        // ملاحظة: `sanitize_name` تقصّ الفراغ الطرفيّ أولًا، فسطرٌ جديد في
        // الذيل يختفي بالقصّ ولا يصل فحص محارف التحكّم. الحالة التي تختبره
        // فعلًا هي محرف تحكّم في وسط الاسم.
        for bad in ["", "   ", "أ/ب", ".مخفي", "..", "ا\nسم", "اسم."] {
            assert_eq!(
                refusal(plan_with(&parent, bad)),
                ("err.name.invalid", Some("folder_name")),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("mkdir-shellish").unwrap();
        let parent = s.dir("أ");

        for name in ["مجلد 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let cmd = plan_with(&parent, name).unwrap();
            assert_eq!(Path::new(&cmd.args[1]), parent.join(name));
            assert_eq!(cmd.args.len(), 2, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_symlinked_parent_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("mkdir-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "ج").unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.join("ج"));
        assert!(cmd.warnings.contains(&"warn.destination.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_the_parent_untouched() {
        let s = Scratch::new("mkdir-clean").unwrap();
        let parent = s.dir("أ");

        for _ in 0..10 {
            plan_with(&parent, "ج").unwrap();
        }
        assert_eq!(s.names(&parent), Vec::<String>::new(), "planning must create nothing");
    }
}
