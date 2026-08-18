//! اختبارات الحدّ الأمني.
//!
//! سؤال واحد يحكم هذا الملف: **هل تستطيع الواجهة أن تجعل النواة تشغّل شيئًا
//! لم يخطّط له الفهرس؟** كل اختبار هنا محاولة لذلك، ويجب أن تفشل المحاولة.

use naffith_core::error::{CoreError, NameRejection};
use naffith_core::paths;
use naffith_core::planner;
use naffith_core::plans::{PlanStore, PlanToken};
use naffith_core::policy::Policy;
use naffith_core::registry;
use naffith_core::value::RawValue;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn inputs(pairs: &[(&str, RawValue)]) -> BTreeMap<String, RawValue> {
    pairs.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect()
}

fn msg(s: &str) -> BTreeMap<String, RawValue> {
    inputs(&[("message", RawValue::Text(s.into()))])
}

/// مساحة اختبار داخل المنزل. `tempfile` يضع مجلداته تحت `/var` وهو خارج
/// الجذور المسموحة، فاختبار السياسة الحقيقية يحتاج موضعًا مسموحًا.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Option<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let base = home.join(format!(".naffith-sec-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).ok()?;
        Some(Scratch(base.canonicalize().ok()?))
    }
    fn dir(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// الرفض كما تراه الواجهة: مفتاح الخطأ والحقل المسؤول عنه.
///
/// أقوى من مطابقة شكل `enum`: خطأٌ صحيح منسوبٌ إلى الحقل الخطأ يظهر للمستخدم
/// تحت حقل لا علاقة له به، وهذا فشلُ منتجٍ لا يمسكه `matches!`.
fn refusal<T>(r: naffith_core::error::Result<T>) -> (&'static str, Option<&'static str>) {
    match r {
        Ok(_) => panic!("expected a refusal, got success"),
        Err(e) => (e.key(), e.input()),
    }
}

fn compress_inputs(source: &Path, destination: &Path, name: &str) -> BTreeMap<String, RawValue> {
    inputs(&[
        ("source", RawValue::Path(source.display().to_string())),
        ("destination", RawValue::Path(destination.display().to_string())),
        ("archive_name", RawValue::Text(name.to_owned())),
    ])
}

// ── ١. الواجهة لا تستطيع التعبير عن أمر ────────────────────────────────

/// السمة كما تُطابَق: **بادئة** لا حرفًا مغلقًا.
///
/// كانت الحرّاس تطابق `#[tauri::command]` بقوسه المغلق، و`#[tauri::command(
/// rename_all = "snake_case")]` صيغة قياسية في Tauri لا تطابقها. أمرٌ مكتوب
/// بالصيغة ذات الخيارات كان يمرّ من تحت كل حارس في هذا الملف وهو يعلن
/// `program` و`args` و`cwd` صراحةً.
const COMMAND_ATTRIBUTE: &str = "#[tauri::command";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// كل ملفات `src/` مقروءة، لا `lib.rs` وحده.
///
/// `include_str!("../src/lib.rs")` كان يجعل الحرّاس عمياء عن أبسط إعادة
/// تنظيم: `pub mod backdoor;` ثم `backdoor::run_raw` في `generate_handler!`.
/// الماكرو يقبل مسارًا مؤهَّلًا بوحدة، فالأمر يُسجَّل ولا يراه أي حارس.
fn source_files() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
        let mut entries: Vec<PathBuf> =
            std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                out.push((path, text));
            }
        }
    }
    let mut out = Vec::new();
    walk(&manifest_dir().join("src"), &mut out);
    assert!(out.len() > 1, "the source walk found nothing — the guards below would be vacuous");
    out
}

/// توقيع كل دالة تحمل سمة أمر، في كل ملف مصدر.
///
/// التوقيع يمتدّ من السمة إلى أول `{`، أي جسم الدالة.
fn command_signatures() -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    for (path, src) in source_files() {
        for (idx, _) in src.match_indices(COMMAND_ATTRIBUTE) {
            let sig_end = src[idx..].find('{').map(|e| idx + e).unwrap_or(src.len());
            out.push((path.clone(), src[idx..sig_end].replace('\n', " ")));
        }
    }
    out
}

