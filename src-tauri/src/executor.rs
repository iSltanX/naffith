//! تشغيل الأوامر.
//!
//! ثلاثة قرارات تحكم هذا الملف:
//!
//! 1. **لا صدفة.** `program` + `args` مباشرة إلى `execve`. لا `sh -c`، ولا
//!    تركيب نصّي، ولا `PATH`. المحارف الخاصة في اسم ملف تبقى محارف.
//!
//! 2. **مجموعة عمليات خاصة.** الطفل يُنقل إلى جلسة جديدة بـ `setsid`، فيصير
//!    قائد مجموعته. الإلغاء يرسل الإشارة إلى المجموعة كلها، فلا يبقى حفيد
//!    يكتب في الملف المؤقّت بعد أن ظنّ المستخدم أنه ألغى.
//!
//! 3. **لا تقدّم مُختلَق.** `ditto` لا تعطي نسبة موثوقة، فنبثّ ما تكتبه فعلًا
//!    ونعرض حالة عمل. رقم مخترع أسوأ من غياب الرقم.

use crate::atomic::ArtifactGuard;
use crate::spec::PlannedCommand;
use serde::Serialize;
use std::process::{ExitStatus, Stdio};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{mpsc, oneshot};

/// حدود البثّ.
///
/// أداة تكتب مليون سطر لا يجوز أن تُغرق الذاكرة ولا الواجهة. الحدّان معلَنان
/// وقابلان للاختبار، وتجاوزهما **يُعلَن** ولا يُخفى: عند بلوغ سقف الأسطر
/// يُرسل سطرٌ أخير من نوع `Truncated` يحمل عدد ما أُسقط، فيعرف المستخدم أن
/// ما يراه ليس كل الخرج. الصمت هنا كذب.
pub const MAX_OUTPUT_LINES: usize = 5_000;

/// أقصى طول سطر واحد بالبايتات. سطر أطول يُقصّ ويُعلَّم بعلامة القصّ.
pub const MAX_LINE_BYTES: usize = 4_000;

/// أقصى ما يُحجز في الذاكرة لسطر واحد قبل إهمال بقيّته.
///
/// بايت واحد فوق `MAX_LINE_BYTES` بالضبط: يكفي كي يعلم `clip` أن السطر تجاوز
/// فيضع علامة القصّ، ولا يزيد الحجز على السقف المعلَن. القارئ السابق كان
/// يراكم حتى الفاصل مهما بلغ، فخرجٌ بلا أسطر — شريط تقدّم بـ `\r` مثلًا —
/// كان يُحجز كاملًا في الذاكرة قبل أن يصل إلى القصّ أصلًا.
const LINE_BUFFER_BYTES: usize = MAX_LINE_BYTES + 1;

/// العلامة التي تُلحق بسطر قُصّ.
pub const TRUNCATION_MARK: &str = " …";

