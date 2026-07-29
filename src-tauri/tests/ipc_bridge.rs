//! جسر IPC الحقيقي: من JSON الذي ترسله الواجهة إلى JSON الذي يعود إليها.
//!
//! ## الفجوة التي يسدّها هذا الملف
//!
//! كل اختبار آخر في هذه النواة ينادي دوالّها مباشرة — `planner::plan`،
//! `executor::run`، `reveal::resolve_target` — أي أنه يعبر الطبقة التي تفصل
//! الواجهة عن النواة **قفزًا فوقها**. والواجهة لا تكلّم النواة إلا عبرها:
//! `#[tauri::command]` يفكّ ترميز ما يصل، ويحوّل أسماء المعاملات من
//! `camelCase` إلى `snake_case`، ويوَلِّف الناتج، ويوَلِّف الخطأ.
//!
//! فمجموع ما نملكه من اختبارات كان يبقى أخضر بأكمله بينما التطبيق مكسور من
//! أول استعمال: يكفي أن يُعاد تسمية حقل في `PlanResponse` لا يعرفه `ipc.ts`،
//! أو أن يتغيّر تمثيل `RawValue` على السلك، أو أن يفقد غلاف الخطأ حقل
//! `detail` الذي تبني عليه `errorText` في `i18n.ts` رسالتها الدقيقة. لا شيء
//! في Rust ولا في TypeScript يمسك أيًّا من هذه: النوعان صحيحان كلٌّ على حدة،
//! والعقد بينهما هو JSON، ولا أحد كان يقرؤه.
//!
//! هنا يُقرأ. الاختبارات تبني تطبيق Tauri بوقت التشغيل الوهمي
//! (`tauri::test::MockRuntime`) وتمرّر `InvokeRequest` حقيقيًا بنفس شكل الجسم
//! الذي يبنيه `@tauri-apps/api/core`، ثم **تؤكّد على أسماء الحقول في JSON
//! العائد نفسه**، لا على قيم من طرف Rust. إعادة تسميةٍ في Rust لا يعرفها
//! `ipc.ts` هي بالضبط العطب المقصود.
//!
//! ## لماذا يقود الاختبار `naffith_core::ipc_handler` لا نسخةً من القائمة
//!
//! لأن نسخةً ثانية من `generate_handler!` هنا كانت ستجعل الاختبار يمرّ على
//! جسرٍ من صنعه هو. المعالج مُخرَجٌ في `lib.rs` دالةً واحدة يركّبها `run()`
//! ويقودها هذا الملف، فما يُختبر هو حرفيًّا ما يُسجَّل.
//!
//! ## المساحة المؤقّتة داخل المنزل لا في `/tmp`
//!
//! `paths.rs` لا يسمح إلا بالمنزل و`/Volumes`. مساحةٌ في `/private/tmp` كانت
//! سترجع `err.path.outside` من كل نداء تخطيط، فيصير الاختبار يقيس رفض
//! السياسة بدل أن يقيس الجسر. نفس ما يفعله `compress_integration.rs`.
//!
//! ## ما لا يقوده هذا الملف عمدًا
//!
//! المسار الناجح لـ `reveal`: نجاحه يعني `open -R` ونافذة Finder تقفز أمام
//! المستخدم. تحليل الهدف — الشرط الحقيقي — مغطّى في `reveal.rs`، وما يخصّ
//! الجسر هنا هو شكل المعامل وشكل الرفض.

use naffith_core::policy::Policy;
use naffith_core::AppState;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::{Listener, Manager};

// ── مساحة عمل داخل المنزل ──────────────────────────────────────────────

const SCRATCH_PREFIX: &str = ".naffith-ipc-";

/// مساحة تُحذف عند الخروج من أي مسار، ومساحات تشغيلٍ قُتل قبل `Drop` تُكنس.
///
/// `Drop` لا يعمل تحت SIGKILL ولا تحت Ctrl-C، فبلا الكنس تتراكم مجلدات في
/// منزل المستخدم. والشرط قبل حذف مجلد غريب أن تكون عمليته قد ماتت فعلًا:
/// `cargo test` آخر يعمل الآن بالتوازي مجلداتُه ليست قمامة.
struct Scratch(PathBuf);

fn process_alive(pid: u32) -> bool {
    std::process::Command::new("/bin/ps")
        .args(["-p", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(true)
}

fn sweep_stale_scratch() {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return };
    let Ok(read) = std::fs::read_dir(&home) else { return };
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(rest) = name.strip_prefix(SCRATCH_PREFIX) else { continue };
        let mut parts = rest.rsplit('-');
        let _random = parts.next();
        let Some(Ok(pid)) = parts.next().map(str::parse::<u32>) else { continue };
        if pid == std::process::id() || process_alive(pid) {
            continue;
        }
        let _ = std::fs::remove_dir_all(entry.path());
    }
}

impl Scratch {
    fn new(tag: &str) -> Self {
        sweep_stale_scratch();
        let home = std::env::var_os("HOME").map(PathBuf::from).expect("HOME must be set");
        let base = home.join(format!(
            "{SCRATCH_PREFIX}{tag}-{}-{}",
            std::process::id(),
            naffith_core::plans::random_suffix()
        ));
        std::fs::create_dir_all(&base).expect("scratch must be creatable");
        Scratch(base.canonicalize().expect("scratch must canonicalise"))
    }

