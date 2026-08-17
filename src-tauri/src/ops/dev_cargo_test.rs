//! تشغيل اختبارات مشروع Rust، باستخدام `cargo test`.
//!
//! ## لماذا `--manifest-path` لا `--manifest-path` قبل الأمر الفرعي
//!
//! أُثبت تجريبيًا: `cargo --manifest-path <p> test` تُرفض بخطأ استعمال
//! (‏«‏Usage: cargo [+toolchain] [OPTIONS] [COMMAND]»)، و`cargo test
//! --manifest-path <p>` تعمل. الراية بعد الأمر الفرعي إذن، لا قبله — انظر
//! `dev_common::CargoManifest` للتوثيق الكامل.
//!
//! ## PATH: أدلّة النظام وحدها
//!
//! ‏`cargo` ملفٌّ تنفيذي أصيل لا شِبَنغ، فلا حاجة إلى دليل Node كما احتاجت
//! `npm`. لكنها تستدعي `cc`/`xcrun` للربط، وهذا مُثبَتٌ تجريبيًا أيضًا: بلا
//! أدلّة النظام تفشل حتى `cargo check` بـ«‏linker `cc` not found».

use crate::error::Result;
use crate::ops::dev_common::{cargo, CargoManifest};
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.cargo.test";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.cargo.test.title",
    description_key: "op.dev.cargo.test.description",
    category: Category::Developer,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: crate::tools::CARGO,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("cargo_path", InputKind::ExistingFile),
    ],
    sort_order: 80,
    search_terms: &["cargo", "test", "rust", "اختبار", "اختبارات", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let cargo_path = inputs.file("cargo_path")?;

    cargo(cargo_path)
        .flag("test", "explain.cargo.test")
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
    fn the_argv_is_test_then_manifest_path_not_before_it() {
        let s = Scratch::new("cargo-test").unwrap();
        let project = s.dir("مشروع");
        let cargo_path = cargo_stub(&s);

        let cmd = plan_with(&project, &cargo_path).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(cmd.program, cargo_path);
        assert_eq!(args[0], "test", "the subcommand must come first");
        assert_eq!(args[1], "--manifest-path");
        assert_eq!(Path::new(&args[2]), project.join("Cargo.toml").as_path());
        assert!(cmd.cwd.is_none(), "the project is named in the argv via --manifest-path, not cwd");
    }

    #[test]
    fn the_child_path_is_exactly_the_system_toolchain_dirs() {
        let s = Scratch::new("cargo-test-path").unwrap();
        let project = s.dir("مشروع");
        let cargo_path = cargo_stub(&s);
        let cmd = plan_with(&project, &cargo_path).unwrap();
        assert_eq!(
            cmd.extra_path,
            vec![
                PathBuf::from("/usr/bin"),
                PathBuf::from("/bin"),
                PathBuf::from("/usr/sbin"),
                PathBuf::from("/sbin"),
            ]
        );
    }

    #[test]
    fn a_cargo_path_outside_the_allowed_roots_is_refused() {
        let s = Scratch::new("cargo-test-outside").unwrap();
        let project = s.dir("مشروع");
        let err = plan_with(&project, Path::new("/etc/hosts")).unwrap_err();
        assert_eq!(err.key(), "err.path.outside");
        assert_eq!(err.input(), Some("cargo_path"));
    }
}
