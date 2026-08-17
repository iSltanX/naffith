//! تشغيل اختبارات مشروع Node.js، باستخدام `npm run test`.
//!
//! نظير `dev_npm_typecheck.rs` تمامًا في كل قرارٍ إلا اسم النصّ — راجعه
//! للتفصيل الكامل.
//!
//! ولماذا `Safe` رغم أن أطر اختبارٍ بعينها قد تكتب تقارير تغطية أو لقطات
//! (‏snapshots) على القرص: هذا سلوك **إطار الاختبار الذي يختاره المشروع**، لا
//! سلوك هذه العملية — تمامًا كما لا تعرف هذه العملية إن كان سكربت `test`
//! يستدعي واحدًا معيّنًا أصلًا. الصدق هنا في وصف ما تعرفه العملية، لا في
//! التكهّن بما قد يفعله المشروع.

use crate::error::Result;
use crate::ops::dev_common::npm_run;
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.npm.test";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.npm.test.title",
    description_key: "op.dev.npm.test.description",
    category: Category::Developer,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: crate::tools::NPM,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("node_path", InputKind::ExistingFile),
    ],
    sort_order: 30,
    search_terms: &["test", "npm", "node", "اختبار", "اختبارات", "vitest", "jest", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let node_path = inputs.file("node_path")?;
    npm_run(node_path, project, "test").reveal(project).read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    fn node_stub(s: &Scratch) -> PathBuf {
        let bin = s.dir("bin");
        for name in ["node", "npm"] {
            let p = bin.join(name);
            std::fs::write(&p, b"#!/bin/sh\nexit 0\n").unwrap();
            let mut perm = std::fs::metadata(&p).unwrap().permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
            std::fs::set_permissions(&p, perm).unwrap();
        }
        bin.join("node")
    }

    fn plan_with(project: &Path, node: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("project".to_owned(), RawValue::Path(project.display().to_string())),
            ("node_path".to_owned(), RawValue::Path(node.display().to_string())),
        ]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.category, Category::Developer);
        assert_eq!(found.danger, Danger::Safe);
    }

    #[test]
    fn the_argv_runs_npm_run_test() {
        let s = Scratch::new("npm-test").unwrap();
        let project = s.dir("مشروع");
        let node = node_stub(&s);
        let cmd = plan_with(&project, &node).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, vec!["run", "test"]);
    }
}
