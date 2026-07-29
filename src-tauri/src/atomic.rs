//! إنشاء النواتج بصورة ذرّية.
//!
//! القاعدة: **لا يُكتب إلى الاسم النهائي مباشرة.** الأداة تكتب إلى اسم مؤقّت
//! داخل *نفس* مجلد الوجهة، ولا يُرقّى إلى اسمه النهائي إلا بعد خروج ناجح.
//!
//! لماذا نفس المجلد: الترقية يجب أن تكون داخل نظام ملفات واحد وإلا صارت نسخًا
//! لا ربطًا، وفقدت ذرّيتها.
//!
//! لماذا `hard_link` + `remove` بدل `rename`: `rename` على يونكس **يستبدل**
//! الوجهة بصمت. `hard_link` يفشل بـ `EEXIST` إن ظهر ملف بالاسم النهائي بين
//! التخطيط والترقية — وهو بالضبط السلوك المطلوب: لا نستبدل شيئًا لم نخطّط
//! لاستبداله.
//!
//! و`Drop` هو شبكة الأمان الأخيرة: أي مسار خروج لم يستدعِ `commit` — فشل، أو
//! إلغاء، أو ذعر، أو إسقاط المستقبل — يحذف المؤقّت. لا يبقى أرشيف جزئي يوحي
//! بالنجاح.

use crate::error::{CoreError, Result};
use crate::plans::random_suffix;
use crate::spec::Artifact;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const TEMP_PREFIX: &str = ".naffith-";
const TEMP_SUFFIX: &str = ".part";
/// أطول اسم ملف تقبله APFS/HFS+ بالبايتات.
const MAX_NAME_BYTES: usize = 255;

/// يبني مسار الملف المؤقّت المقابل لمسار نهائي.
///
/// يبدأ بنقطة كي لا يظهر في Finder أثناء العمل، وينتهي بـ `.part` كي يكون
/// واضحًا إن بقي بعد انهيار مفاجئ، ويحمل ١٦ رمزًا عشوائيًا (‏٦٤ بت) كي لا
/// يصطدم باسم قائم ولا بمؤقّت عمليةٍ أخرى تعمل في اللحظة نفسها.
///
/// الجزء الوصفي (الاسم النهائي) يُقتطع عند الحاجة: الاسم النهائي وحده قد يبلغ
/// ٢٥٥ بايت، وإضافة البادئة فوقه تتجاوز حدّ نظام الملفات فيفشل الإنشاء
/// بـ `ENAMETOOLONG` — وهو فشلٌ يقع على أسماء عربية طويلة قبل غيرها، لأن كل
/// حرف عربي بايتان في UTF-8.
pub fn temp_path_for(final_path: &Path) -> Result<PathBuf> {
    let dir = final_path.parent().ok_or(CoreError::PathNotAbsolute)?;
    let stem = final_path.file_name().ok_or(CoreError::PathNotAbsolute)?;

    let random = random_suffix();
    let fixed = TEMP_PREFIX.len() + random.len() + 1 + TEMP_SUFFIX.len();
    let budget = MAX_NAME_BYTES.saturating_sub(fixed);

    let mut name = OsString::from(TEMP_PREFIX);
    name.push(random.as_str());
    name.push("-");
    name.push(fit_within(stem, budget));
    name.push(TEMP_SUFFIX);
    Ok(dir.join(name))
}

/// يقتطع الجزء الوصفي ليبقى داخل الميزانية، عند حدّ محرف لا حدّ بايت.
///
/// قطعُ بايتٍ في منتصف محرف UTF-8 ينتج اسمًا ترفضه APFS، فالجزء الوصفي رفاهية
/// تُسقَط كاملة إن لم يكن الاسم UTF-8 صالحًا — الاسم المؤقّت عشوائيّ أصلًا،
/// والوصف فيه للتشخيص لا للصحّة.
fn fit_within(stem: &OsStr, budget: usize) -> OsString {
    if stem.as_bytes().len() <= budget {
        return stem.to_os_string();
    }
    match stem.to_str() {
        Some(s) => {
            let mut end = budget.min(s.len());
            while end > 0 && !s.is_char_boundary(end) {
                end -= 1;
            }
            OsString::from(&s[..end])
        }
        None => OsString::new(),
    }
}

/// يحرس ناتجًا مؤقّتًا حتى تُقرّر ترقيته أو إسقاطه.
#[derive(Debug)]
pub struct ArtifactGuard {
    temp: PathBuf,
    final_path: PathBuf,
    settled: bool,
}

impl ArtifactGuard {
    pub fn new(artifact: &Artifact) -> Self {
        Self {
            temp: artifact.temp.clone(),
            final_path: artifact.final_path.clone(),
            settled: false,
        }
    }

    pub fn temp(&self) -> &Path {
        &self.temp
    }

    pub fn final_path(&self) -> &Path {
        &self.final_path
    }

    /// يرقّي المؤقّت إلى اسمه النهائي. يفشل إن لم يُنتج شيء، أو إن ظهر ملف
    /// بالاسم النهائي في هذه الأثناء.
    pub fn commit(mut self) -> Result<PathBuf> {
        if !self.temp.exists() {
            self.settled = true;
            return Err(CoreError::PathMissing);
        }
        match std::fs::hard_link(&self.temp, &self.final_path) {
            Ok(()) => {
                let _ = std::fs::remove_file(&self.temp);
                self.settled = true;
                Ok(self.final_path.clone())
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // شخص أو شيء سبقنا إلى الاسم. لا نستبدل — نتراجع ونبلّغ.
                let _ = std::fs::remove_file(&self.temp);
                self.settled = true;
                Err(CoreError::DestinationExists)
            }
            Err(e) => {
                let _ = std::fs::remove_file(&self.temp);
                self.settled = true;
                Err(CoreError::Io(e))
            }
        }
    }

