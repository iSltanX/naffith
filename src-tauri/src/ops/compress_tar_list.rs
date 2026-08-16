//! فحص محتويات أرشيف `TAR` أو `TAR.GZ` قبل استخراجه.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/tar -t -v -f <الأرشيف>
//! ```
//!
//! `-t` تعرض الفهرس ولا تكتب شيئًا، و`-v` تضيف الصلاحيات والمالك والحجم
//! والتاريخ — أي تجعل السطر يشبه `ls -l` بدل أن يكون اسمًا عاريًا. الفرق مهمّ
//! هنا بالذات: **هذه هي العملية التي يقرأ بها المستخدم أسماء المدخلات قبل
//! الاستخراج**، ولأن TAR لا يُفحص فهرسه في النواة (انظر `compress_tar_extract`)
//! فهي الطريق الوحيد اليوم لرؤية مدخلةٍ اسمها `../..` قبل أن تُفكّ.
//!
//! والضغط يُكتشف من محتوى الملف لا من امتداده، فلا رايةَ صيغةٍ هنا كما في
//! الاستخراج تمامًا.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "compress.tar.list";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.compress.tar.list.title",
    description_key: "op.compress.tar.list.description",
    category: Category::Compress,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::TAR,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("archive", InputKind::ExistingFile)],
    sort_order: 70,
    search_terms: &["tar", "tgz", "قائمة", "محتويات", "list", "فحص", "tarball", "inspect"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let archive_path = inputs.file("archive")?;

    let mut argv = Argv::tool(tools::TAR, "explain.tar.tool")
        .flag("-t", "explain.tar.list")
        .flag("-v", "explain.tar.verbose")
        .flag("-f", "explain.tar.file")
        .path(archive_path);

    if let Some(parent) = archive_path.parent() {
        argv = argv.reveal(parent);
    }
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

    fn plan_with(p: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([("archive".to_owned(), RawValue::Path(p.display().to_string()))]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    #[test]
    fn the_operation_is_listed_as_read_only() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let s = Scratch::new("tarl-argv").unwrap();
        let a = s.file("أرشيف.tar.gz", b"x");
        let cmd = plan_with(&a).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|x| x.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/tar"));
        assert_eq!(args, vec!["-t".to_string(), "-v".into(), "-f".into(), a.display().to_string()]);
        assert!(cmd.artifact.is_none(), "listing must produce nothing");
        assert_eq!(cmd.reveal_target.as_deref(), Some(s.path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("tarl-explain").unwrap();
        let a = s.file("a.tar", b"x");
        let cmd = plan_with(&a).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|x| x.to_string_lossy().into_owned()));
        assert_eq!(cmd.explain.iter().map(|t| t.token.clone()).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("tarl-dash").unwrap();
        let a = s.file("-weird.tar", b"x");
        let cmd = plan_with(&a).unwrap();
        assert!(cannot_be_read_as_a_flag(cmd.args.last().unwrap()));
    }

    #[test]
    fn a_directory_cannot_be_passed_where_a_file_is_declared() {
        let s = Scratch::new("tarl-dir").unwrap();
        let d = s.dir("مجلد");
        let e = plan_with(&d).unwrap_err();
        assert_eq!(e.input(), Some("archive"));
    }
}
