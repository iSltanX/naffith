//! نَفِّذ · النواة.
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

pub mod archive;
pub mod atomic;
pub mod categories;
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
pub mod result;
pub mod reveal;
pub mod spec;
#[cfg(test)]
pub mod testkit;
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

/// قفلٌ يتعافى من التسميم بدل أن يترجمه إلى خطأ في المجال.
///
/// كان كل موضع قفل هنا يكتب `map_err(|_| CoreError::PlanNotFound)`، فذعرٌ
/// واحدٌ داخل المخطّط — أو داخل مهمة تشغيل تمسك القفل — يسمّم `Mutex` إلى
/// الأبد، فيصير كل تخطيط لاحق «الخطة غير موجودة». عطبٌ دائم لسبب عابر،
/// وأسوأ من ذلك أنه يكذب: الخطة موجودة، والقفل هو المكسور.
///
/// البديل المرفوض كان استيراد قفل لا يُسمَّم (`parking_lot`). اعتماد جديد
/// كامل مقابل ما يفعله `clear_poison` في سطرين لا يستحق.
///
/// والتعافي آمن هنا لأن ما تحرسه هذه الأقفال بنى بيانات بسيطة: خريطة خطط
/// وخريطة تشغيلات. أقصى ما يخلّفه ذعرٌ في منتصف عملية هو مدخلة ناقصة، لا
/// بنية مكسورة.
fn recover<T>(lock: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|poisoned| {
        lock.clear_poison();
        poisoned.into_inner()
    })
}

/// يحرّر ما يحرسه مهما كان سبب الخروج — بما في ذلك فكّ المكدّس بعد ذعر.
///
/// حذف خانة التشغيل كان تعليمةً بعد `await`، فذعرٌ داخل المنفّذ يتخطّاها،
/// وتبقى الخانة محجوزة إلى نهاية الجلسة. أربع حالات ذعر تستنفد
/// `MAX_CONCURRENT_RUNS` فلا يعود المستخدم قادرًا على تشغيل شيء. `Drop`
/// يعمل على مسار الفكّ أيضًا، وهذا كل الفرق.
struct SlotGuard<F: FnMut()>(F);

impl<F: FnMut()> Drop for SlotGuard<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}

/// يفحص حدّ التزامن ويستهلك الرمز ويحجز الخانة في قسم حرج **واحد**.
///
/// الترتيب مقصود بأكمله:
///
/// - الفحص قبل الاستهلاك. كان الرمز يُنتزع أولًا ثم يُرفض الطلب لبلوغ الحدّ،
///   فتتلف خطةٌ صالحة لسبب لا علاقة له بها ويعيد المستخدم ملء النموذج كلّه.
///   الرمز أحادي الاستخدام، فإتلافه عقوبةٌ على ازدحامٍ ليس من فعله.
/// - قفلٌ واحد يمتدّ من الفحص إلى الإدراج. كان الفحص في قفل والإدراج في قفل
///   آخر، فتشغيلان متزامنان يريان «ثلاثة» ثم يصيران خمسة.
///
/// مفصولة عن `execute` كي تُختبر بلا تشغيل تطبيق Tauri كامل.
fn reserve_run(
    runs: &Mutex<HashMap<String, Handle>>,
    plans: &Mutex<PlanStore>,
    session: &SessionId,
    token: String,
    cancel: oneshot::Sender<()>,
) -> Result<plans::StoredPlan> {
    let mut runs = recover(runs);
    if runs.len() >= MAX_CONCURRENT_RUNS {
        return Err(CoreError::PlanLimitReached);
    }

    // `take` أحادي الاستخدام: إعادة إرسال نفس الرمز تفشل هنا.
    let stored = {
        let mut store = recover(plans);
        store.take(&PlanToken::from(token), session)?
    };

    // إعادة التحقّق من الشروط قبل الإطلاق مباشرة. الفجوة بين التخطيط والضغط
    // على «نفّذ» فجوة حقيقية يمكن أن يُحذف فيها المصدر أو يظهر الاسم النهائي.
    stored.verify_still_valid()?;

    runs.insert(stored.id.clone(), Handle { cancel });
    Ok(stored)
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
    pub result: result::ResultContract,
    #[serde(flatten)]
    pub outcome: Outcome,
}

