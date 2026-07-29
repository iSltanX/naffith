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

/// حارس بنيوي: يقرأ مصدر حدّ IPC ويثبت أن لا أمر يقبل برنامجًا أو وسائط.
///
/// اختبار سلوكي لا يمكنه إثبات غياب شيء؛ هذا يثبته. إن أضاف أحدهم لاحقًا
/// `#[tauri::command] fn run_raw(argv: Vec<String>)` يسقط البناء هنا.
#[test]
fn no_ipc_command_accepts_a_program_or_arguments() {
    let src = include_str!("../src/lib.rs");

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
    for (idx, _) in src.match_indices("#[tauri::command]") {
        // توقيع الدالة يقع بعد السمة مباشرة وينتهي عند أول `{`.
        let rest = &src[idx..];
        let sig_end = rest.find('{').map(|e| idx + e).unwrap_or(src.len());
        let signature = &src[idx..sig_end];
        for needle in forbidden {
            if signature.contains(needle) {
                offenders.push(format!("`{needle}` in: {}", signature.replace('\n', " ")));
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
    let src = include_str!("../src/lib.rs");
    let mut offenders = Vec::new();
    for (idx, _) in src.match_indices("#[tauri::command]") {
        let rest = &src[idx..];
        let sig_end = rest.find('{').map(|e| idx + e).unwrap_or(src.len());
        let signature = &src[idx..sig_end];
        for needle in ["path:", "PathBuf", "dir:", "file:", "target:", "source:"] {
            if signature.contains(needle) {
                offenders.push(format!("`{needle}` in: {}", signature.replace('\n', " ")));
            }
        }
    }
    assert!(offenders.is_empty(), "an IPC command takes a raw path:\n{}", offenders.join("\n"));
}

#[test]
fn the_ipc_surface_is_exactly_the_six_declared_commands() {
    let src = include_str!("../src/lib.rs");
    let count = src.matches("#[tauri::command]").count();
    assert_eq!(
        count, 6,
        "the IPC surface changed. Every command is attack surface — \
         update this test deliberately, and say why in the commit.\n\
         M1 raised this from 5 to 6 by adding `reveal`, whose only parameter is a run id: \
         the core looks the produced path up in its own journal, so the frontend still \
         cannot express a path. The alternative (an opener plugin) would have given it one."
    );
    for expected in
        ["fn list_operations", "fn plan", "fn execute", "fn cancel", "fn recent_runs", "fn reveal"]
    {
        assert!(src.contains(expected), "missing expected command: {expected}");
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

/// الإضافة الوحيدة خارج النواة هي حوار اختيار مجلد. هذا الاختبار يثبّت
/// صلاحياتها كي لا تتوسّع بالسهو إلى الحفظ أو الرسائل أو نظام الملفات.
#[test]
fn the_capability_file_grants_nothing_beyond_the_folder_picker() {
    let capability = include_str!("../capabilities/default.json");
    let parsed: serde_json::Value = serde_json::from_str(capability).unwrap();
    let granted: Vec<&str> =
        parsed["permissions"].as_array().unwrap().iter().map(|p| p.as_str().unwrap()).collect();

    assert_eq!(
        granted,
        vec!["core:default", "dialog:allow-open"],
        "the capability set changed. Every permission is attack surface — widen it deliberately."
    );
    for banned in ["fs:", "shell:", "http:", "process:", "dialog:allow-save", "opener:"] {
        assert!(
            !capability.contains(banned),
            "`{banned}` must never appear in the capability file"
        );
    }
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
    .unwrap();

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
    .unwrap();

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
    .unwrap();

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
    }
}
