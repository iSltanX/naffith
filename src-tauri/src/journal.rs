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
//! ## ما لا يُسجَّل
//!
//! * **رمز الخطة** — قدرة تُنفَّذ بها العملية. يُستبدل بمعرّف عام لا يفتح شيئًا.
//! * **‏stdout/stderr** — قد يحمل اسم كل ملف داخل الشجرة. السجل يقيّد ماذا
//!   شُغّل ونتيجته، لا ما طبعته الأداة.
//! * **البيئة** — الطفل يعمل ببيئة نظيفة أصلًا.
//!
//! ويبقى البرنامج ووسائطه، وفيهما مسارات المستخدم: هي أقلّ ما لا يصير السجلّ
//! بدونه مراجعةً.

use crate::executor::Outcome;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// المعرّف العام الذي يربط قيود التشغيل الواحد. ليس رمز الخطة.
    pub id: String,
    pub op_id: String,
    /// وقت الحدث بالثواني منذ حقبة يونكس، UTC. الواجهة تعرضه بتوقيت المستخدم.
    pub at: u64,
    /// زمن التنفيذ. في القيد النهائي وحده.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    pub program: String,
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(flatten)]
    pub state: State,
}

impl Entry {
    pub fn new(id: impl Into<String>, op_id: impl Into<String>, state: State) -> Self {
        Entry {
            id: id.into(),
            op_id: op_id.into(),
            at: now_secs(),
            duration_ms: None,
            program: String::new(),
            args: Vec::new(),
            cwd: None,
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
}

pub struct Journal {
    path: Option<PathBuf>,
    /// يُسلسل الكتابة داخل هذه العملية. `flock` يتكفّل بما بين العمليات.
    write_lock: Mutex<()>,
    recent: Mutex<Vec<Entry>>,
    max_recent: usize,
}

impl Journal {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self { path, write_lock: Mutex::new(()), recent: Mutex::new(Vec::new()), max_recent: 200 }
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
        let overflow = recent.len().saturating_sub(self.max_recent);
        if overflow > 0 {
            recent.drain(0..overflow);
        }
    }

    pub fn recent(&self) -> Vec<Entry> {
        self.recent.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// المسار النهائي الذي أنتجه تشغيلٌ ناجح، إن وُجد.
    ///
    /// هذا هو المصدر الوحيد لما يمكن «إظهاره في Finder»: الواجهة تعطي معرّفًا،
    /// والنواة تُخرج المسار من سجلّها. لا مسار يعبر الحدّ في الاتجاه الآخر.
    pub fn produced_for(&self, id: &str) -> Option<String> {
        let recent = self.recent.lock().unwrap_or_else(|e| e.into_inner());
        recent.iter().rev().find_map(|e| match &e.state {
            State::Succeeded { produced: Some(p) } if e.id == id => Some(p.clone()),
            _ => None,
        })
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
    fn acquire(file: &std::fs::File) -> std::io::Result<Self> {
        use std::os::unix::io::AsRawFd;
        let fd = file.as_raw_fd();
        // SAFETY: الواصف مملوك لملفٍ حيّ ويبقى صالحًا حتى تحرير القفل.
        if unsafe { libc::flock(fd, libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(FileLock(fd))
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
pub fn read_all(path: &Path) -> std::io::Result<ReadResult> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(ReadResult::default()),
        Err(e) => return Err(e),
    };
    let mut out = ReadResult::default();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Entry>(line) {
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
    fn reading_a_journal_that_does_not_exist_yet_is_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let read = read_all(&dir.path().join("nothing.jsonl")).unwrap();
        assert!(read.entries.is_empty());
        assert_eq!(read.damaged, 0);
    }
}
