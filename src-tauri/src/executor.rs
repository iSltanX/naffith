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
use std::process::Stdio;
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

    let mut cmd = tokio::process::Command::new(&command.program);
    cmd.args(&command.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
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
    let emitted = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let dropped = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

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
            terminate_group(pid).await;
            child.wait().await
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
        None => Outcome::Success { produced: None },
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
async fn pump<R>(
    stream: Option<R>,
    tx: mpsc::Sender<OutputLine>,
    emitted: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    dropped: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    wrap: fn(String) -> OutputLine,
) where
    R: tokio::io::AsyncRead + Unpin,
{
    use std::sync::atomic::Ordering::Relaxed;
    let Some(s) = stream else { return };
    let mut lines = BufReader::new(s).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if emitted.load(Relaxed) >= MAX_OUTPUT_LINES {
            // لا نتوقّف عن *القراءة*: الكفّ عن تفريغ الأنبوب يجمّد الأداة
            // بدل أن ينهيها. نعدّ ونهمل.
            dropped.fetch_add(1, Relaxed);
            continue;
        }
        emitted.fetch_add(1, Relaxed);
        if tx.send(wrap(clip(line))).await.is_err() {
            break;
        }
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

/// ينهي مجموعة العمليات: طلب لطيف، ثم قتل بعد مهلة.
///
/// الإشارة تُرسل إلى `-pid` أي إلى المجموعة كلها، لأن `setsid` جعلت الطفل
/// قائدها. إرسالها إلى الطفل وحده يترك أحفاده يعملون.
async fn terminate_group(pid: i32) {
    unsafe {
        libc::killpg(pid, libc::SIGTERM);
    }
    tokio::time::sleep(GRACE).await;
    unsafe {
        libc::killpg(pid, libc::SIGKILL);
    }
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
            artifact: Some(Artifact { temp: temp.clone(), final_path: final_path.clone() }),
            estimate: None,
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
            artifact: Some(Artifact { temp: temp.clone(), final_path: final_path.clone() }),
            estimate: None,
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
            artifact: Some(Artifact { temp: temp.clone(), final_path: final_path.clone() }),
            estimate: None,
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
        };
        let (_, lines) = collect(cmd).await;
        assert_eq!(lines.len(), 10);
        assert!(!lines.iter().any(|l| matches!(l, OutputLine::Truncated { .. })));
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
        };
        let (outcome, lines) = collect(cmd).await;
        assert!(outcome.is_success());
        assert!(
            !lines.iter().any(|l| matches!(l, OutputLine::Stdout(s) if s.starts_with("PATH="))),
            "the child must not inherit PATH: {lines:?}"
        );
    }
}
