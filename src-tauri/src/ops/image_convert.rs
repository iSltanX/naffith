//! تحويل صيغة صورة باستخدام `sips`.
//!
//! ## لماذا `sips` لا ImageMagick
//!
//! `sips` جزءٌ من macOS وتمرّ عبر ImageIO — أي الفكّاك نفسه الذي يستعمله
//! Preview وQuick Look. فما تراه في المعاينة هو ما تكتبه الأداة، ولا يوجد
//! تفسيرٌ ثانٍ للملف داخل الجهاز.
//!
//! البديل الشائع — ImageMagick من Homebrew — مرفوضٌ من بابين: `tools.rs` لا
//! يقبل أداةً خارج `/bin` و`/sbin` و`/usr/bin` و`/usr/sbin` لأن ما يستطيع
//! المستخدم استبداله بلا صلاحيات مدير ليس أداة نظام؛ ولأن تثبيتًا خارجيًا
//! يجعل ناتج العملية رهنَ نسخةٍ لا نعرفها ولا نستطيع وصفها في الشرح.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/sips -s format <الصيغة> <المصدر> --out <المؤقّت>
//! ```
//!
//! * `-s` — اضبط خاصيةً من خصائص الصورة. تأخذ رمزين: اسم الخاصية ثم قيمتها.
//! * `format` — اسم الخاصية المضبوطة: صيغة الترميز نفسها.
//! * `--out` — اكتب الناتج في مسارٍ آخر. بدونها تكتب `sips` **فوق المصدر**،
//!   وهو بالضبط ما لا تفعله هذه العملية.
//!
//! ولا `--` هنا: `sips` **لا تفهم** فاصل نهاية الرايات، فإضافته كانت ستصير
//! وسيطًا زائدًا تحاول الأداة قراءته ملفًّا. المسارات مطلقةٌ أصلًا فتبدأ بـ `/`
//! ولا تُقرأ رايةً، واختبارٌ في هذا الملف يثبّت ذلك بدل الاعتماد عليه.
//!
//! ## ما ليس معروضًا هنا: WebP — وقد كان مطلوبًا
//!
//! طُلبت WebP في قائمة المهام، ولا تُعرض. القياس على هذا الجهاز
//! (‏macOS 26.6، بناء 25G72):
//!
//! ```text
//! $ sips --formats | grep webp
//! org.webmproject.webp         webp
//! $ sips -s format webp in.png --out out.webp
//! Error: Can't write format: org.webmproject.webp
//! Error 13: an unknown error occurred
//! ```
//!
//! العمود الثالث في `--formats` هو `Writable`، وWebP لا تحمله: النظام يقرأ
//! WebP ولا يكتبها. فالخيار لو عُرض لَظهر في قائمةٍ مغلقة ثم فشل في كل مرة —
//! وقائمةٌ مغلقة يفشل أحد خياراتها دائمًا أسوأ من خيارٍ غائب، لأن الأول يجعل
//! المستخدم يشكّ في ملفه بينما الثاني يجعله يعرف أن الأداة لا تفعلها.
//!
//! **خارطة الطريق:** WebP تحتاج مُرمِّزًا لا يملكه النظام (‏`cwebp`)، أي أداةً
//! خارج جذور النظام — وهو قرارٌ في `tools.rs` لا في هذا الملف. تبقى مؤجَّلة
//! حتى يُتّخذ ذلك القرار صراحةً، لا أن تتسلّل من باب عمليةٍ واحدة.
//!
//! ## البيانات الوصفية: أثرٌ جانبي لا وعد
//!
//! إعادة الترميز تمرّ بالبيانات الوصفية عبر ImageIO، وما يعبر منها يقرّره
//! **زوج المرمِّزين** لا هذه العملية. قيس: `description` و`copyright`
//! **نجيا** من JPEG إلى PNG، بينما لا تحمل PNG أصلًا كتلة EXIF التي تحملها
//! JPEG في `APP1`.
//!
//! ولهذا لا تُوصف هذه العملية بأنها «تنظّف الصورة»: من يريد إزالة الموقع
//! الجغرافي أو رقم الكاميرا قبل المشاركة لا يجوز أن يُحال إلى تحويلِ صيغة
//! يصادف أن يُسقط بعضها. **عملية «إزالة البيانات الوصفية» ما زالت دَينًا في
//! خارطة الطريق**، والتحذير هنا يقول ذلك بدل أن يسكت فيُفهم صمتُه ضمانًا.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تستبدل، ولا تكتب فوق المصدر، ولا تُنشئ الوجهة. الاسم المأخوذ يوقفها
//! وتُخبر. والوسيط الأخير **مؤقّت** لا نهائي: الناتج يُبنى باسمٍ عابر داخل مجلد
//! الوجهة نفسه، ولا يُرقّى إلى اسمه إلا بعد خروجٍ ناجح — فانقطاعٌ في المنتصف
//! لا يترك صورةً نصفيّة تحمل الاسم الصحيح وتبدو سليمة في Finder.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;
use std::path::Path;

