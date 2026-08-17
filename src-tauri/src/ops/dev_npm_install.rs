//! تثبيت حزم مشروع Node.js، باستخدام `npm install`.
//!
//! ## لماذا `Modifies` لا `Safe`
//!
//! يكتب مجلد `node_modules` — ينشئه أو يغيّره — وقد يعدّل `package-lock.json`
//! إن لم يطابق `package.json` تمامًا. هذا تعديلٌ حقيقي على المشروع، فالشارة
//! تقوله صراحةً بدل أن تعرض العملية بوصفها قراءةً.
//!
//! ## لماذا `npm install` لا `npm run install`
//!
//! `install` أمرٌ فرعي أصيل في `npm`، لا نصًّا معلَنًا في `package.json` —
//! انظر توثيق `dev_common::npm` لماذا هذه العملية وحدها لا تبني على
//! `npm_run`.

use crate::error::Result;
use crate::ops::dev_common::npm;
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.npm.install";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.npm.install.title",
    description_key: "op.dev.npm.install.description",
    category: Category::Developer,
    danger: Danger::Modifies,
    visibility: Visibility::Production,
    tool: crate::tools::NPM,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("node_path", InputKind::ExistingFile),
    ],
    sort_order: 40,
    search_terms: &["install", "npm", "node", "تثبيت", "حزم", "packages", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let node_path = inputs.file("node_path")?;
    npm(node_path, project).flag("install", "explain.npm.install").reveal(project).read_only()
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
    fn the_operation_is_listed_in_its_category_and_marked_as_modifying() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.category, Category::Developer);
        assert_eq!(found.danger, Danger::Modifies);
    }

    #[test]
    fn the_argv_is_npm_install_not_npm_run_install() {
        let s = Scratch::new("npm-install").unwrap();
        let project = s.dir("مشروع");
        let node = node_stub(&s);
        let cmd = plan_with(&project, &node).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, vec!["install"], "must not be ['run', 'install']");
    }
}
