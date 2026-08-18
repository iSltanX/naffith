//! التحقّق من تنسيق مشروع Rust دون تعديله، باستخدام `cargo fmt --check`.
//!
//! ## قراءةٌ فقط — نظير `dev_cargo_fmt.rs` بفرقٍ واحد
//!
//! هذه وتلك تبنيان الأمر نفسه بالضبط عدا `--check`: الأولى تخبر أن هناك ما
//! يحتاج إعادة تنسيق ولا تكتب شيئًا (‏`Safe`)، والثانية تكتبه فعلًا
//! (‏`Modifies` — في `dev_cargo_fmt.rs`). فصلهما عمليتين منفصلتين — لا رايةً
//! اختيارية في عمليةٍ واحدة — يجعل الفارق بينهما مرئيًا في شارة الخطورة قبل
//! أن يضغط المستخدم «نفّذ»، لا مخفيًّا داخل نموذج.

use crate::error::Result;
use crate::ops::dev_common::{cargo, CargoManifest};
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.cargo.fmt.check";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.cargo.fmt.check.title",
    description_key: "op.dev.cargo.fmt.check.description",
    category: Category::Developer,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: crate::tools::CARGO,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("cargo_path", InputKind::ExistingFile),
    ],
    sort_order: 110,
    search_terms: &["cargo", "fmt", "format", "rust", "تنسيق", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let cargo_path = inputs.file("cargo_path")?;

    cargo(cargo_path)
        .flag("fmt", "explain.cargo.fmt")
        .with_manifest(project)
        .flag("--check", "explain.cargo.fmt.check")
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
    fn the_operation_is_listed_in_its_category_and_marked_safe() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.category, Category::Developer);
        assert_eq!(found.danger, Danger::Safe);
    }

    #[test]
    fn the_argv_carries_check_so_nothing_is_rewritten() {
        let s = Scratch::new("cargo-fmt-check").unwrap();
        let project = s.dir("مشروع");
        let cargo_path = cargo_stub(&s);
        let cmd = plan_with(&project, &cargo_path).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args[0], "fmt");
        assert!(args.contains(&"--check".to_owned()));
    }
}