/// أسماء الأوامر كما هي **مسجَّلة** في `generate_handler!`.
///
/// هذه هي القائمة الموثوقة: ما ليس فيها لا تصل إليه الواجهة مهما حمل من
/// سمات، وما فيها يصل مهما كُتب أو أين وُضع. عدّ السمات وحده كان يقيس
/// الشيء الخطأ.
fn registered_commands() -> Vec<String> {
    let src = std::fs::read_to_string(manifest_dir().join("src/lib.rs")).unwrap();
    let start = src.find("generate_handler![").expect("the handler list must exist");
    let open = start + "generate_handler![".len();
    let close = open + src[open..].find(']').expect("unterminated handler list");
    src[open..close]
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// حارس بنيوي: لا أمر يقبل برنامجًا أو وسائط.
///
/// اختبار سلوكي لا يمكنه إثبات غياب شيء؛ هذا يثبته. إن أضاف أحدهم لاحقًا
/// `fn run_raw(argv: Vec<String>)` بسمة أمر — في أي ملف، وبأي صيغة للسمة —
/// يسقط البناء هنا.
#[test]
fn no_ipc_command_accepts_a_program_or_arguments() {
    let forbidden = [
        "argv",
        "args:",
        "program",
        "cmd:",
        "command:",
        "executable",
        "shell",
        "cwd:",
        "working_dir",
        "env:",
    ];

    let mut offenders = Vec::new();
    for (path, signature) in command_signatures() {
        for needle in forbidden {
            if signature.contains(needle) {
                offenders.push(format!("`{needle}` in {}: {signature}", path.display()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "an IPC command exposes a raw-execution parameter:\n{}",
        offenders.join("\n")
    );
}

/// حارس بنيوي ثانٍ: لا أمر IPC يقبل مسارًا خامًا.
///
/// المسارات تعبر الحدّ **داخل مدخلات عملية موصوفة** (`RawValue::Path`) وحدها،
/// فتمرّ حتمًا بـ `value::validate` ثم `paths`. أمرٌ يأخذ `path: String`
/// مباشرة كان سيتجاوز ذلك كلّه.
#[test]
fn no_ipc_command_accepts_a_bare_path() {
    let mut offenders = Vec::new();
    for (path, signature) in command_signatures() {
        for needle in ["path:", "PathBuf", "dir:", "file:", "target:", "source:"] {
            if signature.contains(needle) {
                offenders.push(format!("`{needle}` in {}: {signature}", path.display()));
            }
        }
    }
    assert!(offenders.is_empty(), "an IPC command takes a raw path:\n{}", offenders.join("\n"));
}

/// القائمة المسجَّلة هي الحدّ، وهي تسعة بالاسم.
#[test]
fn the_ipc_surface_is_exactly_the_nine_registered_commands() {
    assert_eq!(
        registered_commands(),
        vec![
            "list_operations",
            "list_categories",
            "plan",
            "execute",
            "cancel",
            "recent_runs",
            "journal_delete",
            "journal_clear",
            "reveal"
        ],
        "the IPC surface changed. Every command is attack surface — \
         update this test deliberately, and say why in the commit.\n\
         M1 raised this from 5 to 6 by adding `reveal`, whose only parameter is a run id: \
         the core looks the produced path up in its own journal, so the frontend still \
         cannot express a path. The alternative (an opener plugin) would have given it one.\n\
         The library milestone raised it from 6 to 9, and each addition was weighed:\n\
         · `list_categories` returns metadata and two counts computed from the registry. \
           It takes no parameter and reads nothing outside the compiled catalogue. The \
           alternative — counting in TypeScript — would have put a second, silently \
           ageing index in the frontend.\n\
         · `journal_delete` takes a run id, exactly like `reveal`. The core decides which \
           lines that id owns; the frontend cannot name a file, a line, or an offset.\n\
         · `journal_clear` takes nothing at all.\n\
         Neither journal command touches anything outside the journal file: a run whose \
         entry is deleted still has its output on disk. Erasing the trace is not erasing \
         what it did."
    );
}

/// كل أمر معلَن يقع في `lib.rs` وحده.
///
/// بلا هذا يظلّ الحدّ قابلًا للتوسّع بملف جديد: الماكرو يقبل `backdoor::run_raw`
/// بلا اعتراض، وكل الحرّاس البنيوية هنا تقرأ ما تعرف مكانه. حصر الإعلان في
/// ملفٍ واحد هو ما يجعل «اقرأ حدّ IPC» جملةً لها معنى.
#[test]
fn every_command_is_declared_in_the_ipc_boundary_file() {
    let mut elsewhere = Vec::new();
    let mut in_lib = 0;
    for (path, src) in source_files() {
        let count = src.matches(COMMAND_ATTRIBUTE).count();
        if count == 0 {
            continue;
        }
        if path.ends_with("lib.rs") {
            in_lib = count;
        } else {
            elsewhere.push(format!("{} declares {count} command(s)", path.display()));
        }
    }
    assert!(
        elsewhere.is_empty(),
        "a Tauri command is declared outside the IPC boundary file:\n{}",
        elsewhere.join("\n")
    );
    assert_eq!(
        in_lib,
        registered_commands().len(),
        "lib.rs declares {in_lib} commands but registers {} — every declared command must be \
         accounted for, and every registered one must be visible to the guards in this file",
        registered_commands().len()
    );
}

/// لا قفلٍ في حدّ IPC يُترجَم إلى خطأ في المجال.
///
/// كان كل موضع يكتب `.lock().map_err(|_| CoreError::PlanNotFound)`، فذعرٌ
/// واحد يسمّم القفل ويصير كل تخطيط بعده «الخطة غير موجودة» إلى نهاية الجلسة:
/// عطبٌ دائم يُبلَّغ عنه بخطأ كاذب. التعافي مركزيّ الآن في `recover`، وهذا
/// الحارس يمنع عودة النمط بالسهو.
#[test]
fn no_lock_failure_is_reported_as_a_domain_error() {
    let src = std::fs::read_to_string(manifest_dir().join("src/lib.rs")).unwrap();
    assert!(
        !src.contains(".lock().map_err"),
        "a poisoned lock must be recovered, not translated into a plan or run error — \
         a transient panic would otherwise brick planning for the rest of the session"
    );
}

/// وكل اسم مسجَّل له دالة بذلك الاسم.
#[test]
fn every_registered_command_names_a_function_in_the_boundary_file() {
    let src = std::fs::read_to_string(manifest_dir().join("src/lib.rs")).unwrap();
    for name in registered_commands() {
        assert!(
            !name.contains("::"),
            "`{name}` is registered through a module path, which puts it outside the guards"
        );
        assert!(
            src.contains(&format!("fn {name}(")),
            "registered command `{name}` has no function"
        );
    }
}

#[test]
fn execute_takes_only_a_token() {
    let src = include_str!("../src/lib.rs");
    let start = src.find("async fn execute").expect("execute must exist");
    let sig = &src[start..start + 220];
    assert!(sig.contains("token: String"), "execute must accept a token");
    // لا مسارات ولا وسائط ولا خيارات إضافية.
    for forbidden in ["Path", "argv", "args", "Vec<String>", "inputs"] {
        assert!(
            !sig.contains(forbidden),
            "execute must not accept `{forbidden}` — everything runnable is already in the core"
        );
    }
}

#[test]
fn reveal_takes_only_a_run_id() {
    let src = include_str!("../src/lib.rs");
    let start = src.find("fn reveal(").expect("reveal must exist");
    let sig = &src[start..start + 120];
    assert!(sig.contains("run_id: String"), "reveal must accept a run id");
    for forbidden in ["Path", "path", "produced", "Vec<"] {
        assert!(!sig.contains(forbidden), "reveal must not accept `{forbidden}`");
    }
}

/// خارج النواة صلاحيتان: حوار الفتح، وفحص التحديث. هذا الاختبار يثبّت
/// الصلاحيات **المحلولة** كي لا تتوسّع بالسهو إلى الحفظ أو النوافذ أو نظام
/// الملفات أو HTTP عام.
///
/// قراءة `default.json` وحده كانت تكذب: `tauri-build` يجمع
/// `capabilities/**/*` كلّه، فملفٌ ثانٍ يمنح
/// `core:webview:allow-create-webview-window` — أي نافذة عرض على عنوان
/// اعتباطي، وهي قناة تسريب تلتفّ حول `default-src 'self'` — كان يمرّ والاختبار
/// أخضر. الملف المحلول في `gen/schemas` هو ما يُبنى منه التطبيق فعلًا.
///
/// و`updater:default` أُضيفت عن قصد لا سهوًا: هي أول صلاحية شبكة هنا، وحارسها
/// التوقيع لا هذا الملف — لا تُثبَّت نسخة إلا بتوقيعٍ يطابق `pubkey`. وتبقى
/// `http:` ممنوعةً أدناه: فرقٌ بين وجهةٍ واحدة معلَنة موقَّعة، وبين منح
/// الواجهة شبكةً عامة.
#[test]
fn the_resolved_capability_set_grants_nothing_beyond_the_open_dialog_and_the_updater() {
    let path = manifest_dir().join("gen/schemas/capabilities.json");
    let resolved = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("{} is written by tauri-build; build the crate before testing: {e}", path.display())
    });

    let parsed: serde_json::Value = serde_json::from_str(&resolved).unwrap();
    let mut granted: Vec<&str> = parsed
        .as_object()
        .expect("the resolved file is a map of capability id to capability")
        .values()
        .flat_map(|cap| cap["permissions"].as_array().unwrap())
        .map(|p| p.as_str().unwrap())
        .collect();
    granted.sort_unstable();

    assert_eq!(
        granted,
        vec!["core:default", "dialog:allow-open", "updater:default"],
        "the resolved capability set changed. Every permission is attack surface — \
         widen it deliberately."
    );
    for banned in [
        "fs:",
        "shell:",
        "http:",
        "process:",
        "dialog:allow-save",
        "opener:",
        "allow-create-webview",
        "core:window:allow-create",
    ] {
        assert!(!resolved.contains(banned), "`{banned}` must never appear in the resolved ACL");
    }
}

/// ولا ملف صلاحيات ثانٍ أصلًا.
///
/// الاختبار السابق يمسك التوسّع بعد البناء؛ هذا يمسكه في المصدر، ويجعل جملة
/// «الصلاحيات في ملف واحد» صحيحة بدل أن تكون عادةً.
///
/// الفرز بالامتداد لا بالنقطة البادئة: جرّبتُ `capabilities/.sneaky.json`
/// فوجدته يُمنح كاملًا في المجموعة المحلولة، بينما `.DS_Store` — لا امتداد
/// له — يتجاهله `tauri-build`. استثناء الملفات المخفيّة كان سيفتح ثغرة،
/// واستثناء عديم الامتداد يمنع فشلًا كاذبًا لا يمنح شيئًا.
#[test]
fn the_capabilities_directory_holds_exactly_one_capability_file() {
    let dir = manifest_dir().join("capabilities");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "json" || e == "json5" || e == "toml"))
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec!["default.json"],
        "tauri-build globs capabilities/**/* — anything parseable here is granted"
    );
}

