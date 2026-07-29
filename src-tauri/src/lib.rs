//! نَفِّذ — سَطْر · النواة.
//!
//! ## الأطروحة
//!
//! `OperationSpec` واحد تُشتقّ منه الواجهة والأمر معًا. نَفِّذ يرسم مدخلاته
//! نموذجًا، وسَطْر يرسم ناتج تخطيطه أمرًا مشروحًا. لأنهما دالّتان على مصدر
//! واحد لا يمكن أن يفترقا، ولا يوجد مسار في الشيفرة يعرض أمرًا غير الذي يُنفَّذ.
//!
//! ## حدّ الأمان
//!
//! الواجهة لا تملك أي صيغة تعبّر عن أمر. أقصى ما ترسله: معرّف عملية من فهرس
//! معروف، ومدخلات موصوفة (`Path` | `Text` | `Flag`). بناء `argv` والتحقّق من
//! المسارات وحلّ الأدوات كلها في Rust. و`execute` لا يقبل إلا رمز خطة صادرًا
//! عن النواة نفسها.

pub mod atomic;
pub mod error;
pub mod estimate;
pub mod executor;
pub mod journal;
pub mod ops;
pub mod paths;
pub mod planner;
pub mod plans;
pub mod policy;
pub mod registry;
pub mod reveal;
pub mod spec;
pub mod tools;
pub mod value;

use error::{CoreError, Result};
use executor::{Handle, Outcome, OutputLine};
use journal::{Entry, Journal};
use planner::PlanResponse;
use plans::{PlanStore, PlanToken, SessionId};
use policy::Policy;
use serde::Serialize;
use spec::OperationSummary;
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{Emitter, Manager, State};
use tokio::sync::{mpsc, oneshot};

/// أقصى عدد تشغيلات متزامنة.
const MAX_CONCURRENT_RUNS: usize = 4;

pub struct AppState {
    policy: Policy,
    session: SessionId,
    plans: Mutex<PlanStore>,
    runs: Mutex<HashMap<String, Handle>>,
    journal: Journal,
}