pub const ID: &str = "image.convert";

/// الخيارات كما تراها الواجهة. مفصولةٌ عن `FORMATS` أدناه لأن `InputKind`
/// يشترط `&'static [ChoiceOption]` ولا سبيل لاشتقاقه من بنيةٍ أغنى في `const`.
/// واختبارٌ في هذا الملف يقارن القائمتين، فانفصالُهما يسقط في البناء لا في يد
/// مستخدمٍ يرى خيارًا بلا شرح.
const FORMAT_OPTIONS: &[ChoiceOption] = &[
    ChoiceOption::new("jpeg", "choice.format.jpeg"),
    ChoiceOption::new("png", "choice.format.png"),
    ChoiceOption::new("tiff", "choice.format.tiff"),
    ChoiceOption::new("heic", "choice.format.heic"),
];

/// صيغةٌ معروضة بما يلزم عنها في التخطيط: قيمتها في الأمر، ومفتاح شرحها،
/// والامتدادات التي توافقها على macOS.
struct Format {
    /// ما يدخل الأمر بعد `-s format`. من الشيفرة المُترجَمة لا من الواجهة.
    value: &'static str,
    /// شرحُ هذه الصيغة بعينها. لكلٍّ مقايضتها، ورمزٌ واحد بشرحٍ واحد لأربعتها
    /// كان سيقول «هذه صيغة الصورة» — وهي جملةٌ لا تُعين أحدًا على الاختيار.
    explain_key: &'static str,
    /// الامتدادات التي يقبلها النظام لهذه الصيغة، بحروفٍ صغيرة.
    extensions: &'static [&'static str],
}

/// الصيغة الأولى هي ما تختاره الواجهة افتراضيًا حين لا يختار المستخدم.
const FORMATS: &[Format] = &[
    Format { value: "jpeg", explain_key: "explain.sips.format.jpeg", extensions: &["jpg", "jpeg"] },
    Format { value: "png", explain_key: "explain.sips.format.png", extensions: &["png"] },
    Format { value: "tiff", explain_key: "explain.sips.format.tiff", extensions: &["tif", "tiff"] },
    Format {
        value: "heic",
        explain_key: "explain.sips.format.heic",
        extensions: &["heic", "heif"],
    },
];

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.image.convert.title",
    description_key: "op.image.convert.description",
    category: Category::Images,
    // تكتب ملفًا جديدًا ولا تمسّ المصدر ولا تستبدل موجودًا.
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::SIPS,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("source", InputKind::ExistingFile),
        InputSpec::new("destination", InputKind::TargetDir),
        // `ext: None` لا `Some(...)`: الامتداد يتبع الصيغة التي يختارها
        // المستخدم لحظتَها، و`InputKind` ثابتٌ في المواصفة فلا يستطيع التعبير
        // عن «امتدادٌ يتبع حقلًا آخر». والبديل — إلحاقُه في `plan` — كان
        // سيجعل الاسم النهائي غير الاسم الذي كتبه المستخدم وقرأه في الشاشة.
        // فبقي الاختيار له، وبقي التحذير علينا حين لا يوافق.
        InputSpec::new("out_name", InputKind::NewName { ext: None }),
        InputSpec::new("format", InputKind::Choice { options: FORMAT_OPTIONS }),
    ],
    sort_order: 10,
    search_terms: &[
        "sips",
        "تحويل",
        "convert",
        "صيغة",
        "format",
        "صورة",
        "image",
        "jpeg",
        "jpg",
        "png",
        "tiff",
        "heic",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.file("source")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("out_name")?;
    let chosen = format_named(inputs.choice("format")?)?;

    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "out_name" } else { "destination" };
        e.on_input(field)
    })?;

    // الاسم النهائي هو المصدر نفسه. المقارنة مقارنة مواضع لا نصوص، فكلا
    // المسارين محلول الروابط. يُفحص **قبل** «الاسم مأخوذ» لأن الرسالتين
    // مختلفتان في ما تطلبانه: الأولى تقول «اختر اسمًا آخر»، وهذه تقول «هذا
    // ملفك الأصلي» — وهي وحدها التي تمنع المستخدم من الظنّ أن اسمًا ثالثًا
    // يحلّ المشكلة.
    if final_path == source {
        return Err(CoreError::SamePath.on_input("out_name"));
    }

    // `symlink_metadata` لا يتبع الروابط: رابطٌ معلَّق بالاسم النهائي تضاربٌ
    // كذلك، وترقيةُ الناتج فوقه كانت ستكتب حيث يشير لا حيث يقف.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("out_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    let argv = Argv::tool(tools::SIPS, "explain.sips.tool")
        .flag("-s", "explain.sips.set")
        .explained_value("format", "explain.sips.format_property")
        .explained_value(chosen.value, chosen.explain_key)
        .explained_path(source, "explain.role.image_source")
        .flag("--out", "explain.sips.out")
        .explained_path(&temp, "explain.role.temp")
        .warn_all(warnings_for(inputs, source, destination, name, chosen));

    argv.producing(Artifact::file(temp, final_path))
}