// ── ٢. الفهرس والسياسة ─────────────────────────────────────────────────

#[test]
fn an_unknown_operation_is_refused() {
    let mut store = PlanStore::new();
    let s = store.register_session().unwrap();
    let r = planner::plan(&mut store, &s, Policy::dev(), "../../etc/passwd", &msg("x"));
    assert!(matches!(r, Err(CoreError::UnknownOperation(_))));
}

#[test]
fn the_internal_operation_is_unreachable_in_production() {
    let mut store = PlanStore::new();
    let s = store.register_session().unwrap();
    let r = planner::plan(&mut store, &s, Policy::production(), "internal.echo", &msg("x"));
    assert!(matches!(r, Err(CoreError::OperationNotAvailable(_))), "got {r:?}");
    assert!(store.is_empty(), "a refused plan must leave no trace");
}

#[test]
fn the_internal_operation_is_absent_from_the_catalogue_in_every_build() {
    for policy in [Policy::production(), Policy::dev()] {
        assert!(registry::list(policy).iter().all(|o| !o.id.starts_with("internal.")));
    }
}

#[test]
fn the_compress_operation_is_reachable_in_production() {
    // الوجه الآخر للاختبار السابق: السياسة تمنع الداخلي ولا تمنع الإنتاجي.
    assert!(registry::find("compress.folder.zip", Policy::production()).is_ok());
}

