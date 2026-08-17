//! السجلّ عبر إعادة التشغيل.
//!
//! اختبارات `journal.rs` الداخلية تُثبت أن الكتابة والقراءة تعملان. ما يثبته
//! هذا الملف أمرٌ آخر: أن **الأمرين اللذين يعتمدان على السجل** — `recent_runs`
//! و`reveal` — ما زالا يعملان بعد إغلاق التطبيق وفتحه.
//!
//! هذا هو الفرق الذي غاب: السجلّ كان يُكتب على القرص ولا يُقرأ منه أبدًا، فكل
//! اختباراته كانت تقرأ ما كتبته العمليةُ نفسها في الجلسة نفسها، وتنجح جميعًا،
//! بينما شاشة السجل عند المستخدم تقول «لا تشغيلات» والملفُّ ممتلئ.
//!
//! و`resolve_target` هنا لا تُستدعى إلا على مسار تحت المنزل: سياسة المسارات
//! ترفض ما سواه، والسجلّ ليس التفافًا عليها.

use naffith_core::journal::{self, Entry, Journal, State};
use naffith_core::reveal;
use std::io::Write;
use std::path::PathBuf;

/// موضع اختبار تحت المنزل: `paths::existing_path` ترفض ما خرج عن الجذور
/// المسموحة، فمجلد مؤقّت في `/var` لا يصلح لاختبار `reveal`.
struct Sandbox(PathBuf);

impl Sandbox {
    fn new(label: &str) -> Option<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let base = home.join(format!(".naffith-test-{label}-{}", std::process::id()));
        std::fs::create_dir_all(&base).ok()?;
        Some(Sandbox(base))
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).ok();
    }
}

fn entry(id: &str, state: State) -> Entry {
    Entry::new(id, "compress.folder.zip", state).with_command(
        "/usr/bin/ditto".into(),
        vec!["-c".into(), "-k".into(), "/Users/x/مجلد".into()],
        None,
    )
}

#[test]
fn the_run_log_is_not_empty_after_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("runs.jsonl");

    {
        let session = Journal::new(Some(path.clone()));
        session.record(entry("run-1", State::Planned));
        session.record(entry("run-1", State::Running));
        session
            .record(entry("run-1", State::Succeeded { produced: Some("/Users/x/أ.zip".into()) }));
        session.record(entry("run-2", State::Cancelled));
    }

    let restarted = Journal::new(Some(path));
    let states: Vec<&str> = restarted.recent().iter().map(|e| e.state.name()).collect();
    assert_eq!(
        states,
        vec!["planned", "running", "succeeded", "cancelled"],
        "recent_runs must return the history the previous session wrote"
    );
}

#[test]
fn reveal_resolves_a_run_recorded_by_a_previous_session() {
    let Some(sandbox) = Sandbox::new("reveal-restart") else { return };
    let archive = sandbox.0.join("أرشيف.zip");
    std::fs::write(&archive, b"PK").unwrap();

    let path = sandbox.0.join("runs.jsonl");
    {
        let session = Journal::new(Some(path.clone()));
        session.record(entry(
            "run-1",
            State::Succeeded { produced: Some(archive.display().to_string()) },
        ));
    }

    // جلسة جديدة: لا شيء في الذاكرة إلا ما استُعيد من الملف.
    let restarted = Journal::new(Some(path));
    let target = reveal::resolve_target(&restarted, "run-1")
        .expect("an archive that is still on disk must stay revealable across a restart");
    assert_eq!(target, archive.canonicalize().unwrap());
}

#[test]
fn a_torn_line_from_a_crash_does_not_cost_the_previous_sessions_runs() {
    let Some(sandbox) = Sandbox::new("torn-restart") else { return };
    let archive = sandbox.0.join("أرشيف.zip");
    std::fs::write(&archive, b"PK").unwrap();

    let path = sandbox.0.join("runs.jsonl");
    {
        let session = Journal::new(Some(path.clone()));
        session.record(entry(
            "run-1",
            State::Succeeded { produced: Some(archive.display().to_string()) },
        ));
    }

    // انهيار في منتصف كتابة القيد التالي، والقطع داخل حرف عربي: بايت 0xD9
    // وحده هو أول بايتَي حرف بلا ثانيهما.
    let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
    f.write_all(b"{\"id\":\"run-2\",\"op_id\":\"compress.folder.zip\",\"program\":\"/Users/x/\xd9")
        .unwrap();
    drop(f);

    let read = journal::read_all(&path).unwrap();
    assert_eq!(read.damaged, 1, "one torn line, counted");
    assert_eq!(read.entries.len(), 1, "and one intact entry, kept");

    let restarted = Journal::new(Some(path.clone()));
    assert!(
        reveal::resolve_target(&restarted, "run-1").is_ok(),
        "a crash may cost the event it interrupted, never the history before it"
    );

    // والجلسة الجديدة تكتب فوق ملف فيه سطر مبتور دون أن تفسد ما قبله.
    restarted.record(entry("run-3", State::Cancelled));
    let read = journal::read_all(&path).unwrap();
    assert_eq!(read.damaged, 1, "the torn line stays one line");
    assert_eq!(read.entries.len(), 2);
    assert_eq!(read.entries[1].id, "run-3");
}

#[test]
fn plan_previews_do_not_revoke_reveal_for_a_finished_run() {
    // الواجهة تعيد التخطيط بعد ٢٥٠ مِلّي ثانية من سكون الكتابة، فكل تعديل على
    // اسم ملف يولّد عشرات قيود `planned`. تشغيلٌ انتهى للتوّ لا يجوز أن تطرده
    // ضغطات المفاتيح من متناول «أظهر في Finder».
    let Some(sandbox) = Sandbox::new("preview-flood") else { return };
    let archive = sandbox.0.join("أرشيف.zip");
    std::fs::write(&archive, b"PK").unwrap();

    let path = sandbox.0.join("runs.jsonl");
    let session = Journal::new(Some(path));
    session
        .record(entry("done", State::Succeeded { produced: Some(archive.display().to_string()) }));
    for i in 0..400 {
        session.record(entry(&format!("preview-{i}"), State::Planned));
    }

    let target = reveal::resolve_target(&session, "done")
        .expect("reveal must survive a flood of plan-time previews");
    assert_eq!(target, archive.canonicalize().unwrap());
}
