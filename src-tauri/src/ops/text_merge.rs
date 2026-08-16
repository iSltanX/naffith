//! دمج ملفين نصّيين في ملفٍ ثالث، باستخدام `cat`.
//!
//! ## لماذا `cat`، ولماذا لا صدفة
//!
//! `cat` تكتب على الخرج القياسي وحده. لا راية فيها تقول «اكتب في هذا الملف»،
//! والصيغة التي يعرفها الناس — `cat أول ثانٍ > الناتج` — ليست من `cat` في
//! شيء: العلامة `>` كلمةٌ في لغة الصدفة، تفتح الصدفةُ بها الملف وتصل واصفَه
//! بخرج الأداة قبل أن تُطلقها. أي أن تنفيذ هذه العملية «كما يكتبها الناس»
//! يعني تسليم نصّ أمرٍ إلى `sh -c` — وهو بالضبط ما يمنعه `executor.rs` في
//! قاعدته الأولى: لا صدفة، ولا تركيب نصّي، ولا `PATH`. ولو فُتح هذا الباب
//! لعمليةٍ واحدة لصار في المنتج مسارٌ تُصبح فيه سلسلةٌ نصّية أمرًا، ولسقطت
//! معه كل ضمانةٍ يقوم عليها «سَطْر».
//!
//! فالتوجيه يقع في النواة لا في نصّ الأمر: `PlannedCommand::stdout_to` يحمل
//! مسار الملف المؤقّت، و`executor` يفتحه بنفسه — بـ`O_NOFOLLOW` فوق حجزٍ
//! حصريّ سبقه في `plans::Preconditions::claim_temp` — ويصل واصفَه بخرج
//! الطفل. لا `>` في أي موضع، ولا مفسّر. و`planner` يرفض أي توجيهٍ إلى غير
//! `artifact.temp`، فالقيد مفروضٌ بفحصٍ لا موصوفٌ في تعليق.
//!
//! ## الصيغة
//!
//! ```text
//! /bin/cat -- <الأول> <الثاني>        والخرج مُوجَّه إلى <المؤقّت>
//! ```
//!
//! المؤقّت **لا يظهر وسيطًا** في الأمر، لأن `cat` لا تعرف وجهة أصلًا. و«سَطْر»
//! يعرض مسار الكتابة في حقله الخاص (`writes_to`) كي لا يبدو الأمر ناقصًا.
//!
//! `--` تفهمها `cat` على macOS، وهي حزام أمانٍ فوق كون المسارين مطلقين
//! أصلًا — ومسارٌ مطلق يبدأ بـ`/` فلا يُقرأ رايةً في أي حال.
//!
//! ## ما لا تفعله `cat`: لا فاصل بين الملفين
//!
//! تنسخ البايتات كما هي ولا تضع بينها شيئًا. قيس على هذا الجهاز: ملفٌ محتواه
//! `a\nb` (بلا سطرٍ جديد في آخره) وملفٌ محتواه `c\nd\n` يخرجان `a\nbc\nd\n`
//! — أي أن آخر سطرٍ في الأول وأول سطرٍ في الثاني صارا سطرًا واحدًا. لذلك
//! تُعلن `warn.text.no_separator` دائمًا (خاصيّةٌ ثابتة في الأداة)، وتُضاف
//! `warn.text.glued_lines` حين **يُقاس** أن الملف الأول لا ينتهي بسطرٍ جديد.
//!
//! والعلاج الذي رُفض: أن نُدخل سطرًا جديدًا بيننا. كان يعني أن الناتج ليس
//! مجموع الملفين بايتًا ببايت، وأن أمرًا معروضًا على الشاشة يفعل أكثر مما
//! يقول. الإخبارُ قبل التنفيذ أصدق من تصحيحٍ صامت.
//!
//! ## ملفان لا أكثر، وهو قرارٌ لا سهو
//!
//! الدمج الطبيعي يقبل عددًا مفتوحًا من الملفات، وذلك يحتاج حقلًا متكرّرًا
//! («أضف ملفًا») لا يعرف النموذج رسمه اليوم: `InputKind` قائمةٌ مغلقة ولا
//! صيغة فيها تعبّر عن «قائمة مسارات». وتزييفُه بحقلٍ نصّي يُقسَّم على فاصلة
//! كان سيعيد تقسيمَ النصّ إلى وسائط من بابٍ خلفي — وهي الصيغة التي لا توجد
//! في `common.rs` عمدًا. فالعملية ملفان، ودمج N مسجَّل في خارطة الطريق باسمه.
//!
//! ## ولا تقدير حجم
//!
//! حجم الناتج هنا معلومٌ يقينًا (مجموع الملفين)، لكن حقل التقدير في الواجهة
//! يقول «حجم المصدر تقديريًا… والأرشيف أصغر عادةً» — وهو نصٌّ عن الضغط.
//! عرضُ رقمٍ يقيني تحت عنوانٍ يقول إنه تقدير يجعل الشاشة تكذب في الاتجاه
//! الآخر، فتُرك الحقل فارغًا حتى يُفصل النصّان.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;
use std::path::Path;

