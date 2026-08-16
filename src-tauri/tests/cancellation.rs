//! الإلغاء ومجموعة العمليات.
//!
//! ادّعاء «الإلغاء يوقف العملية» رخيص. الادّعاء الذي يهمّ هو: **يوقف العملية
//! وكل ما ولدته**. أداةٌ تُنهى وحدها بينما يظلّ حفيدُها يكتب في الملف المؤقّت
//! تترك المستخدم يظنّ أنه ألغى وقد استمرّ العمل.
//!
//! ولذلك هذه الاختبارات تشغّل برنامجًا يُنشئ طفلًا فرعيًا، وتثبت أن **الاثنين**
//! انتهيا — لا أن الأمّ وحدها انتهت.
//!
//! ملحوظة على الأداة المستخدمة: `/bin/sh` هنا **أداة اختبار** لا مسار إنتاج.
//! نحتاج برنامجًا يفرّع طفلًا وينشر معرّفات العمليات كي نتحقّق منها؛ لا يوجد
//! في المنتج أي استدعاء لصدفة، واختبارات `security.rs` و`golden.rs` تثبّت ذلك.

use naffith_core::atomic;
use naffith_core::executor::{self, Outcome, OutputLine};
use naffith_core::spec::{Artifact, PlannedCommand};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};

fn command(program: &str, args: &[&str], artifact: Option<Artifact>) -> PlannedCommand {
    PlannedCommand {
        program: PathBuf::from(program),
        args: args.iter().map(OsString::from).collect(),
        cwd: None,
        explain: vec![],
        warnings: vec![],
        artifact,
        estimate: None,
        stdout_to: None,
        reveal_target: None,
    }
}

/// هل ما زالت العملية موجودة؟ `signal 0` يفحص دون أن يرسل شيئًا.
fn alive(pid: i32) -> bool {
    // SAFETY: نداء فحص لا يرسل إشارة، ورقم العملية مقروء من خرج الاختبار.
    unsafe { libc::kill(pid, 0) == 0 }
}

