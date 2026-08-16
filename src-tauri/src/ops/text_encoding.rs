//! تحويل ملفٍ نصّي من ترميزٍ قديم إلى UTF-8، باستخدام `textutil`.
//!
//! ## المسألة: النصوص العربية القديمة
//!
//! هذه العملية وُجدت للعربية بالذات. ملفٌ كُتب على Windows بترميز
//! `windows-1256`، أو على ماكنتوش قديم بـ Mac Arabic، أو خرج من نظامٍ يستعمل
//! `ISO-8859-6` — كلّها تُفتح اليوم فتُرى «Ø§ÙÙÙ» أو ما يشبهها. البايتات
//! سليمة، والمفقود جدولُ قراءتها. هذه العملية تقول للنظام أي جدولٍ يقرأ به،
//! ثم تكتب الناتج UTF-8 مرّةً واحدة وإلى الأبد.
//!
//! ## لماذا `textutil` لا `iconv`
//!
//! `iconv` تكتب على الخرج القياسي وحدها — وهذا **ليس** سبب استبعادها: النواة
//! تعرف كيف توجّه الخرج إلى ملفها المؤقّت، و`text.merge` تفعل ذلك بـ`cat`.
//! السبب شيئان آخران:
//!
//! 1. `iconv` تعرف تيّار بايتات ولا تعرف مستندًا. وملفات المستخدم على macOS
//!    ليست كلها `.txt`: منها `.rtf` و`.rtfd` و`.doc` و`.docx` و`.html`. تمريرها
//!    على `iconv` يعيد ترميز حشو الصيغة نفسه فيُنتج ملفًا معطوبًا يبدو سليمًا.
//!    و`textutil` تمرّ بنظام النصّ في Cocoa فتفهم الصيغ، و`-convert txt`
//!    تُخرج نصًّا مجرّدًا منها كلها.
//! 2. `textutil` تكتب الملف بنفسها عبر `-output`، فلا توجيهَ ولا افتراضَ عن
//!    الخرج القياسي أصلًا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/textutil -convert txt -inputencoding <الترميز> \
//!                   -encoding UTF-8 -output <المؤقّت> <المصدر>
//! ```
//!
//! `-output` يشير إلى **المؤقّت** لا إلى الاسم النهائي: الترقية لا تقع إلا
//! بعد خروجٍ ناجح، فانقطاعٌ في المنتصف لا يترك في مجلد المستخدم ملفًا نصفيًّا
//! يحمل الاسم الصحيح.
//!
//! `--` معلَنة في `man textutil` («كل ما بعدها أسماء ملفات») ولم تُضف: ما
//! بعدها هنا مسارٌ واحد مطلق يبدأ بـ`/`، فلا شيء لتمنعه. القاعدة في هذا
//! الملف أن تُضاف الراية لأثرها لا لشكلها.
//!
//! ## الحدّ الذي يجب أن يُقال: ترميزٌ خاطئ لا يفشل، بل يشوّه
//!
//! الترميزات القديمة الثلاثة أحادية البايت: كل بايتٍ فيها له حرف، فلا يوجد
//! تسلسلٌ «غير صالح» يجعل الأداة تتوقّف. قيس على هذا الجهاز — البايتات
//! `C7 E1 E1 E5`:
//!
//! * مع `windows-1256` → «الله»
//! * مع `iso-8859-6` → «اففم»
//! * مع `x-mac-arabic` → «اففم»
//! * ومع `macintosh` → «‏«··Â‏»
//!
//! ثلاثة نواتج مختلفة، وكلها خرجت بنجاح. فالاختيار الخاطئ يُنتج ملفًا مشوّهًا
//! ورمز خروج صفرًا، ولذلك تُعلن `warn.encoding.silent_mojibake` مع كل ترميزٍ
//! أحادي البايت. أما `utf-8` فتفشل صراحةً على بايتاتٍ لا تصلح (قيس: «‏Text
//! encoding Unicode (UTF-8) isn't applicable‏»)، فلا تحذير معها.
//!
//! ## ولماذا `x-mac-arabic` لا `macintosh`
//!
//! `macintosh` اسمٌ في سجلّ IANA يعني **MacRoman** لا العربية، وهو ما أثبته
//! القياس أعلاه. إعلانُه تحت وسمٍ يقول «ماك العربية» كان يعني أن الشاشة تسمّي
//! شيئًا وتفعل غيره — وهو نقضٌ لأطروحة المنتج كلها. الاسم الصحيح لجدول ماك
//! العربي في CFString هو `x-mac-arabic`، وهو المعلَن هنا.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;
use std::path::Path;

pub const ID: &str = "text.encoding.utf8";

/// الترميز الذي **لا** يشوّه صامتًا: بايتاتٌ لا تصلح تُوقف الأداة بخطأ.
const SELF_VALIDATING: &str = "utf-8";

