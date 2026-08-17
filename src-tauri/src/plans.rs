//! مخزن الخطط ورموزها.
//!
//! العقد الذي يحمي التنفيذ:
//!
//! * الخطة تُبنى وتُحفظ **داخل النواة**. الواجهة لا ترى `argv` إلا كنص للعرض،
//!   ولا تستطيع إعادة إرساله.
//! * الرمز ‏٢٥٦ بت عشوائية من مولّد النظام — غير قابل للتخمين عمليًا.
//! * **أحادي الاستخدام**: `take` ينتزع الخطة من المخزن، فإعادة الإرسال تفشل.
//! * **قصير العمر**: تنتهي صلاحيته بعد `TTL`، ويُكنس دوريًا — بالساعتين معًا،
//!   الأحادية وساعة الحائط، لأن الأولى تتوقّف مع نوم الجهاز.
//! * **مرتبط بهويّة المسارات** وقت التخطيط، ويُعاد التحقّق منها قبل التشغيل.
//! * **يحجز مساره المؤقّت حجزًا حصريًا** لحظة التحقّق، فلا يبقى بين الفحص
//!   والكتابة نافذةٌ يزرع فيها غيرُنا رابطًا رمزيًا.
//! * **محدود العدد** لكل جلسة وإجمالًا، فلا يمكن إغراق الذاكرة بخطط.

use crate::error::{CoreError, Result, StaleReason};
use crate::spec::{ArtifactKind, PlannedCommand};
use crate::value::Inputs;
use rand::RngCore;
use serde::Serialize;
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// عمر الخطة. أطول من زمن مراجعة بشرية معقولة، وأقصر من أن تصير بصمتها كذبًا.
pub const PLAN_TTL: Duration = Duration::from_secs(180);
pub const MAX_PLANS_PER_SESSION: usize = 16;
pub const MAX_PLANS_TOTAL: usize = 64;
pub const MAX_SESSIONS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct PlanToken(String);

