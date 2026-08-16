//! تقسيم ملفٍ نصّي إلى أجزاءٍ متساوية الأسطر، باستخدام `split`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/split -l <الأسطر> <المصدر> <المؤقّت>/part-
//! ```
//!
//! الوسيط الأخير **بادئة مسار لا مجلد**: `split` تلحق بها لاحقةً من حرفين
//! (`aa`، `ab`، …) لكل جزء، فتصير الأجزاء `part-aa` و`part-ab` داخل المجلد
//! المؤقّت. وهو مسارٌ مطلق كغيره، فلا يمكن أن يُقرأ رايةً.
//!
//! ## لماذا تنزل الأجزاء في مجلدٍ نملكه لا في وجهة المستخدم
//!
//! ثلاثة أسباب، وكلٌّ منها وحده كافٍ:
//!
//! 1. **`split` تدهس.** يقول `man split` صراحةً: «افتراضًا تكتب `split` فوق
//!    أي ملفات ناتجة موجودة». فبادئةٌ داخل مجلد المستخدم تعني أن ملفًا اسمه
//!    `part-aa` عنده يُمحى بلا سؤال — وهذا المنتج لا يستبدل شيئًا أبدًا. أما
//!    المجلد المؤقّت فيُنشأ حصريًّا (`create_dir` في `plans::claim_temp`) قبل
//!    الإطلاق، ولا يمكن أن يحوي ملفًا لأحد.
//! 2. **الأجزاء ناتجٌ واحد.** الترقية الذرّية في `atomic.rs` تعرف اسمًا واحدًا
//!    تُرقّيه؛ وعشرون ملفًا تُنثر في الوجهة لا اسم لها تُرقّى إليه، فيصير
//!    «نجح أم لا» سؤالًا بلا جواب.
//! 3. **الفشل في المنتصف.** قيس على هذا الجهاز: ملفٌ من ٦٧٧ سطرًا مع
//!    `-l 1` أنتج ٦٧٦ جزءًا ثم توقّف بـ«‏split: too many files» ورمز خروج ٦٥.
//!    ولأن الأجزاء كانت في مجلدٍ نملكه، حذفها `ArtifactGuard::abort` كلها ولم
//!    ير المستخدم شيئًا. ولو نُثرت في وجهته لبقيت ٦٧٦ ملفًا باسمٍ اخترناه له
//!    بعد عمليةٍ أُعلنت فاشلة.
//!
//! ## سقف اللواحق: ٦٧٦ جزءًا
//!
//! اللاحقة الافتراضية حرفان من `a-z`، أي ٢٦×٢٦ تركيبة. ما زاد يوقف `split`
//! بخطأ. ولأن عدّ الأسطر يقتضي قراءة الملف كاملًا عند كل ضغطة مفتاح في
//! النموذج، نستعمل حدًّا أعلى لا قياسًا: كل سطرٍ بايتٌ واحد على الأقل (السطر
//! الفارغ هو `\n` نفسه)، فعدد الأسطر لا يزيد على حجم الملف بالبايتات. فإن
//! كان الحجم لا يبلغ ٦٧٦ ضعف الأسطر-لكل-جزء استحال التجاوز ولا تحذير؛ وإن
//! بلغه فالتجاوز **ممكن** لا مؤكَّد — وهو ما يقوله نصّ التحذير بلفظه.
//!
//! ## ملفٌ فارغ
//!
//! قيس: `split` على ملفٍ فارغ تخرج بصفر ولا تكتب جزءًا واحدًا. ولأن
//! `ArtifactGuard` تقيس «أُنتج شيء» بوجود مدخلةٍ في المجلد، ينتهي التشغيل
//! بفشلٍ رسالتُه عن المسار — رسالةٌ لا تدلّ على السبب. فيُقال قبل التنفيذ.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تحذف المصدر ولا تعدّله، ولا تعيد لصق الأجزاء (ذلك دمجٌ، وله عمليته).
//! والتقسيم بالحجم (`-b`) وبالنمط (`-p`) خيارٌ آخر مسجَّل في خارطة الطريق:
//! ثلاثة أنماط في شاشةٍ واحدة تحتاج حقولًا يتغيّر معناها بتغيّر الاختيار،
//! وهو ما لا يرسمه النموذج اليوم.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;
use std::path::Path;

