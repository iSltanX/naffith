//! فحص بناء مشروع Rust سريعًا، باستخدام `cargo check`.
//!
//! نظير `dev_cargo_test.rs` تمامًا في كل قرارٍ إلا الأمر الفرعي — راجعه
//! للتفصيل الكامل: لماذا `--manifest-path` بعد الأمر الفرعي لا قبله، ولماذا
//! PATH يحتاج أدلّة النظام وحدها.
//!
//! والفرق عن `test`: يتحقّق من صحّة الأنواع والاستدلال بلا توليد ملفٍّ
//! تنفيذي كامل، فهو أسرع بكثير — الفحص الذي يشغّله محرّر أثناء الكتابة.

use crate::error::Result;
use crate::ops::dev_common::{cargo, CargoManifest};
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.cargo.check";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.cargo.check.title",
    description_key: "op.dev.cargo.check.description",
    category: Category::Developer,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: crate::tools::CARGO,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("cargo_path", InputKind::ExistingFile),
    ],
    sort_order: 90,
    search_terms: &["cargo", "check", "rust", "فحص", "بناء", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let cargo_path = inputs.file("cargo_path")?;

    cargo(cargo_path)
        .flag("check", "explain.cargo.check")
        .with_manifest(project)
        .reveal(project)
        .read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    fn cargo_stub(s: &Scratch) -> PathBuf {
        let bin = s.dir("bin");
        let p = bin.join("cargo");
        std::fs::write(&p, b"#!/bin/sh\nexit 0\n").unwrap();
        let mut perm = std::fs::metadata(&p).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
        std::fs::set_permissions(&p, perm).unwrap();
        p
    }

    fn plan_with(project: &Path, cargo_path: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("project".to_owned(), RawValue::Path(project.display().to_string())),
            ("cargo_path".to_owned(), RawValue::Path(cargo_path.display().to_string())),
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
    fn the_argv_is_check_then_manifest_path() {
        let s = Scratch::new("cargo-check").unwrap();
        let project = s.dir("مشروع");
        let cargo_path = cargo_stub(&s);
        let cmd = plan_with(&project, &cargo_path).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args[0], "check");
        assert_eq!(args[1], "--manifest-path");
    }
}
