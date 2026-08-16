//! تدوير صورة بزاوية قائمة، باستخدام `sips`.
//!
//! ## `-r` يدوّر **البكسلات**، لا وسم الاتجاه
//!
//! لتدوير الصور طريقتان تبدوان واحدة وتختلفان عند المستلِم:
//!
//! 1. **قلبُ وسم الاتجاه** (‏EXIF Orientation): بايتٌ واحد يُغيَّر، والبكسلات
//!    كما هي. سريعٌ وبلا خسارة — لكن كل برنامجٍ **لا يقرأ الوسم** يعرض الصورة
//!    بوضعها القديم. وذلك ليس نادرًا: كثيرٌ من المتصفّحات القديمة، وأدوات
//!    المعاينة في بعض أنظمة الرفع، ومعظم المكتبات حين تُستدعى بلا خيار.
//! 2. **تدوير البكسلات نفسها**: الصورة تُفكّ وتُدار وتُعاد كتابتها، فتخرج
//!    مدارةً في كل عارضٍ على وجه الأرض.
//!
//! `sips -r` هي الثانية. قيس على هذا الجهاز (‏macOS 26.6، بناء 25G72): صورة
//! ‎200×112‎ خرجت بعد `-r 90` بمقاس ‎112×200‎، وبعد `-r 270` كذلك، وبعد
//! `-r 180` بقيت ‎200×112‎ — أي أن الأبعاد نفسها تبدّلت، وهو ما لا يقع لو كان
//! المتغيّر وسمًا في الترويسة. ووسم الاتجاه في الناتج بقي `upper-left`.
//!
//! والاتجاه **مع عقارب الساعة**: `-r 90` يُنزل أعلى الصورة إلى يمينها.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/sips -r <الزاوية> <المصدر> --out <المؤقّت>
//! ```
//!
//! * `--out` — اكتب الناتج في مسارٍ آخر. **بدونها تدور `sips` الصورة في
//!   مكانها وتكتب فوق المصدر**، ولا رجعة. هذه الراية وحدها هي ما يجعل
//!   العملية `Creates` لا `Modifies`.
//!
//! ولا `--`: `sips` لا تفهم فاصل نهاية الرايات.
//!
//! ## الزوايا المعروضة: ثلاث، ولا حقل حرّ
//!
//! ‎90‎ و‎180‎ و‎270‎ فقط. `sips` تقبل زوايا أخرى، لكن زاويةً غير قائمة تُدخل
//! الصورة في مربّعٍ أكبر وتملأ الأركان بلونٍ لم يختره أحد — وهو ناتجٌ يفاجئ
//! من طلب «أدرها قليلًا». والقائمة المغلقة تمنع ذلك بالبناء لا بالتحذير.
//!
//! ولا زاوية سالبة: `-r -90` كانت ستصل الأداة رايةً لا قيمة، و`Argv` ترفض كل
//! قيمةٍ تبدأ بشرطة. و‎270‎ هي `-90` نفسها، فلا شيء ضاع.
//!
//! ## ما لا تفعله هذه العملية
//!
//! **لا تدوّر بلا خسارة.** التدوير بزاوية قائمة *يمكن* أن يكون بلا خسارة على
//! JPEG (‏`jpegtran -rotate` تفعلها بإعادة ترتيب كتل ‎8×8‎ بلا فكّ ترميز)،
//! لكن `sips` تفكّ وتعيد الترميز في كل حال — فالصورة تخسر جيلًا من الجودة
//! على ملفٍ لم يتغيّر محتواه في الحقيقة. `jpegtran` ليست في النظام، وإدخالها
//! يعني أداةً خارج جذور النظام وهو قرارٌ في `tools.rs` لا هنا. فبقيت `sips`،
//! وبقي القول صريحًا في التحذير بدل أن يُفهم الصمت وعدًا.
//!
//! ولا تغيّر الصيغة، ولا تكتب فوق المصدر، ولا تستبدل اسمًا مأخوذًا.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;
use std::path::Path;

pub const ID: &str = "image.rotate";