/// المهلة بين الطلب اللطيف بالإنهاء والقتل القسري.
const GRACE: Duration = Duration::from_millis(1500);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "stream", content = "line")]
pub enum OutputLine {
    Stdout(String),
    Stderr(String),
    /// بُلغ سقف الأسطر وتوقّف البثّ. `dropped` عدد الأسطر التي لم تُرسل.
    Truncated {
        dropped: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum Outcome {
    /// خرجت الأداة بصفر، ورُقّي الناتج إن وُجد.
    Success { produced: Option<String> },
    /// خرجت بغير صفر. لا ناتج نهائي.
    Failed { code: Option<i32> },
    /// أنهتها إشارة (‏SIGSEGV مثلًا) دون أن نطلب نحن الإلغاء.
    Signalled { signal: Option<i32> },
    /// ألغاها المستخدم. المؤقّت مُنظَّف.
    Cancelled,
    /// فشلٌ قبل أن تبدأ الأداة، أو عند ترقية الناتج.
    Error { key: &'static str },
}

impl Outcome {
    pub fn is_success(&self) -> bool {
        matches!(self, Outcome::Success { .. })
    }
}

pub struct Handle {
    pub cancel: oneshot::Sender<()>,
}

/// يشغّل أمرًا مخطَّطًا ويبثّ خرجه سطرًا سطرًا.
///
/// يعيد `Outcome` واحدًا فقط، وينظّف الناتج المؤقّت في كل مسار خروج غير ناجح.
pub async fn run(
    command: PlannedCommand,
    output: mpsc::Sender<OutputLine>,
    mut cancel: oneshot::Receiver<()>,
) -> Outcome {
    // الحارس يحمل المؤقّت. أي خروج من هذه الدالة دون `commit` يحذفه — بما في
    // ذلك الذعر وإسقاط المستقبل.
    let guard = command.artifact.as_ref().map(ArtifactGuard::new);

    // توجيه الخرج القياسي إلى ملف الناتج، لأدواتٍ لا تعرف `-o`.
    //
    // الفتح هنا لا في `plans`: الملف محجوزٌ حصريًا قبل الوصول إلى هذه الدالة،
    // فهذا فتحٌ لما نملكه. و`custom_flags(O_NOFOLLOW)` رغم ذلك، لأن بين الحجز
    // والفتح لحظةً — والحارس الذي لا يكلّف شيئًا يُوضع.
    let redirect = match &command.stdout_to {
        None => None,
        Some(path) => {
            use std::os::unix::fs::OpenOptionsExt;
            match std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .custom_flags(libc::O_NOFOLLOW)
                .open(path)
            {
                Ok(f) => Some(f),
                Err(_) => {
                    if let Some(g) = guard {
                        g.abort();
                    }
                    return Outcome::Error { key: "err.redirect" };
                }
            }
        }
    };

    let mut cmd = tokio::process::Command::new(&command.program);
    cmd.args(&command.args)
        .stdin(Stdio::null())
        .stdout(match redirect {
            Some(f) => Stdio::from(f),
            None => Stdio::piped(),
        })
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    if let Some(cwd) = &command.cwd {
        cmd.current_dir(cwd);
    }

    // بيئة نظيفة: لا نورّث بيئة المستخدم إلى أداة نظام. لا PATH يعني أنه لا
    // سبيل لاختطاف أداة، وقد حُلّ مسار البرنامج مطلقًا قبل الوصول إلى هنا.
    cmd.env_clear();
    if let Some(home) = std::env::var_os("HOME") {
        cmd.env("HOME", home);
    }
    cmd.env("LC_ALL", "C.UTF-8");

    // ينقل الطفل إلى جلسة/مجموعة جديدة يقودها، كي يطال الإلغاءُ ذرّيتَه.
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => {
            if let Some(g) = guard {
                g.abort();
            }
            return Outcome::Error { key: "err.spawn" };
        }
    };

    let pid = match child.id() {
        Some(p) => p as i32,
        None => {
            if let Some(g) = guard {
                g.abort();
            }
            return Outcome::Error { key: "err.spawn" };
        }
    };

    // البثّ: كل مجرى في مهمّة، فلا يحجب امتلاءُ أحد الأنبوبين الآخر.
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // عدّاد مشترك بين المجريين: السقف على مجموع الخرج لا على كل مجرى وحده،
    // وإلا صار السقف الفعلي ضِعف المعلَن.
    let emitted = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicUsize::new(0));

    let out_tx = output.clone();
    let out_emitted = emitted.clone();
    let out_dropped = dropped.clone();
    let stdout_task = tokio::spawn(async move {
        pump(stdout, out_tx, out_emitted, out_dropped, OutputLine::Stdout).await;
    });

    let err_tx = output.clone();
    let err_emitted = emitted.clone();
    let err_dropped = dropped.clone();
    let stderr_task = tokio::spawn(async move {
        pump(stderr, err_tx, err_emitted, err_dropped, OutputLine::Stderr).await;
    });

    let cancelled;
    let status = tokio::select! {
        status = child.wait() => {
            cancelled = false;
            status
        }
        _ = &mut cancel => {
            cancelled = true;
            terminate_group(pid, &mut child).await
        }
    };

    let _ = stdout_task.await;
    let _ = stderr_task.await;

    let lost = dropped.load(std::sync::atomic::Ordering::Relaxed);
    if lost > 0 {
        let _ = output.send(OutputLine::Truncated { dropped: lost }).await;
    }

    if cancelled {
        if let Some(g) = guard {
            g.abort();
        }
        return Outcome::Cancelled;
    }

    let status = match status {
        Ok(s) => s,
        Err(_) => {
            if let Some(g) = guard {
                g.abort();
            }
            return Outcome::Error { key: "err.wait" };
        }
    };

    if !status.success() {
        if let Some(g) = guard {
            g.abort();
        }
        use std::os::unix::process::ExitStatusExt;
        return match status.signal() {
            Some(sig) => Outcome::Signalled { signal: Some(sig) },
            None => Outcome::Failed { code: status.code() },
        };
    }

    // نجاح: الآن فقط يُرقّى الناتج إلى اسمه النهائي.
    match guard {
        // بلا ناتجٍ مؤقّت، يبقى ما أعلنته العملية موضعًا يستحقّ الفتح: مجلدٌ
        // نُقل إليه، أو مستودعٌ أُنشئ، أو شجرةٌ مُسحت. كان `None` دائمًا، فكان
        // زرّ الإظهار يفشل بعد تشغيلٍ نجح.
        None => Outcome::Success {
            produced: command.reveal_target.as_ref().map(|p| p.display().to_string()),
        },
        Some(g) => match g.commit() {
            Ok(path) => Outcome::Success { produced: Some(path.display().to_string()) },
            Err(crate::error::CoreError::DestinationExists) => {
                Outcome::Error { key: "err.dest.exists" }
            }
            Err(crate::error::CoreError::PathMissing) => Outcome::Error { key: "err.output.empty" },
            Err(_) => Outcome::Error { key: "err.commit" },
        },
    }
}

/// يقرأ مجرًى سطرًا سطرًا ضمن الحدود المعلَنة.
///
/// القراءة **بالبايتات لا بالنصّ**، ولهذا سببان كلاهما كلّف المستخدم شيئًا:
///
/// 1. خرج الأداة بايتات لا نصّ. اسم ملف ليس UTF-8 صالحًا مشروع على macOS،
///    و`ditto` تردّده كما هو حين تشتكي منه. القارئ النصّي `lines()` كان يعيد
///    `Err` عند أول بايت كهذا، والنمط `Ok(Some(_))` يعامل الخطأ معاملة نهاية
///    المجرى: تنتهي المهمّة، فيُسقَط طرف القراءة من الأنبوب، فتموت الأداة
///    بـ `SIGPIPE` ويُحذف أرشيفٌ كان قد اكتمل. بايتٌ واحد لا يجوز أن يكلّف
///    المستخدم عمله. الآن يصير المعطوب محرف استبدال ويُعرض.
///
/// 2. السقف يجب أن يسبق الحجز. `read_line` تراكم حتى الفاصل مهما بلغ،
///    و`clip` لا يعمل إلا بعد أن يصير السطر كلّه في الذاكرة. الحدّ هنا على
///    المخزَن نفسه، فما فوقه يُهمل ولا يُحجز.
///
/// والقاعدة الثابتة في الحالين: **لا نكفّ عن تفريغ الأنبوب**. الكفّ يجمّد
/// الأداة أو يقتلها بدل أن ينهيها.
async fn pump<R>(
    stream: Option<R>,
    tx: mpsc::Sender<OutputLine>,
    emitted: Arc<AtomicUsize>,
    dropped: Arc<AtomicUsize>,
    wrap: fn(String) -> OutputLine,
) where
    R: tokio::io::AsyncRead + Unpin,
{
    let Some(s) = stream else { return };
    let mut reader = BufReader::new(s);
    let mut line: Vec<u8> = Vec::new();
    // ذهب المستقبل: نواصل التفريغ ونعدّ الضائع، ولا نغلق الأنبوب.
    let mut sink = false;

    loop {
        let (consumed, complete) = {
            let chunk = match reader.fill_buf().await {
                Ok(c) => c,
                Err(_) => break,
            };
            if chunk.is_empty() {
                break;
            }
            match chunk.iter().position(|b| *b == b'\n') {
                Some(at) => {
                    append_bounded(&mut line, &chunk[..at]);
                    (at + 1, true)
                }
                None => {
                    let all = chunk.len();
                    append_bounded(&mut line, chunk);
                    (all, false)
                }
            }
        };
        reader.consume(consumed);
        if complete {
            deliver(&tx, &emitted, &dropped, wrap, &mut line, &mut sink).await;
        }
    }

    // آخر سطر بلا فاصل أسطر يُبثّ ولا يُبتلع: رسالة خطأ الأداة قد تكون هو.
    if !line.is_empty() {
        deliver(&tx, &emitted, &dropped, wrap, &mut line, &mut sink).await;
    }
}

/// يضمّ ما تسعه الميزانية من جزءٍ إلى السطر، ويُهمل ما فوقها بلا حجز.
fn append_bounded(line: &mut Vec<u8>, part: &[u8]) {
    let room = LINE_BUFFER_BYTES.saturating_sub(line.len());
    if room == 0 {
        return;
    }
    line.extend_from_slice(&part[..room.min(part.len())]);
}

/// يبثّ سطرًا مكتملًا ويُفرغ مخزنه، محاسبًا ما لم يُبثّ.
async fn deliver(
    tx: &mpsc::Sender<OutputLine>,
    emitted: &AtomicUsize,
    dropped: &AtomicUsize,
    wrap: fn(String) -> OutputLine,
    line: &mut Vec<u8>,
    sink: &mut bool,
) {
    use std::sync::atomic::Ordering::Relaxed;
    // `fetch_add` ثم مقارنة القيمة السابقة، لا `load` ثم `fetch_add`: العدّاد
    // يتقاسمه مجريان، وفحصان منفصلان يمرّان معًا عند ٤٩٩٩ فيتجاوز الخرجُ
    // السقفَ المعلَن. سقفٌ يُخرق بسطر هو سقف لا يُصدَّق.
    let previous = emitted.fetch_add(1, Relaxed);
    if *sink || previous >= MAX_OUTPUT_LINES {
        dropped.fetch_add(1, Relaxed);
        line.clear();
        return;
    }
    let text = clip(String::from_utf8_lossy(line).into_owned());
    line.clear();
    if tx.send(wrap(text)).await.is_err() {
        *sink = true;
    }
}

/// يقصّ سطرًا طويلًا على حدود محرف كامل.
///
/// القصّ على بايت عشوائي يشطر حرفًا عربيًا نصفين ويخرج نصًا معطوبًا، لذا
/// نتراجع إلى أقرب حدّ محرف صالح.
fn clip(mut line: String) -> String {
    if line.len() <= MAX_LINE_BYTES {
        return line;
    }
    let mut end = MAX_LINE_BYTES;
    while end > 0 && !line.is_char_boundary(end) {
        end -= 1;
    }
    line.truncate(end);
    line.push_str(TRUNCATION_MARK);
    line
}

/// ينهي مجموعة العمليات: طلب لطيف، ثم قتل قاطع.
///
/// الإشارة تُرسل إلى `-pid` أي إلى المجموعة كلها، لأن `setsid` جعلت الطفل
/// قائدها. إرسالها إلى الطفل وحده يترك أحفاده يعملون.
///
/// المهلة تُسابَق بخروج القائد ولا تُنتظر كاملة: `sleep` غير مشروط كان يكلّف
/// كل إلغاء ١٫٥ ثانية حتى حين تموت الأداة فورًا على `SIGTERM` — وطوال تلك
/// المدّة يبقى الأرشيف الجزئي على القرص، ولم يُبثّ `run://finished` بعد،
/// فتعرض الواجهة تشغيلًا «جاريًا» ألغاه المستخدم.
///
/// و`SIGKILL` تُرسل إلى المجموعة في الحالين — بعد خروج القائد أيضًا — لأن
/// حفيدًا يتجاهل `SIGTERM` كان سينجو لو ربطنا التصعيد بانقضاء المهلة وحدها.
/// أي أن السباق يوفّر الانتظار ولا يُضعف الضمان.
async fn terminate_group(
    pid: i32,
    child: &mut tokio::process::Child,
) -> std::io::Result<ExitStatus> {
    unsafe {
        libc::killpg(pid, libc::SIGTERM);
    }
    let status = tokio::select! {
        status = child.wait() => status,
        _ = tokio::time::sleep(GRACE) => {
            unsafe { libc::killpg(pid, libc::SIGKILL); }
            child.wait().await
        }
    };
    unsafe {
        libc::killpg(pid, libc::SIGKILL);
    }
    status
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::Artifact;
    use std::ffi::OsString;
    use std::path::PathBuf;

    fn echo(args: &[&str]) -> PlannedCommand {
        PlannedCommand {
            program: PathBuf::from("/bin/echo"),
            args: args.iter().map(OsString::from).collect(),
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: None,
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        }
    }

    async fn collect(cmd: PlannedCommand) -> (Outcome, Vec<OutputLine>) {
        let (tx, mut rx) = mpsc::channel(64);
        let (_cancel_tx, cancel_rx) = oneshot::channel();
        let task = tokio::spawn(run(cmd, tx, cancel_rx));
        let mut lines = Vec::new();
        while let Some(l) = rx.recv().await {
            lines.push(l);
        }
        (task.await.unwrap(), lines)
    }

    #[tokio::test]
    async fn output_is_streamed_and_exit_is_reported() {
        let (outcome, lines) = collect(echo(&["سطر أول"])).await;
        assert!(outcome.is_success());
        assert_eq!(lines, vec![OutputLine::Stdout("سطر أول".into())]);
    }

    #[tokio::test]
    async fn shell_metacharacters_are_passed_through_literally() {
        // لو مرّ هذا بصدفة لحُذف شيء. هنا هو نصّ يُطبع كما هو.
        let payload = "; rm -rf ~ && echo pwned $(whoami) `id`";
        let (outcome, lines) = collect(echo(&[payload])).await;
        assert!(outcome.is_success());
        assert_eq!(
            lines,
            vec![OutputLine::Stdout(payload.to_string())],
            "the payload must be echoed verbatim, never interpreted"
        );
    }

    #[tokio::test]
    async fn a_nonzero_exit_is_reported_as_failure() {
        let cmd = PlannedCommand {
            program: PathBuf::from("/usr/bin/false"),
            args: vec![],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: None,
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        };
        let (outcome, _) = collect(cmd).await;
        assert!(matches!(outcome, Outcome::Failed { code: Some(1) }), "got {outcome:?}");
    }

    #[tokio::test]
    async fn a_missing_program_fails_before_anything_runs() {
        let cmd = PlannedCommand {
            program: PathBuf::from("/usr/bin/definitely-not-here-xyz"),
            args: vec![],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: None,
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        };
        let (outcome, _) = collect(cmd).await;
        assert!(matches!(outcome, Outcome::Error { key: "err.spawn" }));
    }

    #[tokio::test]
    async fn cancelling_stops_the_process_and_removes_the_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let final_path = dir.path().join("out.bin");
        let temp = crate::atomic::temp_path_for(&final_path).unwrap();
        std::fs::write(&temp, b"partial").unwrap();

        // /bin/sleep طويل بما يكفي كي نلغيه في المنتصف.
        let cmd = PlannedCommand {
            program: PathBuf::from("/bin/sleep"),
            args: vec![OsString::from("30")],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: Some(Artifact::file(temp.clone(), final_path.clone())),
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        };

        let (tx, _rx) = mpsc::channel(8);
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let task = tokio::spawn(run(cmd, tx, cancel_rx));

        tokio::time::sleep(Duration::from_millis(150)).await;
        cancel_tx.send(()).unwrap();

        let outcome = task.await.unwrap();
        assert_eq!(outcome, Outcome::Cancelled);
        assert!(!temp.exists(), "cancellation must remove the partial output");
        assert!(!final_path.exists(), "cancellation must not produce a final file");
    }

