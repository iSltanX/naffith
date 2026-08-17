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

/// معرّف القسم. مغلقٌ ومعلَن هنا، فخريطةُ الواجهة منه ثابتة الحجم مهما بلغ
/// عدد العمليات. بياناته الوصفية — الاسم والوصف والأيقونة والترتيب — في
/// `categories.rs`، وعددُ عملياته يُحسب من الفهرس لا يُكتب بيد.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Files,
    Compress,
    Images,
    Text,
    Disk,
    Network,
    Security,
    Git,
    System,
    /// أدوات المطوّرين: مشاريع Node/npm وTauri وCargo/Rust. مفصولةٌ عن
    /// `System` عمدًا — الفرق ليس موضوعيًا وحده، بل بيئة التشغيل: هذه وحدها
    /// تحتاج `PlannedCommand::extra_path` كي تجد أدواتٍ تستدعيها داخليًا.
    Developer,
    /// قسمٌ لا عمليات فيه: مصدره سجلّ التشغيل نفسه. انظر `categories.rs`.
    History,
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

/// خيارٌ واحد في قائمة مغلقة. القيمة تدخل الأمر، والمفتاح يُترجَم في الواجهة.
///
/// قائمةٌ مغلقة لا نصٌّ حر: القيمة تصير وسيطًا (‏`jpeg` بعد `-s format`)، ونصٌّ
/// حر هنا كان سيعني أن الواجهة تكتب جزءًا من الأمر — وهو بالضبط ما بُنيت
/// المعمارية كي يبقى غير قابل للتعبير عنه.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ChoiceOption {
    pub value: &'static str,
    pub label_key: &'static str,
}