/// الصيغة المطابقة لقيمةٍ خرجت من القائمة المغلقة.
///
/// تعيد `Result` لا `unwrap`: القيمة مُتحقَّقة في `value::validate` مقابل
/// `FORMAT_OPTIONS`، لكن الجدولين منفصلان — فصيغةٌ تُضاف إلى الخيارات وتُنسى
/// هنا تصير رفضًا صريحًا منسوبًا إلى حقله، لا شرحًا خاطئًا لصيغةٍ أخرى.
/// واختبار `every_offered_format_is_described` يمنع الحالة من الوقوع أصلًا.
fn format_named(value: &str) -> Result<&'static Format> {
    FORMATS.iter().find(|f| f.value == value).ok_or(CoreError::WrongInputType { id: "format" })
}

/// امتداد الاسم كما يقرؤه النظام، بحروفٍ صغيرة. `None` إن لم يكن له امتداد.
///
/// يُقرأ من الاسم المكتوب لا من ملفٍ على القرص: لا ملف بعدُ لحظة التخطيط،
/// والسؤال أصلًا سؤالٌ عن الاسم — هل يوافق ما ستكتبه `sips` بداخله؟
fn extension_of(name: &str) -> Option<String> {
    Path::new(name).extension().map(|e| e.to_string_lossy().to_lowercase())
}

