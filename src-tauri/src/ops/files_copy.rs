//! نسخ ملف أو مجلد بأمان، باستخدام `ditto`.
//!
//! ## لماذا `ditto` لا `cp -R`
//!
//! `cp` تُسقط resource forks والسمات الممتدّة وأذونات ACL صامتةً. من ينسخ
//! مجلد تصميم أو مشروع Xcode ثم يجد الوسوم والألوان قد ضاعت لن يعرف أين
//! ضاعت. `ditto` جزءٌ من النظام وتحفظها، وهي نفسها ما يستعمله Finder.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/ditto --rsrc --extattr <المصدر> <المؤقّت>
//! ```
//!
//! الوسيط الأخير **مؤقّت** لا نهائي: النسخة تُبنى باسمٍ عابر داخل مجلد الوجهة
//! نفسه، ولا تُرقّى إلى اسمها إلا بعد خروجٍ ناجح. أي أن انقطاعًا في المنتصف لا
//! يترك في وجهة المستخدم مجلدًا نصفيًّا يحمل الاسم الصحيح ويبدو مكتملًا.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تستبدل. إن كان الاسم مأخوذًا في الوجهة تتوقّف وتخبر، ولا تدمج ولا
//! تنحّي جانبًا. الدمج قرارٌ بعواقب لا يصحّ أن يقع بالنيابة، وهو عمليةٌ أخرى
//! مسجَّلة في خارطة الطريق باسمها.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::estimate;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;
use std::path::Path;

pub const ID: &str = "files.copy";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.copy.title",
    description_key: "op.files.copy.description",
    category: Category::Files,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::DITTO,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("source", InputKind::ExistingPath),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("new_name", InputKind::NewName { ext: None }),
    ],
    sort_order: 10,
    search_terms: &["ditto", "نسخ", "copy", "cp", "نسخة", "duplicate", "backup"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.any_path("source")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("new_name")?;

    // كلا المسارين محلول الروابط، فالمقارنة مقارنة مواضع لا نصوص.
    if destination == source {
        return Err(CoreError::SamePath.on_input("destination"));
    }
    if destination.starts_with(source) {
        return Err(CoreError::DestinationInsideSource.on_input("destination"));
    }

    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "new_name" } else { "destination" };
        e.on_input(field)
    })?;

    // النسخة إلى داخل نفسها: مصدرٌ هو أبٌ للناتج. `ditto` كانت ستدور.
    if final_path.starts_with(source) {
        return Err(CoreError::DestinationInsideSource.on_input("destination"));
    }

    // `symlink_metadata` لا يتبع الروابط: رابط معلَّق بالاسم النهائي تضاربٌ.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("new_name"));
    }

    let size = estimate::tree_size(source);
    let copying_a_dir = source.is_dir();
    // اسمُ المؤقّت واحدٌ لملفٍ أو مجلد: `temp_path_for` تبني اسمًا بجوار
    // الاسم النهائي ولا تعنيها طبيعةُ ما سيُكتب فيه. ما يختلف هو **كيف
    // يُحجز وكيف يُرقّى**، وذاك معلَنٌ في `ArtifactKind` أدناه.
    let temp = atomic::temp_path_for(&final_path)?;

    let mut argv = Argv::tool(tools::DITTO, "explain.ditto.tool")
        .flag("--rsrc", "explain.ditto.rsrc")
        .flag("--extattr", "explain.ditto.extattr")
        .explained_path(source, "explain.role.source")
        .explained_path(&temp, "explain.role.temp")
        .with_estimate(size);

    for key in warnings_for(inputs, source, destination, &size) {
        argv = argv.warn(key);
    }

    // شكل الناتج يتبع شكل المصدر: نسخُ ملفٍ يُنتج ملفًا، ونسخُ مجلدٍ يُنتج
    // مجلدًا — ولكلٍّ منهما آلية ترقيةٍ مختلفة في `atomic.rs`.
    let artifact = if copying_a_dir {
        Artifact::dir(temp, final_path)
    } else {
        Artifact::file(temp, final_path)
    };
    argv.producing(artifact)
}