    /// إسقاط صريح عند الفشل أو الإلغاء.
    pub fn abort(mut self) {
        self.cleanup();
        self.settled = true;
    }

    fn cleanup(&self) {
        if self.temp.exists() {
            let _ = std::fs::remove_file(&self.temp);
        }
    }
}

impl Drop for ArtifactGuard {
    fn drop(&mut self) {
        if !self.settled {
            self.cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(dir: &Path, name: &str) -> Artifact {
        let final_path = dir.join(name);
        Artifact { temp: temp_path_for(&final_path).unwrap(), final_path }
    }

    #[test]
    fn the_temp_file_sits_next_to_its_final_name() {
        let a = artifact(Path::new("/tmp/dest"), "أرشيف.zip");
        assert_eq!(a.temp.parent(), a.final_path.parent(), "must share a filesystem to be atomic");
        assert_ne!(a.temp.file_name(), a.final_path.file_name());
        let temp_name = a.temp.file_name().unwrap().to_string_lossy().into_owned();
        assert!(temp_name.starts_with(".naffith-"));
        assert!(temp_name.ends_with(".part"));
    }

    #[test]
    fn the_temp_name_stays_within_the_filesystem_limit() {
        // ١٢٧ حرفًا عربيًا = ٢٥٤ بايت، وهو أطول اسم يقبله `sanitize_name`
        // تقريبًا. البادئة فوقه كانت ستتجاوز ٢٥٥.
        let long = format!("{}.zip", "ا".repeat(120));
        let final_path = Path::new("/tmp/dest").join(&long);
        let temp = temp_path_for(&final_path).unwrap();
        let bytes = temp.file_name().unwrap().as_bytes().len();
        assert!(
            bytes <= MAX_NAME_BYTES,
            "temp name is {bytes} bytes, over the {MAX_NAME_BYTES} limit"
        );
        let name = temp.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with(TEMP_PREFIX) && name.ends_with(TEMP_SUFFIX));
    }

    #[test]
    fn a_long_arabic_name_can_actually_be_created_on_disk() {
        // الحدّ ليس نظريًا: هذا يثبت أن نظام الملفات يقبل الاسم فعلًا.
        let dir = tempfile::tempdir().unwrap();
        let long = format!("{}.zip", "مجلد".repeat(30));
        let temp = temp_path_for(&dir.path().join(&long)).unwrap();
        std::fs::write(&temp, b"x").expect("the temp name must be creatable");
        assert!(temp.exists());
    }

    #[test]
    fn temp_names_do_not_collide() {
        let f = Path::new("/tmp/dest/out.zip");
        let a = temp_path_for(f).unwrap();
        let b = temp_path_for(f).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn success_promotes_the_temp_to_the_final_name() {
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "out.zip");
        std::fs::write(&a.temp, b"payload").unwrap();

        let guard = ArtifactGuard::new(&a);
        let out = guard.commit().unwrap();

        assert_eq!(out, a.final_path);
        assert!(a.final_path.exists());
        assert!(!a.temp.exists(), "the temp must not survive a commit");
        assert_eq!(std::fs::read(&a.final_path).unwrap(), b"payload");
    }

    #[test]
    fn failure_leaves_no_partial_archive_behind() {
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "out.zip");
        std::fs::write(&a.temp, b"half written").unwrap();

        ArtifactGuard::new(&a).abort();

        assert!(!a.temp.exists(), "partial output must be removed");
        assert!(!a.final_path.exists(), "a failed run must not produce a final file");
    }

    #[test]
    fn dropping_the_guard_without_committing_cleans_up() {
        // هذا مسار الإلغاء والذعر: لا شيء يستدعي abort صراحة.
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "out.zip");
        std::fs::write(&a.temp, b"interrupted").unwrap();

        {
            let _guard = ArtifactGuard::new(&a);
        } // drop

        assert!(!a.temp.exists(), "Drop is the last line of defence and it must clean up");
        assert!(!a.final_path.exists());
    }

    #[test]
    fn an_existing_final_file_is_never_silently_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "out.zip");
        std::fs::write(&a.final_path, b"PRECIOUS EXISTING DATA").unwrap();
        std::fs::write(&a.temp, b"new output").unwrap();

        let r = ArtifactGuard::new(&a).commit();

        assert!(matches!(r, Err(CoreError::DestinationExists)));
        assert_eq!(
            std::fs::read(&a.final_path).unwrap(),
            b"PRECIOUS EXISTING DATA",
            "the pre-existing file must be untouched"
        );
        assert!(!a.temp.exists(), "and the temp must still be cleaned up");
    }

    #[test]
    fn committing_when_the_tool_produced_nothing_is_an_error_not_an_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "out.zip");
        // لم يُكتب أي مؤقّت — الأداة خرجت بنجاح ظاهري دون أن تنتج شيئًا.
        let r = ArtifactGuard::new(&a).commit();
        assert!(matches!(r, Err(CoreError::PathMissing)));
        assert!(!a.final_path.exists());
    }

    #[test]
    fn arabic_and_spaced_names_survive_the_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "نسخة احتياطية ٢٠٢٦.zip");
        std::fs::write(&a.temp, b"x").unwrap();
        let out = ArtifactGuard::new(&a).commit().unwrap();
        assert_eq!(out.file_name().unwrap(), std::ffi::OsStr::new("نسخة احتياطية ٢٠٢٦.zip"));
    }
}
