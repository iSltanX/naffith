//! نقل ملف أو مجلد إلى وجهة أخرى باسمٍ جديد، باستخدام `mv`.
//!
//! ## لماذا `mv` لا نسخٌ ثم حذف
//!
//! داخل نظام الملفات الواحد `mv` ليست نقلًا بل `rename(2)`: تغييرُ مدخلةٍ في
//! دليلٍ واحد، ذرّيّ وفوريّ ولا يلمس بايتًا من المحتوى. البديل — «انسخ ثم
//! احذف» — كان يضاعف زمن نقل مجلدٍ بحجم غيغابايتات بلا سبب، ويفتح نافذةً
//! توجد فيها نسختان، ويُسقط في طريقه ما يُسقطه النسخ من سماتٍ ممتدّة.
//!
//! ## الصيغة
//!
//! ```text
//! /bin/mv -n -- <المصدر> <المسار النهائي>
//! ```
//!
//! **لا ناتج مؤقّت هنا، ولا ترقية.** بقيّة عمليات هذا المنتج تكتب باسمٍ عابر
//! ثم تُرقّيه بعد نجاحٍ كامل، لأن أداتها تبني شيئًا جديدًا قد يخرج نصفه.
//! و`mv` لا تبني شيئًا: هي إمّا نجحت فصار الاسم في موضعه، وإمّا فشلت فبقي
//! كما كان. اختراعُ مؤقّتٍ لها كان يعني نقلتين بدل نقلة، وهو أبطأ وأخطر لا
//! أأمن. لذلك تعلن العملية المسار النهائي نفسه ناتجًا قابلًا للإظهار، مع أنه
//! ليس `Artifact` مؤقّتًا: عقد النتيجة يعرض المنقول لا مجلدًا عامًا حوله.
//!
//! ## `-n` حارسٌ **تحت** حارسنا لا بدلًا منه
//!
//! `-n` تمنع `mv` من الكتابة فوق وجهةٍ قائمة. لكنها لا تكفي وحدها، وهذا مقيس
//! لا مُفترَض: إن كان المسار النهائي **مجلدًا قائمًا**، لا تعتبره `mv` تضاربًا
//! أصلًا — بل تنقل المصدر *إلى داخله* وتخرج بنجاح. أي أن مستخدمًا يكتب اسمًا
//! يصادف أنه اسم مجلدٍ موجود كان سيجد ملفه في مكانٍ لم يطلبه، والأمر «نجح».
//! ولذلك الفحص الحاسم هو فحصنا: `symlink_metadata` على المسار النهائي قبل
//! التشغيل. و`-n` تبقى مكتوبةً لأنها تسدّ ما بين لحظة الفحص ولحظة التنفيذ.
//!
//! و`symlink_metadata` لا `metadata`: رابطٌ رمزي معلَّق يحمل الاسم النهائي
//! ليس «لا شيء» — هو اسمٌ مشغول، و`mv` كانت ستدهسه.
//!
//! ## النقل بين قرصين ليس نقلًا
//!
//! حين يقع المصدر والوجهة على نظامَي ملفاتٍ مختلفين (‏`st_dev` مختلف) يتعذّر
//! `rename(2)`، فتتحوّل `mv` من تلقاء نفسها إلى نسخٍ ثم حذف. والفرق ليس في
//! الزمن وحده: العملية تفقد ذرّيّتها، فانقطاعٌ في المنتصف يترك نسخةً ناقصة في
//! الوجهة والأصلَ سليمًا في مكانه. لا نمنع ذلك — نقلُ ملفٍ إلى قرصٍ خارجي طلبٌ
//! مشروع — لكنّا نقوله قبل أن يحدث في `warn.move.cross_device`.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تستبدل، ولا تدمج، ولا تنحّي جانبًا. الاسم المشغول يوقف التخطيط ويُخبر.

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;
use std::path::Path;