impl ChoiceOption {
    pub const fn new(value: &'static str, label_key: &'static str) -> Self {
        Self { value, label_key }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum InputKind {
    /// مجلد قائم، مقروء.
    ExistingDir,
    /// ملف قائم، مقروء.
    ExistingFile,
    /// ملفٌ أو مجلدٌ قائم. لعمليات لا تفرّق بينهما (نسخ، نقل، بصمة صلاحيات).
    ExistingPath,
    /// مجلد وجهة قائم وقابل للكتابة.
    TargetDir,
    /// اسم ملف جديد. يُنقّى ولا يُسمح بأن يحمل فاصل مسار.
    NewName {
        ext: Option<&'static str>,
    },
    /// اسم مجلد جديد يُنشأ داخل الوجهة. يُنقّى كما يُنقّى اسم الملف بلا امتداد.
    NewDirName,
    Text {
        max_len: usize,
    },
    /// اختيار من قائمة مغلقة. القيمة المرسلة يجب أن تكون إحدى `options`.
    Choice {
        options: &'static [ChoiceOption],
    },
    /// عددٌ صحيح ضمن مدى معلَن. يُتحقَّق منه في النواة لا في الحقل.
    Number {
        min: i64,
        max: i64,
        default: i64,
    },
    /// عنوان `http` أو `https` وحدهما. لا `file:` ولا `ftp:` ولا غيرهما.
    Url,
    Flag,
}

#[derive(Debug, Clone, Copy)]
pub struct InputSpec {
    pub id: &'static str,
    pub kind: InputKind,
    pub required: bool,
    /// قيمةٌ لا تُكتب في السجلّ.
    ///
    /// السجلّ يحفظ المدخلات كي يُعاد التشغيل بها، وعنوانٌ قد يحمل رمز وصول في
    /// استعلامه. `secret` يجعل القيمة تُنقّح عند القيد ويبقى الحقل مرئيًا بأنه
    /// كان — فرقٌ بين «لا قيمة» و«قيمةٌ لا تُكتب».
    pub secret: bool,
}

impl InputSpec {
    pub const fn new(id: &'static str, kind: InputKind) -> Self {
        Self { id, kind, required: true, secret: false }
    }

    pub const fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub const fn secret(mut self) -> Self {
        self.secret = true;
        self
    }
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

/// شكل الناتج. يقرّر كيف يُحجز المؤقّت وكيف يُرقّى.
///
/// ملفٌ يُرقّى بربطٍ صلب، ومجلدٌ لا يقبل الربط الصلب فيُرقّى بحجز الاسم ثم
/// `rename`. الفرق معلَن هنا لا مستنتَجٌ من القرص: عمليةٌ تُنتج مجلدًا لا يوجد
/// شيء منه على القرص لحظة التخطيط، فلا سبيل لاستنتاج شكله حينها.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    File,
    Dir,
}

/// ناتج مؤقّت يُكتب باسم عابر ثم يُنقل إلى اسمه النهائي عند النجاح وحده.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub temp: PathBuf,
    pub final_path: PathBuf,
    pub kind: ArtifactKind,
}

impl Artifact {
    pub fn file(temp: PathBuf, final_path: PathBuf) -> Self {
        Self { temp, final_path, kind: ArtifactKind::File }
    }
    pub fn dir(temp: PathBuf, final_path: PathBuf) -> Self {
        Self { temp, final_path, kind: ArtifactKind::Dir }
    }
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
    /// يُوجَّه خرج الأداة القياسي إلى هذا الملف بدل أن يُبثّ.
    ///
    /// لأدوات لا تعرف `-o`: `cat` تدمج، و`unzip -p` تستخرج مدخلةً واحدة، وكلاهما
    /// يكتب إلى `stdout` وحده. البديل — تمرير الأمر إلى صدفة كي تعيد التوجيه —
    /// كان سينقض القاعدة الأولى في `executor.rs`. التوجيه هنا يقع في النواة عبر
    /// واصفٍ مفتوح، فلا نصّ أمرٍ ولا مفسّر.
    ///
    /// **يجب أن يساوي `artifact.temp`**: الملف محجوز حصريًا قبل الإطلاق، فالفتح
    /// هنا فتحٌ لما نملكه لا إنشاءٌ في موضعٍ قد يسبقنا إليه غيرنا. يفرضه
    /// `planner` لا التوثيق.
    pub stdout_to: Option<PathBuf>,
    /// ما يُظهره «إظهار في Finder» بعد نجاحٍ لا يُنتج ناتجًا مؤقّتًا.
    ///
    /// عمليةٌ تُنتج `Artifact` لا تحتاجه: مسارها النهائي هو الجواب. لكن كثيرًا
    /// من العمليات تنجح ولا تُنتج ملفًا — `mv` ينقل إلى مكانه مباشرة، و`git
    /// init` يُنشئ داخل مجلدٍ قائم، و`find` يمسح شجرةً بعينها. لهذه، «أين
    /// أنظر؟» سؤالٌ له جواب، وتركُه بلا جواب كان يعني أن زرّ الإظهار يفشل
    /// بـ«لا يوجد ناتج» بعد تشغيلٍ نجح.
    ///
    /// يُنقّى في `reveal.rs` بسياسة المسارات كاملةً كما يُنقّى أي مسارٍ آخر:
    /// كونه صادرًا عن النواة لا يجعله فوق الفحص.
    pub reveal_target: Option<PathBuf>,
    /// ‏`PATH` الذي يُمنح للطفل، وحده دون بقية بيئة المستخدم.
    ///
    /// فارغةٌ في كل عملية اليوم عدا أدوات المطوّرين: `executor.rs` يمسح بيئة
    /// الطفل كاملةً (‏`env_clear` — لا `PATH` من المستخدم يُورَث أبدًا)، وهذا
    /// يكفي لأداةٍ مفردة كـ`ditto` لا تستدعي غيرها. لكن `cargo`/`npm` **تستدعيان
    /// أدواتٍ أخرى داخليًا** (‏`cc`، `xcrun`، وشِبَنغ `#!/usr/bin/env node` في
    /// `npm` نفسها) عبر `execvp` الذي يبحث في `PATH` — فبلا هذا الحقل تفشل
    /// `cargo check` بـ«‏linker `cc` not found» قبل أن تبدأ.
    ///
    /// كل عنصرٍ هنا إمّا **دليلُ نظامٍ ثابت** (‏`/usr/bin` وأخواتها — مملوكة
    /// لِـroot، محميّة بـSIP، لا يكتب فيها مستخدمٌ عادي) أو **مسارٌ تحقّق منه
    /// `plan()` بنفسه** (مجلد الأداة التي اختارها المستخدم في الإعداد). لا شيء
    /// هنا مُشتقٌّ من `PATH` الفعلي للمستخدم — وهذا هو الفرق بين توسيع القدرة
    /// المتاحة للطفل وإعادة الثقة الضمنية التي صُمِّم مسح البيئة كي يمنعها.
    pub extra_path: Vec<PathBuf>,
}

impl PlannedCommand {
    /// أمرٌ بلا ناتج ولا تقدير: الشكل الغالب في عمليات القراءة.
    pub fn read_only(program: PathBuf, args: Vec<OsString>, explain: Vec<ExplainToken>) -> Self {
        PlannedCommand {
            program,
            args,
            cwd: None,
            explain,
            warnings: vec![],
            artifact: None,
            estimate: None,
            stdout_to: None,
            reveal_target: None,
            extra_path: Vec::new(),
        }
    }
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
    /// موضع العملية داخل قسمها. الأصغر أولًا، والتعادل يُفكّ بالمعرّف.
    ///
    /// معلَنٌ لا مستنتَجٌ من ترتيب المصفوفة: المصفوفة تُعاد ترتيبها بالسهو عند
    /// كل إضافة، والرقم قرارُ منتجٍ يُقرأ في مكانه.
    pub sort_order: u16,
    /// كلماتٌ يجدها البحث فوق العنوان والوصف.
    ///
    /// مكانها هنا لا في الواجهة: البحث يجب أن يجد العملية باسم الأداة
    /// (`ditto`، `sips`) وبمرادفٍ شائع (`unzip`، `فك`) وبالإنجليزية، وأيٌّ من
    /// ذلك ليس في العنوان العربي. وقائمةٌ في الواجهة كانت ستصير فهرسًا ثانيًا
    /// يتقادم صامتًا.
    pub search_terms: &'static [&'static str],
    /// الدالة التي تبني الأمر. شيفرة Rust مُختبَرة، لا قالب نصّي:
    /// خطأ مطبعي في قالب يصبح خطأً على ملفات مستخدم حقيقي، وهنا يصبح خطأ ترجمة.
    pub plan: PlanFn,
}

