//! إنشاء أرشيف `TAR.GZ` من مجلد.
//!
//! ## لماذا TAR.GZ إلى جانب ZIP
//!
//! ليست تكرارًا لعملية الضغط: ZIP يضغط كل ملفٍ وحده، و`tar.gz` يجمع الشجرة
//! أوّلًا ثم يضغط الكتلة كلها — فيستفيد من التشابه بين الملفات ويخرج أصغر
//! كثيرًا على شجرة شيفرةٍ أو مجلد نصوص. وهي الصيغة التي تتوقّعها أدوات
//! الخوادم ولينكس، فمن يرفع شيئًا إلى خادم يريدها لا يريد ZIP.
//!
//! والثمن معلَن في الوصف: `tar.gz` **لا يحفظ** بيانات macOS الوصفية كما تحفظها
//! `ditto`، ولا يمكن قراءة ملفٍ منه دون فكّ ما قبله.
//!
//! ## الصيغة، ولماذا `-C`
//!
//! ```text
//! /usr/bin/tar -c -z -f <المؤقّت> -C <أبو المصدر> <اسم المصدر>
//! ```
//!
//! `-C <الأب>` ثم الاسم النسبي، لا المسار المطلق. لو مُرِّر المسار المطلق
//! لحمل الأرشيف مسارات المستخدم كاملةً في داخله — يتسرّب اسمه واسم مشروعه إلى
//! كل من يفتح الأرشيف — ولتذمّر `tar` من إسقاط الشرطة الأولى. وبهذه الصيغة
//! يحوي الأرشيف مجلدًا واحدًا اسمه اسم المصدر، وهو ما يتوقّعه من يفكّه.
//!
//! والاسم يُمرَّر **بايتات لا نصًّا** (`os_value`): أسماء الملفات على macOS
//! بايتات، والمرور بـ`String` يفسد ما ليس UTF-8 صالحًا وهو مشروع.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::estimate;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;