/// الترميزات المدخلة المعلَنة. عربية كلها عدا الأخير، وهو الحالة التي يكون
/// فيها الملف UTF-8 أصلًا ويُراد تجريده من صيغته (‏`.rtf` مثلًا) لا ترميزه.
const ENCODINGS: &[ChoiceOption] = &[
    ChoiceOption::new("windows-1256", "choice.encoding.cp1256"),
    ChoiceOption::new("iso-8859-6", "choice.encoding.iso8859_6"),
    ChoiceOption::new("x-mac-arabic", "choice.encoding.macarabic"),
    ChoiceOption::new(SELF_VALIDATING, "choice.encoding.utf8"),
];

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.text.encoding.utf8.title",
    description_key: "op.text.encoding.utf8.description",
    category: Category::Text,
    // تكتب ملفًا جديدًا في الوجهة ولا تمسّ المصدر.
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::TEXTUTIL,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("source", InputKind::ExistingFile),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("out_name", InputKind::NewName { ext: Some("txt") }),
        InputSpec::new("input_encoding", InputKind::Choice { options: ENCODINGS }),
    ],
    sort_order: 30,
    search_terms: &[
        "textutil",
        "ترميز",
        "encoding",
        "utf-8",
        "utf8",
        "windows-1256",
        "cp1256",
        "عربي",
        "مشوّه",
        "mojibake",
        "نص",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.file("source")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("out_name")?;
    // `&'static str` من قائمة المواصفة لا نسخةٌ من نصّ الواجهة: ما يدخل الأمر
    // يخرج من الشيفرة المُترجَمة، والواجهة لا تفعل إلا أن تختار بينها.
    let encoding = inputs.choice("input_encoding")?;

    // `.txt` أُضيف في `value.rs` قبل الوصول إلى هنا، و`new_file_in` يفحص
    // الاسم **بعد** إضافته: اسمٌ من ٢٥٥ وحدة يقبله التنقية ثم يصير ٢٥٩.
    // والرفض حينها يخصّ الاسم لا الوجهة، وإلا بدّل المستخدم مجلدًا سليمًا
    // مرّةً بعد مرّة بينما الحقل الذي يصلح العطب هو الاسم.
    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "out_name" } else { "destination" };
        e.on_input(field)
    })?;

    // `symlink_metadata` لا يتبع الروابط: رابطٌ معلَّق بالاسم النهائي تضاربٌ.
    // ويلتقط هذا الفحصُ كذلك محاولةَ الكتابة فوق المصدر نفسه، إذ هو موجود.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("out_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    Argv::tool(tools::TEXTUTIL, "explain.textutil.tool")
        .flag("-convert", "explain.textutil.convert")
        .value("txt")
        .flag("-inputencoding", "explain.textutil.input_encoding")
        .value(encoding)
        .flag("-encoding", "explain.textutil.output_encoding")
        .value("UTF-8")
        .flag("-output", "explain.textutil.output")
        .explained_path(&temp, "explain.role.temp")
        .explained_path(source, "explain.role.source_file")
        .warn_all(warnings_for(inputs, source, destination, encoding))
        .producing(Artifact::file(temp, final_path))
}

