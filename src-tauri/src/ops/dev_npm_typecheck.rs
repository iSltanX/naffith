//! فحص الأنواع لمشروع Node.js، باستخدام `npm run typecheck`.
//!
//! ## لماذا نصّ `package.json` لا `tsc` مباشرة
//!
//! مشاريع TypeScript تُهيّئ `tsc` بخيارات مختلفة (‏`--noEmit`، مشاريع مراجع،
//! ملفّا إعداد لواجهةٍ ونواة، …)، وسكربت `typecheck` في `package.json` هو
//! العقد الذي يعلنه المشروع نفسه عن معنى «فحص الأنواع» عنده. استدعاء `tsc`
//! مباشرةً كان يفترض إعدادًا واحدًا لا يصحّ لكل مشروع.
//!
//! ## الصيغة
//!
//! ```text
//! <npm> run typecheck
//! ```
//! بمجلد عملٍ هو المشروع، وPATH يحوي دليل Node.js وحده — انظر `dev_common.rs`
//! لسبب حاجتها إليه أصلًا.
//!
//! ## ما لا تفعله
//!
//! لا تكتب شيئًا في المشروع: `tsc --noEmit` هو الشكل الشائع لسكربتٍ كهذا،
//! ولو كتب مشروعٌ ملفّات `.d.ts`) فهذا قرار ذلك المشروع لا هذه العملية.
//! والخرج نثرٌ بشريّ بلا نحوٍ ثابت عبر المشاريع، فيُعرض خامًا — لا تُخترع له
//! بنية.

use crate::error::Result;
use crate::ops::dev_common::npm_run;
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.npm.typecheck";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.npm.typecheck.title",
    description_key: "op.dev.npm.typecheck.description",
    category: Category::Developer,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: crate::tools::NPM,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("node_path", InputKind::ExistingFile),
    ],
    sort_order: 10,
    search_terms: &["typecheck", "npm", "node", "فحص", "أنواع", "tsc", "typescript", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let node_path = inputs.file("node_path")?;

    npm_run(node_path, project, "typecheck").reveal(project).read_only()
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
        let node = bin.join("node");
        std::fs::write(&node, b"#!/bin/sh\nexit 0\n").unwrap();
        std::fs::write(bin.join("npm"), b"#!/bin/sh\nexit 0\n").unwrap();
        for p in [&node, &bin.join("npm")] {
            let mut perm = std::fs::metadata(p).unwrap().permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
            std::fs::set_permissions(p, perm).unwrap();
        }
        node
    }

    fn plan_with(project: &Path, node: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("project".to_owned(), RawValue::Path(project.display().to_string())),
            ("node_path".to_owned(), RawValue::Path(node.display().to_string())),
        ]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    fn args_of(cmd: &PlannedCommand) -> Vec<String> {
        cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.category, Category::Developer);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_runs_npm_run_typecheck_in_the_project() {
        let s = Scratch::new("npm-typecheck").unwrap();
        let project = s.dir("مشروع");
        let node = node_stub(&s);

        let cmd = plan_with(&project, &node).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, node.parent().unwrap().join("npm"));
        assert_eq!(args, vec!["run", "typecheck"]);
        assert_eq!(cmd.cwd.as_deref(), Some(project.as_path()));
        assert!(cmd.artifact.is_none());
        assert_eq!(cmd.reveal_target.as_deref(), Some(project.as_path()));
    }

    #[test]
    fn the_child_path_is_exactly_the_node_directory_plus_system_dirs() {
        let s = Scratch::new("npm-typecheck-path").unwrap();
        let project = s.dir("مشروع");
        let node = node_stub(&s);

        let cmd = plan_with(&project, &node).unwrap();
        assert_eq!(cmd.extra_path, vec![node.parent().unwrap().to_path_buf()]);
    }

    #[test]
    fn the_explanation_is_the_argv_itself() {
        let s = Scratch::new("npm-typecheck-explain").unwrap();
        let project = s.dir("مشروع");
        let node = node_stub(&s);

        let cmd = plan_with(&project, &node).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn a_project_without_npm_beside_node_is_refused() {
        let s = Scratch::new("npm-typecheck-missing-npm").unwrap();
        let project = s.dir("مشروع");
        let bin = s.dir("bin-no-npm");
        let node = bin.join("node");
        std::fs::write(&node, b"#!/bin/sh\n").unwrap();
        let mut perm = std::fs::metadata(&node).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
        std::fs::set_permissions(&node, perm).unwrap();

        let err = plan_with(&project, &node).unwrap_err();
        assert_eq!(err.key(), "err.tool.missing");
    }

    #[test]
    fn a_project_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("npm-typecheck-missing-project").unwrap();
        let node = node_stub(&s);
        let err = plan_with(&s.path().join("لا-وجود"), &node).unwrap_err();
        assert_eq!(err.key(), "err.path.missing");
        assert_eq!(err.input(), Some("project"));
    }

    #[test]
    fn a_node_path_outside_the_allowed_roots_is_refused() {
        let s = Scratch::new("npm-typecheck-node-outside").unwrap();
        let project = s.dir("مشروع");
        // ملفٌ حقيقيّ قائم خارج الجذور المسموحة، لا مسارٌ غائب: الغياب
        // يُرفض بمفتاحٍ آخر (`err.path.missing`) قبل أن يُفحص كونه خارجًا.
        let err = plan_with(&project, Path::new("/etc/hosts")).unwrap_err();
        assert_eq!(err.key(), "err.path.outside");
        assert_eq!(err.input(), Some("node_path"));
    }
}
