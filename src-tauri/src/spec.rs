//! `OperationSpec` — الوصف الواحد الذي تُشتقّ منه الواجهة والأمر معًا.
//!
//! هذا هو محور المعمارية. نَفِّذ يرسم `inputs` نموذجًا، وسَطْر يرسم ناتج
//! `plan` أمرًا مشروحًا. لأنهما دالّتان على نفس المصدر، لا يمكن أن يفترقا:
//! إضافة راية في الأمر تظهر في الشرح حتمًا، ولا يوجد مسار في الشيفرة يسمح
//! بعرض أمر غير الذي سيُنفَّذ.

use crate::error::Result;
use crate::value::Inputs;
use serde::Serialize;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Files,
    Compress,
    System,
    /// لا تظهر في الفهرس. للاختبار الداخلي فقط.
    Internal,
}

/// درجة الأثر. تحدد ما تطلبه الواجهة من تأكيد، ولا تُشتق من نص الأمر.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Danger {
    /// لا تكتب شيئًا.
    Safe,
    /// تُنشئ شيئًا جديدًا ولا تمسّ الموجود.
    Creates,
    /// تعدّل موجودًا، والتراجع ممكن.
    Modifies,
    /// تتلف بيانات، والتراجع غير ممكن.
    Destructive,
}

/// ما تفعله العملية إذا كان اسمها النهائي مشغولًا.
///
/// معلَنة في المواصفة لا مستنتَجة من سلوك الدالة، ولسببين: الواجهة تعرضها
/// للمستخدم **قبل** التنفيذ، و`planner` يفرضها مركزيًا — فعمليةٌ تُضاف غدًا
/// وتنسى الفحص في دالتها تُرفَض رغم ذلك.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Conflict {
    /// يُرفض التخطيط ويُترك القرار للمستخدم. لا استبدال ولا تنحية جانبًا.
    Refuse,
    /// العملية لا تُنتج ملفًا، فلا اسم نهائي يتضارب.
    NoArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Production,
    /// مُترجَمة دائمًا كي تُختبر النواة، لكن `Policy` تمنع تخطيطها في الإنتاج.
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum InputKind {
    /// مجلد قائم، مقروء.
    ExistingDir,
    /// ملف قائم، مقروء.
    ExistingFile,
    /// مجلد وجهة قائم وقابل للكتابة.
    TargetDir,
    /// اسم ملف جديد. يُنقّى ولا يُسمح بأن يحمل فاصل مسار.
    NewName {
        ext: Option<&'static str>,
    },
    Text {
        max_len: usize,
    },
    Flag,
}

#[derive(Debug, Clone, Copy)]
pub struct InputSpec {
    pub id: &'static str,
    pub kind: InputKind,
    pub required: bool,
}

/// دور الرمز داخل الأمر. تلوين «سَطْر» يُشتقّ منه، لا من تخمين نصّي:
/// مسارٌ اسمه `-c` كان سيُلوَّن رايةً لو خمّنّا من النصّ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenRole {
    /// اسم الأداة.
    Tool,
    /// راية مثل `-k` أو `--keepParent`.
    Flag,
    /// مسار في نظام الملفات.
    Path,
    /// قيمة نصية.
    Value,
}

/// رمز واحد في الأمر مقرونًا بمفتاح شرحه. هذا ما يبني عرض «سَطْر».
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExplainToken {
    /// النص كما يظهر في الأمر.
    pub token: String,
    /// مفتاح في قاموس الواجهة. `None` يعني «لا شرح لهذا الرمز» (مثل المسارات).
    pub key: Option<&'static str>,
    pub role: TokenRole,
}

impl ExplainToken {
    pub fn new(token: impl Into<String>, key: &'static str) -> Self {
        Self { token: token.into(), key: Some(key), role: TokenRole::Value }
    }
    pub fn bare(token: impl Into<String>) -> Self {
        Self { token: token.into(), key: None, role: TokenRole::Value }
    }
    pub fn with_role(mut self, role: TokenRole) -> Self {
        self.role = role;
        self
    }
}

/// ناتج مؤقّت يُكتب باسم عابر ثم يُنقل إلى اسمه النهائي عند النجاح وحده.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub temp: PathBuf,
    pub final_path: PathBuf,
}

/// أمر مبنيّ بالكامل داخل النواة. لا يحتوي نص shell، ولا يمرّ عبر مفسّر.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedCommand {
    /// مسار مطلق مُتحقَّق منه. لا يُقرأ من `PATH` أبدًا.
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: Option<PathBuf>,
    pub explain: Vec<ExplainToken>,
    /// مفاتيح تحذيرات تعرضها الواجهة قبل التنفيذ.
    pub warnings: Vec<&'static str>,
    pub artifact: Option<Artifact>,
    /// قياس المصدر لحظة التخطيط. `None` إذا كانت العملية لا تمسح شجرة.
    ///
    /// يُحمل هنا لا يُعاد حسابه في `planner`: المسح محدود بزمن، وتكراره يضاعف
    /// كلفة كل ضغطة مفتاح في النموذج.
    pub estimate: Option<crate::estimate::SizeEstimate>,
}

pub type PlanFn = fn(&Inputs) -> Result<PlannedCommand>;

#[derive(Debug)]
pub struct OperationSpec {
    pub id: &'static str,
    /// مفتاح العنوان العربي في قاموس الواجهة (`src/i18n.ts`).
    ///
    /// النصّ العربي يعيش في الواجهة لا في Rust: النواة تُختبر بلا واجهة،
    /// والنصوص تتغيّر أكثر مما يتغيّر المنطق. اختبارٌ في `registry.rs` يثبّت
    /// أن كل عملية تعلن مفتاحيها، واختبارٌ في الواجهة يثبّت أن كل مفتاح مترجَم.
    pub title_key: &'static str,
    /// مفتاح الوصف العربي.
    pub description_key: &'static str,
    pub category: Category,
    pub danger: Danger,
    pub visibility: Visibility,
    /// الأداة المستخدمة، بمسارها المطلق. تُتحقَّق عند التخطيط.
    pub tool: crate::tools::Tool,
    /// السياسة المعلَنة عند تضارب الاسم النهائي. يفرضها `planner`.
    pub conflict: Conflict,
    pub inputs: &'static [InputSpec],
    /// الدالة التي تبني الأمر. شيفرة Rust مُختبَرة، لا قالب نصّي:
    /// خطأ مطبعي في قالب يصبح خطأً على ملفات مستخدم حقيقي، وهنا يصبح خطأ ترجمة.
    pub plan: PlanFn,
}

impl OperationSpec {
    pub fn input(&self, id: &str) -> Option<&InputSpec> {
        self.inputs.iter().find(|i| i.id == id)
    }
}

/// ما تراه الواجهة في الفهرس. لا يحتوي دوال ولا مسارات أدوات.
#[derive(Debug, Clone, Serialize)]
pub struct OperationSummary {
    pub id: &'static str,
    pub title_key: &'static str,
    pub description_key: &'static str,
    pub category: Category,
    pub danger: Danger,
    pub inputs: Vec<InputSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InputSummary {
    pub id: &'static str,
    #[serde(flatten)]
    pub kind: InputKind,
    pub required: bool,
}

impl From<&OperationSpec> for OperationSummary {
    fn from(op: &OperationSpec) -> Self {
        OperationSummary {
            id: op.id,
            title_key: op.title_key,
            description_key: op.description_key,
            category: op.category,
            danger: op.danger,
            inputs: op
                .inputs
                .iter()
                .map(|i| InputSummary { id: i.id, kind: i.kind, required: i.required })
                .collect(),
        }
    }
}