fn warnings_for(
    inputs: &Inputs,
    source: &Path,
    destination: &Path,
    encoding: &str,
) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    // الترميزات أحادية البايت تقبل كل بايت، فالاختيار الخاطئ يُخرج ملفًا
    // مشوّهًا برمز خروج ناجح. و`utf-8` وحدها تتحقّق من نفسها فتفشل صراحةً.
    if encoding != SELF_VALIDATING {
        warnings.push("warn.encoding.silent_mojibake");
    }
    warnings.extend(warn_if_resolved(inputs, "source", source, "warn.source.resolved"));
    warnings.extend(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;

    /// الرايات المعلَنة في هذا الأمر. ما عداها في `argv` بيانات.
    const DECLARED_FLAGS: &[&str] = &["-convert", "-inputencoding", "-encoding", "-output"];

    fn raw(
        source: &Path,
        destination: &Path,
        name: &str,
        encoding: &str,
    ) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("source".to_owned(), RawValue::Path(source.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("out_name".to_owned(), RawValue::Text(name.to_owned())),
            ("input_encoding".to_owned(), RawValue::Text(encoding.to_owned())),
        ])
    }

    fn plan_with(
        source: &Path,
        destination: &Path,
        name: &str,
        encoding: &str,
    ) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(source, destination, name, encoding))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("text.encoding.utf8 must be listed");
        assert_eq!(found.category, Category::Text);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_output_is_the_temp() {
        let s = Scratch::new("enc-argv").unwrap();
        let src = s.file("قديم.txt", b"\xC7\xE1\xE1\xE5\n");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&src, &dst, "جديد", "windows-1256").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/textutil"));
        assert_eq!(args[0], "-convert");
        assert_eq!(args[1], "txt");
        assert_eq!(args[2], "-inputencoding");
        assert_eq!(args[3], "windows-1256");
        assert_eq!(args[4], "-encoding");
        assert_eq!(args[5], "UTF-8");
        assert_eq!(args[6], "-output");

        let artifact = cmd.artifact.as_ref().unwrap();
        assert_eq!(Path::new(&args[7]), artifact.temp.as_path());
        assert_eq!(Path::new(&args[8]), src.as_path());
        assert_eq!(args.len(), 9);
        assert_eq!(artifact.final_path, dst.join("جديد.txt"), "the .txt is added exactly once");
        assert_eq!(artifact.kind, ArtifactKind::File);
        assert!(cmd.stdout_to.is_none(), "textutil writes the file itself through -output");
        assert!(!artifact.temp.exists(), "planning must create nothing");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("enc-explain").unwrap();
        let src = s.file("م", b"x\n");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن", "windows-1256").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("enc-dashes").unwrap();
        let src = s.file("-rf", b"x\n");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "-x", "iso-8859-6").unwrap();
        for arg in &cmd.args {
            if DECLARED_FLAGS.iter().any(|flag| arg.as_os_str() == std::ffi::OsStr::new(flag)) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag");
        }
    }

    #[test]
    fn every_declared_encoding_reaches_the_command_verbatim() {
        let s = Scratch::new("enc-all").unwrap();
        let src = s.file("م", b"x\n");
        let dst = s.dir("و");

        for (index, option) in ENCODINGS.iter().enumerate() {
            let cmd = plan_with(&src, &dst, &format!("ن{index}"), option.value).unwrap();
            assert_eq!(cmd.args[3], std::ffi::OsString::from(option.value));
        }
    }

    #[test]
    fn an_encoding_the_specification_does_not_declare_is_refused() {
        // القائمة مغلقة: نصٌّ حرّ هنا كان يعني أن الواجهة تكتب جزءًا من الأمر.
        let s = Scratch::new("enc-unknown").unwrap();
        let src = s.file("م", b"x\n");
        let dst = s.dir("و");
        for bogus in ["macintosh", "koi8-r", "-o/etc/passwd", ""] {
            assert_eq!(
                refusal(plan_with(&src, &dst, "ن", bogus)),
                ("err.input.type", Some("input_encoding")),
                "{bogus:?} must be refused"
            );
        }
    }

    #[test]
    fn an_existing_name_in_the_destination_stops_the_plan_before_anything_runs() {
        let s = Scratch::new("enc-exists").unwrap();
        let src = s.file("م", b"x\n");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود.txt"), b"PRECIOUS").unwrap();

        assert_eq!(
            refusal(plan_with(&src, &dst, "موجود", "windows-1256")),
            ("err.dest.exists", Some("out_name"))
        );
        assert_eq!(std::fs::read(dst.join("موجود.txt")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn converting_a_file_onto_itself_is_refused_by_the_conflict_check() {
        let s = Scratch::new("enc-onto-self").unwrap();
        let dst = s.dir("و");
        let src = s.file("و/م.txt", b"x\n");
        assert_eq!(
            refusal(plan_with(&src, &dst, "م.txt", "windows-1256")),
            ("err.dest.exists", Some("out_name"))
        );
    }

    #[test]
    fn a_name_carrying_a_separator_is_blamed_on_the_name_field() {
        let s = Scratch::new("enc-badname").unwrap();
        let src = s.file("م", b"x\n");
        let dst = s.dir("و");
        assert_eq!(
            refusal(plan_with(&src, &dst, "أ/ب", "windows-1256")),
            ("err.name.invalid", Some("out_name"))
        );
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("enc-shellish").unwrap();
        let src = s.file("م", b"x\n");
        let dst = s.dir("و");

        for name in ["ناتج 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let cmd = plan_with(&src, &dst, name, "windows-1256").unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(format!("{name}.txt")));
            assert_eq!(cmd.args.len(), 9, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_single_byte_encoding_announces_that_a_wrong_choice_fails_silently() {
        let s = Scratch::new("enc-warn").unwrap();
        let src = s.file("م", b"x\n");
        let dst = s.dir("و");

        for legacy in ["windows-1256", "iso-8859-6", "x-mac-arabic"] {
            let cmd = plan_with(&src, &dst, "ن", legacy).unwrap();
            assert!(
                cmd.warnings.contains(&"warn.encoding.silent_mojibake"),
                "{legacy}: {:?}",
                cmd.warnings
            );
        }

        // `utf-8` تتحقّق من نفسها وتفشل على بايتاتٍ لا تصلح، فلا تحذير.
        let cmd = plan_with(&src, &dst, "ن2", "utf-8").unwrap();
        assert!(!cmd.warnings.contains(&"warn.encoding.silent_mojibake"), "{:?}", cmd.warnings);
    }

    #[test]
    fn a_symlinked_source_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("enc-symlink").unwrap();
        let real = s.file("الحقيقي.txt", b"x\n");
        let link = s.path().join("رابط.txt");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&link, &dst, "ن", "windows-1256").unwrap();
        assert_eq!(Path::new(&cmd.args[8]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_nothing_behind_in_the_destination() {
        let s = Scratch::new("enc-clean").unwrap();
        let src = s.file("م", b"x\n");
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&src, &dst, "ن", "windows-1256").unwrap();
        }
        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
    }
}