    fn dir(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn journal(&self) -> PathBuf {
        self.0.join("runs.jsonl")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn text(p: &Path) -> String {
    p.display().to_string()
}

// ── الجسر ──────────────────────────────────────────────────────────────

/// تطبيق Tauri كامل بوقت تشغيل وهمي، بنفس حالة التطبيق الحقيقي ونفس معالجه.
struct Bridge {
    webview: tauri::WebviewWindow<tauri::test::MockRuntime>,
    /// يبقى حيًّا ما دام الجسر: إسقاطه يُنهي التطبيق تحت النافذة.
    app: tauri::App<tauri::test::MockRuntime>,
}

impl Bridge {
    fn new(policy: Policy, journal: Option<PathBuf>) -> Self {
        let app = mock_builder()
            // المعالج نفسه الذي يركّبه `run()`. لا نسخة منه.
            .invoke_handler(naffith_core::ipc_handler())
            .build(mock_context(noop_assets()))
            .expect("the mock app must build");
        app.manage(AppState::new(policy, journal));
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("the mock webview must build");
        Bridge { webview, app }
    }

    /// يسجّل مستمعًا لحدثٍ ويعيد قناة تصلها حمولاته **نصًّا** كما تصل الواجهة.
    ///
    /// النصّ لا القيمة المفكوكة: ما تستقبله `listen` في `ipc.ts` هو JSON
    /// المُوَلَّف نفسه، وفكّه هنا يجعل التأكيد على أسماء حقوله تأكيدًا على
    /// السلك لا على نوع Rust.
    fn listen(&self, event: &str) -> std::sync::mpsc::Receiver<String> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.app.listen(event, move |e| {
            let _ = tx.send(e.payload().to_owned());
        });
        rx
    }

    /// يمرّر استدعاءً بنفس شكل ما يبنيه `@tauri-apps/api/core`.
    fn invoke(&self, cmd: &str, args: Value) -> Result<Value, Value> {
        get_ipc_response(
            &self.webview,
            InvokeRequest {
                cmd: cmd.to_owned(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_owned(),
            },
        )
        .map(|body| body.deserialize::<Value>().expect("a response must be JSON"))
    }

    fn ok(&self, cmd: &str, args: Value) -> Value {
        self.invoke(cmd, args).unwrap_or_else(|e| panic!("`{cmd}` must succeed, got {e}"))
    }

    fn err(&self, cmd: &str, args: Value) -> Value {
        match self.invoke(cmd, args) {
            Err(e) => e,
            Ok(v) => panic!("`{cmd}` must fail, got {v}"),
        }
    }
}

/// أسماء الحقول في كائن JSON. هذا ما يُقارَن — لا قيمٌ من طرف Rust.
fn keys(v: &Value) -> BTreeSet<String> {
    v.as_object()
        .unwrap_or_else(|| panic!("expected a JSON object, got {v}"))
        .keys()
        .cloned()
        .collect()
}

fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| (*s).to_owned()).collect()
}

/// غلاف الخطأ كما تقرؤه `asCoreError` في `ipc.ts`.
///
/// ثلاثة حقول، وكلها يجب أن تكون **موجودة** على السلك لا مجرد اختيارية في
/// Rust: `errorText` في `i18n.ts` تبني المفتاح الدقيق من `detail`، فحقلٌ
/// محذوف بـ `skip_serializing_if` كان سيُسقط الرسالة الدقيقة إلى العامة
/// صامتًا — وهو أسوأ أنواع العطب: لا يفشل شيء، ويقرأ المستخدم نصًّا أعمّ.
fn assert_error_envelope(e: &Value, expect_key: &str, expect_input: Option<&str>) {
    assert_eq!(
        keys(e),
        set(&["key", "input", "detail"]),
        "the error envelope must carry exactly what `asCoreError` reads: {e}"
    );
    assert_eq!(e["key"], json!(expect_key), "wrong error key: {e}");
    match expect_input {
        Some(id) => assert_eq!(e["input"], json!(id), "the error must name its field: {e}"),
        None => assert_eq!(e["input"], Value::Null, "this error belongs to no field: {e}"),
    }
}

// ── مدخلات بشكلها على السلك ────────────────────────────────────────────

fn path_value(p: &Path) -> Value {
    json!({ "kind": "path", "value": text(p) })
}

fn text_value(s: &str) -> Value {
    json!({ "kind": "text", "value": s })
}

/// جسم نداء `plan` كما تبنيه `ipc.ts`: `{ opId, inputs }` بـ camelCase.
fn plan_args(op_id: &str, inputs: Value) -> Value {
    json!({ "opId": op_id, "inputs": inputs })
}

fn compress_inputs(source: &Path, destination: &Path, name: &str) -> Value {
    json!({
        "source": path_value(source),
        "destination": path_value(destination),
        "archive_name": text_value(name),
    })
}

// ══════════════════════════════════════════════════════════════════════
//  ١ · list_operations
// ══════════════════════════════════════════════════════════════════════

/// الفهرس يعبر الجسر بالحقول التي يعلنها `OperationSummary` في `ipc.ts`.
///
/// وأهمّها `kind` مسطّحًا داخل كل مدخل: `#[serde(tag = "kind")]` على
/// `InputKind` هو ما يجعل الواجهة تختار مربّع اختيار مجلد أو حقل نصّ. لو صار
/// التمثيل متداخلًا (`{"kind":{"new_name":{...}}}`) لرسمت الواجهة نموذجًا
/// فارغًا بلا أن يسقط اختبار واحد في الطرفين.
#[test]
fn the_catalogue_crosses_the_bridge_with_the_fields_the_frontend_declares() {
    let bridge = Bridge::new(Policy::for_build(), None);
    let listed = bridge.ok("list_operations", json!({}));
    let ops = listed.as_array().expect("list_operations must return an array");
    assert!(!ops.is_empty(), "an empty catalogue would make every assertion below vacuous");

    for op in ops {
        assert_eq!(
            keys(op),
            set(&["id", "title_key", "description_key", "category", "danger", "inputs"]),
            "OperationSummary changed shape on the wire: {op}"
        );
        assert!(
            !op["id"].as_str().unwrap().starts_with("internal."),
            "an internal operation reached the catalogue over IPC: {op}"
        );
        for input in op["inputs"].as_array().expect("inputs must be an array") {
            let k = keys(input);
            assert!(
                k.contains("id") && k.contains("required") && k.contains("kind"),
                "InputSummary must carry a flattened `kind` discriminator: {input}"
            );
            assert!(input["kind"].is_string(), "`kind` must be a bare string tag: {input}");
        }
    }

    // العملية الوحيدة التي تشحنها M1، ومدخلاتها الثلاثة بأنواعها كما ترسمها
    // الواجهة. `ext` هو ما يجعل «نَفِّذ» يعرض `.zip` بجوار حقل الاسم.
    let zip = ops
        .iter()
        .find(|o| o["id"] == json!("compress.folder.zip"))
        .expect("M1 ships compress.folder.zip");
    assert_eq!(zip["category"], json!("compress"));
    assert_eq!(zip["danger"], json!("creates"));

    let inputs = zip["inputs"].as_array().unwrap();
    let by_id = |id: &str| {
        inputs.iter().find(|i| i["id"] == json!(id)).unwrap_or_else(|| panic!("no input `{id}`"))
    };
    assert_eq!(by_id("source")["kind"], json!("existing_dir"));
    assert_eq!(by_id("destination")["kind"], json!("target_dir"));

    let name = by_id("archive_name");
    assert_eq!(name["kind"], json!("new_name"));
    assert_eq!(
        keys(name),
        set(&["id", "required", "kind", "ext"]),
        "a `new_name` input must carry its extension on the wire: {name}"
    );
    assert_eq!(name["ext"], json!("zip"), "the frontend shows this next to the field");
    assert_eq!(name["required"], json!(true));
}

// ══════════════════════════════════════════════════════════════════════
//  ٢ · plan
// ══════════════════════════════════════════════════════════════════════

/// الحقول الخمسة عشر التي يعلنها `PlanResponse` في `ipc.ts`، بأسمائها.
const PLAN_RESPONSE_FIELDS: &[&str] = &[
    "token",
    "plan_id",
    "op_id",
    "title_key",
    "description_key",
    "danger",
    "argv_display",
    "explain",
    "warnings",
    "tool",
    "conflict",
    "estimate",
    "produces",
    "writes_to",
    "working_directory",
];

/// `plan` يقبل الجسم الحقيقي ويعيد كل حقل تعلنه الواجهة، بالاسم.
///
/// المقارنة بالمجموعة كاملةً لا بوجود الحقول: حقلٌ يُضاف في Rust ولا تعرفه
/// `ipc.ts` عطبٌ أيضًا — يعني أن أحد الطرفين تحرّك وحده.
#[test]
fn plan_accepts_the_real_payload_and_answers_with_every_declared_field() {
    let scratch = Scratch::new("plan");
    let source = scratch.dir("مصدر");
    std::fs::write(source.join("ملف.txt"), b"bytes").unwrap();
    let destination = scratch.dir("وجهة");

    let bridge = Bridge::new(Policy::for_build(), Some(scratch.journal()));
    let r = bridge.ok(
        "plan",
        plan_args("compress.folder.zip", compress_inputs(&source, &destination, "أرشيف")),
    );

    assert_eq!(
        keys(&r),
        set(PLAN_RESPONSE_FIELDS),
        "PlanResponse and ipc.ts disagree about the wire: {r}"
    );

    // الرمز: ٢٥٦ بت بصيغة ست عشرية، نصًّا مسطّحًا لا كائنًا. `PlanToken` نوع
    // مغلَّف في Rust، ولو سُرِّب مُوَلَّفًا ككائن `{"0":"..."}` لصار
    // `invoke('execute', { token })` يرسل كائنًا فيُرفض دائمًا.
    let token = r["token"].as_str().expect("token must be a bare string");
    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));

    assert!(r["plan_id"].is_string(), "plan_id must be a string: {r}");
    assert_ne!(r["plan_id"].as_str().unwrap(), token, "the public id must not be the capability");
    assert_eq!(r["op_id"], json!("compress.folder.zip"));
    assert_eq!(r["title_key"], json!("op.compress.folder.zip.title"));
    assert_eq!(r["description_key"], json!("op.compress.folder.zip.description"));
    assert_eq!(r["danger"], json!("creates"));
    assert_eq!(r["conflict"], json!("refuse"), "the frontend shows this before executing");

    // `argv_display` نصوص للعرض. أوّلها الأداة بمسارها المطلق.
    let argv: Vec<&str> =
        r["argv_display"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert_eq!(argv[0], "/usr/bin/ditto", "argv_display must start with the resolved tool");
    assert!(argv.contains(&"-c") && argv.contains(&"-k"), "{argv:?}");

    // «سَطْر» يرسم كل رمز من هذه الثلاثية. غياب `role` يجعل التلوين يخمّن من
    // النصّ، وغياب `key` يفقد الشرح.
    let explain = r["explain"].as_array().expect("explain must be an array");
    assert!(!explain.is_empty());
    for token in explain {
        assert_eq!(keys(token), set(&["token", "key", "role"]), "ExplainToken changed: {token}");
        assert!(token["token"].is_string());
        assert!(token["key"].is_string() || token["key"].is_null(), "key is string|null: {token}");
        let role = token["role"].as_str().expect("role must be a string tag");
        assert!(
            ["tool", "flag", "path", "value"].contains(&role),
            "`{role}` is not a TokenRole the frontend knows"
        );
    }

    assert!(r["warnings"].is_array(), "warnings must be an array of keys: {r}");

    assert_eq!(keys(&r["tool"]), set(&["id", "path"]), "ToolView changed: {}", r["tool"]);
    assert_eq!(r["tool"]["id"], json!("ditto"));
    assert_eq!(r["tool"]["path"], json!("/usr/bin/ditto"));

    // تقدير لا قياس. الأسماء نفسها هي العقد: لا حقل اسمه `size`.
    let estimate = &r["estimate"];
    assert_eq!(
        keys(estimate),
        set(&["approx_source_bytes", "scanned_entries", "complete"]),
        "EstimateView changed: {estimate}"
    );
    assert!(estimate["approx_source_bytes"].is_u64());
    assert_eq!(estimate["scanned_entries"], json!(1), "one file was planted in the source");
    assert_eq!(estimate["complete"], json!(true));

    // الاسم النهائي والمؤقّت مختلفان، والوسيط الأخير في الأمر هو المؤقّت.
    let produces = r["produces"].as_str().expect("produces must be a string here");
    let writes_to = r["writes_to"].as_str().expect("writes_to must be a string here");
    assert!(produces.ends_with("أرشيف.zip"), "{produces}");
    assert_ne!(produces, writes_to, "the temporary name must not be the final one");
    assert_eq!(
        *argv.last().unwrap(),
        writes_to,
        "Satr shows the last argument; it must be the very path the command writes"
    );

    assert_eq!(r["working_directory"], Value::Null, "ditto needs no cwd");
}