/// الزوايا القائمة وحدها. القيم بلا إشارة: `Argv` ترفض ما يبدأ بشرطة، و‎270‎
/// تغني عن `-90`.
const DEGREE_OPTIONS: &[ChoiceOption] = &[
    ChoiceOption::new("90", "choice.degrees.90"),
    ChoiceOption::new("180", "choice.degrees.180"),
    ChoiceOption::new("270", "choice.degrees.270"),
];

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.image.rotate.title",
    description_key: "op.image.rotate.description",
    category: Category::Images,
    // `--out` هي ما يجعلها كذلك: بدونها كانت `sips -r` تعدّل المصدر مكانه.
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::SIPS,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("source", InputKind::ExistingFile),
        InputSpec::new("destination", InputKind::TargetDir),
        // الصيغة لا تتغيّر بالتدوير، فالامتداد يتبع المصدر — ولا تستطيع
        // المواصفة إعلانه لأنه يتبع ملفًا يختاره المستخدم لا قيمةً ثابتة.
        InputSpec::new("out_name", InputKind::NewName { ext: None }),
        InputSpec::new("degrees", InputKind::Choice { options: DEGREE_OPTIONS }),
    ],
    sort_order: 30,
    search_terms: &[
        "sips",
        "تدوير",
        "rotate",
        "زاوية",
        "قلب",
        "اتجاه",
        "orientation",
        "صورة",
        "image",
    ],
    plan,
};

/// شرحُ كل زاوية على حدة. زاويةٌ بشرحٍ واحد كانت ستقول «هذه درجة التدوير»،
/// وهي جملةٌ لا تجيب عن السؤال الوحيد الذي يُسأل هنا: مع العقارب أم عكسها؟
fn explanation_for(degrees: &str) -> Result<&'static str> {
    match degrees {
        "90" => Ok("explain.sips.rotate.90"),
        "180" => Ok("explain.sips.rotate.180"),
        "270" => Ok("explain.sips.rotate.270"),
        // لا يقع: القائمة مغلقة و`value::validate` تفرضها. رفضٌ صريح منسوبٌ
        // إلى حقله أصدق من شرحٍ يخصّ زاويةً أخرى، واختبارٌ أدناه يمنع الحالة.
        _ => Err(CoreError::WrongInputType { id: "degrees" }),
    }
}

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.file("source")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("out_name")?;
    let degrees = inputs.choice("degrees")?;
    let explanation = explanation_for(degrees)?;

    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "out_name" } else { "destination" };
        e.on_input(field)
    })?;

    // الناتج هو المصدر نفسه: تدويرٌ في المكان تحت اسم عملية «لا تمسّ المصدر».
    // يُفحص قبل «الاسم مأخوذ» كي تقول الرسالة ما يقع فعلًا.
    if final_path == source {
        return Err(CoreError::SamePath.on_input("out_name"));
    }

    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("out_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    let argv = Argv::tool(tools::SIPS, "explain.sips.tool")
        .flag("-r", "explain.sips.rotate")
        .explained_value(degrees, explanation)
        .explained_path(source, "explain.role.image_source")
        .flag("--out", "explain.sips.out")
        .explained_path(&temp, "explain.role.temp")
        .warn_all(warnings_for(inputs, source, destination, name));

    argv.producing(Artifact::file(temp, final_path))
}

/// امتداد الاسم كما يقرؤه النظام، بحروفٍ صغيرة وبعد توحيد المترادف.
fn extension_of(name: &str) -> Option<String> {
    Path::new(name).extension().map(|e| canonical_extension(&e.to_string_lossy().to_lowercase()))
}

/// يوحّد أسماء الامتداد المترادفة، فـ`jpg` و`jpeg` صيغةٌ واحدة لا صيغتان.
fn canonical_extension(ext: &str) -> String {
    match ext {
        "jpeg" => "jpg",
        "tiff" => "tif",
        "heif" => "heic",
        other => other,
    }
    .to_owned()
}