// ── ٣. المدخلات ────────────────────────────────────────────────────────

#[test]
fn an_undeclared_input_key_is_refused_not_ignored() {
    let mut store = PlanStore::new();
    let s = store.register_session().unwrap();
    let r = planner::plan(
        &mut store,
        &s,
        Policy::dev(),
        "internal.echo",
        &inputs(&[
            ("message", RawValue::Text("ok".into())),
            ("extra_args", RawValue::Text("--force".into())),
        ]),
    );
    assert!(matches!(r, Err(CoreError::UnexpectedInput(k)) if k == "extra_args"));
}

#[test]
fn a_path_cannot_be_smuggled_where_text_is_declared() {
    let mut store = PlanStore::new();
    let s = store.register_session().unwrap();
    let r = planner::plan(
        &mut store,
        &s,
        Policy::dev(),
        "internal.echo",
        &inputs(&[("message", RawValue::Path("/etc/passwd".into()))]),
    );
    assert!(matches!(r, Err(CoreError::WrongInputType { .. })));
}

#[test]
fn an_extra_flag_cannot_be_injected_into_the_compress_operation() {
    let s = Scratch::new("inject").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("م");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let mut raw = compress_inputs(&src, &dst, "ناتج");
    // محاولة إضافة راية تُسقط البيانات الوصفية.
    raw.insert("options".to_owned(), RawValue::Text("--norsrc".to_owned()));

    let r = planner::plan(&mut store, &session, Policy::production(), "compress.folder.zip", &raw);
    assert!(matches!(&r, Err(CoreError::UnexpectedInput(k)) if k == "options"), "got {r:?}");
}