/// العملية التي لا تُنتج ملفًا تُعيد `null` في الحقول الثلاثة لا تحذفها.
///
/// `ipc.ts` يعلن `produces: string | null` — أي حقلًا موجودًا. حذفه بـ
/// `skip_serializing_if` كان سيجعل الوصول إليه `undefined`، وهو ما لا يقوله
/// النوع، ولا يمسكه `tsc` لأن القيمة تأتي من `invoke` غير المطبوعة أصلًا.
#[test]
fn an_operation_without_an_artifact_sends_nulls_not_missing_fields() {
    let bridge = Bridge::new(Policy::dev(), None);
    let r = bridge.ok("plan", plan_args("internal.echo", json!({ "message": text_value("سلام") })));

    assert_eq!(keys(&r), set(PLAN_RESPONSE_FIELDS), "the field set must not depend on the op: {r}");
    assert_eq!(r["produces"], Value::Null);
    assert_eq!(r["writes_to"], Value::Null);
    assert_eq!(r["estimate"], Value::Null, "an op that scans no tree still declares the field");
    assert_eq!(r["conflict"], json!("no_artifact"));
}

/// اسم المعامل على السلك هو `opId` لا `op_id`.
///
/// هذا هو تحويل Tauri التلقائي من camelCase، وهو الجزء الذي لا يراه أي طرف:
/// Rust يكتب `op_id`، و`ipc.ts` يرسل `opId`، وبينهما ماكرو. لو تغيّر التحويل
/// — أو كُتب الأمر يومًا بـ `rename_all = "snake_case"` — لصار كل تخطيط يفشل
/// و`cargo test` أخضر.
#[test]
fn the_wire_spells_the_arguments_in_camel_case() {
    let bridge = Bridge::new(Policy::dev(), None);

    // الشكل الصحيح يمرّ.
    assert!(bridge
        .invoke("plan", plan_args("internal.echo", json!({ "message": text_value("hi") })))
        .is_ok());

    // والشكل بالشرطة السفلية لا يمرّ: المعامل يصل مفقودًا.
    let e = bridge.err(
        "plan",
        json!({ "op_id": "internal.echo", "inputs": { "message": text_value("hi") } }),
    );
    assert!(
        e.as_str().is_some_and(|s| s.contains("opId")),
        "a missing argument must name the camelCase spelling the frontend has to send: {e}"
    );
}