// ════════════════════════════════════════════════════════════════════════
//  حدّ IPC — تسعة أوامر. لا عاشر.
//
//  كانت ستة. أضافت مرحلةُ المكتبة ثلاثة، وكلٌّ منها قرارٌ لا سهو:
//  `list_categories` كي تُرسم شاشة الفئات بأعدادٍ محسوبة في النواة لا مؤلّفة
//  في الواجهة، و`journal_delete` و`journal_clear` كي يملك المستخدم أثرَه —
//  ومعاملهما معرّف تشغيل أو لا شيء، لا مسار ملفٍ ولا سطرٌ فيه.
//
//  لاحظ ما لا يوجد هنا: لا أمر يقبل `program`، ولا `args`، ولا `cwd`، ولا
//  سلسلة أمر، ولا **مسارًا** خارج مدخلات عملية موصوفة. اختبارات في
//  tests/security.rs تثبّت هذه القائمة كي لا تتوسّع بالسهو، وهي تقرأ قائمة
//  `generate_handler!` نفسها لا عدد السمات: التسجيل هو الحدّ، والسمة مجرد
//  إعلان. وتُلزم أن يكون كل أمر معلَنًا في هذا الملف وحده، فتبقى «اقرأ حدّ
//  IPC» جملةً كاملة الصدق. أضِف أمرًا في وحدة أخرى، أو بصيغة السمة ذات
//  الخيارات (‏`rename_all`‎)، وسيسقط البناء — وكلاهما كان يمرّ صامتًا.
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
async fn plan(
    state: State<'_, AppState>,
    op_id: String,
    inputs: BTreeMap<String, value::RawValue>,
) -> Result<PlanResponse> {
    let planned = {
        let mut store = recover(&state.plans);
        planner::plan(&mut store, &state.session, state.policy, &op_id, &inputs)?
    };
    let planner::Planned { response, journal_inputs, journal_argv } = planned;

    // قيدٌ عند التخطيط: خطةٌ رُوجعت ثم تُركت أثرٌ يستحق أن يُرى في السجل.
    state.journal.record(
        Entry::new(response.plan_id.clone(), response.op_id, journal::State::Planned)
            .with_command(
                journal_argv.first().cloned().unwrap_or_default(),
                journal_argv.iter().skip(1).cloned().collect(),
                response.working_directory.clone(),
            )
            .with_category(&category_name(response.category))
            .with_inputs(journal_inputs),
    );

    Ok(response)
}

