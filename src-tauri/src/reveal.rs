//! إظهار الناتج في Finder.
//!
//! ## لماذا أمرٌ في النواة لا إضافة `opener`
//!
//! المطلوب سلوك واحد: «أظهر ما أنتجه هذا التشغيل». إضافة `opener` كانت ستمنح
//! الواجهة القدرة على قول «افتح هذا المسار» — أي على تمرير مسار عبر الحدّ،
//! وهو بالضبط ما بُنيت المعمارية كلها كي يبقى مستحيلًا.
//!
//! هنا الواجهة تقول معرّف تشغيل، والنواة تُخرج المسار من **سجلّها هي**، ثم
//! تعيد التحقّق منه بالسياسة نفسها التي تحكم كل مسار في المنتج. لا يوجد مدخل
//! يجعل هذه الدالة تفتح موضعًا لم تنتجه بنفسها.
//!
//! ولا شيء يُكتب: `open -R` تُبرز الملف في نافذة Finder ولا تفتحه ولا تعدّله.

use crate::error::{CoreError, Result};
use crate::journal::Journal;
use crate::paths;
use crate::tools;
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// يستخرج المسار الذي يجوز إظهاره لهذا التشغيل، أو يرفض.
///
/// أربعة شروط، ولا واحد منها يأتي من الواجهة:
/// التشغيل معروف، ونجح، وأنتج مسارًا، والمسار ما زال قائمًا وضمن السياسة.
pub fn resolve_target(journal: &Journal, run_id: &str) -> Result<PathBuf> {
    let produced = journal.produced_for(run_id).ok_or(CoreError::NothingToReveal)?;
    // المسار خرج من سجلّنا، ومع ذلك يمرّ بالسياسة كاملة: السجل قد يكون قديمًا،
    // والملف قد يكون استُبدل برابط يشير خارج الجذور المسموحة.
    paths::existing_file(Path::new(&produced))
}

/// يُبرز الملف في Finder. لا يفتحه ولا يشغّله.
pub fn reveal(path: &Path) -> Result<()> {
    let program = tools::OPEN.resolve()?;
    // مصفوفة وسائط، لا صدفة. والمسار مطلق (خرج من `canonicalize`) فلا يُقرأ راية.
    debug_assert!(path.is_absolute(), "reveal must be given an absolute path");
    let mut command = std::process::Command::new(program);
    command.arg("-R").arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    spawn_and_reap(command).map(|_| ())
}

/// يطلق عملية عابرة **ويحصدها**، ويعيد رقمها.
///
/// `std::process::Child` لا ينتظر في `Drop` — الوثيقة تنصّ على ذلك صراحة —
/// فإسقاطه دون `wait` يترك عملية زومبي لكل نقرة «أظهر في Finder»، تبقى في
/// جدول عمليات التطبيق حتى يُغلق. خيطٌ قصير ينتظر خروجها أرخص من جرّ وقت
/// تشغيل غير متزامن إلى هذا المسار، وهو الموضع الوحيد في النواة الذي يطلق
/// عملية خارج `executor::run`.
fn spawn_and_reap(mut command: std::process::Command) -> Result<u32> {
    let child = command.spawn().map_err(CoreError::Io)?;
    let pid = child.id();
    std::thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });
    Ok(pid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journal::{Entry, State};

    fn journal_with(id: &str, state: State) -> Journal {
        let j = Journal::new(None);
        j.record(Entry::new(id, "compress.folder.zip", state));
        j
    }

    #[test]
    fn an_unknown_run_reveals_nothing() {
        let j = Journal::new(None);
        assert!(matches!(resolve_target(&j, "no-such-run"), Err(CoreError::NothingToReveal)));
    }

    #[test]
    fn a_cancelled_run_reveals_nothing() {
        let j = journal_with("r1", State::Cancelled);
        assert!(matches!(resolve_target(&j, "r1"), Err(CoreError::NothingToReveal)));
    }

    #[test]
    fn a_failed_run_reveals_nothing() {
        let j = journal_with("r1", State::Failed { reason: "exit".into(), code: Some(1) });
        assert!(matches!(resolve_target(&j, "r1"), Err(CoreError::NothingToReveal)));
    }

    #[test]
    fn a_run_that_is_only_planned_reveals_nothing() {
        let j = journal_with("r1", State::Planned);
        assert!(matches!(resolve_target(&j, "r1"), Err(CoreError::NothingToReveal)));
    }

    #[test]
    fn a_successful_run_whose_file_was_since_deleted_reveals_nothing() {
        let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return };
        let gone = home.join(".naffith-test-deleted-archive.zip");
        let _ = std::fs::remove_file(&gone);

        let j = journal_with("r1", State::Succeeded { produced: Some(gone.display().to_string()) });
        assert!(matches!(resolve_target(&j, "r1"), Err(CoreError::PathMissing)));
    }

    #[test]
    fn a_produced_path_outside_the_allowed_roots_is_still_refused() {
        // لا ينتج هذا من مسارٍ عادي، لكنه سجلٌّ يمكن أن يُحرَّر أو يُهاجَر.
        let j = journal_with("r1", State::Succeeded { produced: Some("/etc/hosts".into()) });
        let r = resolve_target(&j, "r1");
        assert!(
            matches!(r, Err(CoreError::PathOutsideAllowedRoots)),
            "the journal is not a bypass for the path policy: {r:?}"
        );
    }

    #[test]
    fn a_successful_run_resolves_to_its_archive() {
        let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return };
        let base = home.join(format!(".naffith-test-reveal-{}", crate::plans::random_suffix()));
        std::fs::create_dir_all(&base).unwrap();
        let archive = base.join("أرشيف.zip");
        std::fs::write(&archive, b"PK").unwrap();

        let j =
            journal_with("r1", State::Succeeded { produced: Some(archive.display().to_string()) });
        let target = resolve_target(&j, "r1").unwrap();
        assert_eq!(target, archive.canonicalize().unwrap());

        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn the_reveal_tool_is_an_absolute_system_path() {
        assert_eq!(tools::OPEN.absolute, "/usr/bin/open");
        assert!(tools::OPEN.resolve().is_ok());
    }

    #[test]
    fn a_revealed_process_is_reaped_and_leaves_no_zombie() {
        // `/usr/bin/true` لا `open`: نختبر الحصاد لا نافذة Finder. البرنامج
        // يخرج فورًا، فما بقي بعده في جدول العمليات زومبي لا عملية عاملة.
        let mut command = std::process::Command::new("/usr/bin/true");
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        let pid = spawn_and_reap(command).unwrap() as i32;

        // `kill(pid, 0)` ينجح على الزومبي أيضًا: القيد باقٍ حتى يُحصد. فشلها
        // هو الدليل على أن أحدًا انتظر العملية.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        // SAFETY: نداء فحص لا يرسل إشارة.
        while unsafe { libc::kill(pid, 0) } == 0 {
            assert!(
                std::time::Instant::now() < deadline,
                "pid {pid} stayed a zombie: nobody waited on it"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}
