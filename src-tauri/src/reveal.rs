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
    std::process::Command::new(program)
        .arg("-R")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(CoreError::Io)?;
    Ok(())
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
}
