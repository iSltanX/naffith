//! المخطِّط: من مدخلات مُتحقَّق منها إلى أمر مبنيّ وشرحه.
//!
//! هذه هي النقطة التي يلتقي عندها نَفِّذ وسَطْر. استدعاء واحد يُنتج ما ترسمه
//! الواجهة نموذجًا وما تعرضه شرحًا — فلا يمكن أن يفترق ما يُعرَض عمّا يُنفَّذ،
//! لأنهما ناتج الدالة نفسها.

use crate::error::{CoreError, Result};
use crate::plans::{PlanStore, PlanToken, SessionId};
use crate::policy::Policy;
use crate::registry;
use crate::spec::{Conflict, Danger, ExplainToken, PlannedCommand};
use crate::value::{self, RawValue};
use serde::Serialize;
use std::collections::BTreeMap;

/// ما تراه الواجهة بعد التخطيط.
///
/// `argv` هنا **للعرض فقط**. لا يوجد أمر IPC يقبل `argv`، فلا سبيل لإعادة
/// إرساله. الرمز وحده هو ما يُقبل، والأمر الحقيقي محفوظ في النواة.
#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub token: PlanToken,
    /// معرّف عام يربط قيود السجل. ليس قدرة، ولا يُقبل في `execute`.
    pub plan_id: String,
    pub op_id: &'static str,
    pub title_key: &'static str,
    pub description_key: &'static str,
    pub danger: Danger,
    /// البرنامج ووسائطه كنصوص معروضة — لا كمدخل قابل للتنفيذ.
    pub argv_display: Vec<String>,
    pub explain: Vec<ExplainToken>,
    pub warnings: Vec<&'static str>,
    /// الأداة التي ستُنفِّذ.
    pub tool: ToolView,
    /// ما ستفعله العملية إن كان الاسم النهائي مشغولًا. معروضة قبل التنفيذ.
    pub conflict: Conflict,
    /// حجم تقديري للمصدر. `None` إذا كانت العملية لا تمسح شجرة.
    pub estimate: Option<EstimateView>,
    /// المسار النهائي المتوقّع إن كانت العملية تُنتج ملفًا.
    pub produces: Option<String>,
    /// المسار المؤقّت الذي سيكتب إليه الأمر فعلًا.
    ///
    /// معروض لأن الوسيط الأخير في الأمر هو هذا لا `produces`. إخفاؤه كان
    /// سيجعل «سَطْر» يبدو كأنه يعرض أمرًا لا يطابق ما يقوله «نَفِّذ».
    pub writes_to: Option<String>,
    pub working_directory: Option<String>,
}

/// الأداة التي ستُنفِّذ، معلَنةً للمستخدم.
///
/// «سَطْر» يعرض الأمر كاملًا أصلًا، لكن اسم الأداة ومسارها المطلق يستحقان
/// حقلًا صريحًا: هما جواب سؤال «ما الذي سيعمل على ملفاتي؟» ولا ينبغي أن
/// يُستخرَجا بقصّ أول عنصر من `argv_display`.
#[derive(Debug, Serialize)]
pub struct ToolView {
    pub id: &'static str,
    /// المسار المطلق المُتحقَّق منه لحظة التخطيط — لا ما في `PATH`.
    pub path: String,
}

/// تقدير، لا قياس. أسماء الحقول تحمل هذا المعنى كي لا تستطيع الواجهة عرض
/// الرقم كأنه حقيقة: انظر `estimate.rs` لأسباب كونه تقديرًا.
#[derive(Debug, Serialize)]
pub struct EstimateView {
    /// مجموع أحجام ملفات المصدر **قبل** الضغط. الأرشيف الناتج أصغر عادةً،
    /// فهذا حدّ أعلى تقريبي لا حجم الناتج.
    pub approx_source_bytes: u64,
    pub scanned_entries: usize,
    /// `false` يعني أن المسح توقّف عند حدّ: الرقمان أعلاه حدّ أدنى لا مجموع.
    pub complete: bool,
}

fn display(command: &PlannedCommand) -> Vec<String> {
    let mut v = Vec::with_capacity(command.args.len() + 1);
    v.push(command.program.display().to_string());
    for a in &command.args {
        v.push(a.to_string_lossy().into_owned());
    }
    v
}

