//! تصغير صورة بحدٍّ على ضلعها الأطول، باستخدام `sips`.
//!
//! ## `-Z` لا `-z`: الفرق بين حدٍّ ونسبةٍ محفوظة وبين تشويه
//!
//! لـ`sips` رايتان متشابهتان في الشكل مختلفتان في الأثر، والفرق بينهما حرفٌ
//! واحد — وهو بالضبط نوع الخطأ الذي لا يلاحظه أحد إلا بعد أن يرى الصورة:
//!
//! * `-Z <عدد>` — **`resampleHeightWidthMax`**: يحدّ **الضلع الأطول** عند
//!   العدد، ويحسب الضلع الآخر بالنسبة نفسها. الشكل محفوظ.
//! * `-z <ارتفاع> <عرض>` — **`resampleHeightWidth`**: يفرض المقاسين معًا.
//!   قيس: صورة ‎200×112‎ مع `-z 300 300` صارت ‎300×300‎ — أي مسطوحة.
//!
//! هذه العملية تعلن حقلًا واحدًا لأنها تستعمل `-Z` وحدها. ولو أرادت `-z`
//! لاحتاجت حقلين، وأنتجت صورًا مشوّهة لكل من يملأهما بلا حساب النسبة في
//! رأسه. الحدّ الواحد هو ما يريده تسعةٌ من عشرة: «اجعلها أصغر من كذا».
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/sips -Z <الحدّ> <المصدر> --out <المؤقّت>
//! ```
//!
//! * `--out` — اكتب الناتج في مسارٍ آخر. **بدونها تكتب `sips` فوق المصدر**،
//!   فتضيع الصورة الأصلية بلا رجعة. غيابُها هو الفرق بين هذه العملية وبين
//!   تدميرٍ صامت، ولذلك هي معلَنة في الشرح لا مطويّة.
//!
//! ولا `--`: `sips` لا تفهم فاصل نهاية الرايات، وإضافته كانت ستصير وسيطًا
//! زائدًا تحاول الأداة قراءته ملفًّا.
//!
//! ## التكبير: `-Z` **يكبّر**، خلافًا للشائع
//!
//! يُقال إن `-Z` لا يتجاوز المقاس الأصلي. قيس على هذا الجهاز
//! (‏macOS 26.6، بناء 25G72) وكان الأمر عكس ذلك: صورة ‎200×112‎ مع `-Z 900`
//! خرجت ‎900×504‎ — والنسبة محفوظة، لكن البكسلات مخترَعة. تكرّر القياس على
//! PNG وJPEG معًا.
//!
//! فالتحذير هنا يقول ما يقع فعلًا: رقمٌ أكبر من الضلع الأطول **يكبّر** الصورة،
//! والتكبير لا يضيف تفصيلًا لم يكن فيها.
//!
//! **ولماذا يُعلَن دائمًا لا عند التكبير وحده:** لمعرفة أن هذا الرقم أكبر من
//! ضلع الصورة يلزم معرفة أبعادها، ولا سبيل إليها لحظة التخطيط إلا بتشغيل
//! `sips -g` على الملف — أي تشغيل الأداة كي نقرّر هل نحذّر من الأداة. وهذا
//! المنتج لا يشغّل شيئًا قبل أن يعرض الأمر ويوافق المستخدم. البديل الآخر —
//! قراءة ترويسات الصيغ بأنفسنا — يعمل على PNG وJPEG ويتعثّر على HEIC وTIFF،
//! فينتج تحذيرًا يظهر ويغيب بحسب الصيغة، وهو أسوأ من تحذيرٍ ثابت.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا **تغيّر الصيغة**: `-Z` يعيد الترميز بصيغة المصدر نفسها. قيس: تصغيرُ
//! PNG إلى ملفٍ سمّيناه `x.jpg` أنتج **بيانات PNG داخل اسمٍ ينتهي بـ `.jpg`**.
//! لذلك يُقارَن امتداد الاسم بامتداد المصدر ويُحذَّر عند الاختلاف. من يريد
//! تغيير الصيغة فعلًا فله `image.convert`.
//!
//! ولا تكتب فوق المصدر، ولا تستبدل اسمًا مأخوذًا. والوسيط الأخير مؤقّت لا
//! نهائي: لا يُرقّى إلى اسمه إلا بعد خروجٍ ناجح.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;
use std::path::Path;

