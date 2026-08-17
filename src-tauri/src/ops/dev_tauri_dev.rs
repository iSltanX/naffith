//! تشغيل تطبيق Tauri في وضع التطوير، باستخدام `tauri dev`.
//!
//! ## طويلة التشغيل، كـ`dev.npm.dev` — والسبب مختلف قليلًا
//!
//! لا تنتهي من تلقاء نفسها، والإلغاء طريقها الطبيعي الوحيد للإنهاء — نفس
//! المنطق الموثَّق في `dev_npm_dev.rs`. لكنها هنا **تُنشئ فعلًا**: `tauri
//! dev` تستدعي `cargo build`/`cargo run` داخليًا فتكتب نواتج بناءٍ حقيقية في
//! `src-tauri/target`، ولذلك `Danger::Creates` لا `Safe` — طول التشغيل
//! وأثره على القرص محوران مستقلّان، لا يتبع أحدهما الآخر.
//!
//! ## Tauri CLI محليًا لا `npx`
//!
//! `node_modules/.bin/tauri` — نسخة `tauri-cli` التي يُثبّتها `package.json`
//! هذا المشروع بعينه. لا `npx`: تلك قد تُنزّل نسخةً مختلفة صامتًا لو غابت
//! نسخةٌ محلية، وهذا تنزيلٌ شبكي غير معلَن لم يطلبه المستخدم.
//!
//! ## PATH: النظام وNode معًا
//!
//! `tauri.js` شِبَنغ `#!/usr/bin/env node` (فتحتاج دليل Node)، وتستدعي
//! `cargo` التي تحتاج `cc`/`xcrun` (فتحتاج أدلّة النظام). انظر
//! `dev_common::tauri_path_env`.

use crate::error::Result;
use crate::ops::dev_common::tauri_cli;
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.tauri.dev";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.tauri.dev.title",
    description_key: "op.dev.tauri.dev.description",
    category: Category::Developer,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: crate::tools::TAURI_CLI,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("node_path", InputKind::ExistingFile),
    ],
    sort_order: 60,
    search_terms: &["tauri", "dev", "npm", "node", "خادم", "تطوير", "rust", "cargo", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let node_path = inputs.file("node_path")?;
    tauri_cli(node_path, project).value("dev").reveal(project).read_only()
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

    fn plan_with(project: &Path, node: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("project".to_owned(), RawValue::Path(project.display().to_string())),
            ("node_path".to_owned(), RawValue::Path(node.display().to_string())),
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
    fn the_argv_invokes_the_projects_local_tauri_cli_with_dev() {
        let s = Scratch::new("tauri-dev").unwrap();
        let project = project_with_tauri_cli(&s);
        let node = node_stub(&s);

        let cmd = plan_with(&project, &node).unwrap();
        assert_eq!(cmd.program, project.join("node_modules/.bin/tauri"));
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, vec!["dev"]);
    }

    #[test]
    fn the_child_path_carries_both_node_and_system_toolchain_dirs() {
        let s = Scratch::new("tauri-dev-path").unwrap();
        let project = project_with_tauri_cli(&s);
        let node = node_stub(&s);
        let cmd = plan_with(&project, &node).unwrap();
        assert_eq!(
            cmd.extra_path,
            vec![
                node.parent().unwrap().to_path_buf(),
                std::path::PathBuf::from("/usr/bin"),
                std::path::PathBuf::from("/bin"),
                std::path::PathBuf::from("/usr/sbin"),
                std::path::PathBuf::from("/sbin"),
            ]
        );
    }

    #[test]
    fn a_project_without_a_local_tauri_cli_is_refused_with_a_clear_reason() {
        let s = Scratch::new("tauri-dev-missing-cli").unwrap();
        let project = s.dir("مشروع بلا حزم");
        let node = node_stub(&s);
        let err = plan_with(&project, &node).unwrap_err();
        assert_eq!(err.key(), "err.tool.missing");
    }
}
