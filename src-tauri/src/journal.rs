//! سجلّ التنفيذ.
//!
//! الثقة تحتاج أثرًا قابلًا للمراجعة: ماذا شُغّل، بأي وسائط، متى، وبأي نتيجة.
//! سطر واحد لكل حدث بصيغة JSONL — قابل للإلحاق، ولا يفسد ملفًا كاملًا إن
//! انقطع التطبيق في منتصف الكتابة.
//!
//! ## الحالات
//!
//! كل تشغيل يمرّ بسلسلة معلَنة، ولكل حالة قيدها:
//!
//! ```text
//! planned ──▶ running ──▶ succeeded | failed | cancelled
//! ```
//!
//! `planned` يُكتب عند حجز الخطة، لا عند التنفيذ: خطةٌ رُوجعت ثم تُركت أثرٌ
//! يستحق أن يُرى. و`succeeded` **لا يُكتب إلا بعد الترقية إلى الاسم النهائي**،
//! لأن `Outcome::Success` نفسه لا يُبنى قبلها (انظر `executor::run`). سجلٌّ
//! يقول «نجح» وليس على القرص ناتجٌ يكذب.
//!
//! ## التزامن
//!
//! الكتابة مُسلسَلة على مستويين: `Mutex` داخل العملية، و`flock` حصريّ على
//! الملف بين العمليات. والسطر كله يُبنى في الذاكرة ثم يُكتب بنداء `write` واحد
//! على واصفٍ مفتوح بـ `O_APPEND`، فلا يتداخل سطران.
//!
//! ## الانهيار
//!
//! إن مات التطبيق في منتصف سطر، بقي في الملف سطرٌ مبتور. القارئ يتخطّاه
//! ويحتفظ بالباقي — فقدُ حدثٍ واحد مقبول، وفقدُ السجل كله ليس كذلك. والكاتب
//! يبدأ بسطر جديد إن لم ينتهِ الملف بفاصل أسطر، فلا يلتصق قيدُه بالمبتور
//! فيُفسد قيدين بدل واحد.
//!
//! ## النموّ
//!
//! السجلّ يُقرأ عند الإقلاع لتُبنى منه نافذة العرض، ويُضغط عند تجاوز سقفه
//! المعلَن فيبقى منه آخر ما يُراجَع. ولا يوجد في حدّ IPC أمرٌ يمسحه — الحدّ
//! ستة أوامر ولا سابع — فالسقف هو ما يمنعه من النموّ ما دام التطبيق مثبَّتًا.
//!
//! ## ما لا يُسجَّل
//!
//! * **رمز الخطة** — قدرة تُنفَّذ بها العملية. يُستبدل بمعرّف عام لا يفتح شيئًا.
//! * **‏stdout/stderr** — قد يحمل اسم كل ملف داخل الشجرة. السجل يقيّد ماذا
//!   شُغّل ونتيجته، لا ما طبعته الأداة.
//! * **البيئة** — الطفل يعمل ببيئة نظيفة أصلًا.
//!
//! ويبقى البرنامج ووسائطه، وفيهما مسارات المستخدم: هي أقلّ ما لا يصير السجلّ
//! بدونه مراجعةً.

use crate::error::CoreError;
use crate::executor::Outcome;
use crate::result::{
    CollectionRow, MetricValue, RawOutputLine, ResultContract, ResultPayload, ResultProperty,
    StructuredLine,
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// حالة الحدث. `tag` مسطّح داخل القيد كي يبقى السطر مستوًى واحدًا يسهل قراءته
/// بأدوات نصية (‏`grep '"state":"failed"'`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum State {
    /// حُجزت خطة ورمزها. لم يُشغَّل شيء بعد.
    Planned,
    /// أُطلقت العملية.
    Running,
    /// خرجت الأداة بصفر **ورُقّي الناتج إلى اسمه النهائي**.
    Succeeded { produced: Option<String> },
    /// خرجت بغير صفر، أو أنهتها إشارة، أو تعذّرت الترقية.
    Failed { reason: String, code: Option<i32> },
    /// ألغاها المستخدم. المؤقّت مُنظَّف.
    Cancelled,
}

impl State {
    /// يحوّل نتيجة التنفيذ إلى حالة سجل.
    ///
    /// `Error` تُقيَّد فشلًا لا حالةً ثالثة: من زاوية المستخدم، عمليةٌ لم تُنتج
    /// أرشيفًا فشلت — سواء رفضتها الأداة أو تعثّرت الترقية. والسبب محفوظ.
    pub fn from_outcome(outcome: &Outcome) -> Self {
        match outcome {
            Outcome::Success { produced } => State::Succeeded { produced: produced.clone() },
            Outcome::Failed { code } => State::Failed { reason: "exit".to_owned(), code: *code },
            Outcome::Signalled { signal } => {
                State::Failed { reason: "signal".to_owned(), code: *signal }
            }
            Outcome::Cancelled => State::Cancelled,
            Outcome::Error { key } => State::Failed { reason: (*key).to_owned(), code: None },
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            State::Planned => "planned",
            State::Running => "running",
            State::Succeeded { .. } => "succeeded",
            State::Failed { .. } => "failed",
            State::Cancelled => "cancelled",
        }
    }
}

/// مدخلٌ واحد كما يُقيَّد، لإعادة ملء النموذج لاحقًا.
///
/// `value: None` تعني «حقلٌ كان، وقيمته لا تُكتب» لا «حقلٌ فارغ». الفرق مهمّ
/// على الشاشة: زرّ «أعد بهذه القيم» يعرف أن عليه ترك الحقل للمستخدم بدل أن
/// يملأه بفراغ ويبدو كأنه استعاد كل شيء. انظر `InputSpec::secret`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRecord {
    pub id: String,
    #[serde(default)]
    pub value: Option<String>,
}

