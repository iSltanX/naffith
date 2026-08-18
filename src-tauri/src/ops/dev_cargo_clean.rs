//! حذف نواتج بناء مشروع Rust كاملةً، باستخدام `cargo clean`.
//!
//! ## لماذا `Destructive` — وهي الأولى بهذه الدرجة في قسم أدوات المطوّرين
//!
//! تحذف `target/` كاملًا: قد تكون غيغابايتات، ولا تراجع عن الحذف نفسه. لكنها
//! أخفّ من أخطر عملية أخرى في هذا التطبيق (إنهاء عملية جارية، في مرحلةٍ
//! لاحقة): كل ما تحذفه هنا **قابلٌ للتوليد من جديد** — إعادة بناءٍ تكلّف
//! وقتًا لا بياناتٍ مفقودة. الشارة تُعلن الدرجة الصحيحة لهذا بالضبط: تلفٌ لا
//! رجعة فيه، لا كارثة.
//!
//! والباقي نظير `dev_cargo_test.rs` — راجعه لتفصيل PATH واشتقاق الأمر.

use crate::error::Result;
use crate::ops::dev_common::{cargo, CargoManifest};
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.cargo.clean";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.cargo.clean.title",
    description_key: "op.dev.cargo.clean.description",
    category: Category::Developer,
    danger: Danger::Destructive,
    visibility: Visibility::Production,
    tool: crate::tools::CARGO,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("cargo_path", InputKind::ExistingFile),
    ],
    sort_order: 140,
    search_terms: &["cargo", "clean", "rust", "حذف", "تنظيف", "target", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let cargo_path = inputs.file("cargo_path")?;

    cargo(cargo_path)
        .flag("clean", "explain.cargo.clean")
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
    fn the_operation_is_listed_in_its_category_and_marked_as_destructive() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.category, Category::Developer);
        assert_eq!(found.danger, Danger::Destructive);
    }

    #[test]
    fn the_argv_is_clean_then_manifest_path() {
        let s = Scratch::new("cargo-clean").unwrap();
        let project = s.dir("مشروع");
        let cargo_path = cargo_stub(&s);
        let cmd = plan_with(&project, &cargo_path).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args[0], "clean");
        assert_eq!(args[1], "--manifest-path");
    }
}