/// `RawValue` يعبر الجسر بالتمثيل الذي تعلنه `ipc.ts`: `{ kind, value }`.
///
/// `#[serde(tag = "kind", content = "value")]` هو العقد. أي شكل آخر — مصفوفة،
/// أو كائن داخلي، أو نصّ عارٍ — يجب أن يُرفض لا أن يُحوَّل ضمنًا.
#[test]
fn an_input_value_is_only_accepted_in_the_shape_ipc_ts_declares() {
    let bridge = Bridge::new(Policy::dev(), None);

    for bad in [
        json!("سلام"),                                   // نصّ عارٍ
        json!({ "text": "سلام" }),                       // بلا واسم
        json!({ "kind": "text" }),                      // بلا قيمة
        json!({ "kind": "Text", "value": "سلام" }),      // واسم بحرف كبير
        json!({ "kind": "shell", "value": "/bin/sh" }), // صنف رابع لا وجود له
        json!({ "kind": "flag", "value": "true" }),     // راية نصًّا لا منطقًا
    ] {
        let e = bridge.err("plan", plan_args("internal.echo", json!({ "message": bad.clone() })));
        assert!(
            e.as_str().is_some(),
            "a malformed RawValue must be refused at deserialisation, not reach the planner: \
             {bad} produced {e}"
        );
    }

    // والراية المنطقية تمرّ حين تكون منطقية فعلًا — على عملية تعلن نصًّا،
    // فيصل الرفض من `value::validate` لا من serde: النوع صحيح، والدور خطأ.
    let e = bridge.err(
        "plan",
        plan_args("internal.echo", json!({ "message": { "kind": "flag", "value": true } })),
    );
    assert_error_envelope(&e, "err.input.type", Some("message"));
}