impl PlanToken {
    fn generate() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let mut s = String::with_capacity(64);
        for b in bytes {
            s.push_str(&format!("{b:02x}"));
        }
        PlanToken(s)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for PlanToken {
    fn from(s: String) -> Self {
        PlanToken(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct SessionId(String);

impl SessionId {
    pub fn generate() -> Self {
        let mut bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut bytes);
        let mut s = String::with_capacity(32);
        for b in bytes {
            s.push_str(&format!("{b:02x}"));
        }
        SessionId(s)
    }
}

pub fn random_suffix() -> String {
    let mut bytes = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// بصمة مسار وقت التخطيط.
///
/// **ما تضمنه**: أن المسار ما زال يدلّ على *نفس الكائن* في نظام الملفات.
/// `dev`+`ino` يكشفان الاستبدال حتى لو بقي الاسم، و`is_dir` يكشف انقلاب
/// المجلد ملفًا أو رابطًا. هذا وحده ما يحتاجه الأمان: `argv` بُني على مواضع
/// بعينها، والشرط أن تبقى هي هي.
///
/// **وما لا تضمنه — عمدًا**: محتوى شجرة. `mtime` المجلد كان مسجَّلًا هنا
/// بادّعاء أنه «يكشف تعديل المحتوى»، وهو ادّعاء لا تسنده الآلية: مجلدٌ
/// يتحرّك `mtime` بظهور ابنٍ مباشر فيه ولا يتحرّك بتعديل حفيد، فالضمان
/// المعلَن أوسع من الضمان المنفَّذ. وأسوأ منه أن النواة نفسها تحرّكه — فحص
/// الكتابة يُنشئ ملفًا ويحذفه داخل مجلد الوجهة — فكانت خطةٌ ثانية إلى نفس
/// المجلد تُبطل الأولى قبل أن يُنفَّذ شيء، ويُنسب الإبطال إلى «المصدر».
///
/// لذلك يُسجَّل `mtime` **للملفات العادية وحدها**، حيث يعني ما يقوله فعلًا.
/// وتغيّر محتوى المجلد بين التخطيط والتنفيذ ليس خطأً يُرفض أصلًا: `ditto`
/// تقرأ الشجرة لحظة التشغيل، وأرشفةُ ما هو موجود آنذاك هي ما طلبه المستخدم.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PathFingerprint {
    dev: u64,
    ino: u64,
    /// `None` للمجلدات. وبدقّة النانوثانية للملفات: الثانية الكاملة نافذةٌ
    /// يُستبدل فيها ملف بملف يحمل الطابع نفسه.
    mtime: Option<(i64, i64)>,
    is_dir: bool,
}

impl PathFingerprint {
    fn capture(p: &Path) -> Option<Self> {
        let m = std::fs::symlink_metadata(p).ok()?;
        let is_dir = m.is_dir();
        Some(PathFingerprint {
            dev: m.dev(),
            ino: m.ino(),
            mtime: (!is_dir).then(|| (m.mtime(), m.mtime_nsec())),
            is_dir,
        })
    }
}

/// ما يجب أن يبقى صحيحًا بين لحظة التخطيط ولحظة الضغط على «نفّذ».
///
/// الفجوة بينهما فجوة حقيقية يتصرّف فيها المستخدم وغيره: قد يُحذف المصدر، أو
/// يُفصل القرص، أو يظهر ملف بالاسم النهائي، أو يُحدَّث النظام فتختفي الأداة.
/// التحقّق عند التخطيط وحده يعني أن الأمر يُطلق على عالمٍ لم يعد موجودًا.
#[derive(Debug, Clone)]
struct Preconditions {
    /// المسارات القائمة التي بُنيت الخطة عليها.
    inputs: Vec<(PathBuf, PathFingerprint)>,
    /// المسار النهائي الذي يجب أن يبقى **غير موجود** حتى لحظة التشغيل.
    final_path_must_be_absent: Option<PathBuf>,
    /// المسار المؤقّت الذي ستكتب إليه الأداة فعلًا. لا يُفحص بل **يُحجز**.
    temp_to_claim: Option<(PathBuf, ArtifactKind)>,
    /// المجلد الذي سيُكتب فيه الناتج. يُعاد فحص كتابته لا قراءة صلاحياته.
    destination_must_be_writable: Option<PathBuf>,
    /// الأداة التي حُلّت عند التخطيط. اختفاؤها بين اللحظتين حالة واقعية.
    program: PathBuf,
}

impl Preconditions {
    fn capture(inputs: &Inputs, command: &PlannedCommand) -> Result<Self> {
        let mut captured = Vec::new();
        for p in inputs.existing_paths() {
            // لا إسقاط صامت: مسارٌ لم تُؤخذ بصمته مسارٌ لن يُعاد التحقّق منه
            // أبدًا، أي شرطٌ يفشل مفتوحًا داخل بنيةٍ كل غرضها أن تفشل مغلقة.
            // خطةٌ لا تُثبَّت كاملة لا تُحفَظ.
            let fp = PathFingerprint::capture(p).ok_or(CoreError::PathMissing)?;
            captured.push((p.to_path_buf(), fp));
        }
        let artifact = command.artifact.as_ref();
        Ok(Preconditions {
            inputs: captured,
            final_path_must_be_absent: artifact.map(|a| a.final_path.clone()),
            temp_to_claim: artifact.map(|a| (a.temp.clone(), a.kind)),
            destination_must_be_writable: artifact
                .and_then(|a| a.final_path.parent())
                .map(Path::to_path_buf),
            program: command.program.clone(),
        })
    }

    /// يحجز المسار المؤقّت بإنشائه حصريًا، لا بفحصه.
    ///
    /// المؤقّت هو ما تفتحه `ditto` فعلًا — الوسيط الأخير في الأمر — ولم يكن
    /// مشمولًا بأي شرط. واسمه ليس سرًّا رغم عشوائيّته: يُكتب في `runs.jsonl`
    /// قيدَ `planned` قبل أن يضغط المستخدم «نفّذ»، ويعرضه «سَطْر» على الشاشة.
    /// فعمليةٌ أخرى بنفس المستخدم تقرأ الاسم، وتزرع مكانه رابطًا رمزيًا إلى
    /// ملفٍ للمستخدم — و`ditto` تتبع الرابط وتكتب فوق الضحية، ثم تُرقّى
    /// النتيجة ويُسجَّل التشغيل **نجاحًا**. مقيسٌ لا مفترَض.
    ///
    /// الفحص وحده لا يكفي: بين «غير موجود» و«‏`ditto` تفتحه» نافذة. الإنشاء
    /// الحصري يجعل الفحص والحجز خطوة واحدة، فيسقط شرط السباق من أصله، ووجودُ
    /// شيء بالاسم يصير فشلًا صريحًا لا كتابةً عبره.
    ///
    /// و`ditto` تكتب داخل ملف قائم بلا شكوى، فالحجز لا يعطّل الأداة.
    ///
    /// الصلاحيات `0o666` لا `0o600`: القناع يخفضها إلى ما كانت `ditto`
    /// ستُنشئه بنفسها، فلا يتغيّر وجه الأرشيف الذي يراه المستخدم. و`0o600`
    /// كان سيوهم بحماية لا يقدّمها هنا — الخصم في هذا السيناريو عمليةٌ تعمل
    /// بنفس المستخدم أصلًا.
    ///
    /// والسبب `TempPathTaken` لا `FinalPathAppeared`: الموضعان مختلفان — هذا
    /// اسمٌ داخلي يخترعه `atomic::temp_path_for`، وذاك الاسم الذي كتبه
    /// المستخدم. إعارةُ السبب كانت تقول له إن ملفًا ظهر باسم أرشيفه، فيفتح
    /// المجلد ولا يجد شيئًا.
    ///
    /// **أثرٌ يجب أن يقابله `atomic::ArtifactGuard::commit`**: بعد الحجز يصير
    /// المؤقّت موجودًا دائمًا، فشرط «لم يُنتج شيء» لم يعد وجودَ الملف بل
    /// كونَه غير فارغ. `commit` تفحص `exists()` وحدها، فترقية أرشيف بحجم صفر
    /// صارت ممكنة نظريًا؛ الفحص هناك يجب أن يصير على الحجم.
    /// و**المجلد يُحجز بـ`create_dir`**، وهي ذرّية بنفس المعنى وتفشل بـ
    /// `EEXIST` على أي شيء يشغل الاسم. أداةُ الاستخراج تكتب داخله فتجده جاهزًا،
    /// و`ArtifactGuard` تقيس «أُنتج شيء» بوجود مدخلةٍ فيه لا بوجوده هو.
    fn claim_temp(&self) -> Result<()> {
        use std::os::unix::fs::OpenOptionsExt;
        let Some((temp, kind)) = &self.temp_to_claim else { return Ok(()) };
        let claimed = match kind {
            ArtifactKind::File => std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o666)
                .open(temp)
                .map(|_| ()),
            ArtifactKind::Dir => std::fs::create_dir(temp),
        };
        match claimed {
            Ok(()) => Ok(()),
            // `create_new` يفشل على الرابط الرمزي كما يفشل على الملف، ولا
            // يتبعه — وهذا هو المقصود بالضبط. و`create_dir` كذلك.
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(CoreError::PlanStale { detail: StaleReason::TempPathTaken })
            }
            Err(e) => Err(CoreError::Io(e)),
        }
    }

    fn verify(&self) -> Result<()> {
        for (path, expected) in &self.inputs {
            match PathFingerprint::capture(path) {
                None => {
                    return Err(CoreError::PlanStale { detail: StaleReason::SourceGone });
                }
                Some(now) if now != *expected => {
                    return Err(CoreError::PlanStale { detail: StaleReason::SourceReplaced });
                }
                Some(_) => {}
            }
        }
        if let Some(final_path) = &self.final_path_must_be_absent {
            // symlink_metadata لا يتبع الروابط: رابط معلَّق بالاسم النهائي
            // ما زال تضاربًا يجب أن يوقفنا.
            if std::fs::symlink_metadata(final_path).is_ok() {
                return Err(CoreError::PlanStale { detail: StaleReason::FinalPathAppeared });
            }
        }
        if let Some(dir) = &self.destination_must_be_writable {
            if !dir.is_dir() {
                return Err(CoreError::PlanStale { detail: StaleReason::DestinationGone });
            }
            // فحص كتابة جديد: نجاحه عند التخطيط لا يلزم منه بقاؤه. قرص قد
            // يُعاد تركيبه للقراءة فقط، أو تتغيّر ACL، بين اللحظتين.
            if !crate::paths::is_writable_dir(dir) {
                return Err(CoreError::PlanStale { detail: StaleReason::DestinationNotWritable });
            }
        }
        // نفس شرط `Tool::resolve`: ملف عادي **وبتّ تنفيذ**. فحصٌ أضعف منه كان
        // يمرّر أداةً فقدت بتّ التنفيذ فتفشل عند الإطلاق برسالة عامّة، بدل أن
        // تُرفض هنا بسببٍ يقرؤه المستخدم.
        let usable = std::fs::metadata(&self.program)
            .map(|m| {
                use std::os::unix::fs::PermissionsExt;
                m.is_file() && m.permissions().mode() & 0o111 != 0
            })
            .unwrap_or(false);
        if !usable {
            return Err(CoreError::PlanStale { detail: StaleReason::ToolGone });
        }
        // الحجز آخر خطوة: كل ما قبله فحوص لا تخلّف أثرًا، فرفضٌ مبكر لا يترك
        // ملفًا محجوزًا في مجلد المستخدم.
        self.claim_temp()?;
        Ok(())
    }
}

pub struct StoredPlan {
    /// معرّف **عام** غير سرّي، يربط قيود السجل: planned ← running ← النتيجة.
    ///
    /// ليس الرمز. الرمز قدرةٌ تُنفَّذ بها الخطة ولا يجوز أن يُكتب على القرص؛
    /// هذا معرّف مراسلة لا يفتح شيئًا.
    pub id: String,
    pub op_id: &'static str,
    pub inputs: Inputs,
    pub command: PlannedCommand,
    pub session: SessionId,
    created: Instant,
    /// ساعة الحائط بجانب الساعة الأحادية.
    ///
    /// `Instant` على macOS يتوقّف مع نوم الجهاز، فخطةٌ عمرها ثلاث دقائق
    /// تُغلق حاسوبها لليلة كاملة ثم تُفتح، فيجد الرمز نفسه قابلًا للاستهلاك —
    /// وهو بالضبط ما وُضع `TTL` ليمنعه، وبصمةُ نظام الملفات قد لا تكون آنذاك
    /// إلا صورةً قديمة. ولا يُستغنى عن الأحادية بها: ساعة الحائط تُعدَّل
    /// وتُزامَن، والأحادية لا تُعدَّل.
    created_wall: SystemTime,
    preconditions: Preconditions,
}

impl StoredPlan {
    /// يُعاد التحقّق قبل التشغيل مباشرة. الفجوة بين التخطيط والضغط على «نفّذ»
    /// فجوة حقيقية: قد يُحذف المصدر أو يُنشأ ملف بالاسم النهائي خلالها.
    pub fn verify_still_valid(&self) -> Result<()> {
        self.preconditions.verify()
    }
}

pub struct PlanStore {
    plans: HashMap<PlanToken, StoredPlan>,
    sessions: Vec<SessionId>,
    ttl: Duration,
}

impl Default for PlanStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanStore {
    pub fn new() -> Self {
        Self::with_ttl(PLAN_TTL)
    }