// ── ٤. المسارات ────────────────────────────────────────────────────────

#[test]
fn relative_paths_are_refused() {
    assert!(matches!(paths::existing_dir(Path::new("Documents")), Err(CoreError::PathNotAbsolute)));
}

#[test]
fn parent_traversal_is_refused() {
    for p in ["/Users/../etc", "/tmp/../../etc", "/Volumes/disk/../../private"] {
        assert!(
            matches!(paths::existing_dir(Path::new(p)), Err(CoreError::PathTraversal)),
            "traversal not caught for {p}"
        );
    }
}

#[test]
fn system_locations_are_outside_the_allowed_roots() {
    for p in ["/etc", "/usr/bin", "/System", "/private/var", "/Library"] {
        let r = paths::existing_dir(Path::new(p));
        assert!(
            matches!(r, Err(CoreError::PathOutsideAllowedRoots) | Err(CoreError::PathMissing)),
            "{p} should be refused by policy, got {r:?}"
        );
    }
}

#[test]
fn a_system_location_cannot_be_used_as_a_compress_source() {
    let s = Scratch::new("syssrc").expect("HOME must be set for the path policy to be exercised");
    let dst = s.dir("و");
    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();

    let r = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(Path::new("/etc"), &dst, "سرقة"),
    );
    assert_eq!(refusal(r), ("err.path.outside", Some("source")));
    assert!(store.is_empty(), "a refused plan must leave no trace");
}

