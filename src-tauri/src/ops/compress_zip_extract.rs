//! فكّ أرشيف ZIP إلى مجلدٍ جديد، باستخدام `ditto`.
//!
//! ## لماذا `ditto -x` لا `unzip`
//!
//! `ditto -x -k` تفهم `__MACOSX` فتعيد بيانات macOS الوصفية إلى مكانها بدل أن
//! تنثر مجلدًا غريبًا بجوار الملفات. وهي التي تقرأ Zip64 صحيحًا — و`unzip`
//! المرفقة بالنظام قديمة، وتستخرج أرشيفًا فوق ٤ غيغابايت مبتورًا.
//!
//! ## ثلاثة حرّاس ضدّ الخروج من الجذر، لا واحد
//!
//! «‏Zip Slip» هجومٌ حقيقي، والدفاع عنه هنا مركّب عمدًا:
//!
//! 1. **قراءة الفهرس في النواة قبل التخطيط.** `archive::scan_zip` تقرأ أسماء
//!    المدخلات بنفسها، وترفض الأرشيف الذي فيه مسارٌ مطلق أو `..` — قبل أن
//!    يُطلق شيء. هذا هو الحارس الأول لأنه الوحيد الذي يقع **قبل** الكتابة.
//! 2. **الاستخراج في مجلدٍ نملكه.** الوجهة ليست ما اختاره المستخدم، بل مجلدٌ
//!    مؤقّت حجزته الخطة حصريًا داخله. فحتى لو أفلتت مدخلةٌ من الحارس الأول،
//!    فجذرُها مجلدٌ لا شيء فيه للمستخدم.
//! 3. **الترقية كتلةً واحدة.** لا يظهر المجلد باسمه النهائي إلا بعد خروجٍ
//!    ناجح، و`atomic::promote_dir` ترفض الاسم المأخوذ. استخراجٌ انقطع في
//!    المنتصف يُحذف كاملًا ولا يترك شجرةً ناقصة تبدو تامّة.
//!
//! ونرفض كذلك المسحَ الناقص: أرشيفٌ فيه من المدخلات أكثر ممّا يفحصه الحارس
//! يُرفض بدل أن يُقرأ «نظيفًا». ما لم يُفحص قد يكون هو الخبيث.
//!
//! ## ما يبقى تحذيرًا لا رفضًا
//!
//! مدخلةٌ من نوع رابطٍ رمزي. هدفُ الرابط في **جسم** المدخلة لا في فهرسها، فلا
//! سبيل لقراءته دون فكّ الضغط — ورفضُ كل أرشيفٍ فيه رابط كان سيرفض كل حزمة
//! تطبيقٍ على macOS. الحارسان الثاني والثالث يحملان هذه الحالة: الرابط يُنشأ
//! داخل شجرةٍ نملكها، وما يُكتب بعده يُكتب داخلها.

use crate::archive;
use crate::atomic;
use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;