pub const ID: &str = "text.merge";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.text.merge.title",
    description_key: "op.text.merge.description",
    category: Category::Text,
    // تُنشئ ملفًا ثالثًا ولا تمسّ المدخلين.
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::CAT,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("first", InputKind::ExistingFile),
        InputSpec::new("second", InputKind::ExistingFile),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("out_name", InputKind::NewName { ext: None }),
    ],
    sort_order: 10,
    search_terms: &["cat", "دمج", "merge", "concat", "join", "ضمّ", "نص", "text", "ملفين"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let first = inputs.file("first")?;
    let second = inputs.file("second")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("out_name")?;

    // المساران محلولا الروابط، فالمقارنة مقارنة موضعين لا نصّين: رابطٌ رمزي
    // إلى الملف نفسه يُلتقط هنا. ودمج ملفٍ بنفسه ليس خطأً في `cat` — تخرج
    // نسخته مرتين — لكنه لا يكون قصدًا أبدًا، وثمنه ملفٌ باسمٍ اختاره
    // المستخدم يظنّه دمج اثنين.
    if first == second {
        return Err(CoreError::SamePath.on_input("second"));
    }

    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "out_name" } else { "destination" };
        e.on_input(field)
    })?;

    // `symlink_metadata` لا يتبع الروابط: رابطٌ معلَّق بالاسم النهائي تضاربٌ.
    // وهذا الفحص نفسه هو ما يمنع أن يكون الناتج أحد المدخلين، إذ كلاهما
    // موجودٌ بالضرورة — ولو كتبنا فوق أحدهما لقرأت `cat` ما تكتبه.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("out_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    Argv::tool(tools::CAT, "explain.cat.tool")
        .end_of_flags()
        .explained_path(first, "explain.role.merge_first")
        .explained_path(second, "explain.role.merge_second")
        .warn_all(warnings_for(inputs, first, second, destination))
        .producing_via_stdout(Artifact::file(temp, final_path))
}

fn warnings_for(
    inputs: &Inputs,
    first: &Path,
    second: &Path,
    destination: &Path,
) -> Vec<&'static str> {
    // خاصيّةٌ في الأداة لا حالةٌ في المدخلات: `cat` لا تضع فاصلًا أبدًا.
    let mut warnings = vec!["warn.text.no_separator"];
    if lacks_trailing_newline(first) {
        warnings.push("warn.text.glued_lines");
    }
    warnings.extend(warn_if_resolved(inputs, "first", first, "warn.source.resolved"));
    warnings.extend(warn_if_resolved(inputs, "second", second, "warn.source.resolved"));
    warnings.extend(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));
    warnings
}