/// أقصى عدد أسطر خرجٍ تُحفظ مع القيد النهائي.
///
/// ## ولماذا تُحفظ أصلًا
///
/// رأس هذا الملف كان يقول إن الخرج **لا** يُسجَّل، وسببه صحيح: خرج `ditto`
/// على شجرةٍ كبيرة قد يحمل اسم كل ملفٍ فيها، وسجلٌّ يبتلعه يصير نسخةً من
/// فهرس القرص. لكن الغرض من السجلّ أن يُراجَع، ومراجعةُ فشلٍ بلا سطرٍ واحد
/// ممّا قالته الأداة ليست مراجعة: يبقى للمستخدم «رمز الخروج ‎1‎» ولا شيء غيره.
///
/// المقايضة محسومة بالحدّ لا بالمبدأ: **الذيل وحده** — وهو موضع رسالة الخطأ
/// في كل أداةٍ تقريبًا — بعددٍ صغير من الأسطر، وكلٌّ منها مقصوصٌ على حدّه.
/// فشجرةٌ من عشرة آلاف ملف تخلّف اثني عشر سطرًا لا عشرة آلاف.
pub const MAX_TAIL_LINES: usize = 12;

/// أقصى طول سطرٍ محفوظ، بالبايتات.
pub const MAX_TAIL_LINE_BYTES: usize = 240;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// المعرّف العام الذي يربط قيود التشغيل الواحد. ليس رمز الخطة.
    pub id: String,
    pub op_id: String,
    /// القسم الذي تنتمي إليه العملية، مقيَّدًا لحظتها.
    ///
    /// يُكتب ولا يُشتقّ عند القراءة: عمليةٌ تنتقل بين قسمين في إصدارٍ لاحق —
    /// أو تُحذف — تترك قيودها بلا قسمٍ يُصفّى به. القيد أثرٌ لما جرى وقتها.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// وقت الحدث بالثواني منذ حقبة يونكس، UTC. الواجهة تعرضه بتوقيت المستخدم.
    pub at: u64,
    /// زمن التنفيذ. في القيد النهائي وحده.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    pub program: String,
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// المدخلات كما مُلئت، بعد تنقيح السرّي منها. تغذّي «أعد بهذه القيم».
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<InputRecord>,
    /// آخر ما طبعته الأداة. في القيد النهائي وحده، ومحدودٌ بالحدّين أعلاه.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tail: Vec<String>,
    /// عقد النتيجة المنظّم. يغيب عن القيود القديمة وعن planned/running.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<ResultContract>,
    #[serde(flatten)]
    pub state: State,
}

impl Entry {
    pub fn new(id: impl Into<String>, op_id: impl Into<String>, state: State) -> Self {
        Entry {
            id: id.into(),
            op_id: op_id.into(),
            category: None,
            at: now_secs(),
            duration_ms: None,
            program: String::new(),
            args: Vec::new(),
            cwd: None,
            inputs: Vec::new(),
            tail: Vec::new(),
            result: None,
            state,
        }
    }

    pub fn with_command(mut self, program: String, args: Vec<String>, cwd: Option<String>) -> Self {
        self.program = program;
        self.args = args;
        self.cwd = cwd;
        self
    }

    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_owned());
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<InputRecord>) -> Self {
        self.inputs = inputs;
        self
    }

    /// يحفظ ذيل الخرج بعد قصّه على الحدّين المعلَنين.
    ///
    /// القصّ هنا لا عند العرض: ما لا يُكتب لا يُسرَّب، ولا ينمو به الملف.
    pub fn with_tail(mut self, lines: Vec<String>) -> Self {
        let start = lines.len().saturating_sub(MAX_TAIL_LINES);
        self.tail = lines[start..].iter().map(|l| clip_tail(l)).collect();
        self
    }

    pub fn with_result(mut self, mut result: ResultContract) -> Self {
        match &mut result.payload {
            ResultPayload::Artifact { output, .. } => bound_result_output(output),
            ResultPayload::Acknowledgement { details, .. }
            | ResultPayload::Digest { details, .. }
            | ResultPayload::Comparison { details, .. }
            | ResultPayload::Verdict { details, .. } => bound_structured_output(details),
            ResultPayload::Collection { rows, .. } => bound_collection_rows(rows),
            ResultPayload::PropertiesReport { properties, .. } => {
                bound_properties(properties);
            }
            ResultPayload::Metrics { metrics, .. } => bound_metrics(metrics),
            ResultPayload::DiffSearch { items, .. } => bound_structured_output(items),
            ResultPayload::Diagnostic { lines } => bound_result_output(lines),
            ResultPayload::RawOutput { lines } => bound_result_output(lines),
        }
        self.result = Some(result);
        self
    }
}

fn tail_start(len: usize) -> usize {
    len.saturating_sub(MAX_TAIL_LINES)
}

fn bound_structured_output(lines: &mut Vec<StructuredLine>) {
    let start = tail_start(lines.len());
    let bounded = lines
        .drain(start..)
        .map(|mut line| {
            line.value = clip_tail(&line.value);
            line
        })
        .collect();
    *lines = bounded;
}

fn bound_collection_rows(rows: &mut Vec<CollectionRow>) {
    let start = tail_start(rows.len());
    let bounded = rows
        .drain(start..)
        .map(|mut row| {
            row.cells = row.cells.into_iter().map(|cell| clip_tail(&cell)).collect();
            row
        })
        .collect();
    *rows = bounded;
}

fn bound_properties(properties: &mut Vec<ResultProperty>) {
    let start = tail_start(properties.len());
    let bounded = properties
        .drain(start..)
        .map(|mut property| {
            property.label_key = clip_tail(&property.label_key);
            property.value = clip_tail(&property.value);
            property
        })
        .collect();
    *properties = bounded;
}

fn bound_metrics(metrics: &mut Vec<MetricValue>) {
    let start = tail_start(metrics.len());
    let bounded = metrics
        .drain(start..)
        .map(|mut metric| {
            metric.label_key = clip_tail(&metric.label_key);
            metric.value = clip_tail(&metric.value);
            metric.unit = metric.unit.map(|unit| clip_tail(&unit));
            metric
        })
        .collect();
    *metrics = bounded;
}

