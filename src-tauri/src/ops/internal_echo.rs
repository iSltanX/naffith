//! عملية اختبار داخلية — **لا تظهر للمستخدم ولا تدخل فهرس الإصدار**.
//!
//! وجودها غرضه واحد: إثبات أن السلسلة كاملة تعمل من طرف إلى طرف في M0 —
//! تحقّق المدخلات، بناء `argv` في Rust، حجز رمز خطة، إعادة التحقّق قبل
//! التشغيل، إطلاق العملية، بثّ الخرج، رمز الخروج، قيد في السجل — دون أن
//! نبني عملية الضغط الحقيقية قبل موعدها في M1.
//!
//! `echo` اختيرت لأنها لا تكتب شيئًا على القرص، فلا يمكن لخطأ في M0 أن يمسّ
//! ملفات المستخدم.

use crate::error::Result;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsString;

pub const SPEC: OperationSpec = OperationSpec {
    id: "internal.echo",
    title_key: "op.internal.echo.title",
    description_key: "op.internal.echo.description",
    category: Category::Internal,
    danger: Danger::Safe,
    visibility: Visibility::Internal,
    tool: tools::ECHO,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec { id: "message", kind: InputKind::Text { max_len: 512 }, required: true }],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let program = tools::ECHO.resolve()?;
    let message = inputs.text("message")?;

    // النص يمرّ وسيطًا واحدًا. لو كان يحوي `;` أو `$(…)` فهو محارف حرفية:
    // لا صدفة في المسار، فلا شيء يفسّرها. اختبار ذهبي يثبّت هذا.
    let args = vec![OsString::from(message)];

    Ok(PlannedCommand {
        program,
        args,
        cwd: None,
        explain: vec![ExplainToken::new("echo", "explain.echo.cmd"), ExplainToken::bare(message)],
        warnings: vec![],
        artifact: None,
        estimate: None,
    })
}