#[test]
fn a_symlink_pointing_out_of_the_allowed_roots_cannot_be_used_as_a_source() {
    // الرابط داخل المنزل، وهدفه خارجه. السياسة تُفحص بعد الحلّ لا قبله.
    let s = Scratch::new("escape").expect("HOME must be set for the path policy to be exercised");
    let dst = s.dir("و");
    let link = s.0.join("خارج");
    std::os::unix::fs::symlink("/etc", &link).unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let r = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&link, &dst, "ناتج"),
    );
    assert_eq!(refusal(r), ("err.path.outside", Some("source")));
}

#[test]
fn a_symlinked_destination_pointing_out_of_the_allowed_roots_is_refused() {
    // المصدر سليم والوجهة هي الهاربة. فحص المصدر وحده كان سيسمح بكتابة أرشيف
    // خارج الجذور المسموحة.
    let s = Scratch::new("escapedst").expect("HOME must be set");
    let src = s.dir("م");
    let link = s.0.join("وجهة-هاربة");
    std::os::unix::fs::symlink("/private/tmp", &link).unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let r = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &link, "ناتج"),
    );
    assert_eq!(refusal(r), ("err.path.outside", Some("destination")));
}

#[test]
fn a_symlink_in_an_intermediate_component_cannot_smuggle_the_path_outside() {
    // لا الطرف الأول ولا الأخير رابط — الرابط في الوسط. `canonicalize` تحلّ
    // المسار كاملًا، فيقع الفحص على الهدف الحقيقي لا على شكل المسار.
    let s = Scratch::new("midlink").expect("HOME must be set");
    let dst = s.dir("و");
    let bridge = s.0.join("جسر");
    std::os::unix::fs::symlink("/private/etc", &bridge).unwrap();
    let through = bridge.join("ssl");
    if !through.exists() {
        return; // لا يوجد /private/etc/ssl على هذه الآلة
    }

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let r = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&through, &dst, "عبر-الجسر"),
    );
    assert_eq!(refusal(r), ("err.path.outside", Some("source")));
}

#[test]
fn a_symlink_swapped_between_plan_and_execute_cannot_redirect_the_run() {
    // هجوم TOCTOU الكلاسيكي: يُخطَّط على رابط يشير إلى مجلد بريء، ثم يُبدَّل
    // قبل التنفيذ ليشير إلى غيره.
    //
    // الدفاع هنا ليس كشف التبديل، بل أن التبديل لا يغيّر شيئًا: المسار يُحلّ
    // **وقت التخطيط**، والمُحلّ هو ما يدخل `argv`. فالأمر مربوط بالهدف الذي
    // فُحصت سياسته ورآه المستخدم في «سَطْر»، لا بالاسم الذي قد يتبدّل تحته.
    let s = Scratch::new("swap").expect("HOME must be set");
    let innocent = s.dir("بريء");
    let other = s.dir("آخر");
    std::fs::write(innocent.join("a.txt"), b"a").unwrap();
    std::fs::write(other.join("b.txt"), b"b").unwrap();
    let dst = s.dir("و");

    let link = s.0.join("متحوّل");
    std::os::unix::fs::symlink(&innocent, &link).unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&link, &dst, "ناتج"),
    )
    .map(|p| p.response)
    .expect("planning against the innocent target should succeed");

    // الأمر يحمل الهدف المحلول، لا اسم الرابط.
    let innocent_real = innocent.canonicalize().unwrap();
    assert!(
        plan.argv_display.iter().any(|a| Path::new(a) == innocent_real),
        "argv must carry the resolved target: {:?}",
        plan.argv_display
    );
    assert!(
        !plan.argv_display.iter().any(|a| Path::new(a) == link),
        "argv must never carry an unresolved symlink path"
    );

    // بدّل الرابط بعد التخطيط وقبل التنفيذ.
    std::fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(&other, &link).unwrap();

    let stored = store.take(&plan.token, &session).expect("the token is still valid");
    stored.verify_still_valid().expect("the resolved target is untouched, so the plan stays valid");

    // والأهم: ما زال الأمر يشير إلى البريء لا إلى الآخر.
    let args: Vec<String> =
        stored.command.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    assert!(
        args.iter().any(|a| Path::new(a) == innocent_real),
        "the swap must not redirect the run"
    );
    assert!(
        !args.iter().any(|a| Path::new(a) == other.canonicalize().unwrap()),
        "the run must never touch the swapped-in target"
    );
}