fn bound_result_output(lines: &mut Vec<RawOutputLine>) {
    let start = tail_start(lines.len());
    let bounded = lines
        .drain(start..)
        .map(|line| match line {
            RawOutputLine::Stdout(text) => RawOutputLine::Stdout(clip_tail(&text)),
            RawOutputLine::Stderr(text) => RawOutputLine::Stderr(clip_tail(&text)),
            RawOutputLine::Omitted { dropped } => RawOutputLine::Omitted { dropped },
            RawOutputLine::Truncated { dropped } => RawOutputLine::Truncated { dropped },
        })
        .collect();
    *lines = bounded;
}

/// يقصّ سطرًا على حدّ محرفٍ كامل، فلا يُشطر حرفٌ عربي نصفين.
fn clip_tail(line: &str) -> String {
    if line.len() <= MAX_TAIL_LINE_BYTES {
        return line.to_owned();
    }
    let mut end = MAX_TAIL_LINE_BYTES;
    while end > 0 && !line.is_char_boundary(end) {
        end -= 1;
    }
    format!("{} …", &line[..end])
}

/// نافذة الذاكرة: أحدث القيود التي تُعرض بلا لمس القرص.
const MAX_RECENT: usize = 200;

/// أقصى عدد قيود `planned` داخل نافذة الذاكرة.
///
/// حدٌّ منفصل لأن هذه القيود ليست قرارات: الواجهة تعيد التخطيط بعد ٢٥٠ مِلّي
/// ثانية من سكون الكتابة، فتعديل اسم ملف واحد يولّد عشرات المعاينات. بحدٍّ
/// واحد كانت هذه المعاينات تطرد تشغيلًا انتهى للتوّ من النافذة، فتفقد
/// `reveal` مساره وتمتلئ شاشة السجل بضغطات المفاتيح بدل ما جاء المستخدم
/// ليراجعه.
pub const MAX_RECENT_PLANNED: usize = 40;

/// سقف حجم ملف السجل على القرص قبل الضغط.
///
/// معلَن كثابت عامّ أسوة بحدود الخرج في `executor`: حدٌّ لا يُقاس لا يُصدَّق.
pub const MAX_JOURNAL_BYTES: u64 = 1_048_576;

/// عدد القيود التي تبقى بعد الضغط.
pub const RETAINED_ENTRIES: usize = 500;

pub struct Journal {
    path: Option<PathBuf>,
    /// يُسلسل الكتابة داخل هذه العملية. `flock` يتكفّل بما بين العمليات.
    write_lock: Mutex<()>,
    recent: Mutex<Vec<Entry>>,
    max_recent: usize,
    max_bytes: u64,
    retained: usize,
}

