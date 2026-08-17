//! ما تشترك فيه عمليات أدوات المطوّرين: حلّ Node.js، وPATH الموحَّد الذي
//! تحتاجه هذه العمليات وحدها دون بقية المنتج.
//!
//! ## لماذا Node.js مختلفة عن كل أداةٍ أخرى في هذا الملف
//!
//! كل أداةٍ في `tools.rs` مسارها ثابتٌ في الشيفرة (`/usr/bin/ditto`). Node.js
//! ليست كذلك: يثبّتها المستخدم بنفسه (nvm، Homebrew، المثبِّت الرسمي)، في
//! مسارٍ يختلف من جهازٍ لآخر ويقع غالبًا داخل مجلده — لا وجود لمسارٍ نظامي
//! ثابت يشبه `/usr/bin/ditto`. فالمستخدم يختاره مرّةً في الإعدادات (حقل
//! `node_path` في كل عملية هنا)، وتتحقّق النواة منه في كل تخطيط كما تتحقّق من
//! أي أداةٍ أخرى — لا فرق في الصرامة، فقط في مصدر المسار.
//!
//! ## ولماذا PATH أصلًا — أليس `executor.rs` يمسحه؟
//!
//! يمسحه، وهذا صحيحٌ لأداةٍ مفردة كـ`ditto`. لكن `npm` نفسها ملفٌّ يبدأ
//! بشِبَنغ `#!/usr/bin/env node` — أي أن **تشغيل `npm` بمسارها المطلق ذاته**
//! يحتاج نظام التشغيل أن يجد `node` عبر `PATH` كي يفسّر الشِبَنغ، قبل أن يصل
//! التنفيذ إلى `npm` نفسها بخطوةٍ واحدة. أُثبت هذا تجريبيًا: `npm` بمسارٍ
//! مطلق وبيئةٍ ممسوحة بالكامل تفشل بـ«‏env: node: No such file or
//! directory» فورًا. الحلّ هنا ضيّق ومقصود: **دليل Node وحده** يُضاف إلى
//! `PATH`، لا شيء من بيئة المستخدم الفعلية.

use crate::ops::common::Argv;
use std::path::{Path, PathBuf};

/// دليل الأدوات المرافقة لِـNode.js (`npm`، `npx`) — أبو الملف الذي تحقّقت
/// النواة منه في `plan()` كل عملية.
pub fn node_bin_dir(node_path: &Path) -> &Path {
    node_path.parent().unwrap_or(node_path)
}

/// ‏`PATH` الذي تحتاجه أداةٌ تُستدعى بشِبَنغ `#!/usr/bin/env node` — وهذا شكل
/// `npm` نفسها، وحزم npm القابلة للتنفيذ محليًا (`node_modules/.bin/*`) كلّها.
pub fn node_path_env(node_path: &Path) -> Vec<PathBuf> {
    vec![node_bin_dir(node_path).to_path_buf()]
}

/// يبني `npm` وحدها لمشروعٍ ومسار Node تحقّقت منه `plan()` بالفعل، جاهزةً
/// لإضافة الأمر الفرعي (`install`، أو `run <script>`) وإنهائها.
///
/// أساسٌ مشترك لا يُستدعى مباشرةً من عمليةٍ عادةً — `npm_run` و`npm install`
/// (في `dev_npm_install.rs`) يبنيان عليه، لأن اشتقاق `npm` من `node` ومجلد
/// العمل وPATH واحدٌ حرفيًا بين الجميع.
pub fn npm(node_path: &Path, project: &Path) -> Argv {
    let bin = node_bin_dir(node_path);
    let npm_path = bin.join("npm");
    Argv::resolved_tool("npm", &npm_path, "explain.npm.tool")
        .with_extra_path(node_path_env(node_path))
        .in_directory(project)
}

/// يبني `npm run <script>`. **ليست** `npm install` — تلك أمرٌ فرعي أصيل لا
/// نصٌّ معلَنٌ في `package.json`، فتُبنى من `npm()` مباشرةً في
/// `dev_npm_install.rs` لا من هنا؛ `npm run install` كانت ستبحث عن نصٍّ
/// باسم «install» قلّما يُعرَّف، بدل تثبيت الحزمة الفعلي.
pub fn npm_run(node_path: &Path, project: &Path, script: &'static str) -> Argv {
    npm(node_path, project).flag("run", "explain.npm.run").value(script)
}