pub const ID: &str = "text.split";

/// بادئة أسماء الأجزاء. معلَنة هنا لا مكتوبة في موضعين: الاختبار يبني منها
/// المسار المتوقّع، فتغييرها يظلّ تغييرًا في موضعٍ واحد.
const PART_PREFIX: &str = "part-";

/// أقصى عدد أجزاءٍ تسعه لواحق `split` الافتراضية: حرفان من `a-z`.
const MAX_PARTS: u64 = 26 * 26;

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.text.split.title",
    description_key: "op.text.split.description",
    category: Category::Text,
    // تُنشئ مجلدًا جديدًا في الوجهة ولا تمسّ المصدر.
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::SPLIT,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("source", InputKind::ExistingFile),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("folder_name", InputKind::NewDirName),
        // الحدّ الأعلى ليس زينة: عددٌ أكبر منه يعني جزءًا واحدًا عمليًا، وهو
        // ما تفعله «لا تقسّم» لا «قسّم». والافتراضي ١٠٠٠ هو افتراضي `split`
        // نفسها، فمن ترك الحقل كما هو حصل على سلوك الأداة المعروف.
        InputSpec::new(
            "lines_per_part",
            InputKind::Number { min: 1, max: 10_000_000, default: 1000 },
        ),
    ],
    sort_order: 20,
    search_terms: &["split", "تقسيم", "تجزئة", "أجزاء", "chunk", "parts", "نص", "text"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.file("source")?;
    let destination = inputs.target_dir("destination")?;
    let folder_name = inputs.name("folder_name")?;
    let lines = inputs.number("lines_per_part")?;

    let final_path = paths::new_dir_in(destination, OsStr::new(folder_name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "folder_name" } else { "destination" };
        e.on_input(field)
    })?;

    // `symlink_metadata` لا يتبع الروابط: رابطٌ معلَّق بالاسم النهائي تضاربٌ.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("folder_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;
    // بادئةٌ لا مجلد: `split` تكمل الاسم بلاحقتها. و`temp` مطلق فالبادئة مطلقة.
    let prefix = temp.join(PART_PREFIX);

    Argv::tool(tools::SPLIT, "explain.split.tool")
        .flag("-l", "explain.split.lines")
        .value(lines.to_string())
        .explained_path(source, "explain.role.source_file")
        .explained_path(&prefix, "explain.split.prefix")
        .warn_all(warnings_for(inputs, source, destination, lines))
        .producing(Artifact::dir(temp, final_path))
}

fn warnings_for(
    inputs: &Inputs,
    source: &Path,
    destination: &Path,
    lines: i64,
) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    warnings.extend(warn_if_resolved(inputs, "source", source, "warn.source.resolved"));
    warnings.extend(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));

    // نداء `metadata` واحد يجيب عن السؤالين: أفارغٌ هو؟ وهل يمكن أن تتجاوز
    // الأجزاء سقف اللواحق؟ وتعذّرُه يعني «لا ندري» فلا تحذير.
    if let Ok(len) = std::fs::metadata(source).map(|m| m.len()) {
        if len == 0 {
            warnings.push("warn.split.empty_source");
        } else if may_exceed_suffixes(len, lines) {
            warnings.push("warn.split.suffix_limit");
        }
    }
    warnings
}