impl OperationSpec {
    pub fn input(&self, id: &str) -> Option<&InputSpec> {
        self.inputs.iter().find(|i| i.id == id)
    }
}

/// هل تستطيع هذه النسخة من النظام تشغيل هذه العملية؟
///
/// سؤالٌ عن الجهاز لا عن البناء: `git` تأتي مع أدوات Xcode، و`qlmanage` قد
/// تغيب على نظامٍ مقلَّم. الجواب يُحسب لحظة قراءة الفهرس بحلّ أداة العملية.
///
/// والعملية غير المتاحة **تُعرض ولا تُخفى**: قائمةٌ تختصر نفسها صامتةً تجعل
/// المستخدم يظنّ أن المنتج لا يفعل ذلك أصلًا، بدل أن يعرف أن أداةً واحدة تنقصه.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum Availability {
    Available,
    /// الأداة غائبة أو ليست ملفًا تنفيذيًا. `tool` معرّفها كي تسمّيها الشاشة.
    ToolMissing {
        tool: &'static str,
    },
}

impl Availability {
    pub fn of(op: &OperationSpec) -> Self {
        match op.tool.resolve() {
            Ok(_) => Availability::Available,
            Err(_) => Availability::ToolMissing { tool: op.tool.id },
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Availability::Available)
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
    /// السياسة المعلَنة عند تضارب الاسم. تُعرض على البطاقة قبل فتح الشاشة.
    pub conflict: Conflict,
    /// معرّف الأداة، كي يجدها البحث ويسمّيها سببُ التعطيل.
    pub tool: &'static str,
    pub availability: Availability,
    pub sort_order: u16,
    pub search_terms: &'static [&'static str],
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
            conflict: op.conflict,
            tool: op.tool.id,
            availability: Availability::of(op),
            sort_order: op.sort_order,
            search_terms: op.search_terms,
            inputs: op
                .inputs
                .iter()
                .map(|i| InputSummary { id: i.id, kind: i.kind, required: i.required })
                .collect(),
        }
    }
}