/// اسم القسم نصًّا، كما يُكتب في السجل ويُصفّى به.
///
/// من `Serialize` نفسها لا من خريطةٍ ثانية: قسمٌ يُضاف غدًا لا يحتاج سطرًا هنا،
/// ولا يمكن أن يفترق اسمُه في السجلّ عن اسمه على السلك.
fn category_name(category: spec::Category) -> String {
    serde_json::to_value(category)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// ينفّذ خطة محفوظة. المعامل الوحيد رمزٌ صدر عن النواة.
///
/// لا يقبل `argv` ولا مسارًا ولا خيارًا إضافيًا: كل ما يُنفَّذ محفوظ في النواة
/// منذ لحظة التخطيط، والرمز مفتاحه.
///
/// عامٌّ على وقت التشغيل (`R`) لا مثبَّتٌ على `Wry`: هذا ما يجعل `ipc_handler`
/// عامًّا، فيقود اختبارُ الجسر وقتَ التشغيل الوهمي على **نفس** المعالج الذي
/// يركّبه `run()`. التثبيت على `Wry` كان يعني أن الأمر الوحيد الذي يطلق عمليات
/// حقيقية هو الأمر الوحيد الذي لا يمكن أن يمرّ عليه اختبار.
#[tauri::command]
async fn execute(
    app: tauri::AppHandle<impl tauri::Runtime>,
    state: State<'_, AppState>,
    token: String,
) -> Result<String> {
    let (cancel_tx, cancel_rx) = oneshot::channel();
    let stored = reserve_run(&state.runs, &state.plans, &state.session, token, cancel_tx)?;

    // معرّف التشغيل هو معرّف الخطة نفسه، فيتّصل قيد `planned` بما بعده في
    // السجل بدل أن يكون حدثًا يتيمًا.
    let run_id = stored.id.clone();

    let command = stored.command.clone();
    let op_id = stored.op_id;
    let program = command.program.display().to_string();
    let cwd = command.cwd.as_ref().map(|p| p.display().to_string());

    // القسم والمدخلات يُخرجان من الفهرس والخطة المحفوظة، لا ممّا ترسله الواجهة:
    // القيد أثرٌ لما نُفِّذ فعلًا، والمدخلات المحفوظة هي **المُتحقَّق منها** —
    // المسار المحلول لا الرابط الرمزي الذي كُتب في الحقل.
    let (category, journal_inputs, args) = match registry::find(op_id, state.policy) {
        Ok(op) => (
            Some(category_name(op.category)),
            stored.inputs.journal_form(op),
            stored.inputs.journal_args(op, &command.args),
        ),
        // A consumed plan should always resolve to the specification that
        // created it. If that invariant is ever broken, fail closed in the
        // persistent audit trail rather than storing possibly secret argv.
        Err(_) => (None, Vec::new(), vec!["[redacted]".to_owned(); command.args.len()]),
    };

    let mut running = Entry::new(run_id.clone(), op_id, journal::State::Running)
        .with_command(program.clone(), args.clone(), cwd.clone())
        .with_inputs(journal_inputs.clone());
    if let Some(c) = &category {
        running = running.with_category(c);
    }
    state.journal.record(running);

    let (out_tx, mut out_rx) = mpsc::channel::<OutputLine>(256);

    // بثّ الخرج إلى الواجهة، مع الاحتفاظ بذيله للسجل.
    //
    // الذيل يُجمع هنا لا في `executor`: هذا الموضع يرى كل سطرٍ مرّةً واحدة
    // بترتيبه، والمنفّذ يبثّ من مهمّتين متوازيتين. وحدّه في `journal.rs` —
    // اثنا عشر سطرًا مقصوصة — لا يجعله نسخةً من الخرج بل موضعَ رسالة الخطأ.
    let (tail_tx, tail_rx) = oneshot::channel::<(Vec<String>, Vec<OutputLine>, Vec<OutputLine>)>();
    let emit_app = app.clone();
    let emit_run = run_id.clone();
    tokio::spawn(async move {
        let mut tail: std::collections::VecDeque<String> =
            std::collections::VecDeque::with_capacity(journal::MAX_TAIL_LINES);
        let mut result_tail = result::EventOutputTail::with_limit(journal::MAX_TAIL_LINES);
        let mut result_output = result::EventOutputTail::new();
        while let Some(line) = out_rx.recv().await {
            if let OutputLine::Stdout(s) | OutputLine::Stderr(s) = &line {
                if tail.len() == journal::MAX_TAIL_LINES {
                    tail.pop_front();
                }
                tail.push_back(s.clone());
            }
            result_tail.push(line.clone());
            result_output.push(line.clone());
            let _ = emit_app.emit("run://output", RunOutput { run_id: emit_run.clone(), line });
        }
        let _ = tail_tx.send((
            tail.into_iter().collect(),
            result_tail.into_lines(),
            result_output.into_lines(),
        ));
    });

    let done_app = app.clone();
    let done_run = run_id.clone();
    let slot_app = app.clone();
    let slot_run = run_id.clone();
    tokio::spawn(async move {
        // الحارس يمسك الخانة طوال التشغيل ويحرّرها في `Drop`، فيشمل التحرير
        // مسار الذعر لا مسار النجاح وحده.
        let slot = SlotGuard(move || {
            let state: State<'_, AppState> = slot_app.state();
            recover(&state.runs).remove(&slot_run);
        });

        let started = Instant::now();
        // `Outcome::Success` لا يُبنى إلا بعد ترقية الناتج إلى اسمه النهائي،
        // فقيد `succeeded` لا يمكن أن يسبقها. انظر `executor::run`.
        let execution = executor::run_for(op_id, command, out_tx, cancel_rx).await;

        // التحرير صراحةً هنا لا عند نهاية المهمة: الواجهة قد تُطلق تشغيلًا
        // تاليًا فور وصول `run://finished`، فلا يجوز أن تكون الخانة محجوزة
        // بينما يُكتب السجل.
        drop(slot);

        // مهمّة البثّ تُغلق قناتها بعد آخر سطر، فالذيل جاهزٌ هنا حتمًا. وفشل
        // الاستقبال — لو ذُعرت تلك المهمّة — يكلّف الذيل وحده لا القيد كلّه.
        let (tail, result_tail, result_output) = tail_rx.await.unwrap_or_default();

        let reveal_kind = execution
            .outcome
            .produced()
            .and_then(|path| reveal::safe_target_kind(std::path::Path::new(path)));
        let journal_result = result::ResultContract::for_operation(
            op_id,
            execution.semantic,
            execution.outcome.produced(),
            result_tail,
            reveal_kind,
        );
        let event_result = result::ResultContract::for_operation(
            op_id,
            execution.semantic,
            execution.outcome.produced(),
            result_output,
            reveal_kind,
        );
        let outcome = execution.outcome;

        let state: State<'_, AppState> = done_app.state();
        let mut finished =
            Entry::new(done_run.clone(), op_id, journal::State::from_outcome(&outcome))
                .with_command(program, args, cwd)
                .with_duration(started.elapsed().as_millis() as u64)
                .with_inputs(journal_inputs)
                .with_tail(tail)
                .with_result(journal_result);
        if let Some(c) = &category {
            finished = finished.with_category(c);
        }
        state.journal.record(finished);

        let _ = done_app.emit(
            "run://finished",
            RunFinished { run_id: done_run, result: event_result, outcome },
        );
    });

    let _ = app.emit("run://started", RunStarted { run_id: run_id.clone() });
    Ok(run_id)
}

/// يلغي تشغيلًا جاريًا. الإشارة تطال مجموعة العمليات كلها، والمؤقّت يُنظَّف.
#[tauri::command]
fn cancel(state: State<'_, AppState>, run_id: String) -> Result<()> {
    let handle = recover(&state.runs).remove(&run_id).ok_or(CoreError::RunNotFound)?;
    let _ = handle.cancel.send(());
    Ok(())
}

/// الأقسام، وعددُ عمليات كلٍّ منها محسوبًا من الفهرس.
///
/// أمرٌ مستقلّ لا حقلٌ داخل `list_operations`: شاشة الفئات تُرسم قبل أن تُقرأ
/// عملية واحدة، وحساب الأعداد في الواجهة كان يعني أن الشاشة الأولى تنتظر
/// الفهرس كاملًا كي تعرض ثمانية عناوين.
#[tauri::command]
fn list_categories(state: State<'_, AppState>) -> Vec<categories::CategorySummary> {
    registry::categories(state.policy)
}

/// آخر التشغيلات، للمراجعة.
#[tauri::command]
fn recent_runs(state: State<'_, AppState>) -> Vec<Entry> {
    state.journal.recent()
}

/// يحذف كل قيود تشغيلٍ واحد.
///
/// المعامل معرّف تشغيل — نفس ما تقبله `reveal` — لا مسار ملفٍ ولا سطرٌ في
/// الملف: النواة تعرف أي أسطرٍ تخصّه، والواجهة لا تستطيع أن تطلب حذف غيرها.
///
/// ولا يمسّ هذا شيئًا على القرص خارج ملف السجلّ: التشغيل الذي حُذف قيدُه ما زال
/// ناتجه في مكانه. حذفُ الأثر ليس حذف ما فعله.
#[tauri::command]
async fn journal_delete(state: State<'_, AppState>, run_id: String) -> Result<()> {
    state.journal.delete(&run_id)
}

/// يمسح السجلّ كله. الشاشة تسأل قبله؛ هذا الأمر ينفّذ ولا يسأل.
///
/// أمرٌ في الحدّ لا حذفٌ للملف من الواجهة: الواجهة لا تملك مسارًا أصلًا.
#[tauri::command]
async fn journal_clear(state: State<'_, AppState>) -> Result<()> {
    state.journal.clear()
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

/// المعالج الذي يستقبل كل استدعاء من الواجهة. **تركيبٌ واحد لا نسختان.**
///
/// كان `generate_handler!` مكتوبًا داخل `run()`، فلم يكن هناك سبيل لأن يقود
/// اختبارٌ حدَّ IPC الحقيقي: الأوامر خاصة، و`run()` يشغّل تطبيقًا كاملًا بنافذة.
/// فبقيت طبقة `serde` — فكّ ما ترسله الواجهة، وتوليف ما يعود إليها وما يعود
/// إليها من أخطاء — الطبقةَ الوحيدة في المنتج التي لا يمسّها اختبار، مع أن
/// الواجهة لا تكلّم النواة إلا عبرها.
///
/// إخراجه هنا يجعل `tests/ipc_bridge.rs` يقود **هذا** المعالج لا نسخةً منه،
/// فلا يمكن أن يفترق ما يُختبر عمّا يُسجَّل. والقائمة تبقى في هذا الملف وحده،
/// فحرّاس `tests/security.rs` تقرأ الموضع نفسه الذي كانت تقرؤه.
pub fn ipc_handler<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync {
    tauri::generate_handler![
        list_operations,
        list_categories,
        plan,
        execute,
        cancel,
        recent_runs,
        journal_delete,
        journal_clear,
        reveal
    ]
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
        .invoke_handler(ipc_handler())
        .run(tauri::generate_context!())
        .expect("error while running naffith");
}

#[cfg(test)]
mod tests {
    use super::*;
    use spec::PlannedCommand;

    fn dummy_command() -> PlannedCommand {
        PlannedCommand {
            program: std::path::PathBuf::from("/bin/echo"),
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

    fn slot() -> Handle {
        Handle { cancel: oneshot::channel().0 }
    }

    /// قفلٌ مسموم يُتعافى منه ويعود سليمًا.
    ///
    /// الحالة الحقيقية: ذعرٌ في المخطّط يترك `plans` مسمومًا، وكل تخطيط بعده
    /// كان يعود بـ «الخطة غير موجودة» إلى نهاية الجلسة.
    #[test]
    fn a_poisoned_lock_is_recovered_not_treated_as_a_missing_plan() {
        let store: Mutex<PlanStore> = Mutex::new(PlanStore::new());

        let poisoner = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = store.lock().unwrap();
            panic!("planner exploded");
        }));
        assert!(poisoner.is_err(), "the panic must have happened");
        assert!(store.is_poisoned(), "and it must have poisoned the lock");

        let mut guard = recover(&store);
        let session = guard.register_session().expect("planning still works after the panic");
        assert!(guard
            .insert(&session, "internal.echo", Default::default(), dummy_command())
            .is_ok());
        drop(guard);

        assert!(!store.is_poisoned(), "the poison must be cleared, not carried to the next caller");
    }

    /// ذعرٌ داخل مهمة التشغيل لا يترك الخانة محجوزة.
    #[test]
    fn a_panic_inside_the_run_task_releases_the_slot() {
        let runs: Mutex<HashMap<String, Handle>> = Mutex::new(HashMap::new());
        recover(&runs).insert("r1".to_owned(), slot());

        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = SlotGuard(|| {
                recover(&runs).remove("r1");
            });
            panic!("executor exploded");
        }));

        assert!(panicked.is_err());
        assert!(recover(&runs).is_empty(), "a panicking run must not hold its slot forever");
    }

    /// بلوغ حدّ التزامن يرفض الطلب ولا يحرق الخطة.
    #[test]
    fn hitting_the_run_limit_leaves_the_plan_token_usable() {
        let plans = Mutex::new(PlanStore::new());
        let session = recover(&plans).register_session().unwrap();
        let (token, _) = recover(&plans)
            .insert(&session, "internal.echo", Default::default(), dummy_command())
            .unwrap();

        let runs: Mutex<HashMap<String, Handle>> = Mutex::new(HashMap::new());
        for i in 0..MAX_CONCURRENT_RUNS {
            recover(&runs).insert(format!("busy-{i}"), slot());
        }

        let refused =
            reserve_run(&runs, &plans, &session, token.as_str().to_owned(), oneshot::channel().0);
        assert!(
            matches!(refused, Err(CoreError::PlanLimitReached)),
            "the limit must refuse before the token is touched"
        );

        // وهذا بيت القصيد: الرمز ما زال صالحًا. المستخدم ينتظر ويضغط «نفّذ»
        // مرة أخرى بدل أن يعيد ملء النموذج.
        recover(&runs).remove("busy-0");
        assert!(
            reserve_run(&runs, &plans, &session, token.as_str().to_owned(), oneshot::channel().0)
                .is_ok(),
            "the token must survive a refusal that had nothing to do with the plan"
        );
    }

    /// الحدّ يصمد تحت التزامن: الفحص والحجز قسمٌ حرج واحد لا فحصٌ ثم فعل.
    #[test]
    fn the_run_limit_holds_when_many_executions_race() {
        for _ in 0..200 {
            let plans = Mutex::new(PlanStore::new());
            let session = recover(&plans).register_session().unwrap();
            let tokens: Vec<String> = (0..plans::MAX_PLANS_PER_SESSION)
                .map(|_| {
                    recover(&plans)
                        .insert(&session, "internal.echo", Default::default(), dummy_command())
                        .unwrap()
                        .0
                        .as_str()
                        .to_owned()
                })
                .collect();

            let runs: Mutex<HashMap<String, Handle>> = Mutex::new(HashMap::new());
            let barrier = std::sync::Barrier::new(tokens.len());

            std::thread::scope(|scope| {
                for token in &tokens {
                    let token = token.clone();
                    let (runs, plans, session, barrier) = (&runs, &plans, &session, &barrier);
                    scope.spawn(move || {
                        barrier.wait();
                        let _ = reserve_run(runs, plans, session, token, oneshot::channel().0);
                    });
                }
            });

            // لا شيء يحرّر خانة هنا، فالعدد النهائي هو عدد ما نجح حجزه.
            let held = recover(&runs).len();
            assert!(held <= MAX_CONCURRENT_RUNS, "{held} concurrent runs exceeded the limit");
        }
    }
}