    #[tokio::test]
    async fn a_successful_run_promotes_the_temp_to_its_final_name() {
        let dir = tempfile::tempdir().unwrap();
        let final_path = dir.path().join("out.bin");
        let temp = crate::atomic::temp_path_for(&final_path).unwrap();
        std::fs::write(&temp, b"produced").unwrap();

        let cmd = PlannedCommand {
            program: PathBuf::from("/usr/bin/true"),
            args: vec![],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: Some(Artifact::file(temp.clone(), final_path.clone())),
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        };
        let (outcome, _) = collect(cmd).await;

        assert!(outcome.is_success(), "got {outcome:?}");
        assert!(final_path.exists());
        assert!(!temp.exists());
    }

    #[tokio::test]
    async fn a_failing_run_leaves_no_final_file() {
        let dir = tempfile::tempdir().unwrap();
        let final_path = dir.path().join("out.bin");
        let temp = crate::atomic::temp_path_for(&final_path).unwrap();
        std::fs::write(&temp, b"half").unwrap();

        let cmd = PlannedCommand {
            program: PathBuf::from("/usr/bin/false"),
            args: vec![],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: Some(Artifact::file(temp.clone(), final_path.clone())),
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        };
        let (outcome, _) = collect(cmd).await;

        assert!(matches!(outcome, Outcome::Failed { .. }));
        assert!(!temp.exists(), "a failed run must not leave a partial archive");
        assert!(!final_path.exists(), "a failed run must not look like a success");
    }