/// ينتظر اختفاء عملية حتى مهلة. الطفل اليتيم يُحصد من `launchd` لا منّا،
/// فالاختفاء قد يتأخّر أجزاءً من الثانية.
fn wait_until_gone(pid: i32, within: Duration) -> bool {
    let deadline = Instant::now() + within;
    while Instant::now() < deadline {
        if !alive(pid) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    !alive(pid)
}

/// يشغّل أمرًا ويعيد أول سطرين من خرجه ومقبض الإلغاء.
async fn run_collecting(
    cmd: PlannedCommand,
) -> (tokio::task::JoinHandle<Outcome>, oneshot::Sender<()>, mpsc::Receiver<OutputLine>) {
    let (out_tx, out_rx) = mpsc::channel(64);
    let (cancel_tx, cancel_rx) = oneshot::channel();
    let task = tokio::spawn(executor::run(cmd, out_tx, cancel_rx));
    (task, cancel_tx, out_rx)
}

async fn next_number(rx: &mut mpsc::Receiver<OutputLine>) -> i32 {
    let line = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("the child must print within five seconds")
        .expect("the stream must not close early");
    match line {
        OutputLine::Stdout(s) | OutputLine::Stderr(s) => {
            s.trim().parse().unwrap_or_else(|_| panic!("expected a pid, got {s:?}"))
        }
        OutputLine::Truncated { dropped } => {
            panic!("a pid probe cannot overflow the stream ceiling, yet {dropped} were dropped")
        }
    }
}

#[tokio::test]
async fn the_child_runs_in_its_own_process_group_led_by_itself() {
    // `setsid` تجعل الطفل قائد جلسته ومجموعته. بدونها يرث مجموعة التطبيق،
    // وإرسال الإشارة إلى المجموعة كان سيقتل التطبيق نفسه.
    let (task, cancel, mut rx) =
        run_collecting(command("/bin/sh", &["-c", "echo $$; sleep 20"], None)).await;

    let child = next_number(&mut rx).await;

    // SAFETY: نداءان للقراءة فقط.
    let child_group = unsafe { libc::getpgid(child) };
    let our_group = unsafe { libc::getpgid(0) };

    assert_eq!(child_group, child, "the child must lead its own group (setsid)");
    assert_ne!(child_group, our_group, "the child must not share our group");

    cancel.send(()).unwrap();
    assert_eq!(task.await.unwrap(), Outcome::Cancelled);
}

#[tokio::test]
async fn cancelling_kills_the_grandchild_too_not_only_the_child() {
    // البرنامج يطبع معرّفه ثم معرّف طفله، ثم ينتظر. الطفل الفرعي هو ما كان
    // سيبقى حيًّا لو أرسلنا الإشارة إلى الأمّ وحدها.
    let (task, cancel, mut rx) =
        run_collecting(command("/bin/sh", &["-c", "echo $$; sleep 30 & echo $!; wait"], None))
            .await;

    let child = next_number(&mut rx).await;
    let grandchild = next_number(&mut rx).await;

    assert!(alive(child), "the child must be running before we cancel");
    assert!(alive(grandchild), "and so must the grandchild");
    assert_ne!(child, grandchild);

    cancel.send(()).unwrap();
    assert_eq!(task.await.unwrap(), Outcome::Cancelled);

    assert!(wait_until_gone(child, Duration::from_secs(5)), "the child survived cancellation");
    assert!(
        wait_until_gone(grandchild, Duration::from_secs(5)),
        "the grandchild survived cancellation — the signal did not reach the process group"
    );
}

#[tokio::test]
async fn cancelling_reaches_a_whole_chain_of_descendants() {
    // ثلاثة مستويات: طفل ← حفيد ← ابن حفيد. المجموعة تشملهم جميعًا.
    let (task, cancel, mut rx) = run_collecting(command(
        "/bin/sh",
        &["-c", "echo $$; /bin/sh -c 'echo $$; sleep 30 & echo $!; wait' & wait"],
        None,
    ))
    .await;

    let a = next_number(&mut rx).await;
    let b = next_number(&mut rx).await;
    let c = next_number(&mut rx).await;

    cancel.send(()).unwrap();
    assert_eq!(task.await.unwrap(), Outcome::Cancelled);

    for (label, pid) in [("child", a), ("grandchild", b), ("great-grandchild", c)] {
        assert!(
            wait_until_gone(pid, Duration::from_secs(5)),
            "{label} ({pid}) survived cancellation"
        );
    }
}

#[tokio::test]
async fn cancelling_costs_no_more_than_the_child_takes_to_die() {
    // المهلة قبل القتل القسري ١٫٥ ثانية، وكانت تُنتظر كاملة حتى حين تموت
    // الأداة فورًا على `SIGTERM`. طوال ذلك يبقى الأرشيف الجزئي على القرص،
    // ولم يُبثّ `run://finished` بعد، فتعرض الواجهة تشغيلًا ألغاه المستخدم
    // على أنه جارٍ. الانتظار هنا يجب أن يسابقه خروج الطفل لا أن يسبقه.
    let (task, cancel, _rx) = run_collecting(command("/bin/sleep", &["30"], None)).await;
    tokio::time::sleep(Duration::from_millis(120)).await;

    let at = Instant::now();
    cancel.send(()).unwrap();
    assert_eq!(task.await.unwrap(), Outcome::Cancelled);
    let took = at.elapsed();

    assert!(
        took < Duration::from_millis(700),
        "cancelling a process that dies on SIGTERM took {took:?}"
    );
}

#[tokio::test]
async fn a_descendant_that_ignores_sigterm_is_still_killed() {
    // هذا ما يحرس السباق أعلاه: لو رُبط التصعيد بانقضاء المهلة وحدها لنجا منه
    // كل حفيد يتجاهل `SIGTERM` بمجرّد أن يخرج القائد بسرعة. `SIGKILL` تُرسل
    // إلى المجموعة في الحالين، فالسباق يوفّر الانتظار ولا يبيع الضمان.
    //
    // `/bin/sleep` بمسار مطلق: الطفل يعمل ببيئة بلا `PATH` عمدًا، فاسمٌ مجرّد
    // لا يُحلّ ويصير الحلقة دورانًا محمومًا بدل انتظار.
    let (task, cancel, mut rx) = run_collecting(command(
        "/bin/sh",
        &["-c", "(trap '' TERM; while :; do /bin/sleep 1; done) & echo $!; wait"],
        None,
    ))
    .await;

    let stubborn = next_number(&mut rx).await;
    // مهلة قصيرة كي يكون `trap` قد نُصب فعلًا: `$!` يُطبع بعد `fork` مباشرة،
    // وقبل أن ينفّذ الفرعُ أوّل أمر فيه.
    tokio::time::sleep(Duration::from_millis(250)).await;
    assert!(alive(stubborn), "the descendant must be running before we cancel");

    cancel.send(()).unwrap();
    // بمهلة: حفيدٌ ناجٍ يُبقي طرف الأنبوب مفتوحًا، فتعلق `run` بدل أن تعود.
    let outcome = tokio::time::timeout(Duration::from_secs(15), task)
        .await
        .expect("a surviving descendant holds the pipe open and hangs the run")
        .unwrap();
    assert_eq!(outcome, Outcome::Cancelled);

    assert!(
        wait_until_gone(stubborn, Duration::from_secs(5)),
        "a descendant that ignores SIGTERM ({stubborn}) survived cancellation"
    );
}

#[tokio::test]
async fn cancelling_removes_the_temporary_output_and_produces_no_final_file() {
    let dir = tempfile::tempdir().unwrap();
    let final_path = dir.path().join("أرشيف.zip");
    let temp = atomic::temp_path_for(&final_path).unwrap();
    std::fs::write(&temp, "نصف أرشيف").unwrap();

    let artifact = Artifact::file(temp.clone(), final_path.clone());
    let (task, cancel, mut rx) = run_collecting(command(
        "/bin/sh",
        &["-c", "echo $$; sleep 30 & echo $!; wait"],
        Some(artifact),
    ))
    .await;

    let child = next_number(&mut rx).await;
    let grandchild = next_number(&mut rx).await;

    cancel.send(()).unwrap();
    assert_eq!(task.await.unwrap(), Outcome::Cancelled);

    assert!(wait_until_gone(grandchild, Duration::from_secs(5)));
    assert!(wait_until_gone(child, Duration::from_secs(5)));
    assert!(!temp.exists(), "cancellation must remove the partial output");
    assert!(!final_path.exists(), "cancellation must never produce a final archive");
}

#[tokio::test]
async fn cancelling_a_real_ditto_run_leaves_the_destination_clean() {
    // ليس محاكاة: `ditto` حقيقية على شجرة كبيرة بما يكفي كي نلغيها أثناءها.
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("مصدر");
    std::fs::create_dir(&source).unwrap();
    // ملفات كثيرة كي يستغرق الضغط وقتًا يكفي للإلغاء في منتصفه.
    for i in 0..400 {
        std::fs::write(source.join(format!("ملف-{i}.bin")), vec![b'x'; 200_000]).unwrap();
    }
    let dest = dir.path().join("وجهة");
    std::fs::create_dir(&dest).unwrap();

    let final_path = dest.join("ناتج.zip");
    let temp = atomic::temp_path_for(&final_path).unwrap();

    let cmd = command(
        "/usr/bin/ditto",
        &[
            "-c",
            "-k",
            "--sequesterRsrc",
            "--keepParent",
            source.to_str().unwrap(),
            temp.to_str().unwrap(),
        ],
        Some(Artifact::file(temp.clone(), final_path.clone())),
    );

    let (out_tx, _rx) = mpsc::channel(64);
    let (cancel_tx, cancel_rx) = oneshot::channel();
    let task = tokio::spawn(executor::run(cmd, out_tx, cancel_rx));

    tokio::time::sleep(Duration::from_millis(120)).await;
    cancel_tx.send(()).unwrap();

    assert_eq!(task.await.unwrap(), Outcome::Cancelled);
    assert!(!temp.exists(), "the partial archive must be removed");
    assert!(!final_path.exists(), "a cancelled run must not look like a success");
    assert_eq!(
        std::fs::read_dir(&dest).unwrap().count(),
        0,
        "nothing at all may survive in the destination"
    );
    assert_eq!(
        std::fs::read_dir(&source).unwrap().count(),
        400,
        "and the source must be untouched"
    );
}