/// هل يمكن أن تتجاوز الأجزاء سقف اللواحق؟ **بلا قراءة الملف.**
///
/// عدد الأجزاء = ‏⌈الأسطر ÷ الأسطر-لكل-جزء⌉، وعدد الأسطر لا يزيد على الحجم
/// بالبايتات (أقصر سطرٍ ممكن بايتٌ واحد). فـ`⌈الحجم ÷ ن⌉ > ٦٧٦` تكافئ
/// `الحجم > ٦٧٦ × ن`، وهي مقارنةٌ واحدة بلا قسمة ولا تقريب.
///
/// `saturating_mul` لا الضرب العادي: `ن` تبلغ عشرة ملايين، والجداء يبقى في
/// المدى — لكن ثابتةً تُفرض بالحساب أفضل من ثابتةٍ تُفرض بحسن الظنّ.
fn may_exceed_suffixes(len: u64, lines_per_part: i64) -> bool {
    let per_part = lines_per_part.max(1) as u64;
    len > MAX_PARTS.saturating_mul(per_part)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;

    fn raw(
        source: &Path,
        destination: &Path,
        folder: &str,
        lines: i64,
    ) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("source".to_owned(), RawValue::Path(source.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("folder_name".to_owned(), RawValue::Text(folder.to_owned())),
            ("lines_per_part".to_owned(), RawValue::Text(lines.to_string())),
        ])
    }

    fn plan_with(
        source: &Path,
        destination: &Path,
        folder: &str,
        lines: i64,
    ) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(source, destination, folder, lines))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("text.split must be listed");
        assert_eq!(found.category, Category::Text);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_parts_land_inside_the_temp_folder() {
        let s = Scratch::new("split-argv").unwrap();
        let src = s.file("سجل.txt", b"1\n2\n3\n");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&src, &dst, "الأجزاء", 500).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/split"));
        assert_eq!(args[0], "-l");
        assert_eq!(args[1], "500");
        assert_eq!(Path::new(&args[2]), src.as_path());

        let artifact = cmd.artifact.as_ref().unwrap();
        assert_eq!(Path::new(&args[3]), artifact.temp.join(PART_PREFIX).as_path());
        assert_eq!(args.len(), 4);
        assert_eq!(artifact.final_path, dst.join("الأجزاء"));
        assert_eq!(artifact.kind, ArtifactKind::Dir, "the parts live in a folder we promote");
        assert!(cmd.stdout_to.is_none(), "split writes files itself; nothing is redirected");
        assert!(!artifact.temp.exists(), "planning must create nothing");
    }

    #[test]
    fn the_last_argument_is_a_prefix_inside_the_temp_not_the_temp_itself() {
        let s = Scratch::new("split-prefix").unwrap();
        let src = s.file("م.txt", b"a\n");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن", 1000).unwrap();
        let temp = cmd.artifact.as_ref().unwrap().temp.clone();
        let last = Path::new(&cmd.args[3]);
        assert_ne!(last, temp.as_path(), "a bare folder would make split write `.../folderaa`");
        assert_eq!(last.parent(), Some(temp.as_path()));
        assert_eq!(last.file_name().unwrap().to_string_lossy(), PART_PREFIX);
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("split-explain").unwrap();
        let src = s.file("م", b"a\n");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن", 1000).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("split-dashes").unwrap();
        let src = s.file("-rf", b"a\n");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "-x", 1000).unwrap();
        // `-l` وحدها رايةٌ معلَنة؛ ما بعدها كلّه بيانات.
        for arg in cmd.args.iter().skip(1) {
            assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag");
        }
    }

    #[test]
    fn an_existing_name_in_the_destination_stops_the_plan_before_anything_runs() {
        let s = Scratch::new("split-exists").unwrap();
        let src = s.file("م.txt", b"a\n");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود"), b"PRECIOUS").unwrap();

        assert_eq!(
            refusal(plan_with(&src, &dst, "موجود", 1000)),
            ("err.dest.exists", Some("folder_name"))
        );
        assert_eq!(std::fs::read(dst.join("موجود")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn a_folder_name_carrying_a_separator_is_blamed_on_its_own_field() {
        let s = Scratch::new("split-badname").unwrap();
        let src = s.file("م.txt", b"a\n");
        let dst = s.dir("و");
        assert_eq!(
            refusal(plan_with(&src, &dst, "أ/ب", 1000)),
            ("err.name.invalid", Some("folder_name"))
        );
    }

    #[test]
    fn a_line_count_outside_the_declared_range_is_refused_with_its_field() {
        let s = Scratch::new("split-range").unwrap();
        let src = s.file("م.txt", b"a\n");
        let dst = s.dir("و");
        for bad in [0, -1, 10_000_001] {
            assert_eq!(
                refusal(plan_with(&src, &dst, "ن", bad)),
                ("err.input.range", Some("lines_per_part")),
                "{bad} must be refused"
            );
        }
    }

    #[test]
    fn shell_syntax_in_a_folder_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("split-shellish").unwrap();
        let src = s.file("م.txt", b"a\n");
        let dst = s.dir("و");

        for name in ["أجزاء 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let cmd = plan_with(&src, &dst, name, 1000).unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(name));
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn an_empty_source_is_announced_because_split_would_produce_no_part_at_all() {
        let s = Scratch::new("split-empty").unwrap();
        let empty = s.file("فارغ.txt", b"");
        let dst = s.dir("و");

        let cmd = plan_with(&empty, &dst, "ن", 1000).unwrap();
        assert!(cmd.warnings.contains(&"warn.split.empty_source"), "{:?}", cmd.warnings);
    }

    #[test]
    fn the_suffix_ceiling_is_announced_only_when_it_cannot_be_ruled_out() {
        // ٢٠٠٠ بايت مع سطرٍ واحد لكل جزء: التجاوز ممكن (حدّه الأعلى ٢٠٠٠ جزءًا).
        // ونفس الملف مع ١٠٠٠ سطر لكل جزء: مستحيل، فلا تحذير.
        let s = Scratch::new("split-ceiling").unwrap();
        let big = s.file("كبير.txt", &vec![b'x'; 2000]);
        let dst = s.dir("و");

        let risky = plan_with(&big, &dst, "ن1", 1).unwrap();
        assert!(risky.warnings.contains(&"warn.split.suffix_limit"), "{:?}", risky.warnings);

        let safe = plan_with(&big, &dst, "ن2", 1000).unwrap();
        assert!(!safe.warnings.contains(&"warn.split.suffix_limit"), "{:?}", safe.warnings);
    }

    #[test]
    fn the_ceiling_rule_is_an_upper_bound_not_a_guess() {
        // الحدّ الفاصل بالضبط: ٦٧٦ × ن بايتًا لا تتجاوز، وبايتٌ فوقها قد يتجاوز.
        assert!(!may_exceed_suffixes(676, 1));
        assert!(may_exceed_suffixes(677, 1));
        assert!(!may_exceed_suffixes(676_000, 1000));
        assert!(may_exceed_suffixes(676_001, 1000));
        // ولا يفيض الجداء عند أقصى ما تقبله المواصفة فيقلب الجواب.
        assert!(may_exceed_suffixes(u64::MAX, 10_000_000));
    }

    #[test]
    fn an_ordinary_split_raises_no_noisy_warning() {
        let s = Scratch::new("split-quiet").unwrap();
        let src = s.file("م.txt", b"1\n2\n3\n");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن", 1000).unwrap();
        for noisy in ["warn.split.empty_source", "warn.split.suffix_limit"] {
            assert!(!cmd.warnings.contains(&noisy), "unexpected {noisy}: {:?}", cmd.warnings);
        }
    }

    #[test]
    fn a_symlinked_source_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("split-symlink").unwrap();
        let real = s.file("الحقيقي.txt", b"a\n");
        let link = s.path().join("رابط.txt");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&link, &dst, "ن", 1000).unwrap();
        assert_eq!(Path::new(&cmd.args[2]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_nothing_behind_in_either_directory() {
        let s = Scratch::new("split-clean").unwrap();
        let src = s.file("م.txt", b"a\n");
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&src, &dst, "ن", 1000).unwrap();
        }
        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
    }
}
