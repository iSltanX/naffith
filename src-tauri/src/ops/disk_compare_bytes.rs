//! مقارنة ملفين بايتًا بايت، باستخدام `cmp`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/cmp -- <الأيسر> <الأيمن>
//! ```
//!
//! بلا `-s`: الصيغة الصامتة تجيب برمز الخروج وحده، ولا تطبع شيئًا في أيّ من
//! الحالتين. الصيغة هنا تطبع حين تختلفان: `differ: char N, line N` — موضع
//! أوّل بايتٍ يفترقان عنده بعينه. نصٌّ مفيدٌ يُعرَض للمستخدم، لا يُخفى خلف رمز
//! خروجٍ صامت. وحين تتطابقان لا تطبع شيئًا، وهذا صحيحٌ أيضًا: لا فرقَ يُوصَف.
//!
//! ## ما تحلّه هذه العملية ممّا رُفض في `disk.compare.hash`، وما لا تحلّه
//!
//! رأس `disk_compare.rs` يوثّق رفضًا سابقًا لـ`cmp`/`diff -q` بحجّتين. هذه
//! العملية **قرارٌ لاحق** يعيد فتح تلك الحجّة الأولى وحدها، لا يتجاوزها
//! صامتًا:
//!
//! 1. رمزُ خروجٍ يقول «مختلفان» كان سيصل الشاشة **فشلًا أحمر** لا نتيجة —
//!    هذا كان الاعتراض حين لم يوجد في المنتج طريقٌ آخر لعرض رمز الخروج. صار
//!    اليوم موجودًا: `ExitSemantics::Diff`، نفس النمط الذي تستعمله `git.diff`
//!    و`text.diff` فعلًا، يترجم ‎0‎ إلى «متطابقان» و‎1‎ إلى «مختلفان» —
//!    جوابان ناجحان كلاهما، لا فشل. هذه الترجمة تقع في `result.rs` مركزيًا،
//!    وليست من عمل هذا الملف؛ هنا فقط الأمر يُبنى بلا `-s` كي يبقى النصّ
//!    المفيد (`differ: char N, line N`) حاضرًا حين يُحتاج.
//! 2. `cmp` تحتاج الملفين حاضرين معًا على القرص الآن، ولا تفيد من يقارن
//!    ببصمةٍ نُسخت من مكانٍ آخر — صفحة تنزيل، رسالة، جهازٍ ثانٍ. هذا الاعتراض
//!    **لم يُحلّ ولن يُحلّ هنا**؛ هذه العملية لا تدّعي حلّه. هي مكمّلة
//!    لـ`disk.compare.hash` لا بديلة عنها: مقارنةٌ سريعة لملفّين حاضرين معًا
//!    الآن، تتوقّف عند أول بايتٍ مختلف، بينما تبقى `disk.compare.hash`
//!    الخيار الوحيد لمن يقارن ببصمةٍ من مكانٍ آخر. من يملك الملفين معًا فقط
//!    يختار هذه؛ من يملك بصمةً منشورة يختار تلك.
//!
//! ## غيابٌ متعمَّد: لا تحذير عند اختلاف الحجمين
//!
//! `disk_compare.rs` يحذّر حين يختلف حجما الملفين، لأن `shasum` تقرأ الملفّ
//! كاملًا مهما اختلف الحجمان — فالانتظار طويلٌ حتى حين يكون الفرق واضحًا من
//! الحجم وحده. `cmp` لا تحتاج هذا التحذير: هي **تتوقّف عند أول بايتٍ مختلف**،
//! وحجمان مختلفان يعنيان اختلافًا يُكتشف فورًا من أوّل بايتٍ ينتهي عنده
//! الأقصر، لا انتظارًا طويلًا لمعلومةٍ كانت حاضرة. التحذير هناك يخدم مشكلةً
//! لا توجد هنا.
//!
//! ## ملفٌ يُقارن بنفسه
//!
//! مرفوض، لنفس السبب الموثَّق في `disk_compare.rs`: المسارات مُحلّة الروابط
//! قبل الوصول إلى هنا، فرابطان إلى ملفٍ واحد يصلان مسارًا واحدًا. تشغيلُ
//! الأمر عليهما كان سينجح بلا طباعة — جوابٌ صحيح تمامًا ولا يحمل معلومة. الرفض
//! يقع في التخطيط وينسب نفسه إلى الحقل الثاني.
//!
//! ## ما لا تراه هذه المقارنة
//!
//! `cmp` تقرأ محتوى الملفّ وحده. ما لا تراه: السمات الممتدّة، وresource
//! fork، ووسوم Finder، والأذونات. ملفان متطابقا البايتات ومختلفا الوسوم
//! نتيجتهما «متطابقان».

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "disk.compare.bytes";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.disk.compare.bytes.title",
    description_key: "op.disk.compare.bytes.description",
    category: Category::Disk,
    // تقرأ الملفين ولا تكتب شيئًا، ولا تُنتج ملفًا فلا اسم يتضارب.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::CMP,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("left", InputKind::ExistingFile),
        InputSpec::new("right", InputKind::ExistingFile),
    ],
    sort_order: 50,
    search_terms: &[
        "cmp",
        "bytes",
        "compare",
        "identical",
        "مقارنة",
        "بايت",
        "متطابق",
        "سريعة",
        "ملفّين",
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

    let mut argv = Argv::tool(tools::CMP, "explain.cmp.tool")
        .end_of_flags()
        .explained_path(left, "explain.role.compare_bytes_left")
        .explained_path(right, "explain.role.compare_bytes_right");

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
    warnings.extend(warn_if_resolved(inputs, "left", left, "warn.compare.bytes_left_resolved"));
    warnings.extend(warn_if_resolved(inputs, "right", right, "warn.compare.bytes_right_resolved"));
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::ffi::OsStr;

    const DECLARED_FLAGS: &[&str] = &["--"];

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
        let found = listed.iter().find(|o| o.id == ID).expect("disk.compare.bytes must be listed");
        assert_eq!(found.category, Category::Disk);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_keeps_the_order_the_user_chose() {
        let s = Scratch::new("cmpb-argv").unwrap();
        let left = s.file("الأول.bin", b"aaaa");
        let right = s.file("الثاني.bin", b"bbbb");

        let cmd = plan_with(&left, &right).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/cmp"));
        assert_eq!(args[0], "--");
        assert_eq!(Path::new(&args[1]), left.as_path());
        assert_eq!(Path::new(&args[2]), right.as_path());
        assert_eq!(args.len(), 3);
        assert!(cmd.artifact.is_none(), "comparing produces no file");
        assert!(cmd.stdout_to.is_none(), "the differ line is printed, not written to disk");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("cmpb-explain").unwrap();
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
        let s = Scratch::new("cmpb-same").unwrap();
        let file = s.file("واحد.bin", b"data");
        assert_eq!(refusal(plan_with(&file, &file)), ("err.path.same", Some("right")));
    }

    #[test]
    fn a_symlink_to_the_other_file_is_the_same_file_and_is_refused_too() {
        // الحالة التي تجعل المقارنة النصّية غير كافية: اسمان مختلفان تمامًا
        // لموضعٍ واحد. حلّ الروابط قبل المقارنة هو ما يلتقطها.
        let s = Scratch::new("cmpb-same-link").unwrap();
        let real = s.file("الحقيقي.bin", b"data");
        let link = s.path().join("رابط.bin");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        assert_eq!(refusal(plan_with(&real, &link)), ("err.path.same", Some("right")));
    }

    #[test]
    fn a_missing_or_unusable_side_is_refused_and_names_its_own_field() {
        let s = Scratch::new("cmpb-refusals").unwrap();
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
        let s = Scratch::new("cmpb-dashes").unwrap();
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
        let s = Scratch::new("cmpb-shellish").unwrap();
        let plain = s.file("عادي.bin", b"x");
        for name in ["a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "ملف 'اليوم'", "x|y"]
        {
            let odd = s.file(name, b"y");
            let cmd = plan_with(&odd, &plain).unwrap();
            assert_eq!(cmd.args.len(), 3, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), odd.as_path());
        }
    }

    #[test]
    fn two_files_in_one_folder_give_reveal_a_single_answer() {
        let s = Scratch::new("cmpb-reveal-one").unwrap();
        let left = s.file("معًا/أ.bin", b"a");
        let right = s.file("معًا/ب.bin", b"bb");
        let folder = s.path().join("معًا");

        let cmd = plan_with(&left, &right).unwrap();
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
    }

    #[test]
    fn two_files_in_two_folders_get_no_reveal_rather_than_an_arbitrary_one() {
        let s = Scratch::new("cmpb-reveal-none").unwrap();
        let left = s.file("هنا/أ.bin", b"a");
        let right = s.file("هناك/ب.bin", b"bb");

        let cmd = plan_with(&left, &right).unwrap();
        assert!(cmd.reveal_target.is_none(), "half an answer is not an answer");
    }

    #[test]
    fn a_difference_in_size_raises_no_warning_unlike_the_hash_comparison() {
        // بخلاف `disk.compare.hash`: `cmp` تتوقّف عند أول بايتٍ مختلف، فلا
        // انتظار طويلًا يستحقّ التحذير هنا.
        let s = Scratch::new("cmpb-sizes").unwrap();
        let left = s.file("أ.bin", b"aaaa");
        let right = s.file("ب.bin", b"aaaaaaaa");

        assert_eq!(plan_with(&left, &right).unwrap().warnings, Vec::<&'static str>::new());
    }

    #[test]
    fn each_side_announces_its_own_symlink_substitution() {
        let s = Scratch::new("cmpb-symlink").unwrap();
        let real = s.file("الحقيقي.bin", b"aaaa");
        let other = s.file("آخر.bin", b"bbbb");
        let link = s.path().join("رابط.bin");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, &other).unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.compare.bytes_left_resolved"), "{:?}", cmd.warnings);
        assert!(!cmd.warnings.contains(&"warn.compare.bytes_right_resolved"), "{:?}", cmd.warnings);

        let cmd = plan_with(&other, &link).unwrap();
        assert!(cmd.warnings.contains(&"warn.compare.bytes_right_resolved"), "{:?}", cmd.warnings);
        assert!(!cmd.warnings.contains(&"warn.compare.bytes_left_resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_writes_nothing_into_either_folder() {
        let s = Scratch::new("cmpb-clean").unwrap();
        let left = s.file("هنا/أ.bin", b"a");
        let right = s.file("هناك/ب.bin", b"bb");

        for _ in 0..10 {
            plan_with(&left, &right).unwrap();
        }
        assert_eq!(s.names(&s.path().join("هنا")), vec!["أ.bin".to_string()]);
        assert_eq!(s.names(&s.path().join("هناك")), vec!["ب.bin".to_string()]);
    }
}
