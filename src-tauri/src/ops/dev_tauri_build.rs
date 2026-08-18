//! بناء تطبيق Tauri للإصدار، باستخدام `tauri build`.
//!
//! نظير `dev_tauri_dev.rs` في اشتقاق الأمر ومسار PATH بالكامل — راجعه
//! للتفصيل. الفرق الوحيد هنا: هذه **تنتهي من تلقاء نفسها** (بناءٌ طويل لكنه
//! محدود، لا خادمٌ مستمر)، فتسلك مسار التشغيل العادي بلا خصوصية الإلغاء.
//! و`Creates` لنفس السبب هناك: تكتب حزمة تطبيقٍ جديدة في `target/release`.

use crate::error::Result;
use crate::ops::dev_common::tauri_cli;
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.tauri.build";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.tauri.build.title",
    description_key: "op.dev.tauri.build.description",
    category: Category::Developer,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: crate::tools::TAURI_CLI,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("node_path", InputKind::ExistingFile),
        InputSpec::new("cargo_path", InputKind::ExistingFile),
    ],
    sort_order: 70,
    search_terms: &["tauri", "build", "npm", "node", "بناء", "إصدار", "rust", "cargo", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let node_path = inputs.file("node_path")?;
    let cargo_path = inputs.file("cargo_path")?;
    tauri_cli(node_path, cargo_path, project).value("build").reveal(project).read_only()
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
        let p = bin.join("node");
        std::fs::write(&p, b"#!/bin/sh\nexit 0\n").unwrap();
        let mut perm = std::fs::metadata(&p).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
        std::fs::set_permissions(&p, perm).unwrap();
        p
    }

    fn cargo_stub(s: &Scratch) -> PathBuf {
        let bin = s.dir("cargo-bin");
        let p = bin.join("cargo");
        std::fs::write(&p, b"#!/bin/sh\nexit 0\n").unwrap();
        let mut perm = std::fs::metadata(&p).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
        std::fs::set_permissions(&p, perm).unwrap();
        p
    }

    fn project_with_tauri_cli(s: &Scratch) -> PathBuf {
        let project = s.dir("مشروع");
        let bin = project.join("node_modules").join(".bin");
        std::fs::create_dir_all(&bin).unwrap();
        let tauri = bin.join("tauri");
        std::fs::write(&tauri, b"#!/bin/sh\nexit 0\n").unwrap();
        let mut perm = std::fs::metadata(&tauri).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
        std::fs::set_permissions(&tauri, perm).unwrap();
        project
    }

    fn plan_with(project: &Path, node: &Path, cargo_path: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("project".to_owned(), RawValue::Path(project.display().to_string())),
            ("node_path".to_owned(), RawValue::Path(node.display().to_string())),
            ("cargo_path".to_owned(), RawValue::Path(cargo_path.display().to_string())),
        ]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    #[test]
    fn the_operation_is_listed_in_its_category_and_marked_as_creating() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.category, Category::Developer);
        assert_eq!(found.danger, Danger::Creates);
    }

    #[test]
    fn the_argv_invokes_the_projects_local_tauri_cli_with_build() {
        let s = Scratch::new("tauri-build").unwrap();
        let project = project_with_tauri_cli(&s);
        let node = node_stub(&s);
        let cargo_path = cargo_stub(&s);
        let cmd = plan_with(&project, &node, &cargo_path).unwrap();
        assert_eq!(cmd.program, project.join("node_modules/.bin/tauri"));
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, vec!["build"]);
    }

    /// انحدارٌ مباشر لـC-3: بلا `cargo_path`، `tauri build` كانت تفشل عند
    /// أوّل استدعاءٍ لـ`cargo` داخليًا — دليلها غائبٌ عن `PATH` كاملًا.
    #[test]
    fn the_child_path_carries_the_cargo_directory_too() {
        let s = Scratch::new("tauri-build-path").unwrap();
        let project = project_with_tauri_cli(&s);
        let node = node_stub(&s);
        let cargo_path = cargo_stub(&s);
        let cmd = plan_with(&project, &node, &cargo_path).unwrap();
        assert!(
            cmd.extra_path.contains(&cargo_path.parent().unwrap().to_path_buf()),
            "cargo's directory must be on PATH: {:?}",
            cmd.extra_path
        );
    }

    #[test]
    fn a_cargo_path_outside_the_allowed_roots_is_refused() {
        let s = Scratch::new("tauri-build-cargo-outside").unwrap();
        let project = project_with_tauri_cli(&s);
        let node = node_stub(&s);
        let err = plan_with(&project, &node, Path::new("/etc/hosts")).unwrap_err();
        assert_eq!(err.key(), "err.path.outside");
        assert_eq!(err.input(), Some("cargo_path"));
    }
}
