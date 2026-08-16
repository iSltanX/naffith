//! قراءة بيانات صورة: أبعادها ودقّتها وفضاء ألوانها وصيغتها، بـ`sips -g all`.
//!
//! ## عمليةُ قراءةٍ محضة
//!
//! `-g` تعني `--getProperty`: تسأل ولا تكتب. لا `--out` هنا ولا ملف مؤقّت ولا
//! ترقية، ولذلك `danger: Safe` و`conflict: NoArtifact` — لا اسم نهائي تحرسه
//! لأنها لا تكتب اسمًا. والفهرس يفرض هذا الاقتران في `registry.rs` بدل أن
//! يتركه لانتباه من يكتب العملية.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/sips -g all <المصدر>
//! ```
//!
//! `all` قيمةُ الخاصية المطلوبة، لا راية: `-g` تأخذ اسم خاصيةٍ واحدة
//! (‏`pixelWidth` مثلًا)، و`all` هي الكلمة التي تطلبها جميعًا. طلبُ خاصيةٍ
//! واحدة كان سيحتاج حقلًا يختارها، ثم شاشةً تُقرأ سطرًا واحدًا — والسؤال الذي
//! يُطرح فعلًا أمام صورة مجهولة هو «ما هذه؟» لا «كم عرضها بالضبط؟».
//!
//! ولا `--`: `sips` لا تفهم فاصل نهاية الرايات، وإضافته وسيطٌ زائد تحاول
//! الأداة قراءته ملفًّا. المسار مطلقٌ فيبدأ بـ `/` ولا يُقرأ رايةً على أي حال.
//!
//! ## ما يظهر فعلًا
//!
//! قيس على هذا الجهاز (‏macOS 26.6، بناء 25G72) على ملف JPEG:
//!
//! ```text
//!   pixelWidth: 200
//!   pixelHeight: 112
//!   typeIdentifier: public.jpeg
//!   format: jpeg
//!   formatOptions: default
//!   dpiWidth: 72.000
//!   dpiHeight: 72.000
//!   samplesPerPixel: 3
//!   bitsPerSample: 8
//!   hasAlpha: no
//!   space: RGB
//!   profile: sRGB IEC61966-2.1
//! ```
//!
//! ## حدٌّ يُقال ولا يُسكت عنه: الملف الذي ليس صورة
//!
//! `sips -g all` **لا تفشل** على ملفٍ ليس صورة. قيس على ملفٍ نصّي: خرجت
//! بالرمز صفر وطبعت `format: txt` و`dpiWidth: 72.000` و`hasAlpha: no` — أي
//! دقّةً مخترَعة لملفٍ لا دقّة له، بلا `pixelWidth` ولا `space` ولا `profile`.
//!
//! ولا نفحص «هل هذا صورة؟» قبل التشغيل: الفحص الصادق يعني فكّ الملف — أي
//! تشغيل الأداة قبل عرض الأمر والموافقة عليه، وهو ما لا يقع في هذا المنتج.
//! وفحصُ الامتداد وحده كان سيرفض صورةً سليمة بلا امتداد ويقبل ملفًّا نصيًّا
//! اسمه `x.png`. فبقي الأمر كما هو، وبقي الوصف يقول إن السجلّ شبه الفارغ
//! جوابٌ صحيح عن ملفٍ ليس صورة، لا عطلٌ في الأداة.
//!
//! ## «إظهار في Finder»: الملف نفسه لا مجلده
//!
//! العملية لا تُنتج ناتجًا، فسؤال «أين أنظر؟» يحتاج جوابًا معلَنًا —
//! و`reveal_target` هو موضعه. الجواب هنا **الصورة نفسها**، لسببين:
//!
//! 1. `open -R` تُبرز ما يُعطى لها **داخل مجلده الحاوي**. إعطاؤها الصورة يفتح
//!    مجلد الصورة والصورةُ محدَّدة فيه؛ وإعطاؤها المجلد يفتح مجلدَ المجلد —
//!    أي خطوةً أبعد عمّا ينظر إليه المستخدم.
//! 2. `reveal::resolve_target` يعيد التحقّق من المسار بـ`paths::existing_file`،
//!    وهي ترفض المجلدات بـ`PathMissing`. فمجلدٌ هنا كان يعني زرًّا يفشل
//!    بـ«المسار غير موجود» بعد تشغيلٍ نجح — وهي أسوأ رسالةٍ ممكنة لأنها تكذب.
//!
//! والمسار يمرّ بالسياسة كاملةً في `reveal.rs` رغم أنه صادرٌ عن النواة: كونه
//! من عندنا لا يجعله فوق الفحص.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "image.info";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.image.info.title",
    description_key: "op.image.info.description",
    category: Category::Images,
    // لا تكتب بايتًا واحدًا: `-g` تسأل ولا تضبط.
    danger: Danger::Safe,
    // يفرضه `registry.rs` على كل `Safe`: لا اسم نهائي فلا تضارب.
    conflict: Conflict::NoArtifact,
    visibility: Visibility::Production,
    tool: tools::SIPS,
    inputs: &[InputSpec::new("source", InputKind::ExistingFile)],
    sort_order: 40,
    search_terms: &[
        "sips",
        "معلومات",
        "info",
        "أبعاد",
        "dimensions",
        "دقة",
        "dpi",
        "صورة",
        "image",
        "exif",
        "profile",
        "ملف تعريف الألوان",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.file("source")?;

    let mut argv = Argv::tool(tools::SIPS, "explain.sips.tool")
        .flag("-g", "explain.sips.get")
        .explained_value("all", "explain.sips.get_all")
        .explained_path(source, "explain.role.image_source")
        // لا ناتج مؤقّت هنا، فهذا هو الجواب الوحيد عن «أين أنظر؟».
        .reveal(source);

    argv = argv.warn_opt(warn_if_resolved(inputs, "source", source, "warn.source.resolved"));

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

    fn plan_with(source: &Path) -> Result<PlannedCommand> {
        let raw =
            BTreeMap::from([("source".to_owned(), RawValue::Path(source.display().to_string()))]);
        plan(&crate::value::validate(&SPEC, &raw)?)
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
    fn the_operation_is_listed_in_its_category_as_a_read_only_one() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("image.info must be listed");
        assert_eq!(found.category, Category::Images);
        assert_eq!(found.danger, Danger::Safe);
        // الاقتران الذي يفرضه الفهرس: عمليةٌ لا تكتب لا اسم لها تحرسه.
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let s = Scratch::new("img-info-argv").unwrap();
        let src = image(&s, "الصورة.png");

        let cmd = plan_with(&src).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/sips"));
        assert_eq!(args[0], "-g");
        assert_eq!(args[1], "all");
        assert_eq!(Path::new(&args[2]), src.as_path());
        assert_eq!(args.len(), 3);
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("img-info-explain").unwrap();
        let src = image(&s, "م.png");

        let cmd = plan_with(&src).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn nothing_is_produced_and_nothing_is_redirected() {
        let s = Scratch::new("img-info-readonly").unwrap();
        let src = image(&s, "م.png");

        let cmd = plan_with(&src).unwrap();
        assert!(cmd.artifact.is_none(), "a question writes no file");
        assert!(cmd.stdout_to.is_none(), "the answer is streamed, not captured into a file");
        assert!(cmd.cwd.is_none());
        assert!(cmd.estimate.is_none(), "nothing is copied, so nothing is measured");
    }

    #[test]
    fn reveal_points_at_the_image_itself_so_finder_opens_the_folder_it_sits_in() {
        // مجلدُه كان يفتح مجلدَ المجلد، ويُرفض في `reveal::resolve_target`
        // لأنها تتحقّق بـ`paths::existing_file`.
        let s = Scratch::new("img-info-reveal").unwrap();
        let src = image(&s, "داخل/الصورة.png");

        let cmd = plan_with(&src).unwrap();
        assert_eq!(cmd.reveal_target.as_deref(), Some(src.as_path()));
        assert!(cmd.reveal_target.as_ref().unwrap().is_file(), "reveal.rs accepts files only");
    }

    #[test]
    fn no_end_of_flags_separator_is_emitted_because_sips_does_not_understand_one() {
        let s = Scratch::new("img-info-noend").unwrap();
        let src = image(&s, "م.png");

        let cmd = plan_with(&src).unwrap();
        assert!(!cmd.args.iter().any(|a| a == "--"), "{:?}", cmd.args);
    }

    #[test]
    fn a_folder_is_not_an_image_and_is_refused_by_its_own_field() {
        let s = Scratch::new("img-info-dir").unwrap();
        let dir = s.dir("مجلد");
        assert_eq!(refusal(plan_with(&dir)), ("err.path.missing", Some("source")));
    }

    #[test]
    fn a_path_that_is_not_there_is_refused_before_any_command_is_built() {
        let s = Scratch::new("img-info-missing").unwrap();
        let ghost = s.path().join("لا-وجود-لها.png");
        assert_eq!(refusal(plan_with(&ghost)), ("err.path.missing", Some("source")));
    }

    #[test]
    fn a_relative_path_never_becomes_an_argument() {
        assert_eq!(
            refusal(plan_with(Path::new("نسبي/صورة.png"))),
            ("err.path.relative", Some("source"))
        );
    }

    #[test]
    fn a_symlinked_source_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("img-info-symlink").unwrap();
        let real = image(&s, "الحقيقية.png");
        let link = s.path().join("رابط.png");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(&cmd.args[2]), real.as_path());
        assert_eq!(cmd.reveal_target.as_deref(), Some(real.as_path()));
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn reading_an_ordinary_image_raises_no_warning_at_all() {
        let s = Scratch::new("img-info-quiet").unwrap();
        let src = image(&s, "م.png");

        let cmd = plan_with(&src).unwrap();
        assert_eq!(cmd.warnings, Vec::<&str>::new(), "a question has nothing to warn about");
    }

    #[test]
    fn shell_syntax_in_a_file_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("img-info-shellish").unwrap();

        for name in
            ["صورة 'اليوم'.png", "a; rm -rf ~.png", "$(whoami).png", "back`tick`.png", "a & b.png"]
        {
            let src = image(&s, name);
            let cmd = plan_with(&src).unwrap();
            assert_eq!(Path::new(&cmd.args[2]), src.as_path());
            assert_eq!(cmd.args.len(), 3, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("img-info-dashes").unwrap();
        let src = image(&s, "-rf.png");

        let cmd = plan_with(&src).unwrap();
        // الراية الوحيدة التي يعلنها هذا الملف هي `-g`. وما عداها بيانات.
        for (i, a) in cmd.args.iter().enumerate().skip(1) {
            assert!(cannot_be_read_as_a_flag(a), "args[{i}] = {a:?} would be read as a flag");
        }
    }

    #[test]
    fn planning_writes_nothing_anywhere() {
        let s = Scratch::new("img-info-clean").unwrap();
        let src = image(&s, "م.png");

        for _ in 0..10 {
            plan_with(&src).unwrap();
        }

        assert_eq!(s.names(s.path()), vec!["م.png".to_string()]);
    }
}