pub const ID: &str = "compress.tar.create";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.compress.tar.create.title",
    description_key: "op.compress.tar.create.description",
    category: Category::Compress,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::TAR,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("source", InputKind::ExistingDir),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("archive_name", InputKind::NewName { ext: Some("tar.gz") }),
    ],
    sort_order: 50,
    search_terms: &["tar", "gzip", "tgz", "ضغط", "أرشيف", "compress", "archive", "tarball"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.dir("source")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("archive_name")?;

    if destination == source || destination.starts_with(source) {
        return Err(CoreError::DestinationInsideSource.on_input("destination"));
    }

    // الجذر نفسه لا أبَ له ولا اسم، فلا صيغة `-C <أب> <اسم>` له. وهو خارج
    // الجذور المسموحة أصلًا، لكن الرفض هنا صريحٌ بدل أن يكون أثرًا جانبيًا.
    let parent = source.parent().ok_or(CoreError::NotADirectory.on_input("source"))?;
    let base = source.file_name().ok_or(CoreError::NotADirectory.on_input("source"))?;

    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "archive_name" } else { "destination" };
        e.on_input(field)
    })?;

    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("archive_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;
    let size = estimate::tree_size(source);

    let mut argv = Argv::tool(tools::TAR, "explain.tar.tool")
        .flag("-c", "explain.tar.create")
        .flag("-z", "explain.tar.gzip")
        .flag("-f", "explain.tar.file")
        .explained_path(&temp, "explain.role.temp")
        .flag("-C", "explain.tar.directory")
        .path(parent)
        .os_value(base)
        .with_estimate(size);

    argv = argv.warn_opt(warn_if_resolved(inputs, "source", source, "warn.source.resolved"));
    argv = argv.warn_opt(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));
    if size.entries == 0 && size.complete {
        argv = argv.warn("warn.source.empty");
    }
    if !size.complete {
        argv = argv.warn("warn.size.partial");
    }
    // `tar.gz` لا يحفظ resource forks ولا السمات الممتدّة. خسارةٌ صامتة تستحقّ
    // أن تُقال قبل أن يحذف المستخدم مصدره.
    argv = argv.warn("warn.tar.no_macos_metadata");
    match estimate::available_bytes(destination) {
        Some(free) if size.bytes > free => argv = argv.warn("warn.space.low"),
        None => argv = argv.warn("warn.space.unknown"),
        Some(_) => {}
    }

    argv.producing(Artifact::file(temp, final_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn plan_with(source: &Path, destination: &Path, name: &str) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("source".to_owned(), RawValue::Path(source.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("archive_name".to_owned(), RawValue::Text(name.to_owned())),
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
    fn the_archive_is_built_from_a_relative_name_under_the_sources_parent() {
        let s = Scratch::new("tarc-argv").unwrap();
        let src = s.dir("المشروع");
        std::fs::write(src.join("ملف.txt"), b"data").unwrap();
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&src, &dst, "نسخة").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        let artifact = cmd.artifact.as_ref().unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/tar"));
        assert_eq!(args[0], "-c");
        assert_eq!(args[1], "-z");
        assert_eq!(args[2], "-f");
        assert_eq!(Path::new(&args[3]), artifact.temp.as_path());
        assert_eq!(args[4], "-C");
        assert_eq!(
            Path::new(&args[5]),
            s.path(),
            "the parent, so the archive holds no absolute path"
        );
        assert_eq!(args[6], "المشروع", "and the source enters by its bare name");
        assert_eq!(args.len(), 7);
        assert_eq!(artifact.final_path, dst.join("نسخة.tar.gz"));
        assert_eq!(artifact.kind, ArtifactKind::File);
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("tarc-explain").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");
        let cmd = plan_with(&src, &dst, "ن").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        assert_eq!(cmd.explain.iter().map(|t| t.token.clone()).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn the_extension_is_added_once_and_never_doubled() {
        let s = Scratch::new("tarc-ext").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");
        for (given, expected) in [
            ("تقرير", "تقرير.tar.gz"),
            ("تقرير.tar.gz", "تقرير.tar.gz"),
            ("تقرير.TAR.GZ", "تقرير.TAR.GZ"),
        ] {
            let cmd = plan_with(&src, &dst, given).unwrap();
            assert_eq!(
                cmd.artifact.unwrap().final_path.file_name().unwrap(),
                OsStr::new(expected),
                "for input {given:?}"
            );
        }
    }

    #[test]
    fn a_destination_inside_the_source_is_refused() {
        let s = Scratch::new("tarc-nested").unwrap();
        let src = s.dir("مصدر");
        let dst = s.dir("مصدر/داخل");
        assert_eq!(
            refusal(plan_with(&src, &dst, "ن")),
            ("err.dest.inside_source", Some("destination"))
        );
    }

    #[test]
    fn an_existing_archive_name_stops_the_plan_and_leaves_the_file_untouched() {
        let s = Scratch::new("tarc-exists").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود.tar.gz"), b"PRECIOUS").unwrap();

        assert_eq!(
            refusal(plan_with(&src, &dst, "موجود")),
            ("err.dest.exists", Some("archive_name"))
        );
        assert_eq!(std::fs::read(dst.join("موجود.tar.gz")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn a_source_whose_name_starts_with_a_dash_is_refused_rather_than_passed() {
        // `-C <أب> <اسم>` تمرّر الاسم عاريًا، فاسمٌ يبدأ بشرطة كان يُقرأ راية.
        // `os_value` ترفضه، والرفض هنا أصدق من إخفائه خلف `--`.
        let s = Scratch::new("tarc-dash").unwrap();
        let src = s.dir("-rf");
        let dst = s.dir("و");
        assert_eq!(refusal(plan_with(&src, &dst, "ن")).0, "err.arg.flag");
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag_in_an_ordinary_plan() {
        let s = Scratch::new("tarc-safe").unwrap();
        let src = s.dir("مشروع");
        let dst = s.dir("و");
        let cmd = plan_with(&src, &dst, "-x").unwrap();
        for (i, a) in cmd.args.iter().enumerate() {
            // المواضع ٠ و١ و٢ و٤ رايات معلَنة؛ ما عداها بيانات.
            if [0, 1, 2, 4].contains(&i) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "argument {i} ({a:?}) would be read as a flag");
        }
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("tarc-shellish").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");
        for name in ["نسخة 'اليوم'", "a; rm -rf ~", "$(whoami)", "a & b"] {
            let cmd = plan_with(&src, &dst, name).unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(format!("{name}.tar.gz")));
            assert_eq!(cmd.args.len(), 7, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn the_loss_of_macos_metadata_is_always_announced() {
        // خسارةٌ صامتة تُقال قبل أن يحذف المستخدم مصدره اعتمادًا على الأرشيف.
        let s = Scratch::new("tarc-meta").unwrap();
        let src = s.dir("م");
        std::fs::write(src.join("ملف"), b"x").unwrap();
        let dst = s.dir("و");
        assert!(plan_with(&src, &dst, "ن")
            .unwrap()
            .warnings
            .contains(&"warn.tar.no_macos_metadata"));
    }

    #[test]
    fn planning_leaves_both_directories_clean() {
        let s = Scratch::new("tarc-clean").unwrap();
        let src = s.dir("م");
        std::fs::write(src.join("ملف"), b"x").unwrap();
        let dst = s.dir("و");
        for _ in 0..10 {
            plan_with(&src, &dst, "ن").unwrap();
        }
        assert_eq!(s.names(&dst), Vec::<String>::new());
        assert_eq!(s.names(&src), vec!["ملف".to_string()]);
    }
}