/// هل ينتهي الملف بغير سطرٍ جديد؟ يُقرأ **بايتٌ واحد** من آخره لا الملف كلّه.
///
/// التخطيط يجري عند كل تغيير في النموذج، فقراءة الملف كاملًا للإجابة كانت
/// تجعل كل ضغطة مفتاح تمرّ على القرص كلّه. القفز إلى آخر بايت يجيب عن السؤال
/// نفسه بقراءةٍ واحدة مهما بلغ حجم الملف.
///
/// وتعذّر القراءة يعني «لا ندري»، فلا تحذير: تحذيرٌ مبنيّ على جهلٍ يعلّم
/// المستخدم أن يتجاهل التحذيرات.
fn lacks_trailing_newline(path: &Path) -> bool {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut file) = std::fs::File::open(path) else { return false };
    let Ok(len) = file.seek(SeekFrom::End(0)) else { return false };
    // ملفٌ فارغ لا يلصق شيئًا بشيء، فلا محلّ للتحذير.
    if len == 0 {
        return false;
    }
    if file.seek(SeekFrom::End(-1)).is_err() {
        return false;
    }
    let mut last = [0u8; 1];
    match file.read_exact(&mut last) {
        Ok(()) => last[0] != b'\n',
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;

    fn raw(
        first: &Path,
        second: &Path,
        destination: &Path,
        name: &str,
    ) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("first".to_owned(), RawValue::Path(first.display().to_string())),
            ("second".to_owned(), RawValue::Path(second.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("out_name".to_owned(), RawValue::Text(name.to_owned())),
        ])
    }

    fn plan_with(
        first: &Path,
        second: &Path,
        destination: &Path,
        name: &str,
    ) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(first, second, destination, name))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("text.merge must be listed");
        assert_eq!(found.category, Category::Text);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_output_goes_through_stdout() {
        let s = Scratch::new("merge-argv").unwrap();
        let a = s.file("الأول.txt", b"a\n");
        let b = s.file("الثاني.txt", b"b\n");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&a, &b, &dst, "المدموج.txt").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|x| x.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/bin/cat"));
        assert_eq!(args[0], "--");
        assert_eq!(Path::new(&args[1]), a.as_path());
        assert_eq!(Path::new(&args[2]), b.as_path());
        assert_eq!(args.len(), 3);

        let artifact = cmd.artifact.as_ref().unwrap();
        assert_eq!(artifact.final_path, dst.join("المدموج.txt"));
        assert_eq!(artifact.kind, ArtifactKind::File);
        assert_eq!(
            cmd.stdout_to.as_deref(),
            Some(artifact.temp.as_path()),
            "the redirect must land in the plan's own temp and nowhere else"
        );
        assert!(
            !args.iter().any(|arg| Path::new(arg) == artifact.temp.as_path()),
            "cat knows no destination: the temp must not appear as an argument"
        );
        assert!(!artifact.temp.exists(), "planning must create nothing");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("merge-explain").unwrap();
        let a = s.file("أ", b"1\n");
        let b = s.file("ب", b"2\n");
        let dst = s.dir("و");

        let cmd = plan_with(&a, &b, &dst, "ن").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|x| x.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("merge-dashes").unwrap();
        let a = s.file("-rf", b"x\n");
        let b = s.file("--force", b"y\n");
        let dst = s.dir("و");

        let cmd = plan_with(&a, &b, &dst, "-x").unwrap();
        // الوسيط الأول `--` رايةٌ معلَنة؛ ما بعدها بيانات.
        for arg in cmd.args.iter().skip(1) {
            assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag");
        }
    }

    #[test]
    fn merging_a_file_with_itself_is_refused() {
        let s = Scratch::new("merge-same").unwrap();
        let one = s.file("واحد.txt", b"x\n");
        let dst = s.dir("و");
        assert_eq!(refusal(plan_with(&one, &one, &dst, "ن")), ("err.path.same", Some("second")));
    }

    #[test]
    fn a_symlink_to_the_same_file_is_caught_because_paths_are_resolved_first() {
        let s = Scratch::new("merge-same-link").unwrap();
        let real = s.file("حقيقي.txt", b"x\n");
        let link = s.path().join("رابط.txt");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");
        assert_eq!(refusal(plan_with(&real, &link, &dst, "ن")), ("err.path.same", Some("second")));
    }

    #[test]
    fn an_existing_name_in_the_destination_stops_the_plan_before_anything_runs() {
        let s = Scratch::new("merge-exists").unwrap();
        let a = s.file("أ", b"1\n");
        let b = s.file("ب", b"2\n");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود"), b"PRECIOUS").unwrap();

        assert_eq!(
            refusal(plan_with(&a, &b, &dst, "موجود")),
            ("err.dest.exists", Some("out_name"))
        );
        assert_eq!(std::fs::read(dst.join("موجود")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn writing_the_result_over_one_of_its_own_inputs_is_refused() {
        // المدخل موجودٌ بالضرورة، فيلتقطه فحص التضارب نفسه: لا حاجة إلى فحصٍ
        // ثانٍ، والاختبار يثبّت أن الحالة مغطّاة فعلًا.
        let s = Scratch::new("merge-onto-input").unwrap();
        let dst = s.dir("و");
        let a = s.file("و/أ.txt", b"1\n");
        let b = s.file("ب.txt", b"2\n");
        assert_eq!(
            refusal(plan_with(&a, &b, &dst, "أ.txt")),
            ("err.dest.exists", Some("out_name"))
        );
    }

    #[test]
    fn a_name_carrying_a_separator_is_blamed_on_the_name_field() {
        let s = Scratch::new("merge-badname").unwrap();
        let a = s.file("أ", b"1\n");
        let b = s.file("ب", b"2\n");
        let dst = s.dir("و");
        assert_eq!(
            refusal(plan_with(&a, &b, &dst, "مجلد/ملف")),
            ("err.name.invalid", Some("out_name"))
        );
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("merge-shellish").unwrap();
        let a = s.file("أ", b"1\n");
        let b = s.file("ب", b"2\n");
        let dst = s.dir("و");

        for name in ["ناتج 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let cmd = plan_with(&a, &b, &dst, name).unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(name));
            assert_eq!(cmd.args.len(), 3, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn the_missing_separator_is_always_announced() {
        let s = Scratch::new("merge-warn-always").unwrap();
        let a = s.file("أ", "ينتهي بسطر\n".as_bytes());
        let b = s.file("ب", "ثانٍ\n".as_bytes());
        let dst = s.dir("و");

        let cmd = plan_with(&a, &b, &dst, "ن").unwrap();
        assert!(cmd.warnings.contains(&"warn.text.no_separator"), "{:?}", cmd.warnings);
    }

    #[test]
    fn gluing_is_announced_only_when_the_first_file_really_lacks_its_last_newline() {
        let s = Scratch::new("merge-warn-glue").unwrap();
        let dst = s.dir("و");
        let ends_with_newline = s.file("مكتمل", b"a\nb\n");
        let does_not = s.file("ناقص", b"a\nb");
        let empty = s.file("فارغ", b"");
        let other = s.file("آخر", b"c\n");

        let quiet = plan_with(&ends_with_newline, &other, &dst, "ن1").unwrap();
        assert!(!quiet.warnings.contains(&"warn.text.glued_lines"), "{:?}", quiet.warnings);

        let loud = plan_with(&does_not, &other, &dst, "ن2").unwrap();
        assert!(loud.warnings.contains(&"warn.text.glued_lines"), "{:?}", loud.warnings);

        let empty_first = plan_with(&empty, &other, &dst, "ن3").unwrap();
        assert!(
            !empty_first.warnings.contains(&"warn.text.glued_lines"),
            "an empty first file glues nothing: {:?}",
            empty_first.warnings
        );
    }

    #[test]
    fn a_symlinked_input_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("merge-symlink").unwrap();
        let real = s.file("الحقيقي.txt", b"x\n");
        let link = s.path().join("رابط.txt");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let other = s.file("آخر.txt", b"y\n");
        let dst = s.dir("و");

        let cmd = plan_with(&link, &other, &dst, "ن").unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_nothing_behind_in_the_destination() {
        let s = Scratch::new("merge-clean").unwrap();
        let a = s.file("أ", b"1\n");
        let b = s.file("ب", b"2\n");
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&a, &b, &dst, "ن").unwrap();
        }
        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
    }
}
