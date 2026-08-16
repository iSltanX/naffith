//! مقارنة ملفين ببصمتيهما، باستخدام `shasum`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/shasum -a 256 -- <الأول> <الثاني>
//! ```
//!
//! تطبع الأداة سطرين بترتيب الوسائط: بصمة، ثم مسار صاحبها. والمقارنة بينهما
//! يجريها **المستخدم** بعينه.
//!
//! ## لماذا لا تقول هذه العملية «متطابقان» أو «مختلفان»
//!
//! لأن قولها يقتضي أن يقرأ التطبيق ما طبعته الأداة ويفسّره. وهذا الشيء
//! بالذات لا يفعله هذا المنتج في أي موضع: النواة تبثّ السطور كما خرجت،
//! والشاشة تعرضها كما وصلت. لحظةَ يُضاف مفسّرٌ للخرج يصير في المنتج **مصدرا
//! حقيقةٍ اثنان** — ما طبعته الأداة، وما فهمه التطبيق منه — ولا شيء يضمن
//! توافقهما حين تغيّر الأداة صيغتها في تحديث نظام. عندها يقول التطبيق
//! «متطابقان» فوق سطرين يقولان غير ذلك، ولن يعرف المستخدم أيّهما يصدّق.
//!
//! والثمن معلَن ولا يُخفى: المستخدم يقارن سلسلتين من ٦٤ خانة ستّ عشرية بعينه.
//! أول أربع خانات وآخر أربع تكفي عمليًّا للفرق العابر، ومن أراد اليقين نسخ
//! السطرين. يُقال هذا في وصف العملية بدل أن يُكتشف أمام الشاشة.
//!
//! ## البديل الذي رُفض: `cmp` و`diff -q`
//!
//! `/usr/bin/cmp -s` تجيب بنعم/لا في **رمز خروجها**، والنواة تعلن رمز الخروج
//! أصلًا — أي أن الجواب كان سيصل بلا تفسير أي نصّ. ومع ذلك رُفض، لسببين:
//!
//! 1. رمزُ خروجٍ يقول «مختلفان» يصل الشاشة على أنه **فشل** لا نتيجة. عمليةٌ
//!    تنجح في عملها ثم تُعرض حمراء تُعلّم المستخدم أن يتجاهل لون الحالة.
//! 2. `cmp` تقارن ملفين حاضرين معًا، ولا تترك بيدك شيئًا. البصمة تُنسخ
//!    وتُرسل وتُقارن ببصمةٍ منشورة على صفحة تنزيل بعد شهر. هذا هو ما يريده
//!    عمليًّا من يسأل «هل الملفان واحد؟».
//!
//! وثمن رفضها صادق كذلك: `cmp` تتوقّف عند أول بايت مختلف، و`shasum` تقرأ
//! الملفين كاملين مهما اختلفا من البداية. مقارنةُ ملفين كبيرين هنا أبطأ.
//! ولذلك حين يختلف الحجمان — والاختلاف يعني حتمًا اختلاف البصمتين — يُعلَن
//! ذلك قبل التنفيذ في `warn.compare.size_differs`، فيقرّر المستخدم إن كان
//! الانتظار يستحقّ.
//!
//! ## ملفٌ يُقارن بنفسه
//!
//! مرفوض. المسارات مُحلّة الروابط قبل الوصول إلى هنا، فرابطان إلى ملفٍ واحد
//! يصلان مسارًا واحدًا. تشغيلُ الأمر عليهما كان سينجح ويطبع بصمتين متطابقتين
//! حتمًا — جوابٌ صحيح تمامًا ولا يحمل معلومة، وقد يُقرأ تأكيدًا لسؤالٍ لم
//! يُطرح. الرفض يقع في التخطيط وينسب نفسه إلى الحقل الثاني.
//!
//! ## ما لا تراه هذه المقارنة
//!
//! ما لا تراه البصمة: السمات الممتدّة، و resource fork، ووسوم Finder،
//! والأذونات. ملفان متطابقا البايتات ومختلفا الوسوم بصمتاهما واحدة.

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "disk.compare.hash";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.disk.compare.hash.title",
    description_key: "op.disk.compare.hash.description",
    category: Category::Disk,
    // تقرأ الملفين ولا تكتب شيئًا، ولا تُنتج ملفًا فلا اسم يتضارب.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::SHASUM,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("left", InputKind::ExistingFile),
        InputSpec::new("right", InputKind::ExistingFile),
    ],
    sort_order: 30,
    search_terms: &[
        "shasum",
        "sha256",
        "مقارنة",
        "مطابقة",
        "متطابق",
        "بصمة",
        "compare",
        "identical",
        "hash",
        "checksum",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let left = inputs.file("left")?;
    let right = inputs.file("right")?;

    // كلا المسارين محلول الروابط، فالمقارنة مقارنة مواضع لا نصوص: رابطٌ
    // واسمُ هدفه يصلان هنا مسارًا واحدًا.
    if left == right {
        return Err(CoreError::SamePath.on_input("right"));
    }

    let mut argv = Argv::tool(tools::SHASUM, "explain.shasum.tool")
        .flag("-a", "explain.shasum.algorithm")
        .explained_value("256", "explain.shasum.sha256")
        .end_of_flags()
        .explained_path(left, "explain.role.compare_left")
        .explained_path(right, "explain.role.compare_right");

    // «أين أنظر؟» له جوابٌ واحد فقط حين يسكن الملفان مجلدًا واحدًا. وحين لا
    // يسكنانه فالجوابان اثنان، واختيارُ أحدهما بالقرعة يفتح للمستخدم نافذةً
    // على نصف ما قارنه. لا جواب خيرٌ من جوابٍ نصفيّ.
    if let Some(shared) = shared_folder(left, right) {
        argv = argv.reveal(shared);
    }

    for key in warnings_for(inputs, left, right) {
        argv = argv.warn(key);
    }
    argv.read_only()
}