    #[test]
    fn a_long_line_is_clipped_on_a_character_boundary_not_a_byte() {
        // العربية متعددة البايتات: القصّ الأعمى يشطر حرفًا وينتج نصًا معطوبًا.
        let line = "ع".repeat(MAX_LINE_BYTES); // بايتان لكل حرف
        let out = clip(line);
        assert!(out.len() <= MAX_LINE_BYTES + TRUNCATION_MARK.len());
        assert!(out.ends_with(TRUNCATION_MARK));
        assert!(std::str::from_utf8(out.as_bytes()).is_ok(), "clipping must not corrupt UTF-8");
    }

    #[test]
    fn a_short_line_is_returned_untouched() {
        let line = "سطر قصير".to_string();
        assert_eq!(clip(line.clone()), line);
    }

    #[tokio::test]
    async fn output_beyond_the_declared_ceiling_is_dropped_and_the_loss_is_announced() {
        // /usr/bin/seq يكتب أسطرًا كثيرة بسرعة دون صدفة.
        let cmd = PlannedCommand {
            program: PathBuf::from("/usr/bin/seq"),
            args: vec![OsString::from("1"), OsString::from((MAX_OUTPUT_LINES + 500).to_string())],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: None,
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        };
        let (outcome, lines) = collect(cmd).await;
        assert!(outcome.is_success(), "got {outcome:?}");

        let normal = lines.iter().filter(|l| !matches!(l, OutputLine::Truncated { .. })).count();
        assert_eq!(normal, MAX_OUTPUT_LINES, "the ceiling must hold exactly");

        let announced: usize = lines
            .iter()
            .filter_map(|l| match l {
                OutputLine::Truncated { dropped } => Some(*dropped),
                _ => None,
            })
            .sum();
        assert_eq!(announced, 500, "the number of dropped lines must be reported, not hidden");
    }