#[test]
fn a_source_replaced_by_a_symlink_after_planning_invalidates_the_plan() {
    // الاتجاه المعاكس: مجلد حقيقي وقت التخطيط، يُستبدل برابط قبل التنفيذ.
    let s = Scratch::new("becomelink").expect("HOME must be set");
    let src = s.dir("مصدر");
    std::fs::write(src.join("a.txt"), b"a").unwrap();
    let elsewhere = s.dir("مكان-آخر");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "ناتج"),
    )
    .unwrap()
    .response;

    std::fs::remove_dir_all(&src).unwrap();
    std::os::unix::fs::symlink(&elsewhere, &src).unwrap();

    let stored = store.take(&plan.token, &session).unwrap();
    assert!(
        matches!(stored.verify_still_valid(), Err(CoreError::PlanStale { .. })),
        "a source that became a symlink must invalidate the plan"
    );
}

#[test]
fn sensitive_directories_inside_home_are_protected() {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return };
    let ssh = home.join(".ssh");
    if !ssh.is_dir() {
        return; // لا مفاتيح على هذه الآلة — لا شيء لنثبته
    }
    assert!(
        matches!(paths::existing_dir(&ssh), Err(CoreError::PathProtected)),
        "~/.ssh must be refused even though it sits inside an allowed root"
    );
}

#[test]
fn a_protected_directory_cannot_be_compressed() {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return };
    let ssh = home.join(".ssh");
    if !ssh.is_dir() {
        return;
    }
    let s =
        Scratch::new("protected").expect("HOME must be set for the path policy to be exercised");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let r = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&ssh, &dst, "مفاتيح"),
    );
    assert_eq!(
        refusal(r),
        ("err.path.protected", Some("source")),
        "compressing ~/.ssh is exfiltration, not a service"
    );
}

#[test]
fn the_destination_may_not_sit_inside_the_source() {
    let s = Scratch::new("nested").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("مصدر");
    let dst = s.dir("مصدر/داخل");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let r = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "ناتج"),
    );
    assert_eq!(refusal(r), ("err.dest.inside_source", Some("destination")));
}

#[test]
fn a_file_name_cannot_carry_a_path_separator() {
    for name in ["../escape.zip", "a/b.zip", "/absolute.zip"] {
        assert!(
            matches!(
                paths::sanitize_name(name),
                Err(CoreError::InvalidName { reason: NameRejection::ContainsSeparator })
            ),
            "{name} must not be accepted as a file name"
        );
    }
}

#[test]
fn an_archive_name_cannot_escape_the_destination() {
    let s =
        Scratch::new("escape-name").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("م");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    for name in ["../خارج.zip", "../../Library/سرقة.zip", "sub/dir.zip"] {
        let r = planner::plan(
            &mut store,
            &session,
            Policy::production(),
            "compress.folder.zip",
            &compress_inputs(&src, &dst, name),
        );
        assert_eq!(
            refusal(r),
            ("err.name.invalid", Some("archive_name")),
            "name {name:?} must not escape the destination"
        );
    }
}

#[test]
fn a_file_name_cannot_carry_a_nul_or_control_byte() {
    assert!(matches!(
        paths::sanitize_name("a\0b"),
        Err(CoreError::InvalidName { reason: NameRejection::ContainsNul })
    ));
    assert!(matches!(
        paths::sanitize_name("a\rb"),
        Err(CoreError::InvalidName { reason: NameRejection::ContainsControl })
    ));
}