/// المجلد الذي يحوي الملفين، إن كان واحدًا.
fn shared_folder<'a>(left: &'a Path, right: &Path) -> Option<&'a Path> {
    let folder = left.parent()?;
    (folder == right.parent()?).then_some(folder)
}

fn warnings_for(inputs: &Inputs, left: &Path, right: &Path) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    warnings.extend(warn_if_resolved(inputs, "left", left, "warn.compare.left_resolved"));
    warnings.extend(warn_if_resolved(inputs, "right", right, "warn.compare.right_resolved"));

    // حجمان مختلفان يعنيان بصمتين مختلفتين حتمًا: لا حاجة إلى قراءة بايت
    // واحد كي يُعرف ذلك. لا يمنع التنفيذ — قد يريد المستخدم البصمتين نفسيهما
    // لا الجواب — لكن ألّا يُقال يعني انتظارًا طويلًا لمعلومةٍ كانت حاضرة.
    if let (Some(a), Some(b)) = (file_bytes(left), file_bytes(right)) {
        if a != b {
            warnings.push("warn.compare.size_differs");
        }
    }
    warnings
}

fn file_bytes(p: &Path) -> Option<u64> {
    std::fs::metadata(p).ok().map(|m| m.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::ffi::OsStr;

    const DECLARED_FLAGS: &[&str] = &["-a", "--"];

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

    fn data_args(cmd: &PlannedCommand) -> Vec<&OsStr> {
        let end =
            cmd.args.iter().position(|a| a.as_os_str() == "--").expect("`--` must be in the argv");
        cmd.args[end + 1..].iter().map(|a| a.as_os_str()).collect()
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("disk.compare.hash must be listed");
        assert_eq!(found.category, Category::Disk);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_keeps_the_order_the_user_chose() {
        let s = Scratch::new("cmp-argv").unwrap();
        let left = s.file("الأول.bin", b"aaaa");
        let right = s.file("الثاني.bin", b"bbbb");

        let cmd = plan_with(&left, &right).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/shasum"));
        assert_eq!(args[0], "-a");
        assert_eq!(args[1], "256");
        assert_eq!(args[2], "--");
        assert_eq!(Path::new(&args[3]), left.as_path(), "the first line is the first field");
        assert_eq!(Path::new(&args[4]), right.as_path());
        assert_eq!(args.len(), 5);
        assert!(cmd.artifact.is_none(), "comparing produces no file");
        assert!(cmd.stdout_to.is_none(), "the two hashes are printed, not written to disk");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("cmp-explain").unwrap();
        let left = s.file("أ", b"a");
        let right = s.file("ب", b"b");

        let cmd = plan_with(&left, &right).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn comparing_a_file_with_itself_is_refused_because_the_answer_is_known() {
        let s = Scratch::new("cmp-same").unwrap();
        let file = s.file("واحد.bin", b"data");
        assert_eq!(refusal(plan_with(&file, &file)), ("err.path.same", Some("right")));
    }

    #[test]
    fn a_symlink_to_the_other_file_is_the_same_file_and_is_refused_too() {
        // الحالة التي تجعل المقارنة النصّية غير كافية: اسمان مختلفان تمامًا
        // لموضعٍ واحد. حلّ الروابط قبل المقارنة هو ما يلتقطها.
        let s = Scratch::new("cmp-same-link").unwrap();
        let real = s.file("الحقيقي.bin", b"data");
        let link = s.path().join("رابط.bin");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        assert_eq!(refusal(plan_with(&real, &link)), ("err.path.same", Some("right")));
    }

    #[test]
    fn a_missing_or_unusable_side_is_refused_and_names_its_own_field() {
        let s = Scratch::new("cmp-refusals").unwrap();
        let file = s.file("قائم.bin", b"x");
        let dir = s.dir("مجلد");
        let ghost = s.path().join("لا-وجود-له");

        assert_eq!(refusal(plan_with(&ghost, &file)), ("err.path.missing", Some("left")));
        assert_eq!(refusal(plan_with(&file, &ghost)), ("err.path.missing", Some("right")));
        assert_eq!(refusal(plan_with(&dir, &file)), ("err.path.missing", Some("left")));
        assert_eq!(refusal(plan_with(&file, &dir)), ("err.path.missing", Some("right")));

        // حقلٌ ناقص: العمليتان مطلوبتان معًا، ولا يُبنى أمرٌ بجانبٍ واحد.
        let only_left =
            BTreeMap::from([("left".to_owned(), RawValue::Path(file.display().to_string()))]);
        let r = crate::value::validate(&SPEC, &only_left).map(|i| plan(&i).unwrap());
        assert_eq!(refusal(r), ("err.input.missing", Some("right")));
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("cmp-dashes").unwrap();
        let left = s.file("-a", b"x");
        let right = s.file("--check", b"y");

        let cmd = plan_with(&left, &right).unwrap();
        for a in data_args(&cmd) {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
        for a in cmd.args.iter().take_while(|a| a.as_os_str() != "--") {
            let text = a.to_string_lossy().into_owned();
            assert!(
                DECLARED_FLAGS.contains(&text.as_str()) || cannot_be_read_as_a_flag(a.as_os_str()),
                "{text:?} is neither a declared flag nor safe data"
            );
        }
    }

    #[test]
    fn shell_syntax_in_either_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("cmp-shellish").unwrap();
        let plain = s.file("عادي.bin", b"x");
        for name in ["a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "ملف 'اليوم'", "x|y"]
        {
            let odd = s.file(name, b"y");
            let cmd = plan_with(&odd, &plain).unwrap();
            assert_eq!(cmd.args.len(), 5, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[3]), odd.as_path());
        }
    }

    #[test]
    fn two_files_in_one_folder_give_reveal_a_single_answer() {
        let s = Scratch::new("cmp-reveal-one").unwrap();
        let left = s.file("معًا/أ.bin", b"a");
        let right = s.file("معًا/ب.bin", b"bb");
        let folder = s.path().join("معًا");

        let cmd = plan_with(&left, &right).unwrap();
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
    }

    #[test]
    fn two_files_in_two_folders_get_no_reveal_rather_than_an_arbitrary_one() {
        let s = Scratch::new("cmp-reveal-none").unwrap();
        let left = s.file("هنا/أ.bin", b"a");
        let right = s.file("هناك/ب.bin", b"bb");

        let cmd = plan_with(&left, &right).unwrap();
        assert!(cmd.reveal_target.is_none(), "half an answer is not an answer");
    }

    #[test]
    fn a_difference_in_size_is_announced_before_the_long_read_begins() {
        let s = Scratch::new("cmp-sizes").unwrap();
        let left = s.file("أ.bin", b"aaaa");
        let right = s.file("ب.bin", b"aaaaaaaa");

        let cmd = plan_with(&left, &right).unwrap();
        assert!(cmd.warnings.contains(&"warn.compare.size_differs"), "{:?}", cmd.warnings);
        assert_eq!(cmd.args.len(), 5, "a warning must not change the command");
    }

    #[test]
    fn two_files_of_one_size_raise_no_warning_even_when_their_bytes_differ() {
        let s = Scratch::new("cmp-quiet").unwrap();
        let left = s.file("أ.bin", b"aaaa");
        let right = s.file("ب.bin", b"bbbb");

        assert_eq!(plan_with(&left, &right).unwrap().warnings, Vec::<&'static str>::new());
    }

    #[test]
    fn each_side_announces_its_own_symlink_substitution() {
        let s = Scratch::new("cmp-symlink").unwrap();
        let real = s.file("الحقيقي.bin", b"aaaa");
        let other = s.file("آخر.bin", b"bbbb");
        let link = s.path().join("رابط.bin");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, &other).unwrap();
        assert_eq!(Path::new(&cmd.args[3]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.compare.left_resolved"), "{:?}", cmd.warnings);
        assert!(!cmd.warnings.contains(&"warn.compare.right_resolved"), "{:?}", cmd.warnings);

        let cmd = plan_with(&other, &link).unwrap();
        assert!(cmd.warnings.contains(&"warn.compare.right_resolved"), "{:?}", cmd.warnings);
        assert!(!cmd.warnings.contains(&"warn.compare.left_resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_writes_nothing_into_either_folder() {
        let s = Scratch::new("cmp-clean").unwrap();
        let left = s.file("هنا/أ.bin", b"a");
        let right = s.file("هناك/ب.bin", b"bb");

        for _ in 0..10 {
            plan_with(&left, &right).unwrap();
        }
        assert_eq!(s.names(&s.path().join("هنا")), vec!["أ.bin".to_string()]);
        assert_eq!(s.names(&s.path().join("هناك")), vec!["ب.bin".to_string()]);
    }
}