fn warnings_for(
    inputs: &Inputs,
    source: &Path,
    destination: &Path,
    size: &estimate::SizeEstimate,
) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    warnings.extend(warn_if_resolved(inputs, "source", source, "warn.source.resolved"));
    warnings.extend(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.destination.resolved",
    ));

    if source.is_dir() && size.entries == 0 && size.complete {
        warnings.push("warn.source.empty");
    }
    if !size.complete {
        warnings.push("warn.size.partial");
    }
    match estimate::available_bytes(destination) {
        // النسخ لا يضغط، فالمقارنة هنا **ليست** متحفّظة كما هي في الضغط:
        // شجرةٌ أكبر من المساحة الحرة لن تُنسخ، نقطة.
        Some(free) if size.bytes > free => warnings.push("warn.space.low"),
        None => warnings.push("warn.space.unknown"),
        Some(_) => {}
    }
    warnings
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
        let found = listed.iter().find(|o| o.id == ID).expect("files.copy must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Creates);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_last_argument_is_the_temp() {
        let s = Scratch::new("copy-argv").unwrap();
        let src = s.dir("المصدر");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&src, &dst, "نسخة").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/ditto"));
        assert_eq!(args[0], "--rsrc");
        assert_eq!(args[1], "--extattr");
        assert_eq!(Path::new(&args[2]), src.as_path());
        let artifact = cmd.artifact.as_ref().unwrap();
        assert_eq!(Path::new(&args[3]), artifact.temp.as_path());
        assert_eq!(args.len(), 4);
        assert_eq!(artifact.final_path, dst.join("نسخة"));
        assert_eq!(artifact.kind, ArtifactKind::Dir, "copying a folder produces a folder");
        assert!(!artifact.temp.exists(), "planning must create nothing");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("copy-explain").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn copying_a_file_produces_a_file_artifact() {
        let s = Scratch::new("copy-file").unwrap();
        let file = s.file("مستند.txt", b"data");
        let dst = s.dir("و");

        let cmd = plan_with(&file, &dst, "نسخة.txt").unwrap();
        assert_eq!(cmd.artifact.unwrap().kind, ArtifactKind::File);
    }

    #[test]
    fn a_destination_inside_the_source_is_refused() {
        let s = Scratch::new("copy-nested").unwrap();
        let src = s.dir("مصدر");
        let dst = s.dir("مصدر/داخل");
        assert_eq!(
            refusal(plan_with(&src, &dst, "ن")),
            ("err.dest.inside_source", Some("destination"))
        );
    }

    #[test]
    fn copying_a_directory_onto_itself_is_refused() {
        let s = Scratch::new("copy-same").unwrap();
        let dir = s.dir("واحد");
        assert_eq!(refusal(plan_with(&dir, &dir, "ن")), ("err.path.same", Some("destination")));
    }

    #[test]
    fn an_existing_name_in_the_destination_stops_the_plan_before_anything_runs() {
        let s = Scratch::new("copy-exists").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود"), b"PRECIOUS").unwrap();

        assert_eq!(refusal(plan_with(&src, &dst, "موجود")), ("err.dest.exists", Some("new_name")));
        assert_eq!(std::fs::read(dst.join("موجود")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("copy-shellish").unwrap();
        let src = s.dir("م");
        let dst = s.dir("و");

        for name in ["نسخة 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let cmd = plan_with(&src, &dst, name).unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(name));
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("copy-dashes").unwrap();
        let src = s.dir("-rf");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "-x").unwrap();
        for a in cmd.args.iter().skip(2) {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn a_symlinked_source_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("copy-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&link, &dst, "ن").unwrap();
        assert_eq!(Path::new(&cmd.args[2]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_copy_raises_no_noisy_warning() {
        let s = Scratch::new("copy-quiet").unwrap();
        let src = s.dir("م");
        std::fs::write(src.join("ملف"), vec![0u8; 64]).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ن").unwrap();
        for noisy in ["warn.source.empty", "warn.size.partial", "warn.space.low"] {
            assert!(!cmd.warnings.contains(&noisy), "unexpected {noisy}: {:?}", cmd.warnings);
        }
    }

    #[test]
    fn planning_leaves_nothing_behind_in_either_directory() {
        let s = Scratch::new("copy-clean").unwrap();
        let src = s.dir("م");
        std::fs::write(src.join("ملف"), b"data").unwrap();
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&src, &dst, "ن").unwrap();
        }

        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
        assert_eq!(s.names(&src), vec!["ملف".to_string()], "the source must not be written to");
    }
}