fn warnings_for(
    inputs: &Inputs,
    source: &Path,
    destination: &Path,
    name: &str,
) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    warnings.extend(warn_if_resolved(inputs, "source", source, "warn.source.resolved"));
    warnings.extend(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));

    // التدوير لا يغيّر الصيغة، فامتدادٌ مخالف لامتداد المصدر يكذب على كل
    // أداةٍ تثق بالامتداد — و`sips` لا تعترض، تكتفي بملاحظةٍ على الخرج.
    match (extension_of(name), source.extension()) {
        (None, _) => warnings.push("warn.image.no_extension"),
        (Some(chosen), Some(origin)) => {
            let origin = canonical_extension(&origin.to_string_lossy().to_lowercase());
            if chosen != origin {
                warnings.push("warn.image.source_extension");
            }
        }
        (Some(_), None) => {}
    }

    // ثوابت لا شروط: إعادة الترميز خاصيةٌ في `sips` لا حالةٌ في هذا الملف،
    // وكذلك ما يعبر من البيانات الوصفية، وكذلك ملاحظة الامتداد على الخرج.
    warnings.push("warn.image.recompress");
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
        degrees: &str,
    ) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("source".to_owned(), RawValue::Path(source.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("out_name".to_owned(), RawValue::Text(name.to_owned())),
            ("degrees".to_owned(), RawValue::Text(degrees.to_owned())),
        ])
    }

    fn plan_with(
        source: &Path,
        destination: &Path,
        name: &str,
        degrees: &str,
    ) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(source, destination, name, degrees))?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    fn image(s: &Scratch, name: &str) -> std::path::PathBuf {
        s.file(name, b"\x89PNG\r\n\x1a\n")
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("image.rotate must be listed");
        assert_eq!(found.category, Category::Images);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_last_argument_is_the_temp() {
        let s = Scratch::new("img-rotate-argv").unwrap();
        let src = image(&s, "الأصل.png");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&src, &dst, "مُدارة.png", "90").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/sips"));
        assert_eq!(args[0], "-r");
        assert_eq!(args[1], "90");
        assert_eq!(Path::new(&args[2]), src.as_path());
        assert_eq!(args[3], "--out");
        let artifact = cmd.artifact.as_ref().unwrap();
        assert_eq!(Path::new(&args[4]), artifact.temp.as_path());
        assert_eq!(args.len(), 5);

        assert_eq!(artifact.final_path, dst.join("مُدارة.png"));
        assert_eq!(artifact.kind, ArtifactKind::File);
        assert!(!artifact.temp.exists(), "planning must create nothing");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("img-rotate-explain").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن.png", "180").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_offered_angle_reaches_the_command_and_carries_its_own_explanation() {
        let s = Scratch::new("img-rotate-angles").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        let mut keys = std::collections::HashSet::new();
        for option in DEGREE_OPTIONS {
            let cmd = plan_with(&src, &dst, "ن.png", option.value).unwrap();
            assert_eq!(cmd.args[1].to_string_lossy(), option.value);
            let key = cmd.explain[2].key.expect("the angle must be explained");
            assert!(keys.insert(key), "{} reuses the explanation {key}", option.value);
            assert!(explanation_for(option.value).is_ok());
        }
        // زاويةٌ لا تعرضها المواصفة لا شرح لها، فترفَض بدل أن تُشرح خطأً.
        assert!(explanation_for("45").is_err());
    }

    #[test]
    fn an_angle_the_specification_does_not_offer_never_reaches_the_command() {
        let s = Scratch::new("img-rotate-badangle").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        // زاويةٌ غير قائمة تملأ الأركان بلونٍ لم يختره أحد، وسالبةٌ تُقرأ راية.
        for bad in ["45", "-90", "360", "0", "٩٠"] {
            assert_eq!(
                refusal(plan_with(&src, &dst, "ن.png", bad)),
                ("err.input.type", Some("degrees")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn no_offered_angle_could_ever_be_read_as_a_flag() {
        // ‎270‎ معروضة بدل `-90` تحديدًا لهذا: القيمة السالبة كانت ستصل الأداة
        // رايةً، و`Argv::explained_value` كانت سترفض الأمر كلّه.
        for option in DEGREE_OPTIONS {
            assert!(!option.value.starts_with('-'), "{} could be read as a flag", option.value);
        }
    }

    #[test]
    fn writing_the_result_over_the_source_image_is_refused_by_name() {
        let s = Scratch::new("img-rotate-same").unwrap();
        let src = image(&s, "صورة.png");
        assert_eq!(
            refusal(plan_with(&src, s.path(), "صورة.png", "90")),
            ("err.path.same", Some("out_name"))
        );
    }

    #[test]
    fn an_existing_name_in_the_destination_stops_the_plan_before_anything_runs() {
        let s = Scratch::new("img-rotate-exists").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجودة.png"), b"PRECIOUS").unwrap();

        assert_eq!(
            refusal(plan_with(&src, &dst, "موجودة.png", "90")),
            ("err.dest.exists", Some("out_name"))
        );
        assert_eq!(std::fs::read(dst.join("موجودة.png")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn an_invalid_output_name_is_blamed_on_its_own_field() {
        let s = Scratch::new("img-rotate-name").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        assert_eq!(
            refusal(plan_with(&src, &dst, ".مخفية.png", "90")),
            ("err.name.invalid", Some("out_name"))
        );
        assert_eq!(
            refusal(plan_with(&src, &dst, "أعلى/ن.png", "90")),
            ("err.name.invalid", Some("out_name"))
        );
    }

    #[test]
    fn a_name_whose_extension_contradicts_the_source_format_is_announced() {
        let s = Scratch::new("img-rotate-ext").unwrap();
        let src = image(&s, "الأصل.png");
        let dst = s.dir("و");

        let lying = plan_with(&src, &dst, "مُدارة.jpg", "90").unwrap();
        assert!(lying.warnings.contains(&"warn.image.source_extension"), "{:?}", lying.warnings);

        let honest = plan_with(&src, &dst, "مُدارة.PNG", "90").unwrap();
        assert!(!honest.warnings.contains(&"warn.image.source_extension"), "{:?}", honest.warnings);
    }

    #[test]
    fn a_name_with_no_extension_at_all_is_announced_as_its_own_case() {
        let s = Scratch::new("img-rotate-noext").unwrap();
        let src = image(&s, "الأصل.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "بلا امتداد", "90").unwrap();
        assert!(cmd.warnings.contains(&"warn.image.no_extension"), "{:?}", cmd.warnings);
        assert!(!cmd.warnings.contains(&"warn.image.source_extension"), "{:?}", cmd.warnings);
    }

    #[test]
    fn the_re_encoding_notice_is_stated_on_every_plan() {
        // التدوير بزاوية قائمة *يمكن* أن يكون بلا خسارة، و`sips` لا تفعله
        // كذلك. الصمت هنا كان سيُقرأ وعدًا بما لا يقع.
        let s = Scratch::new("img-rotate-lossy").unwrap();
        let src = image(&s, "م.jpg");
        let dst = s.dir("و");

        for angle in ["90", "180", "270"] {
            let cmd = plan_with(&src, &dst, "ن.jpg", angle).unwrap();
            assert!(cmd.warnings.contains(&"warn.image.recompress"), "{angle}: {:?}", cmd.warnings);
            assert!(cmd.warnings.contains(&"warn.image.metadata"), "{:?}", cmd.warnings);
            assert!(cmd.warnings.contains(&"warn.image.suffix_notice"), "{:?}", cmd.warnings);
        }
    }

    #[test]
    fn a_symlinked_source_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("img-rotate-symlink").unwrap();
        let real = image(&s, "الحقيقية.png");
        let link = s.path().join("رابط.png");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&link, &dst, "ن.png", "90").unwrap();
        assert_eq!(Path::new(&cmd.args[2]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("img-rotate-shellish").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        for name in
            ["صورة 'اليوم'.png", "a; rm -rf ~.png", "$(whoami).png", "back`tick`.png", "a & b.png"]
        {
            let cmd = plan_with(&src, &dst, name, "90").unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(name));
            assert_eq!(cmd.args.len(), 5, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("img-rotate-dashes").unwrap();
        let src = image(&s, "-rf.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "-x.png", "90").unwrap();
        // الرايتان الوحيدتان يعلنهما هذا الملف: `-r` و`--out`.
        for (i, a) in cmd.args.iter().enumerate() {
            if i == 0 || i == 3 {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "args[{i}] = {a:?} would be read as a flag");
        }
    }

    #[test]
    fn planning_leaves_nothing_behind_in_either_directory() {
        let s = Scratch::new("img-rotate-clean").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&src, &dst, "ن.png", "90").unwrap();
        }

        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
        assert!(src.exists(), "the source must not be touched");
    }
}