/// أدلّة النظام التي تحتاجها `cargo` — التي تستدعيها `tauri dev`/`tauri
/// build` داخليًا — لاستدعاء المصرّف والرابط (`cc`، `xcrun`).
///
/// أربعة أدلّة ثابتة مملوكة لِـroot ومحميّة بـSIP، لا صلة لها بـPATH
/// المستخدم الفعلي. أُثبتت تجريبيًا: `cargo check` تفشل بـ«‏linker `cc` not
/// found» بلا هذه الأدلّة بالضبط، وتنجح بها.
pub const SYSTEM_TOOLCHAIN_DIRS: &[&str] = &["/usr/bin", "/bin", "/usr/sbin", "/sbin"];

/// مسار Tauri CLI المحلّي للمشروع (`node_modules/.bin/tauri`)، لا `npm`
/// العامّة: نسخة `tauri-cli` التي يُثبّتها `package.json` هذا المشروع بعينه،
/// لا أي نسخةٍ أخرى قد تُحمَّل عبر `npx` صامتًا لو غابت محليًا.
pub fn tauri_cli_path(project: &Path) -> PathBuf {
    project.join("node_modules").join(".bin").join("tauri")
}

/// ‏`PATH` الذي تحتاجه `tauri dev`/`tauri build`: دليل Node (فـ`tauri.js`
/// نفسها شِبَنغ `#!/usr/bin/env node`) ثم أدلّة نظام `cargo` — كلتا الحاجتين
/// معًا لا إحداهما، لأن الأمر يستدعي الاثنين.
pub fn tauri_path_env(node_path: &Path) -> Vec<PathBuf> {
    let mut dirs = node_path_env(node_path);
    dirs.extend(SYSTEM_TOOLCHAIN_DIRS.iter().map(PathBuf::from));
    dirs
}

/// يبني أمر Tauri CLI لمشروعٍ ومسار Node تحقّقت منه `plan()` بالفعل، جاهزًا
/// لإضافة الوسيط الخاص (`dev` أو `build`) وإنهائه.
pub fn tauri_cli(node_path: &Path, project: &Path) -> Argv {
    let cli = tauri_cli_path(project);
    Argv::resolved_tool("tauri", &cli, "explain.tauri.tool")
        .with_extra_path(tauri_path_env(node_path))
        .in_directory(project)
}

/// يبني أمر Cargo مسارُه تحقّقت منه `plan()` بالفعل، جاهزًا لإضافة الأمر
/// الفرعي (`test`، `check`، …) ثم `.with_manifest(project)` وإنهائه.
///
/// **‏`--manifest-path` لا مجلد عمل**: خلافًا لـ`npm` — التي لا تملك رايةً
/// مكافئة، فتحتاج `.in_directory()` لتجد `package.json` — تملك `cargo` هذه
/// الراية أصلًا، فتُستعمل صراحةً كما تُستعمل `-C` في عمليات Git: الأمر
/// المعروض في «سَطْر» يقول أين يعمل بدل أن يُترك ذلك ضمنيًّا في `cwd` غير
/// المرئي. وهي — عمدًا — ليست وسيطًا لهذه الدالّة: تُضاف بعد الأمر الفرعي
/// عبر `with_manifest`، لا قبله.
pub fn cargo(cargo_path: &Path) -> Argv {
    Argv::resolved_tool("cargo", cargo_path, "explain.cargo.tool")
        .with_extra_path(SYSTEM_TOOLCHAIN_DIRS.iter().map(PathBuf::from).collect())
}

/// راية `--manifest-path`، مؤجَّلةٌ عمدًا حتى بعد الأمر الفرعي.
///
/// أُثبت تجريبيًا: `cargo --manifest-path <p> check` تُرفض («‏Usage: cargo
/// [+toolchain] [OPTIONS] [COMMAND]»)، و`cargo check --manifest-path <p>`
/// تعمل. فترتيب البناء في كل عملية `cargo(...).flag("check", …).with_manifest(project)`
/// لا العكس — والترتيب هنا موثَّقٌ لا اعتباطي، فمن يعكسه يكسر الأمر صامتًا.
pub trait CargoManifest {
    fn with_manifest(self, project: &Path) -> Self;
}

impl CargoManifest for Argv {
    fn with_manifest(self, project: &Path) -> Self {
        self.flag("--manifest-path", "explain.cargo.manifest_path")
            .path(&project.join("Cargo.toml"))
    }
}