// ══════════════════════════════════════════════════════════════════════
//  ٣ · execute
// ══════════════════════════════════════════════════════════════════════

/// `execute` لا يقبل إلا `{ token }`، ولا يعرف أي اسم آخر.
///
/// الحدّ كلّه قائم على هذا: كل ما يُنفَّذ محفوظ في النواة منذ التخطيط. أمرٌ
/// يقبل — بالسهو — مفتاحًا آخر يعني أن الواجهة تستطيع قول شيء عن التنفيذ.
#[test]
fn execute_accepts_a_token_and_knows_no_other_argument() {
    let bridge = Bridge::new(Policy::dev(), None);

    for bogus in [
        json!({}),
        json!({ "plan_id": "0".repeat(64) }),
        json!({ "argv": ["/bin/sh", "-c", "id"] }),
        json!({ "program": "/bin/sh", "args": ["-c", "id"] }),
        json!({ "tokens": ["0"] }),
    ] {
        let e = bridge.err("execute", bogus.clone());
        assert!(
            e.as_str().is_some_and(|s| s.contains("token")),
            "execute must refuse {bogus} for want of a token, got {e}"
        );
    }

    // ورمز بنوع خاطئ يُرفض عند فكّ الترميز لا يُحوَّل.
    assert!(bridge.err("execute", json!({ "token": 12345 })).as_str().is_some());
}

/// رمز مزوَّر يعود بغلاف الخطأ الذي تقرؤه `asCoreError`.
#[test]
fn a_forged_token_comes_back_as_the_error_envelope_the_frontend_reads() {
    let bridge = Bridge::new(Policy::dev(), None);
    let e = bridge.err("execute", json!({ "token": "f".repeat(64) }));
    assert_error_envelope(&e, "err.plan.not_found", None);
    assert_eq!(e["detail"], Value::Null);
}

/// الرمز أحادي الاستخدام، ويُثبَت ذلك **عبر الجسر** لا في المخزن.
///
/// إعادة الإرسال هي ما يفعله زرٌّ يُضغط مرتين. لو نجحت المرة الثانية لعملت
/// العملية مرتين على نفس الملفات.
#[test]
fn a_replayed_token_is_refused_the_second_time() {
    let scratch = Scratch::new("replay");
    let bridge = Bridge::new(Policy::dev(), Some(scratch.journal()));

    let planned =
        bridge.ok("plan", plan_args("internal.echo", json!({ "message": text_value("مرحبا") })));
    let token = planned["token"].as_str().unwrap().to_owned();

    let run_id = bridge.ok("execute", json!({ "token": token }));
    let run_id = run_id.as_str().expect("execute must return a run id string").to_owned();
    assert_eq!(
        run_id,
        planned["plan_id"].as_str().unwrap(),
        "the run id must be the plan id, so the journal entries connect"
    );

    let e = bridge.err("execute", json!({ "token": token }));
    assert_error_envelope(&e, "err.plan.not_found", None);

    settle(&bridge, &run_id);
}

/// أحداث التشغيل الثلاثة تعبر الجسر بحمولاتها كما يعلنها `ipc.ts`.
///
/// النصف الآخر من الجسر: `execute` يعيد معرّفًا واحدًا ثم **يبثّ** كل ما
/// عداه. لولا هذا الاختبار لبقيت `RunStarted` و`RunOutput` و`RunFinished`
/// أنواعًا لا يوَلِّفها شيء مُختبَر، مع أن شاشة التشغيل لا ترسم غيرها.
///
/// والتسطيح (`#[serde(flatten)]`) هو ما يجعل `e.payload.stream` و
/// `e.payload.status` حقولًا مستوية في الحدث الواحد؛ تمثيلٌ متداخل كان
/// سيجعل الشاشة تعرض تشغيلًا بلا خرج وبلا نتيجة، بلا خطأ في أي طرف.
#[test]
fn the_run_events_carry_the_payloads_the_frontend_listens_for() {
    let scratch = Scratch::new("events");
    let bridge = Bridge::new(Policy::dev(), Some(scratch.journal()));

    // التسجيل قبل الإطلاق: الأحداث لا تُخزَّن، ومن يتأخّر يفوته البثّ.
    let started = bridge.listen("run://started");
    let output = bridge.listen("run://output");
    let finished = bridge.listen("run://finished");

    let planned =
        bridge.ok("plan", plan_args("internal.echo", json!({ "message": text_value("سلام") })));
    let token = planned["token"].as_str().unwrap().to_owned();
    let run_id = bridge.ok("execute", json!({ "token": token }));
    let run_id = run_id.as_str().unwrap().to_owned();

    let next = |rx: &std::sync::mpsc::Receiver<String>, what: &str| -> Value {
        let raw = rx
            .recv_timeout(std::time::Duration::from_secs(20))
            .unwrap_or_else(|e| panic!("`{what}` never arrived: {e}"));
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("`{what}` was not JSON: {e}"))
    };

    let s = next(&started, "run://started");
    assert_eq!(keys(&s), set(&["run_id"]), "RunStarted changed shape: {s}");
    assert_eq!(s["run_id"], json!(run_id));

    let o = next(&output, "run://output");
    assert_eq!(
        keys(&o),
        set(&["run_id", "stream", "line"]),
        "RunOutputEvent must be flat: run_id, stream, line — {o}"
    );
    assert_eq!(o["run_id"], json!(run_id));
    assert_eq!(o["stream"], json!("stdout"));
    assert_eq!(o["line"], json!("سلام"), "Arabic output must survive the wire");

    let f = next(&finished, "run://finished");
    assert_eq!(
        keys(&f),
        set(&["run_id", "status", "produced"]),
        "a successful RunFinished carries exactly these — {f}"
    );
    assert_eq!(f["run_id"], json!(run_id));
    assert_eq!(f["status"], json!("success"), "the tag the run screen switches on");
    assert_eq!(f["produced"], Value::Null, "echo writes nothing");

    settle(&bridge, &run_id);
}

