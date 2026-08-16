//! فكّ أرشيف `TAR` أو `TAR.GZ` إلى مجلدٍ جديد.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/tar -x -f <الأرشيف> -C <المجلد المؤقّت>
//! ```
//!
//! لا رايةَ ضغطٍ: `bsdtar` تكتشف `gzip` و`bzip2` و`xz` من محتوى الملف نفسه لا
//! من امتداده، فأرشيفٌ سُمّي `.tar` وهو مضغوط يُفكّ صحيحًا، والعكس كذلك. رايةٌ
//! صريحة كانت ستُفشل الحالتين على اسمٍ مضلِّل، وهي حالةٌ شائعة في التنزيلات.
//!
//! ## الخروج من الجذر — وأين يختلف هذا عن ZIP
//!
//! في ZIP نقرأ الفهرس في النواة ونرفض قبل التشغيل (‏`archive::scan_zip`). في
//! TAR **لا نستطيع ذلك اليوم**: أسماء المدخلات موزَّعة على ترويسات ٥١٢ بايت
//! داخل المجرى، وهو مضغوطٌ غالبًا — فقراءتها تعني فكّ ضغطٍ كاملًا في النواة،
//! أي إمّا اعتماد مكتبة inflate أو كتابتها. وقد أُجّل، وسُجّل في خارطة الطريق.
//!
//! فالحرّاس هنا ثلاثة، أوّلُهم من الأداة لا منّا — وهذا يُقال صراحةً بدل أن
//! يُخفى خلف صمت:
//!
//! 1. **`bsdtar` ترفض من نفسها.** بلا `-P` تُسقط الشرطة الأولى من كل مسار
//!    مطلق، وترفض أي مدخلة يحوي مسارها `..`. اختبار تكاملٍ يبني أرشيفًا خبيثًا
//!    فعلًا ويثبت أن شيئًا لم يخرج — فالضمان **مقيسٌ** لا منقولٌ عن التوثيق،
//!    ولو تغيّر سلوك الأداة يومًا سقط الاختبار قبل أن يقع الضرر.
//! 2. **الاستخراج في مجلدٍ نملكه.** الوجهة مجلدٌ مؤقّت حجزته الخطة حصريًا،
//!    لا ما اختاره المستخدم. فما يفلت يقع في مجلدٍ لا شيء فيه له.
//! 3. **الترقية كتلةً واحدة** بعد خروجٍ ناجح، وحذفُ الشجرة كاملةً عند الفشل
//!    أو الإلغاء.
//!
//! وحتمًا لا `-P` ولا `--absolute-paths` في هذا الأمر، ولا سبيل لإضافتهما:
//! الوسائط تُبنى هنا في Rust ولا تُركَّب من نصٍّ قادمٍ من الواجهة.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;