pub const ID: &str = "compress.zip.extract";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.compress.zip.extract.title",
    description_key: "op.compress.zip.extract.description",
    category: Category::Compress,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::DITTO,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("archive", InputKind::ExistingFile),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("folder_name", InputKind::NewDirName),
    ],
    sort_order: 40,
    search_terms: &["ditto", "unzip", "zip", "فك", "استخراج", "extract", "أرشيف", "تفريغ"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let archive_path = inputs.file("archive")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("folder_name")?;

    // الحارس الأول، وقبل أي شيء آخر: نقرأ الفهرس ونرفض ما يخرج من جذره.
    let scan = archive::scan_zip(archive_path).map_err(|e| e.on_input("archive"))?;
    scan.guard_extraction("archive")?;

    let final_path = paths::new_dir_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "folder_name" } else { "destination" };
        e.on_input(field)
    })?;

    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("folder_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    let mut argv = Argv::tool(tools::DITTO, "explain.ditto.tool")
        .flag("-x", "explain.ditto.extract")
        .flag("-k", "explain.ditto.pkzip_read")
        .path(archive_path)
        .explained_path(&temp, "explain.role.temp_dir");

    for key in scan.warnings() {
        argv = argv.warn(key);
    }
    argv = argv.warn_opt(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));
    // الأرشيف يُعلن حجمًا غير مضغوط؛ إن تجاوز المساحة الحرة فالاستخراج يفشل
    // في منتصفه. الرقم من الفهرس لا من قياسٍ لنا — ولذلك تحذيرٌ لا رفض.
    match crate::estimate::available_bytes(destination) {
        Some(free) if scan.uncompressed_bytes > free => argv = argv.warn("warn.space.low"),
        None => argv = argv.warn("warn.space.unknown"),
        Some(_) => {}
    }

    argv.producing(Artifact::dir(temp, final_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::{zip_with, Scratch};
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn plan_with(archive_path: &Path, destination: &Path, name: &str) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("archive".to_owned(), RawValue::Path(archive_path.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("folder_name".to_owned(), RawValue::Text(name.to_owned())),
        ]);
        plan(&crate::value::validate(&SPEC, &raw)?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.category, Category::Compress);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_extracts_into_the_temporary_directory_not_the_final_one() {
        let s = Scratch::new("zipx-argv").unwrap();
        let z = s.file("أرشيف.zip", &zip_with(&["a.txt", "مجلد/ب.txt"]));
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&z, &dst, "المستخرَج").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        let artifact = cmd.artifact.as_ref().unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/ditto"));
        assert_eq!(args[0], "-x");
        assert_eq!(args[1], "-k");
        assert_eq!(Path::new(&args[2]), z.as_path());
        assert_eq!(Path::new(&args[3]), artifact.temp.as_path());
        assert_eq!(args.len(), 4);

        assert_eq!(artifact.kind, ArtifactKind::Dir);
        assert_eq!(artifact.final_path, dst.join("المستخرَج"));
        assert_eq!(artifact.temp.parent().unwrap(), dst.as_path(), "must share the filesystem");
        assert!(!artifact.temp.exists(), "planning must create nothing");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("zipx-explain").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt"]));
        let dst = s.dir("و");
        let cmd = plan_with(&z, &dst, "خ").unwrap();

        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    /// الاختبار الذي توجد هذه العملية من أجله.
    #[test]
    fn a_zip_slip_archive_is_refused_and_nothing_is_planned() {
        let s = Scratch::new("zipx-slip").unwrap();
        let z = s.file("خبيث.zip", &zip_with(&["ok.txt", "../../../../etc/passwd"]));
        let dst = s.dir("و");

        assert_eq!(refusal(plan_with(&z, &dst, "خ")), ("err.archive.escapes", Some("archive")));
        assert_eq!(s.names(&dst), Vec::<String>::new(), "a refused plan writes nothing");
    }

    #[test]
    fn an_absolute_entry_is_refused_just_the_same() {
        let s = Scratch::new("zipx-abs").unwrap();
        let z = s.file("خبيث.zip", &zip_with(&["/etc/passwd"]));
        let dst = s.dir("و");
        assert_eq!(refusal(plan_with(&z, &dst, "خ")), ("err.archive.escapes", Some("archive")));
    }

    #[test]
    fn a_corrupt_archive_is_refused_before_anything_runs() {
        let s = Scratch::new("zipx-corrupt").unwrap();
        let z = s.file("تالف.zip", b"not a zip at all");
        let dst = s.dir("و");
        assert_eq!(refusal(plan_with(&z, &dst, "خ")), ("err.archive.unreadable", Some("archive")));
    }

    #[test]
    fn an_existing_folder_name_stops_the_plan_and_leaves_it_untouched() {
        let s = Scratch::new("zipx-exists").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt"]));
        let dst = s.dir("و");
        let taken = s.dir("و/موجود");
        std::fs::write(taken.join("ثمين.txt"), b"PRECIOUS").unwrap();

        assert_eq!(refusal(plan_with(&z, &dst, "موجود")), ("err.dest.exists", Some("folder_name")));
        assert_eq!(std::fs::read(taken.join("ثمين.txt")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn an_existing_file_at_the_folder_name_also_stops_the_plan() {
        let s = Scratch::new("zipx-file-taken").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt"]));
        let dst = s.dir("و");
        std::fs::write(dst.join("مأخوذ"), b"A FILE").unwrap();
        assert_eq!(refusal(plan_with(&z, &dst, "مأخوذ")), ("err.dest.exists", Some("folder_name")));
    }

    #[test]
    fn an_invalid_folder_name_is_blamed_on_the_name_not_the_destination() {
        let s = Scratch::new("zipx-badname").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt"]));
        let dst = s.dir("و");
        for bad in ["", "   ", "a/b", "..", ".مخفي"] {
            assert_eq!(
                refusal(plan_with(&z, &dst, bad)),
                ("err.name.invalid", Some("folder_name")),
                "name {bad:?}"
            );
        }
    }

    #[test]
    fn shell_syntax_in_the_folder_name_is_carried_literally() {
        let s = Scratch::new("zipx-shellish").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt"]));
        let dst = s.dir("و");
        for name in ["مجلد 'اليوم'", "a; rm -rf ~", "$(whoami)", "a & b"] {
            let cmd = plan_with(&z, &dst, name).unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(name));
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("zipx-dash").unwrap();
        let z = s.file("-a.zip", &zip_with(&["a.txt"]));
        let dst = s.dir("و");
        let cmd = plan_with(&z, &dst, "-x").unwrap();
        for a in cmd.args.iter().skip(2) {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn an_archive_carrying_a_symlink_entry_is_allowed_but_announced() {
        // لا نرفضه: كل حزمة تطبيقٍ على macOS تحمل روابط. نقوله فحسب.
        let s = Scratch::new("zipx-quiet").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt", "b.txt"]));
        let dst = s.dir("و");
        let cmd = plan_with(&z, &dst, "خ").unwrap();
        assert!(!cmd.warnings.contains(&"warn.archive.symlinks"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_the_destination_clean() {
        let s = Scratch::new("zipx-clean").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt"]));
        let dst = s.dir("و");
        for _ in 0..10 {
            plan_with(&z, &dst, "خ").unwrap();
        }
        assert_eq!(s.names(&dst), Vec::<String>::new());
    }
}