// ══════════════════════════════════════════════════════════════════════
//  ٤ · cancel
// ══════════════════════════════════════════════════════════════════════

/// `cancel` يقبل `{ runId }` بهذه الهجاء وحده.
#[test]
fn cancel_takes_a_run_id_spelled_the_way_the_frontend_sends_it() {
    let bridge = Bridge::new(Policy::dev(), None);

    let e = bridge.err("cancel", json!({ "runId": "no-such-run" }));
    assert_error_envelope(&e, "err.run.not_found", None);

    // الهجاء بالشرطة السفلية لا يصل، فلا يجوز أن يُقرأ نجاحًا.
    let e = bridge.err("cancel", json!({ "run_id": "no-such-run" }));
    assert!(e.as_str().is_some_and(|s| s.contains("runId")), "{e}");

    assert!(bridge.err("cancel", json!({})).as_str().is_some());
}

/// إلغاء تشغيلٍ جارٍ يمرّ عبر الجسر ويعيد `null` لا كائنًا.
///
/// `invoke<void>` في `ipc.ts` يتوقّع جسمًا فارغًا؛ `()` في Rust تُوَلَّف
/// `null`. هذا ما يجعل `await cancel(id)` يعود بلا مفاجأة.
#[test]
fn cancelling_a_live_run_answers_with_null() {
    let scratch = Scratch::new("cancel");
    let bridge = Bridge::new(Policy::dev(), Some(scratch.journal()));

    let planned =
        bridge.ok("plan", plan_args("internal.echo", json!({ "message": text_value("طويل") })));
    let token = planned["token"].as_str().unwrap().to_owned();
    let run_id = bridge.ok("execute", json!({ "token": token }));
    let run_id = run_id.as_str().unwrap().to_owned();

    // `echo` عملية قصيرة جدًّا: الإلغاء قد يصل بعد خروجها، وحينها
    // `err.run.not_found` هو الجواب الصحيح لا عطب. المُثبَت هنا أن كِلا
    // الجوابين يعبر الجسر بشكله المعلَن، لا أن السباق يُحسم في اتجاه بعينه.
    match bridge.invoke("cancel", json!({ "runId": run_id })) {
        Ok(v) => assert_eq!(v, Value::Null, "`invoke<void>` must receive null"),
        Err(e) => assert_error_envelope(&e, "err.run.not_found", None),
    }

    settle(&bridge, &run_id);
}