pub const ID: &str = "compress.tar.extract";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.compress.tar.extract.title",
    description_key: "op.compress.tar.extract.description",
    category: Category::Compress,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::TAR,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("archive", InputKind::ExistingFile),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("folder_name", InputKind::NewDirName),
    ],
    sort_order: 60,
    search_terms: &["tar", "gzip", "tgz", "فك", "استخراج", "extract", "tarball", "أرشيف"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let archive_path = inputs.file("archive")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("folder_name")?;

    let final_path = paths::new_dir_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "folder_name" } else { "destination" };
        e.on_input(field)
    })?;

    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("folder_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    let mut argv = Argv::tool(tools::TAR, "explain.tar.tool")
        .flag("-x", "explain.tar.extract")
        .flag("-f", "explain.tar.file")
        .path(archive_path)
        .flag("-C", "explain.tar.into")
        .explained_path(&temp, "explain.role.temp_dir")
        // فحصُ الفهرس قبل التشغيل غير متاح لـTAR اليوم، والصمت عن ذلك أسوأ
        // من قوله: الحارس هنا من الأداة ومن عزل المجلد، لا من قراءةٍ سبقت.
        .warn("warn.tar.no_pre_scan");

    argv = argv.warn_opt(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));
    if crate::estimate::available_bytes(destination).is_none() {
        argv = argv.warn("warn.space.unknown");
    }

    argv.producing(Artifact::dir(temp, final_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
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
    }

    #[test]
    fn the_argv_extracts_into_the_temporary_directory_and_carries_no_absolute_path_flag() {
        let s = Scratch::new("tarx-argv").unwrap();
        let a = s.file("أرشيف.tar.gz", b"\x1f\x8b placeholder");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&a, &dst, "المستخرَج").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|x| x.to_string_lossy().into_owned()).collect();
        let artifact = cmd.artifact.as_ref().unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/tar"));
        assert_eq!(args[0], "-x");
        assert_eq!(args[1], "-f");
        assert_eq!(Path::new(&args[2]), a.as_path());
        assert_eq!(args[3], "-C");
        assert_eq!(Path::new(&args[4]), artifact.temp.as_path());
        assert_eq!(args.len(), 5);

        assert_eq!(artifact.kind, ArtifactKind::Dir);
        assert_eq!(artifact.final_path, dst.join("المستخرَج"));
        assert!(!artifact.temp.exists(), "planning must create nothing");
    }

    /// الثابتة التي يقوم عليها الحارس الأول.
    #[test]
    fn the_command_can_never_carry_the_absolute_path_flag() {
        let s = Scratch::new("tarx-noP").unwrap();
        let a = s.file("a.tar", b"x");
        let dst = s.dir("و");
        let cmd = plan_with(&a, &dst, "خ").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|x| x.to_string_lossy().into_owned()).collect();
        for forbidden in ["-P", "--absolute-paths", "--absolute-names"] {
            assert!(!args.contains(&forbidden.to_string()), "{forbidden} must never appear");
        }
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("tarx-explain").unwrap();
        let a = s.file("a.tar", b"x");
        let dst = s.dir("و");
        let cmd = plan_with(&a, &dst, "خ").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|x| x.to_string_lossy().into_owned()));
        assert_eq!(cmd.explain.iter().map(|t| t.token.clone()).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn the_absence_of_a_pre_scan_is_announced_rather_than_hidden() {
        let s = Scratch::new("tarx-warn").unwrap();
        let a = s.file("a.tar", b"x");
        let dst = s.dir("و");
        assert!(plan_with(&a, &dst, "خ").unwrap().warnings.contains(&"warn.tar.no_pre_scan"));
    }

    #[test]
    fn an_existing_folder_name_stops_the_plan_and_leaves_it_untouched() {
        let s = Scratch::new("tarx-exists").unwrap();
        let a = s.file("a.tar", b"x");
        let dst = s.dir("و");
        let taken = s.dir("و/موجود");
        std::fs::write(taken.join("ثمين.txt"), b"PRECIOUS").unwrap();

        assert_eq!(refusal(plan_with(&a, &dst, "موجود")), ("err.dest.exists", Some("folder_name")));
        assert_eq!(std::fs::read(taken.join("ثمين.txt")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn an_invalid_folder_name_is_blamed_on_the_name_not_the_destination() {
        let s = Scratch::new("tarx-badname").unwrap();
        let a = s.file("a.tar", b"x");
        let dst = s.dir("و");
        for bad in ["", "a/b", "..", ".مخفي"] {
            assert_eq!(
                refusal(plan_with(&a, &dst, bad)),
                ("err.name.invalid", Some("folder_name")),
                "name {bad:?}"
            );
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("tarx-dash").unwrap();
        let a = s.file("-a.tar", b"x");
        let dst = s.dir("و");
        let cmd = plan_with(&a, &dst, "-x").unwrap();
        for (i, arg) in cmd.args.iter().enumerate() {
            if [0, 1, 3].contains(&i) {
                continue; // رايات معلَنة
            }
            assert!(cannot_be_read_as_a_flag(arg), "argument {i} ({arg:?})");
        }
    }

    #[test]
    fn shell_syntax_in_the_folder_name_is_carried_literally() {
        let s = Scratch::new("tarx-shellish").unwrap();
        let a = s.file("a.tar", b"x");
        let dst = s.dir("و");
        for name in ["مجلد 'اليوم'", "a; rm -rf ~", "$(whoami)"] {
            let cmd = plan_with(&a, &dst, name).unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(name));
            assert_eq!(cmd.args.len(), 5, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn planning_leaves_the_destination_clean() {
        let s = Scratch::new("tarx-clean").unwrap();
        let a = s.file("a.tar", b"x");
        let dst = s.dir("و");
        for _ in 0..10 {
            plan_with(&a, &dst, "خ").unwrap();
        }
        assert_eq!(s.names(&dst), Vec::<String>::new());
    }
}