#[test]
fn shell_metacharacters_are_legal_in_a_file_name() {
    // ليست تساهلًا: الأمر argv فلا مفسّر يراها. رفضها كان سيمنع أسماء مشروعة.
    for name in ["report; final.zip", "$HOME backup.zip", "a&b.zip", "مشروع (٢).zip"] {
        assert!(paths::sanitize_name(name).is_ok(), "{name} is a legitimate file name");
    }
}

// ── ٥. رموز الخطط ──────────────────────────────────────────────────────

#[test]
fn a_forged_token_is_refused() {
    let mut store = PlanStore::new();
    let s = store.register_session().unwrap();
    store.insert(&s, "internal.echo", Default::default(), dummy()).unwrap();

    for forged in ["", "0", &"f".repeat(64), &"0".repeat(64)] {
        assert!(
            matches!(
                store.take(&PlanToken::from(forged.to_string()), &s),
                Err(CoreError::PlanNotFound)
            ),
            "forged token {forged:?} was accepted"
        );
    }
}

#[test]
fn a_token_cannot_be_replayed() {
    let mut store = PlanStore::new();
    let s = store.register_session().unwrap();
    let (t, _) = store.insert(&s, "internal.echo", Default::default(), dummy()).unwrap();

    assert!(store.take(&t, &s).is_ok(), "first use succeeds");
    assert!(
        matches!(store.take(&t, &s), Err(CoreError::PlanNotFound)),
        "a replayed token must be refused"
    );
}

#[test]
fn a_real_compress_token_cannot_be_replayed_either() {
    // إعادة الإرسال على عملية حقيقية، لا على خطة وهمية: الضغط مرتين بنفس
    // الرمز يعني أرشيفين أو كتابةً فوق ناتج.
    let s = Scratch::new("replay").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("م");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "ناتج"),
    )
    .unwrap()
    .response;

    let token = PlanToken::from(plan.token.as_str().to_owned());
    assert!(store.take(&token, &session).is_ok());
    assert!(matches!(store.take(&token, &session), Err(CoreError::PlanNotFound)));
}

#[test]
fn the_plan_id_in_the_response_is_not_the_token() {
    let s = Scratch::new("planid").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("م");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "ناتج"),
    )
    .unwrap()
    .response;

    assert_ne!(plan.plan_id, plan.token.as_str());
    // ولا يُقبل معرّف الخطة مكان الرمز.
    assert!(matches!(
        store.take(&PlanToken::from(plan.plan_id.clone()), &session),
        Err(CoreError::PlanNotFound)
    ));
}

#[test]
fn an_expired_token_is_refused() {
    let mut store = PlanStore::with_ttl(std::time::Duration::from_millis(10));
    let s = store.register_session().unwrap();
    let (t, _) = store.insert(&s, "internal.echo", Default::default(), dummy()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(30));
    assert!(matches!(store.take(&t, &s), Err(CoreError::PlanNotFound)));
}

#[test]
fn plans_are_bounded_so_the_core_cannot_be_flooded() {
    let mut store = PlanStore::new();
    let s = store.register_session().unwrap();
    let mut accepted = 0;
    for _ in 0..1000 {
        if store.insert(&s, "internal.echo", Default::default(), dummy()).is_ok() {
            accepted += 1;
        }
    }
    assert!(accepted <= 1000);
    assert!(
        store.len() <= naffith_core::plans::MAX_PLANS_PER_SESSION,
        "the live plan count is what is bounded: older plans are evicted, not refused"
    );
}

fn dummy() -> naffith_core::spec::PlannedCommand {
    naffith_core::spec::PlannedCommand {
        program: PathBuf::from("/bin/echo"),
        args: vec![],
        cwd: None,
        explain: vec![],
        warnings: vec![],
        artifact: None,
        estimate: None,
        stdout_to: None,
        reveal_target: None,
        extra_path: Vec::new(),
    }
}