    /// عمر الخطة قابل للضبط كي تتمكّن الاختبارات من إثبات انتهاء الصلاحية
    /// دون انتظار ثلاث دقائق، ودون فتح باب خلفي في نوع الإنتاج.
    pub fn with_ttl(ttl: Duration) -> Self {
        Self { plans: HashMap::new(), sessions: Vec::new(), ttl }
    }

    pub fn register_session(&mut self) -> Result<SessionId> {
        self.sweep();
        if self.sessions.len() >= MAX_SESSIONS {
            return Err(CoreError::PlanLimitReached);
        }
        let id = SessionId::generate();
        self.sessions.push(id.clone());
        Ok(id)
    }

    pub fn insert(
        &mut self,
        session: &SessionId,
        op_id: &'static str,
        inputs: Inputs,
        command: PlannedCommand,
    ) -> Result<(PlanToken, String)> {
        self.sweep();

        // خطة الجلسة الأقدم تُطرح لتفسح للأحدث.
        //
        // «نَفِّذ» يعيد التخطيط عند كل تغيير في النموذج كي تبقى المعاينة صادقة،
        // فكتابة اسم من عشرين حرفًا تحجز خططًا أكثر من السقف. الرفض هنا كان
        // سيعني أن نموذجًا يعمل يتوقّف عن العمل كلما أطال المستخدم الكتابة.
        //
        // والسقف يبقى محفوظًا: العدد الحيّ لا يتجاوز `MAX_PLANS_PER_SESSION`
        // أبدًا، والطرح يُفشل الرمز المطروح — أي يفشل مغلقًا.
        while self.plans.values().filter(|p| p.session == *session).count() >= MAX_PLANS_PER_SESSION
        {
            let Some(oldest) = self
                .plans
                .iter()
                .filter(|(_, p)| p.session == *session)
                .min_by_key(|(_, p)| p.created)
                .map(|(token, _)| token.clone())
            else {
                break;
            };
            self.plans.remove(&oldest);
        }

        // السقف الإجمالي يبقى رفضًا: تجاوزه يعني جلسات كثيرة لا مستخدمًا يكتب.
        if self.plans.len() >= MAX_PLANS_TOTAL {
            return Err(CoreError::PlanLimitReached);
        }

        let preconditions = Preconditions::capture(&inputs, &command)?;
        let token = PlanToken::generate();
        let id = random_suffix();
        self.plans.insert(
            token.clone(),
            StoredPlan {
                id: id.clone(),
                op_id,
                inputs,
                command,
                session: session.clone(),
                created: Instant::now(),
                created_wall: SystemTime::now(),
                preconditions,
            },
        );
        Ok((token, id))
    }