fn warnings_for(
    inputs: &Inputs,
    source: &Path,
    destination: &Path,
    name: &str,
    chosen: &Format,
) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    warnings.extend(warn_if_resolved(inputs, "source", source, "warn.source.resolved"));
    warnings.extend(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));

    // `sips` **لا تستنتج الصيغة من امتداد `--out`**. قيس: تحويلٌ إلى JPEG مع
    // ناتجٍ اسمه `x.png` يكتب بيانات JPEG داخل ملفٍ ينتهي بـ `.png`، فيُفتح
    // مشوّهًا في كل أداةٍ تثق بالامتداد. الأداة تسكت عن هذا (تكتفي بملاحظة
    // على الخرج)، فالتحذير قبل التنفيذ هو الموضع الوحيد الذي يُقرأ فيه.
    match extension_of(name) {
        None => warnings.push("warn.image.no_extension"),
        Some(ext) if !chosen.extensions.contains(&ext.as_str()) => {
            warnings.push("warn.image.format_extension")
        }
        Some(_) => {}
    }

    // ثابتان لا شرطيّان، وكلاهما خاصيةٌ في الأداة لا حالةٌ في الملف:
    warnings.push("warn.image.metadata");
    warnings.push("warn.image.suffix_notice");
    warnings
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
        name: &str,
        format: &str,
    ) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("source".to_owned(), RawValue::Path(source.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("out_name".to_owned(), RawValue::Text(name.to_owned())),
            ("format".to_owned(), RawValue::Text(format.to_owned())),
        ])
    }

    fn plan_with(
        source: &Path,
        destination: &Path,
        name: &str,
        format: &str,
    ) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(source, destination, name, format))?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    /// صورةٌ اصطناعية. محتواها لا يُقرأ في التخطيط — `sips` وحدها تقرؤه، وهي
    /// لا تعمل في الاختبارات — والمطلوب ملفٌ قائم باسمٍ ذي امتداد.
    fn image(s: &Scratch, name: &str) -> std::path::PathBuf {
        s.file(name, b"\x89PNG\r\n\x1a\n")
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("image.convert must be listed");
        assert_eq!(found.category, Category::Images);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_last_argument_is_the_temp() {
        let s = Scratch::new("img-convert-argv").unwrap();
        let src = image(&s, "الأصل.png");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&src, &dst, "الناتج.jpg", "jpeg").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/sips"));
        assert_eq!(args[0], "-s");
        assert_eq!(args[1], "format");
        assert_eq!(args[2], "jpeg");
        assert_eq!(Path::new(&args[3]), src.as_path());
        assert_eq!(args[4], "--out");
        let artifact = cmd.artifact.as_ref().unwrap();
        assert_eq!(Path::new(&args[5]), artifact.temp.as_path());
        assert_eq!(args.len(), 6);

        assert_eq!(artifact.final_path, dst.join("الناتج.jpg"));
        assert_eq!(artifact.kind, ArtifactKind::File);
        assert!(!artifact.temp.exists(), "planning must create nothing");
        assert!(cmd.stdout_to.is_none(), "sips writes through --out, not through stdout");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("img-convert-explain").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن.jpg", "jpeg").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_end_of_flags_separator_is_emitted_because_sips_does_not_understand_one() {
        // `--` بعد `sips` وسيطٌ زائد تحاول الأداة قراءته ملفًّا، لا حزام أمان.
        let s = Scratch::new("img-convert-noend").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن.png", "png").unwrap();
        assert!(!cmd.args.iter().any(|a| a == "--"), "{:?}", cmd.args);
    }

    #[test]
    fn every_offered_format_reaches_the_command_and_carries_its_own_explanation() {
        let s = Scratch::new("img-convert-formats").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        let mut keys = std::collections::HashSet::new();
        for option in FORMAT_OPTIONS {
            let cmd = plan_with(&src, &dst, "ن", option.value).unwrap();
            assert_eq!(cmd.args[2].to_string_lossy(), option.value);
            let key = cmd.explain[3].key.expect("the format value must be explained");
            assert!(keys.insert(key), "{} reuses the explanation {key}", option.value);
        }
    }

    #[test]
    fn the_offered_options_and_the_described_formats_are_the_same_list() {
        // جدولان منفصلان بالضرورة (‏`InputKind` يشترط `ChoiceOption`)، فالحارس
        // هنا هو ما يمنعهما من الانفصال معنًى بعد أن انفصلا شكلًا.
        let offered: Vec<&str> = FORMAT_OPTIONS.iter().map(|o| o.value).collect();
        let described: Vec<&str> = FORMATS.iter().map(|f| f.value).collect();
        assert_eq!(offered, described);
        for f in FORMATS {
            assert!(!f.extensions.is_empty(), "{} lists no extension", f.value);
            assert!(format_named(f.value).is_ok());
        }
        assert!(format_named("webp").is_err(), "webp is deliberately not offered");
    }

    #[test]
    fn a_format_the_specification_does_not_offer_never_reaches_the_command() {
        let s = Scratch::new("img-convert-badfmt").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");
        // WebP مرفوضة عند التحقّق لا عند التخطيط: القائمة مغلقة، والواجهة لا
        // تفعل إلا أن تختار بين قيمٍ من الشيفرة المُترجَمة.
        assert_eq!(
            refusal(plan_with(&src, &dst, "ن.webp", "webp")),
            ("err.input.type", Some("format"))
        );
    }

    #[test]
    fn writing_the_result_over_the_source_image_is_refused_by_name() {
        let s = Scratch::new("img-convert-same").unwrap();
        let src = image(&s, "صورة.png");
        // الوجهة هي مجلد المصدر، والاسم هو اسم المصدر: الناتج هو المدخل.
        assert_eq!(
            refusal(plan_with(&src, s.path(), "صورة.png", "jpeg")),
            ("err.path.same", Some("out_name"))
        );
    }

    #[test]
    fn an_existing_name_in_the_destination_stops_the_plan_before_anything_runs() {
        let s = Scratch::new("img-convert-exists").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود.jpg"), b"PRECIOUS").unwrap();

        assert_eq!(
            refusal(plan_with(&src, &dst, "موجود.jpg", "jpeg")),
            ("err.dest.exists", Some("out_name"))
        );
        assert_eq!(std::fs::read(dst.join("موجود.jpg")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn a_dangling_symlink_holding_the_final_name_is_a_conflict_too() {
        let s = Scratch::new("img-convert-dangling").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");
        std::os::unix::fs::symlink(dst.join("لا-وجود-له"), dst.join("رابط.jpg")).unwrap();

        assert_eq!(
            refusal(plan_with(&src, &dst, "رابط.jpg", "jpeg")),
            ("err.dest.exists", Some("out_name"))
        );
    }

    #[test]
    fn an_invalid_output_name_is_blamed_on_its_own_field() {
        let s = Scratch::new("img-convert-name").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");
        // اسمٌ يبدأ بنقطة يُنشئ ملفًا مخفيًا لا يجده المستخدم بعد أن ينتظره.
        assert_eq!(
            refusal(plan_with(&src, &dst, ".مخفي.jpg", "jpeg")),
            ("err.name.invalid", Some("out_name"))
        );
        // واسمٌ يحمل فاصل مسار ليس اسمًا.
        assert_eq!(
            refusal(plan_with(&src, &dst, "أعلى/ن.jpg", "jpeg")),
            ("err.name.invalid", Some("out_name"))
        );
    }

    #[test]
    fn a_name_whose_extension_contradicts_the_chosen_format_is_announced() {
        let s = Scratch::new("img-convert-mismatch").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        // قيس: `sips` تكتب بيانات JPEG داخل ملفٍ اسمه `.png` بلا اعتراض.
        let wrong = plan_with(&src, &dst, "ن.png", "jpeg").unwrap();
        assert!(wrong.warnings.contains(&"warn.image.format_extension"), "{:?}", wrong.warnings);

        // والامتدادات الموافقة لا تُطلق التحذير، وفيها البديل الشائع لكل صيغة.
        for (name, format) in [
            ("ن.jpg", "jpeg"),
            ("ن.jpeg", "jpeg"),
            ("ن.PNG", "png"),
            ("ن.tif", "tiff"),
            ("ن.heif", "heic"),
        ] {
            let cmd = plan_with(&src, &dst, name, format).unwrap();
            assert!(
                !cmd.warnings.contains(&"warn.image.format_extension"),
                "{name} with {format}: {:?}",
                cmd.warnings
            );
        }
    }

    #[test]
    fn a_name_with_no_extension_at_all_is_announced_as_its_own_case() {
        let s = Scratch::new("img-convert-noext").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "بلا امتداد", "jpeg").unwrap();
        assert!(cmd.warnings.contains(&"warn.image.no_extension"), "{:?}", cmd.warnings);
        assert!(!cmd.warnings.contains(&"warn.image.format_extension"), "{:?}", cmd.warnings);
    }

    #[test]
    fn the_metadata_and_suffix_notices_are_stated_on_every_plan() {
        // كلاهما خاصيةٌ في الأداة لا حالةٌ في الملف، فلا شرط يصحّ أن يُسكتهما:
        // الأولى تمنع فهم التحويل تنظيفًا، والثانية تفسّر ملاحظةً سيراها
        // المستخدم في سجل التشغيل عن الاسم المؤقّت الذي لم يكتبه هو.
        let s = Scratch::new("img-convert-always").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن.jpg", "jpeg").unwrap();
        assert!(cmd.warnings.contains(&"warn.image.metadata"), "{:?}", cmd.warnings);
        assert!(cmd.warnings.contains(&"warn.image.suffix_notice"), "{:?}", cmd.warnings);
    }

    #[test]
    fn a_symlinked_source_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("img-convert-symlink").unwrap();
        let real = image(&s, "الحقيقية.png");
        let link = s.path().join("رابط.png");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&link, &dst, "ن.jpg", "jpeg").unwrap();
        assert_eq!(Path::new(&cmd.args[3]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("img-convert-shellish").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        for name in
            ["صورة 'اليوم'.jpg", "a; rm -rf ~.jpg", "$(whoami).jpg", "back`tick`.jpg", "a & b.jpg"]
        {
            let cmd = plan_with(&src, &dst, name, "jpeg").unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(name));
            assert_eq!(cmd.args.len(), 6, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("img-convert-dashes").unwrap();
        let src = image(&s, "-rf.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "-x.jpg", "jpeg").unwrap();
        // الرايتان الوحيدتان يعلنهما هذا الملف: `-s` و`--out`. وما عداهما بيانات.
        for (i, a) in cmd.args.iter().enumerate() {
            if i == 0 || i == 4 {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "args[{i}] = {a:?} would be read as a flag");
        }
    }

    #[test]
    fn planning_leaves_nothing_behind_in_either_directory() {
        let s = Scratch::new("img-convert-clean").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&src, &dst, "ن.jpg", "jpeg").unwrap();
        }

        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
        assert!(src.exists(), "the source must not be touched");
    }
}