pub const ID: &str = "image.resize";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.image.resize.title",
    description_key: "op.image.resize.description",
    category: Category::Images,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::SIPS,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("source", InputKind::ExistingFile),
        InputSpec::new("destination", InputKind::TargetDir),
        // الصيغة لا تتغيّر، فالامتداد يجب أن يبقى امتداد المصدر — ولا سبيل
        // لإعلانه هنا لأنه يتبع الملف الذي سيختاره المستخدم لا المواصفة.
        // فبقي `None`، وبقي على التخطيط أن يحذّر عند الاختلاف.
        InputSpec::new("out_name", InputKind::NewName { ext: None }),
        // الحدّ الأدنى ‎16‎: أصغر منه ليس صورةً يُنظر إليها بل أيقونة، ولها
        // أدواتها. والأعلى ‎20000‎: فوق أكبر مستشعرٍ متداول، وتحته يبقى الناتج
        // في حدود ما تفتحه أدوات العرض بلا اختناق.
        // والافتراضي ‎1600‎: يكفي شاشةً كاملة ومرفقَ بريدٍ معًا.
        InputSpec::new("max_pixels", InputKind::Number { min: 16, max: 20_000, default: 1600 }),
    ],
    sort_order: 20,
    search_terms: &[
        "sips",
        "تصغير",
        "resize",
        "أبعاد",
        "مقاس",
        "scale",
        "صورة",
        "image",
        "thumbnail",
        "مصغّرة",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.file("source")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("out_name")?;
    let max_pixels = inputs.number("max_pixels")?;

    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "out_name" } else { "destination" };
        e.on_input(field)
    })?;

    // الناتج هو المصدر نفسه. يُفحص قبل «الاسم مأخوذ» لأن الرسالتين تطلبان
    // شيئين مختلفين: تلك تقول «اختر اسمًا آخر»، وهذه تقول «هذه صورتك الأصلية»
    // — ولولا التفريق لظنّ المستخدم أن اسمًا ثالثًا يحلّ ما ليس بمشكلة اسم.
    if final_path == source {
        return Err(CoreError::SamePath.on_input("out_name"));
    }

    // لا يتبع الروابط: رابطٌ معلَّق يحمل الاسم النهائي تضاربٌ كذلك.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("out_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    // `max_pixels` مُتحقَّق في `value::validate` ضمن ‎16..=20000‎، فلا يبدأ
    // بشرطة أبدًا — و`Argv::explained_value` ترفض ما يبدأ بها على أي حال.
    let argv = Argv::tool(tools::SIPS, "explain.sips.tool")
        .flag("-Z", "explain.sips.resample_max")
        .explained_value(max_pixels.to_string(), "explain.sips.max_pixels")
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
///
/// بدونه كان تصغيرُ `صورة.jpeg` إلى `مصغّرة.jpg` يُطلق تحذير «الامتداد يخالف
/// المصدر» عن اسمين يشيران إلى الصيغة نفسها — وتحذيرٌ يكذب مرّةً لا يُقرأ
/// في المرّة التي يصدق فيها.
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

    // الصيغة لا تتغيّر مع `-Z`، فامتدادٌ مخالف لامتداد المصدر يكذب على كل
    // أداةٍ تثق بالامتداد. تُقارَن الصيغتان بعد توحيد المترادف، ولا يُقارَن
    // شيء إن كان المصدر نفسه بلا امتداد.
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

    // ثوابت لا شروط، وكلٌّ منها خاصيةٌ في الأداة لا حالةٌ في هذا الملف:
    // التكبير (‏لا نعرف الأبعاد قبل التشغيل)، والبيانات الوصفية، وملاحظة
    // الامتداد التي تطبعها `sips` عن الاسم المؤقّت.
    warnings.push("warn.image.upscale");
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
        max_pixels: &str,
    ) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("source".to_owned(), RawValue::Path(source.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("out_name".to_owned(), RawValue::Text(name.to_owned())),
            ("max_pixels".to_owned(), RawValue::Text(max_pixels.to_owned())),
        ])
    }

    fn plan_with(
        source: &Path,
        destination: &Path,
        name: &str,
        max_pixels: &str,
    ) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(source, destination, name, max_pixels))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("image.resize must be listed");
        assert_eq!(found.category, Category::Images);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_last_argument_is_the_temp() {
        let s = Scratch::new("img-resize-argv").unwrap();
        let src = image(&s, "الأصل.png");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&src, &dst, "مصغّرة.png", "1600").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/sips"));
        assert_eq!(args[0], "-Z", "the capital Z bounds the longest side; -z would distort");
        assert_eq!(args[1], "1600");
        assert_eq!(Path::new(&args[2]), src.as_path());
        assert_eq!(args[3], "--out");
        let artifact = cmd.artifact.as_ref().unwrap();
        assert_eq!(Path::new(&args[4]), artifact.temp.as_path());
        assert_eq!(args.len(), 5);

        assert_eq!(artifact.final_path, dst.join("مصغّرة.png"));
        assert_eq!(artifact.kind, ArtifactKind::File);
        assert!(!artifact.temp.exists(), "planning must create nothing");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("img-resize-explain").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن.png", "800").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn the_lowercase_z_that_would_distort_the_picture_is_never_emitted() {
        // `-z` يفرض المقاسين معًا فيسطح الصورة. حرفٌ واحد يفرّق بينهما، وهو
        // نوع الخطأ الذي لا يُرى إلا في الناتج.
        let s = Scratch::new("img-resize-case").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن.png", "640").unwrap();
        assert!(!cmd.args.iter().any(|a| a == "-z"), "{:?}", cmd.args);
        assert!(!cmd.args.iter().any(|a| a == "--"), "sips does not understand --");
    }

    #[test]
    fn the_bound_reaches_the_command_exactly_as_the_user_gave_it() {
        let s = Scratch::new("img-resize-number").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        for value in ["16", "1600", "20000"] {
            let cmd = plan_with(&src, &dst, "ن.png", value).unwrap();
            assert_eq!(cmd.args[1].to_string_lossy(), value);
        }
    }

    #[test]
    fn a_bound_outside_the_declared_range_is_refused_and_names_its_field() {
        let s = Scratch::new("img-resize-range").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        // `-1` بينها عمدًا: يُقرأ عددًا صحيحًا ويُرفض بالمدى، فلا يبلغ
        // `Argv` أصلًا كي يُسأل هل يبدو رايةً — والحارسان معًا لا أحدهما.
        for outside in ["15", "0", "-1", "20001", "99999"] {
            assert_eq!(
                refusal(plan_with(&src, &dst, "ن.png", outside)),
                ("err.input.range", Some("max_pixels")),
                "{outside} must be refused"
            );
        }
        // ونصٌّ ليس عددًا لا يصير وسيطًا بأي حال.
        for bad in ["16; rm -rf ~", "١٦٠٠", "", "1600.5"] {
            assert_eq!(
                refusal(plan_with(&src, &dst, "ن.png", bad)),
                ("err.input.type", Some("max_pixels")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn writing_the_result_over_the_source_image_is_refused_by_name() {
        let s = Scratch::new("img-resize-same").unwrap();
        let src = image(&s, "صورة.png");
        assert_eq!(
            refusal(plan_with(&src, s.path(), "صورة.png", "800")),
            ("err.path.same", Some("out_name"))
        );
    }

    #[test]
    fn an_existing_name_in_the_destination_stops_the_plan_before_anything_runs() {
        let s = Scratch::new("img-resize-exists").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجودة.png"), b"PRECIOUS").unwrap();

        assert_eq!(
            refusal(plan_with(&src, &dst, "موجودة.png", "800")),
            ("err.dest.exists", Some("out_name"))
        );
        assert_eq!(std::fs::read(dst.join("موجودة.png")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn an_invalid_output_name_is_blamed_on_its_own_field() {
        let s = Scratch::new("img-resize-name").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        assert_eq!(
            refusal(plan_with(&src, &dst, ".مخفية.png", "800")),
            ("err.name.invalid", Some("out_name"))
        );
        assert_eq!(
            refusal(plan_with(&src, &dst, "أعلى/ن.png", "800")),
            ("err.name.invalid", Some("out_name"))
        );
    }

    #[test]
    fn a_name_whose_extension_contradicts_the_source_format_is_announced() {
        let s = Scratch::new("img-resize-ext").unwrap();
        let src = image(&s, "الأصل.png");
        let dst = s.dir("و");

        // قيس: `-Z` لا يحوّل الصيغة، فهذا ملف PNG باسمٍ ينتهي بـ `.jpg`.
        let lying = plan_with(&src, &dst, "مصغّرة.jpg", "800").unwrap();
        assert!(lying.warnings.contains(&"warn.image.source_extension"), "{:?}", lying.warnings);

        // ونفس الامتداد — بأي حالة أحرف — لا يُطلق شيئًا.
        for name in ["مصغّرة.png", "مصغّرة.PNG"] {
            let cmd = plan_with(&src, &dst, name, "800").unwrap();
            assert!(!cmd.warnings.contains(&"warn.image.source_extension"), "{name}");
        }
    }

    #[test]
    fn synonymous_extensions_are_one_format_not_two() {
        let s = Scratch::new("img-resize-synonym").unwrap();
        let src = image(&s, "الأصل.jpeg");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "مصغّرة.jpg", "800").unwrap();
        assert!(
            !cmd.warnings.contains(&"warn.image.source_extension"),
            "jpg and jpeg name the same format: {:?}",
            cmd.warnings
        );
    }

    #[test]
    fn a_name_with_no_extension_at_all_is_announced_as_its_own_case() {
        let s = Scratch::new("img-resize-noext").unwrap();
        let src = image(&s, "الأصل.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "بلا امتداد", "800").unwrap();
        assert!(cmd.warnings.contains(&"warn.image.no_extension"), "{:?}", cmd.warnings);
        assert!(!cmd.warnings.contains(&"warn.image.source_extension"), "{:?}", cmd.warnings);
    }

    #[test]
    fn the_enlargement_notice_is_stated_on_every_plan_including_a_shrink() {
        // لا سبيل لمعرفة أبعاد الصورة لحظة التخطيط إلا بتشغيل الأداة، وهو ما
        // لا يقع قبل موافقة المستخدم. فالتحذير وصفُ سلوكٍ ثابت لا استنتاجُ حالة.
        let s = Scratch::new("img-resize-upscale").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        for bound in ["16", "20000"] {
            let cmd = plan_with(&src, &dst, "ن.png", bound).unwrap();
            assert!(cmd.warnings.contains(&"warn.image.upscale"), "{bound}: {:?}", cmd.warnings);
            assert!(cmd.warnings.contains(&"warn.image.metadata"), "{:?}", cmd.warnings);
            assert!(cmd.warnings.contains(&"warn.image.suffix_notice"), "{:?}", cmd.warnings);
        }
    }

    #[test]
    fn a_symlinked_source_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("img-resize-symlink").unwrap();
        let real = image(&s, "الحقيقية.png");
        let link = s.path().join("رابط.png");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&link, &dst, "ن.png", "800").unwrap();
        assert_eq!(Path::new(&cmd.args[2]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("img-resize-shellish").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        for name in
            ["صورة 'اليوم'.png", "a; rm -rf ~.png", "$(whoami).png", "back`tick`.png", "a & b.png"]
        {
            let cmd = plan_with(&src, &dst, name, "800").unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(name));
            assert_eq!(cmd.args.len(), 5, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("img-resize-dashes").unwrap();
        let src = image(&s, "-rf.png");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "-x.png", "800").unwrap();
        // الرايتان الوحيدتان يعلنهما هذا الملف: `-Z` و`--out`.
        for (i, a) in cmd.args.iter().enumerate() {
            if i == 0 || i == 3 {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "args[{i}] = {a:?} would be read as a flag");
        }
    }

    #[test]
    fn planning_leaves_nothing_behind_in_either_directory() {
        let s = Scratch::new("img-resize-clean").unwrap();
        let src = image(&s, "م.png");
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&src, &dst, "ن.png", "800").unwrap();
        }

        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
        assert!(src.exists(), "the source must not be touched");
    }
}
