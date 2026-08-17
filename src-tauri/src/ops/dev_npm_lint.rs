//! فحص الأسلوب لمشروع Node.js، باستخدام `npm run lint`.
//!
//! نظير `dev_npm_typecheck.rs` تمامًا في كل قرارٍ إلا اسم النصّ. راجع ذلك
//! الملف للتفصيل الكامل: لماذا نصّ `package.json` لا الأداة مباشرةً، ولماذا
//! PATH يحتاج دليل Node وحده.

use crate::error::Result;
use crate::ops::dev_common::npm_run;
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.npm.lint";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.npm.lint.title",
    description_key: "op.dev.npm.lint.description",
    category: Category::Developer,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: crate::tools::NPM,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("node_path", InputKind::ExistingFile),
    ],
    sort_order: 20,
    search_terms: &["lint", "npm", "node", "أسلوب", "eslint", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let node_path = inputs.file("node_path")?;
    npm_run(node_path, project, "lint").reveal(project).read_only()
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
    fn the_argv_runs_npm_run_lint() {
        let s = Scratch::new("npm-lint").unwrap();
        let project = s.dir("مشروع");
        let node = node_stub(&s);
        let cmd = plan_with(&project, &node).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, vec!["run", "lint"]);
        assert_eq!(cmd.cwd.as_deref(), Some(project.as_path()));
    }
}