impl Journal {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self::with_limits(path, MAX_RECENT, MAX_JOURNAL_BYTES, RETAINED_ENTRIES)
    }

    /// حدود مصغّرة للاختبار: الدوران يُختبر بحدّ صغير لا بكتابة ميغابايت.
    fn with_limits(
        path: Option<PathBuf>,
        max_recent: usize,
        max_bytes: u64,
        retained: usize,
    ) -> Self {
        let journal = Self {
            path,
            write_lock: Mutex::new(()),
            recent: Mutex::new(Vec::new()),
            max_recent,
            max_bytes,
            retained,
        };
        journal.reload();
        journal
    }

    /// يستعيد نافذة الذاكرة من القرص عند الإقلاع.
    ///
    /// بدون هذا كان السجلّ يُكتب ولا يُقرأ: بعد إعادة التشغيل تعود
    /// `recent_runs` فارغة و`reveal` تفشل لكل تشغيل سابق، والملف على القرص
    /// يحمل التاريخ كاملًا. أي أن «الأثر القابل للمراجعة» الذي يوجد هذا الملف
    /// من أجله كان أثرًا لا يقرأه أحد.
    fn reload(&self) {
        let Some(path) = &self.path else { return };
        match read_all(path) {
            Ok(read) => {
                if read.damaged > 0 {
                    eprintln!("naffith: skipped {} damaged journal line(s)", read.damaged);
                }
                let mut recent = self.recent.lock().unwrap_or_else(|e| e.into_inner());
                *recent = read.entries;
                trim(&mut recent, self.max_recent);
            }
            // سجلّ لا يُقرأ لا يمنع تشغيلًا جديدًا من أن يُقيَّد.
            Err(e) => eprintln!("naffith: could not read journal: {e}"),
        }
    }

    pub fn record(&self, entry: Entry) {
        if self.path.is_some() {
            if let Err(e) = self.append(&entry) {
                // فشل الكتابة لا يُسقط تشغيلًا نجح. يُبلَّغ ولا يُخفى.
                eprintln!("naffith: could not write journal entry: {e}");
            }
        }
        let mut recent = self.recent.lock().unwrap_or_else(|e| e.into_inner());
        recent.push(entry);
        trim(&mut recent, self.max_recent);
    }

    pub fn recent(&self) -> Vec<Entry> {
        self.recent.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// المسار النهائي الذي أنتجه تشغيلٌ ناجح، إن وُجد.
    ///
    /// هذا هو المصدر الوحيد لما يمكن «إظهاره في Finder»: الواجهة تعطي معرّفًا،
    /// والنواة تُخرج المسار من سجلّها. لا مسار يعبر الحدّ في الاتجاه الآخر.
    ///
    /// والبحث لا يقف عند النافذة: النافذة محدودة والأرشيف ليس كذلك. تشغيلٌ
    /// خرج منها — بإعادة تشغيل التطبيق أو بازدحام قيود لاحقة — ما زال ملفه على
    /// القرص وقيده في الملف، فالرجوع إلى الملف أصدق من قول «لا شيء لإظهاره».
    pub fn produced_for(&self, id: &str) -> Option<String> {
        {
            let recent = self.recent.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(found) = produced_in(&recent, id) {
                return Some(found);
            }
        }
        let path = self.path.as_ref()?;
        let read = read_all(path).ok()?;
        produced_in(&read.entries, id)
    }

    /// أحدث قيدٍ لتشغيلٍ بعينه، من النافذة ثم من الملف.
    ///
    /// يخدم «أعد العملية بالقيم السابقة»: الشاشة تعطي معرّفًا، والنواة تُخرج
    /// المدخلات من سجلّها هي. لا مدخلات تعبر الحدّ في الاتجاه الآخر لتُستعاد.
    pub fn entry(&self, id: &str) -> Option<Entry> {
        {
            let recent = self.recent.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(found) = recent.iter().rev().find(|e| e.id == id) {
                return Some(found.clone());
            }
        }
        let path = self.path.as_ref()?;
        let read = read_all(path).ok()?;
        read.entries.into_iter().rev().find(|e| e.id == id)
    }

    /// يحذف كل قيود تشغيلٍ واحد — من النافذة ومن الملف معًا.
    ///
    /// «قيدًا واحدًا» من زاوية المستخدم هو تشغيلٌ واحد، وهو في الملف ثلاثة
    /// أسطر أو أربعة (‏planned ثم running ثم النتيجة). حذفُ سطرٍ منها كان
    /// سيترك نصفَ تشغيلٍ معروضًا بحالةٍ لا تنتهي.
    pub fn delete(&self, id: &str) -> Result<(), CoreError> {
        let existed = {
            let mut recent = self.recent.lock().unwrap_or_else(|e| e.into_inner());
            let before = recent.len();
            recent.retain(|e| e.id != id);
            before != recent.len()
        };
        let on_disk = self.rewrite(|e| e.id != id)?;
        if existed || on_disk > 0 {
            Ok(())
        } else {
            Err(CoreError::JournalEntryNotFound)
        }
    }

    /// يمسح السجلّ كله. الشاشة تسأل قبله؛ هذه الدالة تنفّذ ولا تسأل.
    pub fn clear(&self) -> Result<(), CoreError> {
        self.recent.lock().unwrap_or_else(|e| e.into_inner()).clear();
        self.rewrite(|_| false)?;
        Ok(())
    }

    /// يعيد كتابة الملف مُبقيًا ما يجتاز الشرط. يعيد عدد ما حُذف.
    ///
    /// في المكان وتحت `flock`، للسبب نفسه الذي يحكم `compact`: القفل قفلُ هذا
    /// الـ inode، و`rename` كان سيتركنا نحرس inode مهجورًا.
    fn rewrite(&self, keep: impl Fn(&Entry) -> bool) -> std::io::Result<usize> {
        let Some(path) = &self.path else { return Ok(0) };
        let _serialised = self.write_lock.lock().unwrap_or_else(|e| e.into_inner());

        let mut file = match std::fs::OpenOptions::new().read(true).write(true).open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e),
        };
        let _lock = FileLock::acquire(&file)?;

        file.seek(SeekFrom::Start(0))?;
        let mut raw = Vec::new();
        file.read_to_end(&mut raw)?;

        let mut out = Vec::with_capacity(raw.len());
        let mut removed = 0usize;
        for line in raw.split(|b| *b == b'\n') {
            if line.iter().all(|b| b.is_ascii_whitespace()) {
                continue;
            }
            match serde_json::from_slice::<Entry>(line) {
                Ok(entry) if keep(&entry) => {
                    out.extend_from_slice(line);
                    out.push(b'\n');
                }
                // القيد المطابق يُحذف، والسطر المعطوب يُحذف معه: إعادةُ الكتابة
                // فرصةُ تنظيفٍ لا داعيَ لتفويتها.
                Ok(_) => removed += 1,
                Err(_) => {}
            }
        }

        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(&out)?;
        file.flush()?;
        Ok(removed)
    }

    fn append(&self, entry: &Entry) -> std::io::Result<()> {
        let Some(path) = &self.path else { return Ok(()) };
        let _serialised = self.write_lock.lock().unwrap_or_else(|e| e.into_inner());

        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut file =
            std::fs::OpenOptions::new().read(true).create(true).append(true).open(path)?;

        let _lock = FileLock::acquire(&file)?;

        // سقفٌ على الملف نفسه لا على الذاكرة وحدها: بلا هذا ينمو `runs.jsonl`
        // ما دام التطبيق مثبَّتًا، وليس في حدّ IPC أمرٌ يمسحه.
        if file.metadata()?.len() > self.max_bytes {
            compact(&mut file, self.retained)?;
        }

        let mut line = Vec::with_capacity(256);
        // سطر سابق مبتور (انهيار في منتصف كتابة) لا يبتلع قيدنا.
        if needs_leading_newline(&mut file)? {
            line.push(b'\n');
        }
        serde_json::to_writer(&mut line, entry)?;
        line.push(b'\n');

        // كتابة واحدة على واصف `O_APPEND`: الإزاحة والكتابة ذرّيتان معًا.
        file.write_all(&line)?;
        file.flush()
    }
}

fn produced_in(entries: &[Entry], id: &str) -> Option<String> {
    entries.iter().rev().find_map(|e| match &e.state {
        State::Succeeded { produced: Some(p) } if e.id == id => Some(p.clone()),
        _ => None,
    })
}

/// يقلّم نافذة الذاكرة على مستويين.
///
/// المستوى الأول يخصّ `planned` وحدها لأنها معاينات لا أحداث، والثاني هو
/// السقف العامّ. ترتيبهما مقصود: تُطرح المعاينات القديمة أولًا، فلا يدفع
/// تشغيلٌ منتهٍ ثمن ضغطات المفاتيح.
fn trim(entries: &mut Vec<Entry>, max_recent: usize) {
    let planned = entries.iter().filter(|e| matches!(e.state, State::Planned)).count();
    if planned > MAX_RECENT_PLANNED {
        let mut excess = planned - MAX_RECENT_PLANNED;
        entries.retain(|e| {
            if excess > 0 && matches!(e.state, State::Planned) {
                excess -= 1;
                false
            } else {
                true
            }
        });
    }
    let overflow = entries.len().saturating_sub(max_recent);
    if overflow > 0 {
        entries.drain(0..overflow);
    }
}