pub const ID: &str = "files.move";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.move.title",
    description_key: "op.files.move.description",
    category: Category::Files,
    danger: Danger::Modifies,
    visibility: Visibility::Production,
    tool: tools::MV,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("source", InputKind::ExistingPath),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("new_name", InputKind::NewName { ext: None }),
    ],
    sort_order: 20,
    search_terms: &["mv", "نقل", "move", "تحريك", "إعادة تسمية", "rename", "نقل ملف"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.any_path("source")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("new_name")?;

    // كلا المسارين محلول الروابط، فالمقارنة مقارنة مواضع لا نصوص: من اختار
    // رابطًا رمزيًا ومقصده معًا يستحق أن يُقال له إنهما موضعٌ واحد.
    if destination == source {
        return Err(CoreError::SamePath.on_input("destination"));
    }
    // نقل مجلدٍ إلى داخل نفسه: `mv` ترفضه برسالةٍ من طبقة النظام، ونحن نرفضه
    // بلغةٍ تُصلح — وقبل أن يُعرض أمرٌ لا يمكن أن ينجح.
    if destination.starts_with(source) {
        return Err(CoreError::DestinationInsideSource.on_input("destination"));
    }

    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "new_name" } else { "destination" };
        e.on_input(field)
    })?;

    // الحارس الحقيقي. انظر شرح `-n` في رأس الملف: الرايةُ وحدها كانت تسمح
    // بالنقل إلى داخل مجلدٍ يحمل الاسم النهائي، وتخرج بنجاح.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("new_name"));
    }

    let mut argv = Argv::tool(tools::MV, "explain.mv.tool")
        .flag("-n", "explain.mv.no_clobber")
        // `mv` تفهم `--`، وقد جُرّب: اسمٌ يبدأ بشرطة يصير وسيطًا لا راية.
        // المسارات هنا مطلقة أصلًا فتبدأ بـ`/`، والفاصل حزام أمانٍ فوق ذلك.
        .end_of_flags()
        .explained_path(source, "explain.role.source")
        .explained_path(&final_path, "explain.role.moved")
        // لا `Artifact` مؤقّت، لكن المسار النهائي هو الناتج الفعلي المملوك
        // للتشغيل. `reveal(run_id)` يعيد التحقق منه قبل أي فتح.
        .reveal(&final_path);

    for key in warnings_for(inputs, source, destination) {
        argv = argv.warn(key);
    }

    // `read_only` هنا تعني «بلا ناتجٍ مؤقّت يُرقّى» لا «بلا أثر»: الأثر معلَنٌ
    // في `Danger::Modifies`، والاسم يصف ما تبنيه الخطة لا ما تفعله الأداة.
    argv.read_only()
}

fn warnings_for(inputs: &Inputs, source: &Path, destination: &Path) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    warnings.extend(warn_if_resolved(inputs, "source", source, "warn.source.resolved"));
    warnings.extend(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));
    if on_different_devices(source, destination) == Some(true) {
        warnings.push("warn.move.cross_device");
    }
    warnings
}