    /// ينتزع الخطة. النجاح مرّة واحدة فقط — وهذا ما يجعل الرمز أحادي الاستخدام.
    pub fn take(&mut self, token: &PlanToken, session: &SessionId) -> Result<StoredPlan> {
        self.sweep();
        match self.plans.remove(token) {
            // خطة جلسة أخرى تُعامل كأنها غير موجودة: لا نكشف وجودها.
            Some(p) if p.session != *session => {
                self.plans.insert(token.clone(), p);
                Err(CoreError::PlanNotFound)
            }
            Some(p) => Ok(p),
            None => Err(CoreError::PlanNotFound),
        }
    }

    pub fn discard(&mut self, token: &PlanToken) {
        self.plans.remove(token);
    }

    pub fn len(&self) -> usize {
        self.plans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plans.is_empty()
    }

    fn sweep(&mut self) {
        let now = Instant::now();
        let wall_now = SystemTime::now();
        let ttl = self.ttl;
        // تنتهي الصلاحية إن قالت **أيّ** من الساعتين ذلك — أي نأخذ الأقصر:
        // الأحادية تحرس ضدّ تقديم ساعة النظام، وساعة الحائط تحرس ضدّ النوم
        // الذي يوقف الأحادية. وقفزةُ الساعة إلى الخلف تُقرأ صفرًا، فلا تُسقط
        // خطةً قبل أوانها ولا تصير هي نفسها بابًا لإطالة العمر.
        self.plans.retain(|_, p| {
            now.duration_since(p.created) < ttl
                && wall_now.duration_since(p.created_wall).unwrap_or_default() < ttl
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{Artifact, PlannedCommand};
    use crate::value::Value;

    /// يزحزح `mtime` بلا انتظار ثانيةٍ كاملة.
    ///
    /// دقّة `mtime` في `stat` ثانية واحدة، فاختبارٌ يكتب ملفًا ثم يقارن يمرّ
    /// بالصدفة داخل نفس الثانية. الاختبارات هنا تثبّت قاعدةً لا تقيس زمنًا،
    /// فالإزاحة الصريحة أصدق من النوم وأسرع منه.
    fn shift_mtime(p: &Path, seconds: i64) {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        let m = std::fs::symlink_metadata(p).unwrap();
        let c = CString::new(p.as_os_str().as_bytes()).unwrap();
        let t = libc::timeval { tv_sec: m.mtime() as libc::time_t + seconds, tv_usec: 0 };
        let times = [t, t];
        assert_eq!(unsafe { libc::utimes(c.as_ptr(), times.as_ptr()) }, 0, "utimes must succeed");
    }

    fn dummy_command() -> PlannedCommand {
        PlannedCommand {
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

    #[test]
    fn tokens_are_256_bit_and_never_repeat() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            let t = PlanToken::generate();
            assert_eq!(t.as_str().len(), 64, "expected 32 bytes hex-encoded");
            assert!(seen.insert(t), "token collision");
        }
    }

    #[test]
    fn a_plan_can_be_taken_exactly_once() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let (t, _) = store.insert(&s, "x", Inputs::default(), dummy_command()).unwrap();

        assert!(store.take(&t, &s).is_ok());
        assert!(matches!(store.take(&t, &s), Err(CoreError::PlanNotFound)));
    }

    #[test]
    fn a_forged_token_is_rejected() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let forged = PlanToken("00".repeat(32));
        assert!(matches!(store.take(&forged, &s), Err(CoreError::PlanNotFound)));
    }

    #[test]
    fn an_expired_plan_is_rejected_and_dropped() {
        let mut store = PlanStore::with_ttl(Duration::from_millis(10));
        let s = store.register_session().unwrap();
        let (t, _) = store.insert(&s, "x", Inputs::default(), dummy_command()).unwrap();
        std::thread::sleep(Duration::from_millis(30));
        assert!(matches!(store.take(&t, &s), Err(CoreError::PlanNotFound)));
        assert!(store.is_empty(), "expired plans must not linger");
    }

    #[test]
    fn a_plan_cannot_be_taken_by_another_session() {
        let mut store = PlanStore::new();
        let a = store.register_session().unwrap();
        let b = store.register_session().unwrap();
        let (t, _) = store.insert(&a, "x", Inputs::default(), dummy_command()).unwrap();

        assert!(matches!(store.take(&t, &b), Err(CoreError::PlanNotFound)));
        // ولا يُستهلك بمحاولة الجلسة الأخرى.
        assert!(store.take(&t, &a).is_ok());
    }

    #[test]
    fn the_live_count_per_session_never_exceeds_the_cap() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        // نموذجٌ يُعاد تخطيطه عند كل ضغطة مفتاح: مئة خطة، والسقف ستّ عشرة.
        for _ in 0..100 {
            store.insert(&s, "x", Inputs::default(), dummy_command()).unwrap();
            assert!(store.len() <= MAX_PLANS_PER_SESSION, "the cap must always hold");
        }
        assert_eq!(store.len(), MAX_PLANS_PER_SESSION);
    }