/// يضغط ملف السجل في مكانه: يُبقي آخر `retained` قيدًا سليمًا ويطرح ما قبلها.
///
/// في المكان لا بملف جانبي ثم `rename`: القفل الذي نحمله هو قفل هذا الـ inode،
/// و`rename` كان سيتركنا نحرس inode مهجورًا بينما تكتب عمليةٌ أخرى في الملف
/// الجديد بلا قفل — أي أن البديل «الأأمن ضدّ الانهيار» يشتري الأمان بكسر
/// التسلسل بين العمليات الذي يقوم عليه هذا الملف. والضغط يطرح أيضًا الأسطر
/// المعطوبة، فالانهيار القديم لا يُورَّث إلى الأبد.
///
/// والمعاينات تُطرح قبل غيرها، للسبب نفسه الذي يحكم نافذة الذاكرة: الملف هو
/// ما ترجع إليه `produced_for` حين تخرج التشغيلة من النافذة، فلو ملأته
/// المعاينات لصار الرجوع إليه بلا فائدة.
fn compact(file: &mut std::fs::File, retained: usize) -> std::io::Result<()> {
    file.seek(SeekFrom::Start(0))?;
    let mut raw = Vec::new();
    file.read_to_end(&mut raw)?;

    let mut kept: Vec<(&[u8], bool)> = raw
        .split(|b| *b == b'\n')
        .filter_map(|line| match serde_json::from_slice::<Entry>(line) {
            Ok(entry) => Some((line, matches!(entry.state, State::Planned))),
            Err(_) => None,
        })
        .collect();

    let planned_budget = retained / 5;
    let planned = kept.iter().filter(|(_, is_planned)| *is_planned).count();
    if planned > planned_budget {
        let mut excess = planned - planned_budget;
        kept.retain(|(_, is_planned)| {
            if excess > 0 && *is_planned {
                excess -= 1;
                false
            } else {
                true
            }
        });
    }
    if kept.len() > retained {
        kept.drain(0..kept.len() - retained);
    }

    let mut out = Vec::new();
    for (line, _) in kept {
        out.extend_from_slice(line);
        out.push(b'\n');
    }

    file.set_len(0)?;
    // الواصف مفتوح بـ `O_APPEND`، والنهاية صارت صفرًا، فالكتابة تبدأ من أوّله.
    file.write_all(&out)?;
    file.flush()
}

/// هل ينتهي الملف بمحتوى بلا فاصل أسطر؟
fn needs_leading_newline(file: &mut std::fs::File) -> std::io::Result<bool> {
    let len = file.seek(SeekFrom::End(0))?;
    if len == 0 {
        return Ok(false);
    }
    file.seek(SeekFrom::End(-1))?;
    let mut last = [0u8; 1];
    file.read_exact(&mut last)?;
    Ok(last[0] != b'\n')
}

/// قفل حصريّ على الملف طوال الكتابة، يُحرَّر في `Drop`.
struct FileLock(std::os::unix::io::RawFd);

impl FileLock {
    /// أقصى انتظارٍ لقفلٍ يمسكه غيرنا، ثم نُخفق بدل أن نقف.
    ///
    /// كان `flock(LOCK_EX)` وحده: انتظارٌ **بلا سقف**. ومن يمسك القفل قد يكون
    /// نسخةً ثانية من التطبيق (‏`app_data_dir` مشتقّ من المعرّف، فالنسختان
    /// تكتبان `runs.jsonl` نفسه) أو خيطَ المنفّذ في هذه العملية نفسها —
    /// و`flock` يُسلسل بين واصفَي فتحٍ مختلفَين ولو كانا لعمليةٍ واحدة.
    ///
    /// والثمن كان تجمّد الواجهة: `plan` أمرٌ متزامن، والأوامر المتزامنة في
    /// Tauri تعمل على **الخيط الرئيسي**، فيقف الخيط داخل نداء نظامٍ لا يعود
    /// منه، وتتوقّف حلقة أحداث AppKit، ويقول macOS «التطبيق لا يستجيب».
    ///
    /// سجلٌّ لا يُكتب سطرُه أهون من واجهةٍ لا تستجيب — والإخفاق مُبلَّغٌ في
    /// `record` لا مُبتلَع.
    const MAX_WAIT: std::time::Duration = std::time::Duration::from_millis(2_000);
    const RETRY_EVERY: std::time::Duration = std::time::Duration::from_millis(5);

    fn acquire(file: &std::fs::File) -> std::io::Result<Self> {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        let deadline = std::time::Instant::now() + Self::MAX_WAIT;
        loop {
            // SAFETY: الواصف مملوك لملفٍ حيّ ويبقى صالحًا حتى تحرير القفل.
            if unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) } == 0 {
                return Ok(FileLock(fd));
            }
            let err = std::io::Error::last_os_error();
            // ‏`EWOULDBLOCK` وحدها تعني «القفل مشغول». أي خطأٍ آخر عطلٌ حقيقي
            // يُرفع فورًا بدل أن يُعاد المحاولة عليه إلى نهاية المهلة.
            if err.raw_os_error() != Some(libc::EWOULDBLOCK) {
                return Err(err);
            }
            if std::time::Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::WouldBlock,
                    "journal lock is held by another writer",
                ));
            }
            std::thread::sleep(Self::RETRY_EVERY);
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // SAFETY: نفس الواصف وما زال مفتوحًا: الملف يعيش أطول من القفل.
        unsafe { libc::flock(self.0, libc::LOCK_UN) };
    }
}

/// ما استُخرج من ملف سجلّ على القرص.
#[derive(Debug, Default)]
pub struct ReadResult {
    pub entries: Vec<Entry>,
    /// أسطر تعذّرت قراءتها — غالبًا سطر مبتور من انهيار.
    pub damaged: usize,
}