    #[tokio::test]
    async fn output_within_the_ceiling_carries_no_truncation_notice() {
        let cmd = PlannedCommand {
            program: PathBuf::from("/usr/bin/seq"),
            args: vec![OsString::from("1"), OsString::from("10")],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: None,
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        };
        let (_, lines) = collect(cmd).await;
        assert_eq!(lines.len(), 10);
        assert!(!lines.iter().any(|l| matches!(l, OutputLine::Truncated { .. })));
    }

    /// أداة اختبار لا مسار إنتاج: نحتاج برنامجًا يكتب بايتات لا نملك أداة
    /// إنتاجية تكتبها عمدًا. لا يوجد في المنتج أي استدعاء لصدفة، وتثبّت ذلك
    /// اختبارات `security.rs` و`golden.rs`.
    fn sh(script: String) -> PlannedCommand {
        PlannedCommand {
            program: PathBuf::from("/bin/sh"),
            args: vec![OsString::from("-c"), OsString::from(script)],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: None,
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        }
    }

    #[tokio::test]
    async fn one_invalid_utf8_byte_neither_ends_the_stream_nor_costs_the_archive() {
        // اسم ملف ليس UTF-8 صالحًا مشروع على macOS، و`ditto` تردّده كما هو.
        // القارئ النصّي كان يعامل بايتًا كهذا معاملة نهاية المجرى، فيُغلق
        // الأنبوب وتموت الأداة بـ SIGPIPE ويُحذف أرشيفٌ قد اكتمل.
        let dir = tempfile::tempdir().unwrap();
        let final_path = dir.path().join("أرشيف.zip");
        let temp = crate::atomic::temp_path_for(&final_path).unwrap();

        let mut cmd = sh(format!(
            "printf 'PK' > '{}'; printf 'قبل\\n' >&2; printf '\\377\\n' >&2; \
             printf 'بعد-١\\nبعد-٢\\n' >&2; exit 0",
            temp.display()
        ));
        cmd.artifact = Some(Artifact::file(temp.clone(), final_path.clone()));

        let (outcome, lines) = collect(cmd).await;
        assert!(
            outcome.is_success(),
            "a tool that exited zero must not be reported failed: {outcome:?}"
        );
        assert!(final_path.exists(), "one bad byte must not cost the user the finished archive");

        let texts: Vec<String> = lines
            .iter()
            .filter_map(|l| match l {
                OutputLine::Stderr(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        assert!(texts.iter().any(|s| s == "قبل"), "got {texts:?}");
        assert!(
            texts.iter().any(|s| s == "بعد-٢"),
            "output after the bad byte must keep flowing: {texts:?}"
        );
    }

    #[tokio::test]
    async fn a_stream_with_no_newline_at_all_does_not_grow_the_core() {
        // ٢٠٠ ميغابايت بلا فاصل أسطر. القارئ السابق كان يراكمها كلها في سطر
        // واحد قبل أن يصل إلى القصّ — أي أن الحدّ المعلَن يحكم ما يُرسل ولا
        // يحكم ما يُحجز.
        let before = peak_rss();
        let cmd = PlannedCommand {
            program: PathBuf::from("/bin/dd"),
            args: vec![
                OsString::from("if=/dev/zero"),
                OsString::from("bs=1048576"),
                OsString::from("count=200"),
            ],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: None,
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        };
        let (outcome, lines) = collect(cmd).await;
        assert!(outcome.is_success(), "got {outcome:?}");

        let grew = peak_rss().saturating_sub(before);
        assert!(
            grew < 64 * 1024 * 1024,
            "reading 200 MB without a newline grew peak RSS by {grew} bytes"
        );
        for l in &lines {
            if let OutputLine::Stdout(s) = l {
                assert!(s.len() <= MAX_LINE_BYTES + TRUNCATION_MARK.len(), "line not capped");
            }
        }
    }

    /// ذروة الذاكرة المقيمة للعملية بالبايتات. `ru_maxrss` تراكمية، فالفرق
    /// بين قياسين يحدّ ما أضافه ما بينهما ولا ينقص عنه.
    fn peak_rss() -> u64 {
        // SAFETY: بنية مصفّرة تُملأ بنداء قراءة فقط.
        let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
        unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
        usage.ru_maxrss as u64
    }

    #[test]
    fn a_line_buffer_never_holds_more_than_the_declared_budget() {
        let mut line = Vec::new();
        for _ in 0..1000 {
            append_bounded(&mut line, &vec![b'x'; 10_000]);
        }
        assert_eq!(line.len(), LINE_BUFFER_BYTES);
    }

    #[test]
    fn the_shared_ceiling_holds_exactly_when_both_streams_race_on_it() {
        // خيوط حقيقية لا مهامّ على منفّذ واحد: الفحص والزيادة كانا نداءين
        // منفصلين، ولا يمرّان معًا إلا بتوازٍ فعلي. والحاجز يضع الجميع على
        // العتبة نفسها: سطر واحد باقٍ تحت السقف وثمانية يطلبونه.
        //
        // التجربة تتكرّر لأن سباقًا يُثبَت بتكراره لا بمرّة واحدة — النسخة
        // التي تفحص ثم تزيد تخرق السقف في نحو ثلث المحاولات، فتُظهرها
        // خمسٌ وعشرون محاولة يقينًا. والنسخة الصحيحة دقيقة في كل مرّة.
        use std::sync::atomic::Ordering::Relaxed;

        let writers = 8;
        for trial in 0..25 {
            let (tx, mut rx) = mpsc::channel::<OutputLine>(writers + 4);
            let emitted = Arc::new(AtomicUsize::new(MAX_OUTPUT_LINES - 1));
            let dropped = Arc::new(AtomicUsize::new(0));
            let ready = Arc::new(AtomicUsize::new(0));
            let go = Arc::new(std::sync::atomic::AtomicBool::new(false));

            let mut threads = Vec::new();
            for _ in 0..writers {
                let tx = tx.clone();
                let emitted = emitted.clone();
                let dropped = dropped.clone();
                let ready = ready.clone();
                let go = go.clone();
                threads.push(std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
                    rt.block_on(async move {
                        let mut line = b"line".to_vec();
                        let mut sink = false;
                        // انتظار دوّار لا `Barrier`: الحاجز يوقظ الخيوط
                        // متتابعةً بميكروثوانٍ بينها، وهي دهرٌ أمام نافذة
                        // السباق. الدوران يجعلها تنطلق معًا فعلًا.
                        ready.fetch_add(1, Relaxed);
                        while !go.load(std::sync::atomic::Ordering::Acquire) {
                            std::hint::spin_loop();
                        }
                        deliver(&tx, &emitted, &dropped, OutputLine::Stdout, &mut line, &mut sink)
                            .await;
                    });
                }));
            }
            drop(tx);
            while ready.load(Relaxed) < writers {
                std::hint::spin_loop();
            }
            go.store(true, std::sync::atomic::Ordering::Release);
            for t in threads {
                t.join().unwrap();
            }

            let mut delivered = 0usize;
            while rx.blocking_recv().is_some() {
                delivered += 1;
            }
            assert_eq!(delivered, 1, "trial {trial}: the ceiling must hold exactly, not roughly");
            assert_eq!(
                dropped.load(Relaxed),
                writers - 1,
                "trial {trial}: every line that did not pass must be counted"
            );
        }
    }

    #[tokio::test]
    async fn the_child_does_not_inherit_the_users_path() {
        // /usr/bin/env بلا PATH يطبع بيئة لا تحتوي PATH.
        let cmd = PlannedCommand {
            program: PathBuf::from("/usr/bin/env"),
            args: vec![],
            cwd: None,
            explain: vec![],
            warnings: vec![],
            artifact: None,
            estimate: None,
            stdout_to: None,
            reveal_target: None,
        };
        let (outcome, lines) = collect(cmd).await;
        assert!(outcome.is_success());
        assert!(
            !lines.iter().any(|l| matches!(l, OutputLine::Stdout(s) if s.starts_with("PATH="))),
            "the child must not inherit PATH: {lines:?}"
        );
    }
}