    #[test]
    fn the_oldest_plan_is_the_one_evicted_and_it_fails_closed() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();

        let (first, _) = store.insert(&s, "x", Inputs::default(), dummy_command()).unwrap();
        // خطط `created` تُقاس بـ Instant، ودقّته دون المللي ثانية على macOS،
        // لكن ننام قليلًا كي يكون الترتيب لا لبس فيه.
        std::thread::sleep(Duration::from_millis(2));
        let mut newest = first.clone();
        for _ in 0..MAX_PLANS_PER_SESSION {
            newest = store.insert(&s, "x", Inputs::default(), dummy_command()).unwrap().0;
        }

        assert!(
            matches!(store.take(&first, &s), Err(CoreError::PlanNotFound)),
            "the evicted token must fail, not execute something stale"
        );
        assert!(store.take(&newest, &s).is_ok(), "the newest plan must still be redeemable");
    }

    #[test]
    fn one_session_cannot_evict_another_sessions_plans() {
        let mut store = PlanStore::new();
        let a = store.register_session().unwrap();
        let b = store.register_session().unwrap();

        let (kept, _) = store.insert(&a, "x", Inputs::default(), dummy_command()).unwrap();
        for _ in 0..MAX_PLANS_PER_SESSION * 2 {
            store.insert(&b, "x", Inputs::default(), dummy_command()).unwrap();
        }

        assert!(store.take(&kept, &a).is_ok(), "eviction is per session, not global");
    }

    #[test]
    fn session_count_is_capped() {
        let mut store = PlanStore::new();
        for _ in 0..MAX_SESSIONS {
            store.register_session().unwrap();
        }
        assert!(matches!(store.register_session(), Err(CoreError::PlanLimitReached)));
    }