impl AppState {
    pub fn new(policy: Policy, journal_path: Option<std::path::PathBuf>) -> Self {
        let mut store = PlanStore::new();
        let session = store.register_session().expect("first session always fits");
        AppState {
            policy,
            session,
            plans: Mutex::new(store),
            runs: Mutex::new(HashMap::new()),
            journal: Journal::new(journal_path),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct RunStarted {
    pub run_id: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct RunOutput {
    pub run_id: String,
    #[serde(flatten)]
    pub line: OutputLine,
}

#[derive(Debug, Serialize, Clone)]
pub struct RunFinished {
    pub run_id: String,
    #[serde(flatten)]
    pub outcome: Outcome,
}

// ════════════════════════════════════════════════════════════════════════
//  حدّ IPC — ستة أوامر. لا سابع.
//
//  لاحظ ما لا يوجد هنا: لا أمر يقبل `program`، ولا `args`، ولا `cwd`، ولا
//  سلسلة أمر، ولا **مسارًا** خارج مدخلات عملية موصوفة. اختبار في
//  tests/security.rs يثبّت هذه القائمة كي لا تتوسّع بالسهو.
//
//  M1 أضافت `reveal` وحده، ومعاملها معرّف تشغيل لا مسار: النواة تُخرج المسار
//  من سجلّها هي. البديل — إضافة `opener` — كان سيمنح الواجهة «افتح هذا
//  المسار»، وهو ما بُنيت المعمارية كي يبقى غير قابل للتعبير عنه.
// ════════════════════════════════════════════════════════════════════════

/// فهرس العمليات الظاهرة. لا يتضمّن العمليات الداخلية في أي بناء.
#[tauri::command]
fn list_operations(state: State<'_, AppState>) -> Vec<OperationSummary> {
    registry::list(state.policy)
}

/// يخطّط عملية ويعيد رمزًا. لا ينفّذ شيئًا ولا ينتج ملفًا.
///
/// الأثر الوحيد على القرص هو ملف فحص كتابة بحجم صفر داخل مجلد الوجهة، يُحذف
/// فورًا في كل مسار خروج — لأن قراءة بتات الصلاحيات تكذب تحت ACL أو قرص
/// للقراءة فقط. اختبار في `paths.rs` و`ops/compress_ditto.rs` يثبت ألّا بقيّة.
#[tauri::command]
fn plan(
    state: State<'_, AppState>,
    op_id: String,
    inputs: BTreeMap<String, value::RawValue>,
) -> Result<PlanResponse> {
    let response = {
        let mut store = state.plans.lock().map_err(|_| CoreError::PlanNotFound)?;
        planner::plan(&mut store, &state.session, state.policy, &op_id, &inputs)?
    };

    // قيدٌ عند التخطيط: خطةٌ رُوجعت ثم تُركت أثرٌ يستحق أن يُرى في السجل.
    state.journal.record(
        Entry::new(response.plan_id.clone(), response.op_id, journal::State::Planned).with_command(
            response.argv_display.first().cloned().unwrap_or_default(),
            response.argv_display.iter().skip(1).cloned().collect(),
            response.working_directory.clone(),
        ),
    );

    Ok(response)
}

/// ينفّذ خطة محفوظة. المعامل الوحيد رمزٌ صدر عن النواة.
///
/// لا يقبل `argv` ولا مسارًا ولا خيارًا إضافيًا: كل ما يُنفَّذ محفوظ في النواة
/// منذ لحظة التخطيط، والرمز مفتاحه.
#[tauri::command]
async fn execute(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    token: String,
) -> Result<String> {
    let stored = {
        let mut store = state.plans.lock().map_err(|_| CoreError::PlanNotFound)?;
        // `take` أحادي الاستخدام: إعادة إرسال نفس الرمز تفشل هنا.
        store.take(&PlanToken::from(token), &state.session)?
    };

    // إعادة التحقّق من الشروط قبل الإطلاق مباشرة. الفجوة بين التخطيط والضغط
    // على «نفّذ» فجوة حقيقية يمكن أن يُحذف فيها المصدر أو يظهر الاسم النهائي.
    stored.verify_still_valid()?;

    {
        let runs = state.runs.lock().map_err(|_| CoreError::PlanNotFound)?;
        if runs.len() >= MAX_CONCURRENT_RUNS {
            return Err(CoreError::PlanLimitReached);
        }
    }

    // معرّف التشغيل هو معرّف الخطة نفسه، فيتّصل قيد `planned` بما بعده في
    // السجل بدل أن يكون حدثًا يتيمًا.
    let run_id = stored.id.clone();
    let (cancel_tx, cancel_rx) = oneshot::channel();
    state
        .runs
        .lock()
        .map_err(|_| CoreError::PlanNotFound)?
        .insert(run_id.clone(), Handle { cancel: cancel_tx });

    let command = stored.command.clone();
    let op_id = stored.op_id;
    let program = command.program.display().to_string();
    let args: Vec<String> = command.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    let cwd = command.cwd.as_ref().map(|p| p.display().to_string());

    state.journal.record(Entry::new(run_id.clone(), op_id, journal::State::Running).with_command(
        program.clone(),
        args.clone(),
        cwd.clone(),
    ));

    let (out_tx, mut out_rx) = mpsc::channel::<OutputLine>(256);

    // بثّ الخرج إلى الواجهة.
    let emit_app = app.clone();
    let emit_run = run_id.clone();
    tokio::spawn(async move {
        while let Some(line) = out_rx.recv().await {
            let _ = emit_app.emit("run://output", RunOutput { run_id: emit_run.clone(), line });
        }
    });

    let done_app = app.clone();
    let done_run = run_id.clone();
    tokio::spawn(async move {
        let started = Instant::now();
        // `Outcome::Success` لا يُبنى إلا بعد ترقية الناتج إلى اسمه النهائي،
        // فقيد `succeeded` لا يمكن أن يسبقها. انظر `executor::run`.
        let outcome = executor::run(command, out_tx, cancel_rx).await;

        let state: State<'_, AppState> = done_app.state();
        state.runs.lock().map(|mut r| r.remove(&done_run)).ok();
        state.journal.record(
            Entry::new(done_run.clone(), op_id, journal::State::from_outcome(&outcome))
                .with_command(program, args, cwd)
                .with_duration(started.elapsed().as_millis() as u64),
        );

        let _ = done_app.emit("run://finished", RunFinished { run_id: done_run, outcome });
    });

    let _ = app.emit("run://started", RunStarted { run_id: run_id.clone() });
    Ok(run_id)
}

/// يلغي تشغيلًا جاريًا. الإشارة تطال مجموعة العمليات كلها، والمؤقّت يُنظَّف.
#[tauri::command]
fn cancel(state: State<'_, AppState>, run_id: String) -> Result<()> {
    let handle = state
        .runs
        .lock()
        .map_err(|_| CoreError::RunNotFound)?
        .remove(&run_id)
        .ok_or(CoreError::RunNotFound)?;
    let _ = handle.cancel.send(());
    Ok(())
}

/// آخر التشغيلات، للمراجعة.
#[tauri::command]
fn recent_runs(state: State<'_, AppState>) -> Vec<Entry> {
    state.journal.recent()
}

/// يُبرز في Finder ما أنتجه تشغيلٌ ناجح.
///
/// المعامل معرّف تشغيل لا مسار: النواة تُخرج المسار من سجلّها هي ثم تُمرّه
/// بسياسة المسارات كاملة. لا سبيل لأن تطلب الواجهة إظهار موضع لم يُنتَج هنا.
#[tauri::command]
fn reveal(state: State<'_, AppState>, run_id: String) -> Result<()> {
    let target = reveal::resolve_target(&state.journal, &run_id)?;
    reveal::reveal(&target)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let journal_path = app.path().app_data_dir().ok().map(|d| d.join("runs.jsonl"));
            app.manage(AppState::new(Policy::for_build(), journal_path));
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_operations,
            plan,
            execute,
            cancel,
            recent_runs,
            reveal
        ])
        .run(tauri::generate_context!())
        .expect("error while running naffith");
}