/// يقرأ سجلًّا من القرص متسامحًا مع سطر مبتور.
///
/// سطرٌ لا يُحلَّل يُعدّ ولا يوقف القراءة. الغرض ألّا يُفقد السجلّ كله بسبب
/// انقطاع في اللحظة الخطأ.
///
/// والقراءة **بايتات لا نصًّا**: `read_to_string` كان يفشل على الملف كلّه إن
/// وُجد فيه بايت واحد غير صالح. وبتر السطر في منتصف حرف عربي — وحروف مسارات
/// المستخدم عربية غالبًا، وكلّ منها بايتان — ينتج بالضبط هذا البايت. أي أن
/// الانهيار الذي وعد رأس الملف بأن يكلّف حدثًا واحدًا كان يكلّف السجلّ كلّه،
/// وإلى الأبد: القيود التالية تُلحق بملف صار غير مقروء أصلًا.
pub fn read_all(path: &Path) -> std::io::Result<ReadResult> {
    let raw = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(ReadResult::default()),
        Err(e) => return Err(e),
    };
    let mut out = ReadResult::default();
    for line in raw.split(|b| *b == b'\n') {
        if line.iter().all(|b| b.is_ascii_whitespace()) {
            continue;
        }
        match serde_json::from_slice::<Entry>(line) {
            Ok(e) => out.entries.push(e),
            Err(_) => out.damaged += 1,
        }
    }
    Ok(out)
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, state: State) -> Entry {
        Entry::new(id, "compress.folder.zip", state).with_command(
            "/usr/bin/ditto".into(),
            vec!["-c".into(), "-k".into(), "/Users/x/مجلد".into()],
            None,
        )
    }

    /// قيدٌ في السجلّ لا يجوز أن يجمّد من يستدعيه مهما فعل جارُه.
    ///
    /// `flock(LOCK_EX)` بلا `LOCK_NB` انتظارٌ بلا سقف: نسخةٌ ثانية من التطبيق
    /// — أو خيطُ المنفّذ نفسه — تمسك القفل، فيقف المستدعي إلى الأبد. وحين
    /// يكون المستدعي **الخيط الرئيسي** (وأمر `plan` متزامن، فهو كذلك) تتوقّف
    /// حلقة أحداث AppKit ويقول macOS «التطبيق لا يستجيب».
    ///
    /// الاختبار يمسك القفل من واصفٍ مستقلّ ثم يقيس: `record` يجب أن تعود
    /// خلال مهلةٍ معلَنة، لا أن تنتظر إلى ما لا نهاية.
    #[test]
    fn a_peer_holding_the_journal_lock_cannot_freeze_the_caller() {
        use std::os::unix::io::AsRawFd;
        use std::sync::mpsc;

        let dir = std::env::temp_dir().join(format!("naffith-flock-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("runs.jsonl");
        std::fs::write(&path, b"").unwrap();

        // جارٌ يمسك القفل الحصريّ ولا يُفلته — كالنسخة الثانية من التطبيق.
        let peer = std::fs::OpenOptions::new().read(true).write(true).open(&path).unwrap();
        assert_eq!(unsafe { libc::flock(peer.as_raw_fd(), libc::LOCK_EX) }, 0);

        let j = Journal::new(Some(path.clone()));
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            j.record(Entry::new("run-1", "files.copy", State::Planned));
            let _ = tx.send(());
        });

        let returned = rx.recv_timeout(std::time::Duration::from_secs(5)).is_ok();

        unsafe { libc::flock(peer.as_raw_fd(), libc::LOCK_UN) };
        drop(peer);
        let _ = std::fs::remove_dir_all(&dir);

        assert!(
            returned,
            "record() لم تعد بينما يمسك جارٌ القفل — انتظارٌ بلا سقف يجمّد الخيط الرئيسي"
        );
    }

    #[test]
    fn every_state_is_recorded_not_just_success() {
        let j = Journal::new(None);
        j.record(entry("1", State::Planned));
        j.record(entry("1", State::Running));
        j.record(entry("2", State::Cancelled));
        j.record(entry("3", State::Failed { reason: "exit".into(), code: Some(2) }));

        let names: Vec<&str> = j.recent().iter().map(|e| e.state.name()).collect();
        assert_eq!(names, vec!["planned", "running", "cancelled", "failed"]);
    }

    #[test]
    fn the_lifecycle_of_one_run_shares_a_single_id() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = Journal::new(Some(path.clone()));
        j.record(entry("abc", State::Planned));
        j.record(entry("abc", State::Running));
        j.record(entry("abc", State::Succeeded { produced: Some("/Users/x/out.zip".into()) }));

        let read = read_all(&path).unwrap();
        assert_eq!(read.damaged, 0);
        assert_eq!(read.entries.len(), 3);
        assert!(read.entries.iter().all(|e| e.id == "abc"));
        assert_eq!(
            read.entries.iter().map(|e| e.state.name()).collect::<Vec<_>>(),
            vec!["planned", "running", "succeeded"]
        );
    }

    #[test]
    fn entries_are_appended_as_one_json_object_per_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = Journal::new(Some(path.clone()));
        j.record(entry("1", State::Succeeded { produced: None }));
        j.record(entry("2", State::Cancelled));

        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        for l in lines {
            let v: serde_json::Value = serde_json::from_str(l).expect("each line parses alone");
            assert!(v.get("id").is_some());
            assert!(v.get("state").is_some());
        }
    }

    #[test]
    fn arabic_arguments_survive_the_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = Journal::new(Some(path.clone()));
        j.record(entry("1", State::Succeeded { produced: None }));

        let read = read_all(&path).unwrap();
        assert_eq!(read.entries[0].args[2], "/Users/x/مجلد");
    }

    #[test]
    fn journals_written_before_the_result_contract_still_load() {
        let old = serde_json::json!({
            "id": "old",
            "op_id": "disk.free",
            "at": 1,
            "program": "/bin/df",
            "args": ["-h"],
            "state": "succeeded",
            "produced": null
        });
        let entry: Entry = serde_json::from_value(old).unwrap();
        assert!(entry.result.is_none());
        assert!(matches!(entry.state, State::Succeeded { produced: None }));
    }

    #[test]
    fn a_terminal_result_contract_survives_the_journal_round_trip() {
        let result = crate::result::ResultContract::for_operation(
            "text.search",
            crate::result::ResultSemantic::NoMatches,
            None,
            vec![crate::result::RawOutputLine::Stderr("none".into())],
            Some(crate::result::RevealKind::Directory),
        );
        let original = entry("result", State::Succeeded { produced: None }).with_result(result);
        let encoded = serde_json::to_vec(&original).unwrap();
        let decoded: Entry = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.result, original.result);
    }

    #[test]
    fn typed_result_output_obeys_the_journal_tail_limits_too() {
        let mut lines = vec![RawOutputLine::Stdout("/tmp/image.png".into())];
        lines.extend(
            (0..20).map(|index| {
                RawOutputLine::Stdout(format!("property{index}: {}", "x".repeat(500)))
            }),
        );
        let result = crate::result::ResultContract::for_operation(
            "image.info",
            crate::result::ResultSemantic::Completed,
            None,
            lines,
            None,
        );
        let entry =
            entry("bounded-result", State::Succeeded { produced: None }).with_result(result);
        let Some(ResultPayload::PropertiesReport { properties, .. }) =
            entry.result.map(|result| result.payload)
        else {
            panic!("expected a properties report")
        };
        assert_eq!(properties.len(), MAX_TAIL_LINES);
        for property in properties {
            assert_eq!(property.stream, crate::result::OutputStream::Stdout);
            assert!(property.value.len() <= MAX_TAIL_LINE_BYTES + " …".len());
        }
    }

    #[test]
    fn a_truncated_last_line_costs_one_entry_not_the_whole_journal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = Journal::new(Some(path.clone()));
        j.record(entry("1", State::Succeeded { produced: None }));
        j.record(entry("2", State::Cancelled));

        // انهيار في منتصف كتابة القيد الثالث.
        let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(br#"{"id":"3","op_id":"compress.folder.zip","at":17000"#).unwrap();
        drop(f);

        let read = read_all(&path).unwrap();
        assert_eq!(read.entries.len(), 2, "the intact entries must survive");
        assert_eq!(read.damaged, 1);
        assert_eq!(read.entries[0].id, "1");
    }

    #[test]
    fn a_new_entry_after_a_crash_does_not_glue_itself_to_the_broken_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = Journal::new(Some(path.clone()));
        j.record(entry("1", State::Cancelled));

        let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(br#"{"id":"2","truncat"#).unwrap();
        drop(f);

        // التطبيق يُعاد تشغيله ويقيّد حدثًا جديدًا.
        let j2 = Journal::new(Some(path.clone()));
        j2.record(entry("3", State::Planned));

        let read = read_all(&path).unwrap();
        assert_eq!(read.damaged, 1, "only the crash line stays damaged");
        assert_eq!(read.entries.len(), 2);
        assert_eq!(read.entries[1].id, "3", "the new entry must be readable on its own line");
    }

    #[test]
    fn concurrent_writers_never_interleave_a_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = std::sync::Arc::new(Journal::new(Some(path.clone())));

        let mut handles = Vec::new();
        for w in 0..8 {
            let j = j.clone();
            handles.push(std::thread::spawn(move || {
                for i in 0..50 {
                    j.record(entry(&format!("{w}-{i}"), State::Running));
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        let read = read_all(&path).unwrap();
        assert_eq!(read.damaged, 0, "no line may be torn by a concurrent write");
        assert_eq!(read.entries.len(), 400);
    }

    #[test]
    fn a_success_entry_carries_the_final_path_not_the_temporary_one() {
        let j = Journal::new(None);
        j.record(entry("1", State::Succeeded { produced: Some("/Users/x/أرشيف.zip".into()) }));
        assert_eq!(j.produced_for("1").as_deref(), Some("/Users/x/أرشيف.zip"));
    }

    #[test]
    fn nothing_is_reported_as_produced_for_a_run_that_did_not_succeed() {
        let j = Journal::new(None);
        j.record(entry("1", State::Planned));
        j.record(entry("1", State::Running));
        j.record(entry("1", State::Failed { reason: "exit".into(), code: Some(1) }));
        j.record(entry("2", State::Cancelled));

        assert_eq!(j.produced_for("1"), None);
        assert_eq!(j.produced_for("2"), None);
        assert_eq!(j.produced_for("no-such-run"), None);
    }

    #[test]
    fn outcomes_map_onto_the_declared_states() {
        use crate::executor::Outcome;
        assert_eq!(
            State::from_outcome(&Outcome::Success { produced: Some("/x".into()) }).name(),
            "succeeded"
        );
        assert_eq!(State::from_outcome(&Outcome::Failed { code: Some(1) }).name(), "failed");
        assert_eq!(State::from_outcome(&Outcome::Signalled { signal: Some(9) }).name(), "failed");
        assert_eq!(State::from_outcome(&Outcome::Cancelled).name(), "cancelled");
        // تعثّر الترقية ليس نجاحًا. هذا هو الفرق الذي يجعل السجل صادقًا.
        assert_eq!(
            State::from_outcome(&Outcome::Error { key: "err.dest.exists" }).name(),
            "failed"
        );
    }

    #[test]
    fn no_journal_entry_carries_a_plan_token() {
        // الرمز قدرة. كتابته على القرص تعني أن قراءة السجل تكفي لتنفيذ خطة.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = Journal::new(Some(path.clone()));
        j.record(entry("1", State::Planned));

        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("token"), "the journal must not name a token at all");
    }

    #[test]
    fn the_recent_buffer_is_bounded() {
        let j = Journal::new(None);
        for i in 0..500 {
            j.record(entry(&i.to_string(), State::Running));
        }
        assert_eq!(j.recent().len(), 200);
        assert_eq!(j.recent().first().unwrap().id, "300", "oldest entries drop first");
    }

    #[test]
    fn a_restart_finds_the_journal_where_it_left_it() {
        // الأثر القابل للمراجعة يعني: بعد إغلاق التطبيق وفتحه. النسخة السابقة
        // كانت تكتب الملف ثم تبدأ الجلسة التالية بنافذة فارغة، فتعرض شاشة
        // السجل «لا تشغيلات» وملفُّ القرص يحمل التاريخ كاملًا.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");

        let first = Journal::new(Some(path.clone()));
        first.record(entry("abc", State::Planned));
        first
            .record(entry("abc", State::Succeeded { produced: Some("/Users/x/أرشيف.zip".into()) }));
        drop(first);

        let after_restart = Journal::new(Some(path));
        assert_eq!(after_restart.recent().len(), 2, "the run log must survive a restart");
        assert_eq!(
            after_restart.produced_for("abc").as_deref(),
            Some("/Users/x/أرشيف.zip"),
            "and reveal must still know what that run produced"
        );
    }

    #[test]
    fn a_line_torn_inside_an_arabic_character_costs_one_line_not_the_history() {
        // البتر عند بايت ASCII هو الحالة السهلة. الحالة الواقعية أن المسار
        // عربي، فكل حرف بايتان، والقطع يقع داخل الحرف أكثر مما يقع بينه.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = Journal::new(Some(path.clone()));
        j.record(entry("1", State::Succeeded { produced: Some("/Users/x/أ.zip".into()) }));
        j.record(entry("2", State::Cancelled));
        drop(j);

        // انهيار في منتصف كتابة القيد الثالث، والقطع داخل حرف عربي:
        // 0xD9 أول بايتَي «م» بلا ثانيهما.
        let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
        f.write_all(b"{\"id\":\"3\",\"op_id\":\"c\",\"program\":\"/Users/x/\xd9").unwrap();
        drop(f);

        let read = read_all(&path).unwrap();
        assert_eq!(read.damaged, 1, "one torn line must cost one line");
        assert_eq!(read.entries.len(), 2, "and the intact history must survive it");

        let after_restart = Journal::new(Some(path));
        assert_eq!(after_restart.recent().len(), 2);
        assert_eq!(after_restart.produced_for("1").as_deref(), Some("/Users/x/أ.zip"));
    }

    #[test]
    fn plan_previews_never_evict_a_run_that_just_finished() {
        // بلا قرص: الاختبار على النافذة وحدها، كي لا يسترها الرجوع إلى الملف.
        let j = Journal::new(None);
        j.record(entry("done", State::Succeeded { produced: Some("/Users/x/أرشيف.zip".into()) }));
        // تعديل اسم ملف واحد على واجهة تعيد التخطيط كل ٢٥٠ مِلّي ثانية.
        for i in 0..400 {
            j.record(entry(&format!("preview-{i}"), State::Planned));
        }

        assert_eq!(
            j.produced_for("done").as_deref(),
            Some("/Users/x/أرشيف.zip"),
            "keystrokes must not revoke reveal for a run whose archive is still on disk"
        );
        let planned = j.recent().iter().filter(|e| matches!(e.state, State::Planned)).count();
        assert!(planned <= MAX_RECENT_PLANNED, "previews are bounded on their own: {planned}");
    }

    #[test]
    fn compaction_drops_previews_before_it_drops_a_finished_run() {
        // الملف هو ما ترجع إليه `produced_for` بعد خروج التشغيلة من النافذة.
        // ضغطٌ يُبقي آخر ما كُتب فحسب كان سيملأه بالمعاينات ويُفرغه من معناه.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = Journal::with_limits(Some(path.clone()), 8, 6_000, 40);
        j.record(entry("done", State::Succeeded { produced: Some("/Users/x/أرشيف.zip".into()) }));
        for i in 0..400 {
            j.record(entry(&format!("preview-{i}"), State::Planned));
        }

        assert!(std::fs::metadata(&path).unwrap().len() <= 6_000 + 1_024);
        assert_eq!(
            j.produced_for("done").as_deref(),
            Some("/Users/x/أرشيف.zip"),
            "a finished run must outlive the keystrokes that followed it"
        );
    }

    #[test]
    fn a_run_pushed_out_of_the_window_is_still_resolvable_from_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        let j = Journal::with_limits(Some(path), 8, MAX_JOURNAL_BYTES, RETAINED_ENTRIES);
        j.record(entry("done", State::Succeeded { produced: Some("/Users/x/أرشيف.zip".into()) }));
        for i in 0..50 {
            j.record(entry(&format!("later-{i}"), State::Running));
        }

        assert!(
            !j.recent().iter().any(|e| e.id == "done"),
            "the window must really have dropped it, or the test proves nothing"
        );
        assert_eq!(
            j.produced_for("done").as_deref(),
            Some("/Users/x/أرشيف.zip"),
            "a bounded window may not shrink what the user can reveal"
        );
    }

    #[test]
    fn the_file_is_compacted_once_it_passes_its_declared_ceiling() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("runs.jsonl");
        // حدود مصغّرة: الدوران سلوكٌ يُختبر بحدّ صغير لا بكتابة ميغابايت.
        let j = Journal::with_limits(Some(path.clone()), 200, 8_000, 10);
        for i in 0..200 {
            j.record(entry(&i.to_string(), State::Running));
        }

        let size = std::fs::metadata(&path).unwrap().len();
        assert!(size <= 8_000 + 1_024, "the journal file must stay bounded, it is {size} bytes");

        let read = read_all(&path).unwrap();
        assert_eq!(read.damaged, 0, "compaction must not tear a line");
        assert_eq!(
            read.entries.last().unwrap().id,
            "199",
            "compaction drops the oldest, never the newest"
        );
    }

    #[test]
    fn reading_a_journal_that_does_not_exist_yet_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let read = read_all(&dir.path().join("nothing.jsonl")).unwrap();
        assert!(read.entries.is_empty());
        assert_eq!(read.damaged, 0);
    }
}