/// يخطّط عملية: يبحث عنها، يتحقّق من مدخلاتها، يبني أمرها، ويحجز لها رمزًا.
pub fn plan(
    store: &mut PlanStore,
    session: &SessionId,
    policy: Policy,
    op_id: &str,
    raw_inputs: &BTreeMap<String, RawValue>,
) -> Result<PlanResponse> {
    let op = registry::find(op_id, policy)?;
    let inputs = value::validate(op, raw_inputs)?;
    let command = (op.plan)(&inputs)?;

    // فرض السياسة المعلَنة مركزيًا. `compress_ditto` يفحص التضارب في دالته
    // أيضًا كي يُرجع خطأً منسوبًا إلى الحقل الصحيح؛ هذا الفحص هو الشبكة تحته،
    // فعمليةٌ تُضاف لاحقًا وتنسى الفحص لا تستطيع أن تدهس ملفًا.
    if op.conflict == Conflict::Refuse {
        if let Some(a) = &command.artifact {
            if std::fs::symlink_metadata(&a.final_path).is_ok() {
                return Err(CoreError::DestinationExists);
            }
        }
    }

    let response_argv = display(&command);
    let produces = command.artifact.as_ref().map(|a| a.final_path.display().to_string());
    let writes_to = command.artifact.as_ref().map(|a| a.temp.display().to_string());
    let cwd = command.cwd.as_ref().map(|p| p.display().to_string());
    let explain = command.explain.clone();
    let warnings = command.warnings.clone();
    let tool = ToolView { id: op.tool.id, path: command.program.display().to_string() };
    let estimate = command.estimate.map(|e| EstimateView {
        approx_source_bytes: e.bytes,
        scanned_entries: e.entries,
        complete: e.complete,
    });

    let (token, plan_id) = store.insert(session, op.id, inputs, command)?;

    Ok(PlanResponse {
        token,
        plan_id,
        op_id: op.id,
        title_key: op.title_key,
        description_key: op.description_key,
        danger: op.danger,
        argv_display: response_argv,
        explain,
        warnings,
        tool,
        conflict: op.conflict,
        estimate,
        produces,
        writes_to,
        working_directory: cwd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs(pairs: &[(&str, RawValue)]) -> BTreeMap<String, RawValue> {
        pairs.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect()
    }

    #[test]
    fn planning_produces_a_token_and_a_displayable_command() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let r = plan(
            &mut store,
            &s,
            Policy::dev(),
            "internal.echo",
            &inputs(&[("message", RawValue::Text("مرحبا".into()))]),
        )
        .unwrap();

        assert_eq!(r.argv_display, vec!["/bin/echo".to_string(), "مرحبا".to_string()]);
        assert_eq!(r.token.as_str().len(), 64);
        assert_eq!(store.len(), 1, "the plan must be retained in the core, not handed out");
    }

    #[test]
    fn production_cannot_plan_the_internal_operation() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let r = plan(
            &mut store,
            &s,
            Policy::production(),
            "internal.echo",
            &inputs(&[("message", RawValue::Text("x".into()))]),
        );
        assert!(matches!(r, Err(CoreError::OperationNotAvailable(_))));
        assert!(store.is_empty(), "a refused plan must not be stored");
    }

    #[test]
    fn an_input_the_spec_does_not_declare_is_refused() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let r = plan(
            &mut store,
            &s,
            Policy::dev(),
            "internal.echo",
            &inputs(&[
                ("message", RawValue::Text("hi".into())),
                // محاولة تهريب مفتاح لا تعرفه المواصفة
                ("args", RawValue::Text("--dangerous".into())),
            ]),
        );
        assert!(matches!(r, Err(CoreError::UnexpectedInput(k)) if k == "args"));
    }

    #[test]
    fn a_wrong_input_type_is_refused_rather_than_coerced() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let r = plan(
            &mut store,
            &s,
            Policy::dev(),
            "internal.echo",
            &inputs(&[("message", RawValue::Path("/etc/passwd".into()))]),
        );
        assert!(matches!(r, Err(CoreError::WrongInputType { id: "message" })));
    }

    #[test]
    fn a_missing_required_input_is_refused() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let r = plan(&mut store, &s, Policy::dev(), "internal.echo", &inputs(&[]));
        assert!(matches!(r, Err(CoreError::MissingInput("message"))));
    }
}