    #[test]
    fn a_plan_goes_stale_when_its_source_is_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source");
        std::fs::create_dir(&src).unwrap();

        let pre = Preconditions {
            inputs: vec![(src.clone(), PathFingerprint::capture(&src).unwrap())],
            final_path_must_be_absent: None,
            temp_to_claim: None,
            destination_must_be_writable: None,
            program: PathBuf::from("/bin/echo"),
        };
        assert!(pre.verify().is_ok());

        // نفس الاسم، عقدة مختلفة — وهذا ما يكشفه dev+ino ولا يكشفه الاسم.
        std::fs::remove_dir(&src).unwrap();
        std::fs::create_dir(&src).unwrap();
        assert!(matches!(
            pre.verify(),
            Err(CoreError::PlanStale { detail: StaleReason::SourceReplaced })
        ));
    }

    #[test]
    fn a_plan_goes_stale_when_the_final_name_appears_meanwhile() {
        let dir = tempfile::tempdir().unwrap();
        let final_path = dir.path().join("out.zip");

        let pre = Preconditions {
            inputs: vec![],
            final_path_must_be_absent: Some(final_path.clone()),
            temp_to_claim: None,
            destination_must_be_writable: None,
            program: PathBuf::from("/bin/echo"),
        };
        assert!(pre.verify().is_ok());

        std::fs::write(&final_path, b"someone got here first").unwrap();
        assert!(matches!(
            pre.verify(),
            Err(CoreError::PlanStale { detail: StaleReason::FinalPathAppeared })
        ));
    }

    #[test]
    fn the_public_plan_id_is_unique_and_is_not_the_token() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let (t1, id1) = store.insert(&s, "x", Inputs::default(), dummy_command()).unwrap();
        let (_, id2) = store.insert(&s, "x", Inputs::default(), dummy_command()).unwrap();

        assert_ne!(id1, id2, "each plan needs its own correlation id");
        assert_ne!(id1, t1.as_str(), "the journal id must never be the capability token");
        assert!(!t1.as_str().contains(&id1), "the id must not leak part of the token");
    }

    #[test]
    fn a_plan_goes_stale_when_its_tool_disappears() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let fake_tool = dir.path().join("tool");
        std::fs::write(&fake_tool, b"#!/bin/sh\n").unwrap();
        // أداةٌ بلا بتّ تنفيذ ليست أداة: `Tool::resolve` ترفضها، وإعادةُ
        // التحقّق صارت ترفضها كذلك.
        std::fs::set_permissions(&fake_tool, std::fs::Permissions::from_mode(0o755)).unwrap();

        let pre = Preconditions {
            inputs: vec![],
            final_path_must_be_absent: None,
            temp_to_claim: None,
            destination_must_be_writable: None,
            program: fake_tool.clone(),
        };
        assert!(pre.verify().is_ok());

        std::fs::remove_file(&fake_tool).unwrap();
        assert!(matches!(
            pre.verify(),
            Err(CoreError::PlanStale { detail: StaleReason::ToolGone })
        ));
    }

    #[test]
    fn a_plan_goes_stale_when_the_destination_stops_being_writable() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("dest");
        std::fs::create_dir(&dest).unwrap();

        let pre = Preconditions {
            inputs: vec![],
            final_path_must_be_absent: None,
            temp_to_claim: None,
            destination_must_be_writable: Some(dest.clone()),
            program: PathBuf::from("/bin/echo"),
        };
        assert!(pre.verify().is_ok(), "a writable destination must pass");

        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o500)).unwrap();
        let r = pre.verify();
        std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o700)).unwrap();

        assert!(
            matches!(r, Err(CoreError::PlanStale { detail: StaleReason::DestinationNotWritable })),
            "the check at plan time is not a promise for execute time: {r:?}"
        );
    }

    #[test]
    fn a_plan_goes_stale_when_the_destination_is_removed() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("dest");
        std::fs::create_dir(&dest).unwrap();

        let pre = Preconditions {
            inputs: vec![],
            final_path_must_be_absent: None,
            temp_to_claim: None,
            destination_must_be_writable: Some(dest.clone()),
            program: PathBuf::from("/bin/echo"),
        };
        std::fs::remove_dir(&dest).unwrap();
        assert!(matches!(
            pre.verify(),
            Err(CoreError::PlanStale { detail: StaleReason::DestinationGone })
        ));
    }

    #[test]
    fn re_verifying_leaves_no_write_probe_in_the_destination() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("dest");
        std::fs::create_dir(&dest).unwrap();

        let pre = Preconditions {
            inputs: vec![],
            final_path_must_be_absent: None,
            temp_to_claim: None,
            destination_must_be_writable: Some(dest.clone()),
            program: PathBuf::from("/bin/echo"),
        };
        for _ in 0..10 {
            pre.verify().unwrap();
        }
        assert_eq!(std::fs::read_dir(&dest).unwrap().count(), 0, "probes must not accumulate");
    }

    // ── المؤقّت: يُحجز لا يُفحص ────────────────────────────────────────

    fn artifact_preconditions(dir: &Path, name: &str) -> Preconditions {
        let final_path = dir.join(name);
        let temp = crate::atomic::temp_path_for(&final_path).unwrap();
        Preconditions {
            inputs: vec![],
            final_path_must_be_absent: Some(final_path),
            temp_to_claim: Some((temp, ArtifactKind::File)),
            destination_must_be_writable: None,
            program: PathBuf::from("/bin/echo"),
        }
    }

    #[test]
    fn a_symlink_planted_at_the_temporary_path_stops_the_run() {
        // الهجوم كاملًا: الاسم المؤقّت منشور في السجل وقت التخطيط، فيُقرأ
        // ويُزرع مكانه رابط إلى ملف للمستخدم. `ditto` كانت ستتبعه وتكتب فوق
        // الضحية، ثم يُبلَّغ التشغيل نجاحًا.
        let dir = tempfile::tempdir().unwrap();
        let victim = dir.path().join("مستند مهم.txt");
        std::fs::write(&victim, b"PRECIOUS USER DATA").unwrap();

        let pre = artifact_preconditions(dir.path(), "نسخة.zip");
        std::os::unix::fs::symlink(&victim, &pre.temp_to_claim.as_ref().unwrap().0).unwrap();

        let r = pre.verify();
        assert!(matches!(r, Err(CoreError::PlanStale { .. })), "got {r:?}");
        assert_eq!(
            std::fs::read(&victim).unwrap(),
            b"PRECIOUS USER DATA",
            "and the victim must not have been touched"
        );
    }

    #[test]
    fn a_regular_file_squatting_the_temporary_path_stops_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let pre = artifact_preconditions(dir.path(), "نسخة.zip");
        std::fs::write(&pre.temp_to_claim.as_ref().unwrap().0, b"squatter").unwrap();

        assert!(matches!(pre.verify(), Err(CoreError::PlanStale { .. })));
    }

    #[test]
    fn verification_claims_the_temporary_path_so_no_one_can_get_there_after_it() {
        // الفحص وحده يترك نافذة بين «غير موجود» وفتح الأداة له. الحجز يغلقها:
        // بعد تحقّق ناجح يصير الاسم ملكًا لنا، وتحقّقٌ ثانٍ يفشل.
        let dir = tempfile::tempdir().unwrap();
        let pre = artifact_preconditions(dir.path(), "نسخة.zip");
        let temp = pre.temp_to_claim.clone().unwrap().0;

        pre.verify().expect("a clean temp path must verify");
        assert!(temp.exists(), "the temp path must be claimed, not merely inspected");

        assert!(
            matches!(pre.verify(), Err(CoreError::PlanStale { .. })),
            "a second claim on the same name must fail closed"
        );
    }

    #[test]
    fn a_refusal_before_the_claim_leaves_the_destination_clean() {
        // ترتيب الفحوص مقصود: الحجز آخرها، فرفضٌ لسببٍ آخر لا يخلّف ملفًا.
        let dir = tempfile::tempdir().unwrap();
        let mut pre = artifact_preconditions(dir.path(), "نسخة.zip");
        pre.program = dir.path().join("لا-وجود-لها");

        assert!(matches!(
            pre.verify(),
            Err(CoreError::PlanStale { detail: StaleReason::ToolGone })
        ));
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0, "nothing may be left behind");
    }

    #[test]
    fn the_temporary_path_is_pinned_from_the_planned_artifact_not_invented_later() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let final_path = dir.path().join("out.zip");
        let temp = crate::atomic::temp_path_for(&final_path).unwrap();

        let mut command = dummy_command();
        command.artifact = Some(Artifact::file(temp.clone(), final_path));
        let (t, _) = store.insert(&s, "x", Inputs::default(), command).unwrap();

        let stored = store.take(&t, &s).unwrap();
        assert_eq!(
            stored.preconditions.temp_to_claim.as_ref().map(|(p, _)| p.as_path()),
            Some(temp.as_path())
        );
    }

    // ── البصمة: هويّة لا محتوى ────────────────────────────────────────

    #[test]
    fn unrelated_churn_in_a_fingerprinted_directory_does_not_invalidate_the_plan() {
        // لقطة شاشة تهبط على سطح المكتب، أو فحصُ الكتابة الخاص بخطةٍ ثانية
        // إلى نفس المجلد. كلاهما كان يُبطل خطةً صالحة ويسمّي السبب «المصدر».
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("dest");
        std::fs::create_dir(&dest).unwrap();

        let pre = Preconditions {
            inputs: vec![(dest.clone(), PathFingerprint::capture(&dest).unwrap())],
            final_path_must_be_absent: None,
            temp_to_claim: None,
            destination_must_be_writable: None,
            program: PathBuf::from("/bin/echo"),
        };
        assert!(pre.verify().is_ok());

        std::fs::write(dest.join("لقطة.png"), b"x").unwrap();
        shift_mtime(&dest, 3600);

        assert!(
            pre.verify().is_ok(),
            "a directory's mtime is neither a content guarantee nor ours to depend on"
        );
    }

    #[test]
    fn replacing_a_fingerprinted_directory_still_invalidates_the_plan() {
        // ما أسقطناه هو `mtime` وحده. الهويّة — وهي ما يقوم عليه الأمان —
        // ما زالت مرصودة.
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("source");
        std::fs::create_dir(&src).unwrap();

        let pre = Preconditions {
            inputs: vec![(src.clone(), PathFingerprint::capture(&src).unwrap())],
            final_path_must_be_absent: None,
            temp_to_claim: None,
            destination_must_be_writable: None,
            program: PathBuf::from("/bin/echo"),
        };
        std::fs::remove_dir(&src).unwrap();
        std::fs::create_dir(&src).unwrap();

        assert!(matches!(
            pre.verify(),
            Err(CoreError::PlanStale { detail: StaleReason::SourceReplaced })
        ));
    }

    #[test]
    fn a_file_input_still_reports_a_content_change() {
        // `mtime` يبقى للملفات العادية، حيث يعني ما يقوله فعلًا.
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("مستند.txt");
        std::fs::write(&file, b"before").unwrap();

        let pre = Preconditions {
            inputs: vec![(file.clone(), PathFingerprint::capture(&file).unwrap())],
            final_path_must_be_absent: None,
            temp_to_claim: None,
            destination_must_be_writable: None,
            program: PathBuf::from("/bin/echo"),
        };
        assert!(pre.verify().is_ok());

        std::fs::write(&file, b"after").unwrap();
        shift_mtime(&file, 3600);
        assert!(matches!(
            pre.verify(),
            Err(CoreError::PlanStale { detail: StaleReason::SourceReplaced })
        ));
    }

    #[test]
    fn a_plan_whose_input_cannot_be_fingerprinted_is_never_stored() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let ghost = PathBuf::from("/nonexistent-naffith-input").join(random_suffix());
        let inputs = Inputs::from_pairs(vec![("source", Value::Dir(ghost))]);

        let r = store.insert(&s, "x", inputs, dummy_command());
        assert!(matches!(r, Err(CoreError::PathMissing)), "got {r:?}");
        assert!(store.is_empty(), "a plan that cannot be pinned must not be stored");
    }

    // ── العمر: الساعتان معًا ──────────────────────────────────────────

    #[test]
    fn a_plan_expires_on_the_wall_clock_too_so_sleep_cannot_extend_it() {
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let (t, _) = store.insert(&s, "x", Inputs::default(), dummy_command()).unwrap();

        // نوم الجهاز يوقف `Instant` ولا يوقف ساعة الحائط. نحاكيه بإرجاع طابع
        // الحائط وحده، تاركين الساعة الأحادية طازجة كما تكون بعد الاستيقاظ.
        for p in store.plans.values_mut() {
            p.created_wall -= Duration::from_secs(3600);
        }

        assert!(matches!(store.take(&t, &s), Err(CoreError::PlanNotFound)));
        assert!(store.is_empty(), "a plan that slept past its TTL must not linger");
    }

    #[test]
    fn a_backwards_wall_clock_does_not_expire_a_fresh_plan() {
        // مزامنة NTP تُرجع الساعة. الفرق السالب يُقرأ صفرًا، فالخطة تبقى
        // محكومة بالساعة الأحادية وحدها.
        let mut store = PlanStore::new();
        let s = store.register_session().unwrap();
        let (t, _) = store.insert(&s, "x", Inputs::default(), dummy_command()).unwrap();

        for p in store.plans.values_mut() {
            p.created_wall += Duration::from_secs(3600);
        }

        assert!(store.take(&t, &s).is_ok(), "a clock jump backwards must not kill a live plan");
    }

    #[test]
    fn a_tool_that_lost_its_execute_bit_is_a_stale_plan_not_a_spawn_failure() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let tool = dir.path().join("tool");
        std::fs::write(&tool, b"#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o755)).unwrap();

        let pre = Preconditions {
            inputs: vec![],
            final_path_must_be_absent: None,
            temp_to_claim: None,
            destination_must_be_writable: None,
            program: tool.clone(),
        };
        assert!(pre.verify().is_ok());

        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(matches!(
            pre.verify(),
            Err(CoreError::PlanStale { detail: StaleReason::ToolGone })
        ));
    }

    #[test]
    fn a_dangling_symlink_at_the_final_name_still_counts_as_a_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let final_path = dir.path().join("out.zip");
        std::os::unix::fs::symlink(dir.path().join("nowhere"), &final_path).unwrap();

        let pre = Preconditions {
            inputs: vec![],
            final_path_must_be_absent: Some(final_path.clone()),
            temp_to_claim: None,
            destination_must_be_writable: None,
            program: PathBuf::from("/bin/echo"),
        };
        assert!(
            matches!(pre.verify(), Err(CoreError::PlanStale { .. })),
            "a dangling symlink would still be clobbered by a rename"
        );
    }
}