/// ينتظر بلوغ التشغيل حالةً نهائية قبل أن تنتهي الاختبارة.
///
/// ليس تجميلًا: قيد الحالة النهائية يُكتب في مهمة تعمل **بعد** عودة
/// `execute`، و`Journal::append` تُنشئ مجلد السجل إن غاب. فاختبارٌ ينتهي قبل
/// أن يستقرّ التشغيل كان يحذف مساحته في `Drop` ثم يُعيدها القيدُ المتأخّر
/// إلى الوجود — فتتراكم مجلدات في منزل المستخدم بعد كل `cargo test`.
fn settle(bridge: &Bridge, run_id: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while std::time::Instant::now() < deadline {
        let listed = bridge.ok("recent_runs", json!({}));
        let settled = listed.as_array().unwrap().iter().any(|e| {
            e["id"] == json!(run_id)
                && ["succeeded", "failed", "cancelled"]
                    .contains(&e["state"].as_str().unwrap_or_default())
        });
        if settled {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    panic!("run `{run_id}` never reached a terminal state");
}

// ══════════════════════════════════════════════════════════════════════
//  ٥ · recent_runs
// ══════════════════════════════════════════════════════════════════════

/// قيود السجل تعبر الجسر بشكل `JournalEntry`، وحالتها وسمٌ مسطّح.
///
/// `#[serde(flatten, tag = "state")]` هو ما يجعل `entry.state === 'succeeded'`
/// و`entry.produced` يعملان معًا في `run-log.tsx`. تمثيلٌ متداخل كان سيجعل
/// شاشة السجل تعرض صفوفًا بلا حالة.
#[test]
fn the_journal_crosses_the_bridge_as_the_entries_the_run_log_draws() {
    let scratch = Scratch::new("journal");
    let bridge = Bridge::new(Policy::dev(), Some(scratch.journal()));

    assert_eq!(bridge.ok("recent_runs", json!({})), json!([]), "a fresh journal is an empty array");

    let planned =
        bridge.ok("plan", plan_args("internal.echo", json!({ "message": text_value("قيد") })));
    let run_id = planned["plan_id"].as_str().unwrap().to_owned();
    let token = planned["token"].as_str().unwrap().to_owned();
    bridge.ok("execute", json!({ "token": token }));

    let entries = wait_for_state(&bridge, &run_id, "succeeded");

    // القيد الأول هو المعاينة: التخطيط وحده يُقيَّد قبل أن يُنفَّذ شيء.
    let states: Vec<&str> =
        entries.iter().map(|e| e["state"].as_str().expect("state must be a flat tag")).collect();
    assert_eq!(states, vec!["planned", "running", "succeeded"], "{entries:#?}");

    for e in &entries {
        let k = keys(e);
        for required in ["id", "op_id", "at", "program", "args", "state"] {
            assert!(k.contains(required), "JournalEntry is missing `{required}`: {e}");
        }
        assert_eq!(e["id"], json!(run_id), "one run, one id");
        assert_eq!(e["op_id"], json!("internal.echo"));
        assert!(e["at"].is_u64(), "`at` must be seconds since the epoch: {e}");
        assert_eq!(e["program"], json!("/bin/echo"));
        assert_eq!(e["args"], json!(["قيد"]), "Arabic arguments must survive the wire");
    }

    // الحقول المشروطة بالحالة، وهي ما تعلنه `ipc.ts` اختيارية.
    let finished = entries.last().unwrap();
    assert_eq!(finished["produced"], Value::Null, "echo produces no artifact");
    assert!(finished["duration_ms"].is_u64(), "the final entry carries its duration: {finished}");
    assert!(
        entries[0].get("duration_ms").is_none(),
        "a plan preview has no duration; the field must be absent, not a lie"
    );
}

/// ينتظر بلوغ حالة بعينها **بالسؤال عبر الجسر** لا بقراءة السجل مباشرة.
fn wait_for_state(bridge: &Bridge, run_id: &str, state: &str) -> Vec<Value> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let listed = bridge.ok("recent_runs", json!({}));
        let mine: Vec<Value> = listed
            .as_array()
            .expect("recent_runs must return an array")
            .iter()
            .filter(|e| e["id"] == json!(run_id))
            .cloned()
            .collect();
        if mine.iter().any(|e| e["state"] == json!(state)) {
            return mine;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "run `{run_id}` never reached `{state}`; last seen: {mine:#?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

// ══════════════════════════════════════════════════════════════════════
//  ٦ · reveal
// ══════════════════════════════════════════════════════════════════════

/// `reveal` يقبل `{ runId }` ولا يقبل مسارًا بأي هجاء.
///
/// المسار الناجح غير مقود هنا عن قصد: نجاحه يُقفز نافذة Finder أمام
/// المستخدم. تحليل الهدف — الشرط الذي يقرّر ما يجوز إظهاره — مغطّى كاملًا في
/// وحدات `reveal.rs`؛ ما يخصّ الجسر هو شكل المعامل وشكل الرفض.
#[test]
fn reveal_takes_a_run_id_and_refuses_anything_it_did_not_produce() {
    let scratch = Scratch::new("reveal");
    let bridge = Bridge::new(Policy::dev(), Some(scratch.journal()));

    let e = bridge.err("reveal", json!({ "runId": "no-such-run" }));
    assert_error_envelope(&e, "err.reveal.nothing", None);

    // تشغيلٌ خُطِّط ولم يُنفَّذ لم يُنتج شيئًا يُظهَر.
    let planned =
        bridge.ok("plan", plan_args("internal.echo", json!({ "message": text_value("م") })));
    let e = bridge.err("reveal", json!({ "runId": planned["plan_id"] }));
    assert_error_envelope(&e, "err.reveal.nothing", None);

    // ولا سبيل لتمرير مسار: لا بالاسم الصحيح ولا بغيره.
    for bogus in [
        json!({ "path": "/etc/passwd" }),
        json!({ "produced": "/etc/passwd" }),
        json!({ "run_id": "x" }),
        json!({}),
    ] {
        assert!(
            bridge.err("reveal", bogus.clone()).as_str().is_some(),
            "reveal must refuse {bogus} for want of a runId"
        );
    }
}

// ══════════════════════════════════════════════════════════════════════
//  ٧ · ما ليس في الحدّ لا وجود له وقت التشغيل
// ══════════════════════════════════════════════════════════════════════

/// أمرٌ غير مسجَّل لا يُنفَّذ، ويُثبَت ذلك **بطلبه فعلًا** لا بقراءة المصدر.
///
/// حرّاس `security.rs` تمسح الشيفرة، وهي تثبت أن القائمة ستة. هذا يثبت الشيء
/// المكمّل: أن وقت التشغيل نفسه يرفض السابع. لو أُضيف يومًا مسار التفافي —
/// إضافة تسجّل أوامرها، أو معالج ثانٍ — لكان هذا هو الاختبار الذي يسقط.
#[test]
fn a_command_that_is_not_in_the_boundary_does_not_exist_at_runtime() {
    let bridge = Bridge::new(Policy::dev(), None);

    for absent in [
        "run_raw",
        "exec",
        "execute_raw",
        "open_path",
        "read_file",
        "list_operations_internal",
        "plugin:shell|execute",
        "plugin:fs|read_file",
        "plugin:opener|open",
    ] {
        let e = bridge.err(absent, json!({ "argv": ["/bin/sh", "-c", "id"] }));
        let message = e.as_str().unwrap_or_default();
        assert!(
            message.contains("not found") || message.contains("not allowed"),
            "`{absent}` must not be reachable over IPC, got {e}"
        );
    }

    // والستة المسجَّلة موجودة فعلًا: لولا هذا لكان الحارس أعلاه يمرّ على
    // تطبيق بلا معالج أصلًا.
    for present in ["list_operations", "plan", "execute", "cancel", "recent_runs", "reveal"] {
        let answered = match bridge.invoke(present, json!({})) {
            Ok(_) => true,
            Err(e) => !e.as_str().unwrap_or_default().contains("not found"),
        };
        assert!(answered, "registered command `{present}` was not reachable");
    }
}

// ══════════════════════════════════════════════════════════════════════
//  ٨ · غلاف الخطأ عبر أنواع مختلفة
// ══════════════════════════════════════════════════════════════════════

/// اسم أرشيف غير صالح: مفتاح **وتفصيل نصّي**.
///
/// هذا هو الحقل الذي تعتمد عليه `errorText` في `i18n.ts` كي تعرض «الاسم لا
/// يجوز أن يحوي فاصل مسار» بدل «الاسم غير صالح». والتفصيل يجب أن يصل **نصًّا**
/// لا كائنًا: الدالة تفحص `typeof detail === 'string'` وتسقط صامتة إلى الرسالة
/// العامة إن كان غير ذلك.
#[test]
fn an_invalid_archive_name_carries_a_string_detail_the_ui_can_specialise() {
    let scratch = Scratch::new("name");
    let source = scratch.dir("مصدر");
    let destination = scratch.dir("وجهة");
    let bridge = Bridge::new(Policy::for_build(), Some(scratch.journal()));

    let e = bridge.err(
        "plan",
        plan_args("compress.folder.zip", compress_inputs(&source, &destination, "أ/ب")),
    );
    assert_error_envelope(&e, "err.name.invalid", Some("archive_name"));
    assert_eq!(
        e["detail"],
        json!("contains_separator"),
        "the detail must be the snake_case reason, as a bare string: {e}"
    );
    assert!(e["detail"].is_string(), "errorText() only specialises on a string detail: {e}");

    // وسببٌ آخر يصل بنصّه هو أيضًا، فالتفصيل ليس ثابتًا مموّهًا.
    let e = bridge
        .err("plan", plan_args("compress.folder.zip", compress_inputs(&source, &destination, "")));
    assert_error_envelope(&e, "err.name.invalid", Some("archive_name"));
    assert_eq!(e["detail"], json!("empty"));
}

/// خطأ منسوب إلى حقل: المفتاح يقول ماذا، و`input` يقول أين.
#[test]
fn a_path_error_names_the_field_that_can_fix_it() {
    let scratch = Scratch::new("paths");
    let destination = scratch.dir("وجهة");
    let missing = scratch.0.join("لا-وجود-له");
    let bridge = Bridge::new(Policy::for_build(), Some(scratch.journal()));

    let e = bridge.err(
        "plan",
        plan_args("compress.folder.zip", compress_inputs(&missing, &destination, "أرشيف")),
    );
    assert_error_envelope(&e, "err.path.missing", Some("source"));

    // ومسارٌ خارج الجذور المسموحة يُرفض بمفتاحه هو لا بـ «غير موجود».
    let e = bridge.err(
        "plan",
        plan_args("compress.folder.zip", compress_inputs(Path::new("/etc"), &destination, "أرشيف")),
    );
    assert_error_envelope(&e, "err.path.outside", Some("source"));

    // ومسار نسبيّ: الواجهة تسمح بالكتابة اليدوية في الحقل.
    let e = bridge.err(
        "plan",
        plan_args("compress.folder.zip", compress_inputs(Path::new("مجلد"), &destination, "أرشيف")),
    );
    assert_error_envelope(&e, "err.path.relative", Some("source"));
}

/// عملية مجهولة، ومدخل لا تعلنه المواصفة: خطآن بلا حقل مسؤول.
#[test]
fn catalogue_errors_belong_to_no_field_and_carry_no_detail() {
    let bridge = Bridge::new(Policy::dev(), None);

    let e = bridge.err("plan", plan_args("no.such.operation", json!({})));
    assert_error_envelope(&e, "err.op.unknown", None);
    assert_eq!(e["detail"], Value::Null);

    // مفتاح مهرَّب لا تعرفه المواصفة يُرفض ولا يُتجاهَل.
    let e = bridge.err(
        "plan",
        plan_args(
            "internal.echo",
            json!({ "message": text_value("hi"), "args": text_value("--dangerous") }),
        ),
    );
    assert_error_envelope(&e, "err.input.unexpected", None);

    // ومدخل مطلوب ناقص يُنسب إلى حقله.
    let e = bridge.err("plan", plan_args("internal.echo", json!({})));
    assert_error_envelope(&e, "err.input.missing", Some("message"));
}

// ══════════════════════════════════════════════════════════════════════
//  ٩ · السياسة عبر الجسر
// ══════════════════════════════════════════════════════════════════════

/// السياسة قرارُ وقت تشغيل، ويُرى أثره على السلك.
///
/// `Policy::for_build()` تعطي `dev` تحت `cargo test` لأن `debug_assertions`
/// مفعّلة، فالعملية الداخلية تُخطَّط هنا. الاختبار لا يفترض ذلك بل يثبته، ثم
/// يبني جسرًا ثانيًا بسياسة الإنتاج ويثبت أن **نفس** الطلب يعود
/// `err.op.unavailable` — الرفض الذي سيراه المستخدم في البناء الموزَّع.
#[test]
fn the_production_policy_refuses_the_internal_operation_over_the_wire() {
    assert_eq!(
        Policy::for_build(),
        Policy::dev(),
        "cargo test builds with debug_assertions, so the app policy here is dev"
    );

    let args = plan_args("internal.echo", json!({ "message": text_value("hi") }));

    let dev = Bridge::new(Policy::dev(), None);
    assert!(dev.invoke("plan", args.clone()).is_ok());

    let production = Bridge::new(Policy::production(), None);
    let e = production.err("plan", args);
    assert_error_envelope(&e, "err.op.unavailable", None);

    // وفي الحالتين لا تظهر العملية الداخلية في الفهرس أصلًا.
    for bridge in [&dev, &production] {
        let listed = bridge.ok("list_operations", json!({}));
        assert!(!listed.to_string().contains("internal."), "{listed}");
    }
}