/// هل يقع الموضعان على نظامَي ملفاتٍ مختلفين؟ `None` إن تعذّر السؤال.
///
/// `st_dev` هو ما يقرّره النظام لا امتداد المسار: `/Volumes/…` قد يكون قرصًا
/// خارجيًا وقد يكون تركيبًا لنفس القرص، ومجلدٌ في المنزل قد يكون نقطة تركيبٍ
/// لشبكة. والتخمين من شكل المسار كان يحذّر حيث لا داعي ويصمت حيث يجب.
///
/// والجواب `Option` لا `bool`: تعذّر القراءة ليس «نفس القرص». نحن نحذّر عند
/// اليقين وحده، لأن تحذيرًا يظهر بلا سبب يُعلَّم المستخدم أن يتجاهل التحذيرات.
fn on_different_devices(source: &Path, destination: &Path) -> Option<bool> {
    use std::os::unix::fs::MetadataExt;
    let a = std::fs::metadata(source).ok()?;
    let b = std::fs::metadata(destination).ok()?;
    Some(a.dev() != b.dev())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;

    fn raw(source: &Path, destination: &Path, name: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("source".to_owned(), RawValue::Path(source.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("new_name".to_owned(), RawValue::Text(name.to_owned())),
        ])
    }

    fn plan_with(source: &Path, destination: &Path, name: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(source, destination, name))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("files.move must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Modifies);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_is_promoted_afterwards() {
        let s = Scratch::new("move-argv").unwrap();
        let src = s.file("مستند.txt", b"data");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&src, &dst, "بعد النقل.txt").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/bin/mv"));
        assert_eq!(args[0], "-n");
        assert_eq!(args[1], "--");
        assert_eq!(Path::new(&args[2]), src.as_path());
        assert_eq!(Path::new(&args[3]), dst.join("بعد النقل.txt"));
        assert_eq!(args.len(), 4);

        // `mv` تنقل إلى مكانه مباشرة: لا مؤقّت، ولا ترقية، ولا توجيه خرج.
        assert!(cmd.artifact.is_none(), "mv renames in place; there is nothing to promote");
        assert!(cmd.stdout_to.is_none());
        assert_eq!(cmd.reveal_target.as_deref(), Some(dst.join("بعد النقل.txt").as_path()));
    }

    #[tokio::test]
    async fn a_successful_result_names_the_actual_moved_path() {
        let s = Scratch::new("move-result").unwrap();
        let src = s.file("قبل.txt", b"data");
        let dst = s.dir("الوجهة");
        let final_path = dst.join("بعد.txt");
        let cmd = plan_with(&src, &dst, "بعد.txt").unwrap();
        let (tx, _rx) = tokio::sync::mpsc::channel(8);
        let (_cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();

        let execution = crate::executor::run_for(ID, cmd, tx, cancel_rx).await;
        assert_eq!(
            execution.outcome,
            crate::executor::Outcome::Success { produced: Some(final_path.display().to_string()) }
        );
        assert!(!src.exists());
        assert_eq!(std::fs::read(&final_path).unwrap(), b"data");

        let reveal = crate::reveal::safe_target_kind(&final_path);
        let result = crate::result::ResultContract::for_operation(
            ID,
            execution.semantic,
            execution.outcome.produced(),
            Vec::new(),
            reveal,
        );
        assert_eq!(result.category, crate::result::ResultCategory::Artifact);
        assert!(matches!(
            result.payload,
            crate::result::ResultPayload::Artifact { ref path, .. }
                if path == &final_path.display().to_string()
        ));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("move-explain").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("move-dashes").unwrap();
        let src = s.dir("-rf");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "-x").unwrap();
        for a in cmd.args.iter().skip(2) {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn moving_a_path_onto_itself_is_refused() {
        let s = Scratch::new("move-same").unwrap();
        let dir = s.dir("واحد");
        assert_eq!(refusal(plan_with(&dir, &dir, "ن")), ("err.path.same", Some("destination")));
    }

    #[test]
    fn a_destination_inside_the_source_is_refused() {
        let s = Scratch::new("move-nested").unwrap();
        let src = s.dir("مصدر");
        let dst = s.dir("مصدر/داخل");
        assert_eq!(
            refusal(plan_with(&src, &dst, "ن")),
            ("err.dest.inside_source", Some("destination"))
        );
    }

    #[test]
    fn an_existing_name_in_the_destination_stops_the_plan_before_anything_moves() {
        let s = Scratch::new("move-exists").unwrap();
        let src = s.file("مصدر.txt", b"SOURCE");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود"), b"PRECIOUS").unwrap();

        assert_eq!(refusal(plan_with(&src, &dst, "موجود")), ("err.dest.exists", Some("new_name")));
        assert_eq!(std::fs::read(dst.join("موجود")).unwrap(), b"PRECIOUS");
        assert_eq!(std::fs::read(&src).unwrap(), b"SOURCE");
    }

    #[test]
    fn an_existing_directory_at_the_final_name_is_a_conflict_even_though_mv_would_accept_it() {
        // الحالة التي لا تمسكها `-n`: `mv -n -- ملف مجلدٌ_قائم` تنقل الملف
        // *إلى داخل* المجلد وتخرج بصفر. مقيسة لا مفترَضة.
        let s = Scratch::new("move-into-dir").unwrap();
        let src = s.file("مصدر.txt", b"x");
        let dst = s.dir("و");
        s.dir("و/مجلد");

        assert_eq!(refusal(plan_with(&src, &dst, "مجلد")), ("err.dest.exists", Some("new_name")));
    }

    #[test]
    fn a_dangling_symlink_at_the_final_name_still_counts_as_taken() {
        let s = Scratch::new("move-dangling").unwrap();
        let src = s.file("مصدر.txt", b"x");
        let dst = s.dir("و");
        std::os::unix::fs::symlink(s.path().join("لا-وجود-له"), dst.join("رابط")).unwrap();

        assert_eq!(refusal(plan_with(&src, &dst, "رابط")), ("err.dest.exists", Some("new_name")));
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("move-shellish").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");

        for name in ["ملف 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let cmd = plan_with(&src, &dst, name).unwrap();
            assert_eq!(Path::new(&cmd.args[3]), dst.join(name));
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_symlinked_source_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("move-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&link, &dst, "ن").unwrap();
        assert_eq!(Path::new(&cmd.args[2]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn a_move_within_one_volume_does_not_claim_to_be_a_cross_device_move() {
        let s = Scratch::new("move-onedisk").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن").unwrap();
        assert!(!cmd.warnings.contains(&"warn.move.cross_device"), "{:?}", cmd.warnings);
        assert_eq!(on_different_devices(&src, &dst), Some(false));
    }

    #[test]
    fn a_device_that_cannot_be_read_is_unknown_not_identical() {
        // صمتٌ عن السؤال ليس جوابًا بـ«نفس القرص».
        assert_eq!(on_different_devices(Path::new("/no/such/path/xyz"), Path::new("/")), None);
    }

    #[test]
    fn planning_moves_nothing_and_creates_nothing() {
        let s = Scratch::new("move-clean").unwrap();
        let src = s.dir("م");
        std::fs::write(src.join("ملف"), b"data").unwrap();
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&src, &dst, "ن").unwrap();
        }

        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
        assert_eq!(s.names(&src), vec!["ملف".to_string()], "the source must not be touched");
    }
}
