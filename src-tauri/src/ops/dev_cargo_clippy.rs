//! فحص Clippy لمشروع Rust، باستخدام `cargo clippy --all-targets -- -D warnings`.
//!
//! نظير `dev_cargo_test.rs` في اشتقاق الأمر وPATH — راجعه للتفصيل.
//!
//! ## لماذا `--all-targets`
//!
//! بلا هذه الراية يتخطّى Clippy أهداف الاختبار والمقاييس (‏`tests/`،
//! `benches/`)، فكودٌ فيها قد يحمل تحذيرات لا تظهر أبدًا. هذا هو الشكل
//! الذي يستعمله هذا المشروع نفسه في `lint:core` — وليس تعميمًا من عندنا،
//! بل معيار Clippy الشائع لفحصٍ كامل لا جزئي.
//!
//! ## ولماذا `-D warnings` بعد `--`
//!
//! `--` تفصل رايات `cargo clippy` عن الرايات التي تمرّ إلى Clippy نفسه.
//! ‏`-D warnings` تحويل كل تحذير إلى خطأ — فحصٌ صارم يوافق ما يفعله CI في
//! أي مشروع جادّ، لا تساهلًا يخفي مشاكل حقيقية.

use crate::error::Result;
use crate::ops::dev_common::{cargo, CargoManifest};
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.cargo.clippy";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.cargo.clippy.title",
    description_key: "op.dev.cargo.clippy.description",
    category: Category::Developer,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: crate::tools::CARGO,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("cargo_path", InputKind::ExistingFile),
    ],
    sort_order: 100,
    search_terms: &["cargo", "clippy", "rust", "فحص", "أسلوب", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let cargo_path = inputs.file("cargo_path")?;

    cargo(cargo_path)
        .flag("clippy", "explain.cargo.clippy")
        .with_manifest(project)
        .flag("--all-targets", "explain.cargo.all_targets")
        .flag("--", "explain.end_of_flags")
        .flag("-D", "explain.cargo.deny")
        .value("warnings")
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
    fn the_argv_matches_the_documented_form_exactly() {
        let s = Scratch::new("cargo-clippy").unwrap();
        let project = s.dir("مشروع");
        let cargo_path = cargo_stub(&s);
        let cmd = plan_with(&project, &cargo_path).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(
            args,
            vec![
                "clippy",
                "--manifest-path",
                &project.join("Cargo.toml").to_string_lossy(),
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ]
        );
    }
}
