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
//! ولماذا لا `hard_link` وحدها: FAT32 و exFAT لا تعرفان الروابط الصلبة، و‏
//! `/Volumes` جذرٌ مسموح عمدًا كي تكون ذاكرة USB وبطاقة الكاميرا وجهةً صالحة —
//! وهما تُهيّآن بإحدى الصيغتين. انظر `promote` أدناه.
//!
//! و`Drop` هو شبكة الأمان الأخيرة: أي مسار خروج لم يستدعِ `commit` — فشل، أو
//! إلغاء، أو ذعر، أو إسقاط المستقبل — يحذف المؤقّت. لا يبقى أرشيف جزئي يوحي
//! بالنجاح.
//!
//! وخروجٌ ناجح ليس وحده شرط الترقية: `plans::Preconditions::claim_temp` يحجز
//! المؤقّت بإنشائه قبل الإطلاق، فهو **موجود دائمًا** لحظة `commit`. لذلك
//! «أُنتج شيء» تُقاس بالحجم لا بالوجود، وإلّا رُقّي أرشيف بحجم صفر وقيل للمستخدم
//! إنه نجاح. انظر `commit`.

use crate::error::{CoreError, Result};
use crate::plans::random_suffix;
use crate::spec::{Artifact, ArtifactKind};
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
    kind: ArtifactKind,
    settled: bool,
}

impl ArtifactGuard {
    pub fn new(artifact: &Artifact) -> Self {
        Self {
            temp: artifact.temp.clone(),
            final_path: artifact.final_path.clone(),
            kind: artifact.kind,
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
    ///
    /// **«لم يُنتج شيء» = الحجم صفر، لا الغياب.** الشرط كان `exists()` وكان
    /// صادقًا يوم كُتب؛ ثم صار `plans::Preconditions::claim_temp` يُنشئ المؤقّت
    /// حصريًا (‏`O_CREAT|O_EXCL`‎) قبل الإطلاق كي يسدّ سباق زرع رابط رمزي مكانه،
    /// فصار المؤقّت **موجودًا دائمًا** لحظة الوصول إلى هنا. أي أن الشرط لم
    /// يعد قابلًا للتحقّق أبدًا: أداةٌ تخرج بصفر دون أن تكتب بايتًا واحدًا كانت
    /// تُرقّي أرشيفًا فارغًا ويُقيَّد التشغيل **نجاحًا**، والمستخدم يفتح ملفًا
    /// بحجم صفر. الفحص على الحجم يستعيد الثابتة التي يعلنها هذا الملف.
    ///
    /// والحذف صراحةً قبل الخروج: الملف الصفري المحجوز موجود فعلًا، وتركُه
    /// بعد تسوية الحارس (`settled`) كان سيخلّف `.naffith-*.part` في مجلد
    /// المستخدم بعد كل فشلٍ من هذا النوع — و`Drop` لا يمرّ عليه بعد التسوية.
    pub fn commit(mut self) -> Result<PathBuf> {
        self.settled = true;
        match self.kind {
            ArtifactKind::File => {
                let produced = std::fs::metadata(&self.temp).map(|m| m.len() > 0).unwrap_or(false);
                if !produced {
                    self.cleanup();
                    return Err(CoreError::PathMissing);
                }
                let promoted = promote(&self.temp, &self.final_path);
                // في مسار الرابط الصلب يبقى المؤقّت بعد النجاح، وفي مسار النقل
                // لا يبقى. حذفٌ غير مشروط يغطّي الحالتين، وفشله لا يعني شيئًا.
                let _ = std::fs::remove_file(&self.temp);
                promoted.map(|()| self.final_path.clone())
            }
            ArtifactKind::Dir => {
                // «أُنتج شيء» للمجلد = مدخلةٌ واحدة على الأقل. أداةٌ خرجت بصفر
                // ولم تكتب شيئًا تُنتج مجلدًا فارغًا، وترقيتُه تقول للمستخدم إن
                // الاستخراج نجح ثم لا يجد فيه شيئًا.
                let produced =
                    std::fs::read_dir(&self.temp).map(|mut d| d.next().is_some()).unwrap_or(false);
                if !produced {
                    self.cleanup();
                    return Err(CoreError::PathMissing);
                }
                let promoted = promote_dir(&self.temp, &self.final_path);
                if promoted.is_err() {
                    self.cleanup();
                }
                promoted.map(|()| self.final_path.clone())
            }
        }
    }

    /// إسقاط صريح عند الفشل أو الإلغاء.
    pub fn abort(mut self) {
        self.cleanup();
        self.settled = true;
    }

    fn cleanup(&self) {
        match self.kind {
            ArtifactKind::File => {
                if self.temp.exists() {
                    let _ = std::fs::remove_file(&self.temp);
                }
            }
            // `remove_dir_all` لا تتبع الروابط الرمزية داخل الشجرة (تحذفها
            // كروابط)، فتنظيفُ استخراجٍ نصفيّ لا يمكن أن يمتدّ خارج المؤقّت.
            ArtifactKind::Dir => {
                if self.temp.symlink_metadata().is_ok() {
                    let _ = std::fs::remove_dir_all(&self.temp);
                }
            }
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

/// يرقّي المؤقّت إلى اسمه النهائي، ولا يستبدل ملفًا موجودًا في أي حال.
///
/// `hard_link` هي المسار المفضّل: نداءٌ واحد يمنح رفض الاستبدال ذرّيًا
/// (‏`EEXIST`) بلا نافذة زمنية أصلًا. لكنها ليست متاحة في كل مكان — و‏
/// **فشلها كان يُتلف عملًا مكتملًا**: على exFAT ترجع `ENOTSUP` (٤٥) بعد أن
/// تكون `ditto` قد كتبت الأرشيف كاملًا، فيُحذف المؤقّت ويرى المستخدم
/// `err.commit` بعد انتظار طويل ولا يبقى له شيء. وهذه ليست حالة نادرة: كل
/// ذاكرة USB وبطاقة كاميرا تصل بـ exFAT أو FAT32.
fn promote(temp: &Path, final_path: &Path) -> Result<()> {
    match std::fs::hard_link(temp, final_path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(CoreError::DestinationExists)
        }
        // أي فشل آخر — لا نميّز رقم الخطأ لأن `ENOTSUP` يصل Rust بنوع
        // `Uncategorized` غير قابل للمطابقة — يُجرَّب بالبديل، وهو نفسه يرفض
        // الاستبدال، فتجريبه لا يخاطر بشيء.
        Err(_) => promote_without_links(temp, final_path),
    }
}

/// ترقية على نظام ملفات لا يعرف الروابط الصلبة.
///
/// نحجز الاسم النهائي أولًا بـ `O_CREAT|O_EXCL` — وهي ذرّية على FAT و exFAT
/// و SMB كما هي على APFS — فإن كان الاسم مأخوذًا فشل الحجز ولم نمسّ شيئًا.
/// ثم `rename` تنقل المؤقّت فوق **حجزنا نحن**، فما تستبدله ملفٌ فارغ أنشأناه
/// قبل سطر واحد لا ملف مستخدم.
///
/// `renamex_np` بـ `RENAME_EXCL` جُرِّبت أولًا لأنها تلغي النافذة بين الحجز
/// والنقل، وسقطت: على exFAT ترجع `ENOTSUP` هي الأخرى، فلم تكن تحلّ الحالة
/// التي وُجدت من أجلها.
/// يرقّي مجلدًا مؤقّتًا إلى اسمه النهائي، ولا يستبدل شيئًا موجودًا.
///
/// `hard_link` لا تعمل على المجلدات على أي نظام ملفات، فلا مسار مفضّل هنا:
/// الآلية الوحيدة هي **حجز الاسم ثم النقل فوق الحجز**، وهي نفس فكرة
/// `promote_without_links`.
///
/// `create_dir` ذرّية وتفشل بـ `EEXIST` إن كان الاسم مأخوذًا — بملفٍ أو مجلدٍ
/// أو رابطٍ معلَّق — فالحجز نفسه هو الفحص. ثم `rename(temp، final)` على يونكس
/// تنجح حين تكون الوجهة **مجلدًا فارغًا**، وهو بالضبط ما أنشأناه قبل سطر.
///
/// وإن ملأه أحدٌ في تلك اللحظة فشل النقل بـ `ENOTEMPTY` — فشلٌ مغلق، ونحذف
/// حجزنا كي لا يبقى مجلدٌ فارغ يحمل الاسم الذي اختاره المستخدم ويوحي بأن
/// الاستخراج تمّ. والحذف `remove_dir` لا `remove_dir_all`: ما نحذفه حجزٌ فارغ،
/// وإن لم يكن فارغًا فهو ليس حجزنا ولا يُمسّ.
fn promote_dir(temp: &Path, final_path: &Path) -> Result<()> {
    match std::fs::create_dir(final_path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(CoreError::DestinationExists)
        }
        Err(e) => return Err(CoreError::Io(e)),
    }
    std::fs::rename(temp, final_path).map_err(|e| {
        let _ = std::fs::remove_dir(final_path);
        CoreError::Io(e)
    })
}

fn promote_without_links(temp: &Path, final_path: &Path) -> Result<()> {
    match std::fs::OpenOptions::new().write(true).create_new(true).open(final_path) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(CoreError::DestinationExists)
        }
        Err(e) => return Err(CoreError::Io(e)),
    }
    std::fs::rename(temp, final_path).map_err(|e| {
        // فشل النقل بعد الحجز: لا يجوز أن يبقى ملف فارغ يحمل الاسم الذي
        // اختاره المستخدم ويوحي بأن شيئًا أُنتج.
        let _ = std::fs::remove_file(final_path);
        CoreError::Io(e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(dir: &Path, name: &str) -> Artifact {
        let final_path = dir.join(name);
        Artifact::file(temp_path_for(&final_path).unwrap(), final_path)
    }

    /// ناتجٌ مجلد، ومؤقّته مُنشأ كما يُنشئه `plans::Preconditions::claim_temp`.
    fn dir_artifact(dir: &Path, name: &str) -> Artifact {
        let final_path = dir.join(name);
        let temp = temp_path_for(&final_path).unwrap();
        std::fs::create_dir(&temp).unwrap();
        Artifact::dir(temp, final_path)
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
    fn a_claimed_but_never_written_temp_is_still_nothing_produced() {
        // الحالة الواقعية بعد أن صار `claim_temp` يُنشئ المؤقّت حصريًا قبل
        // الإطلاق: الملف **موجود** دائمًا وحجمه صفر. فحصُ `exists()` كان قد
        // صار لا يُحقَّق أبدًا، فأرشيفٌ فارغ يُرقّى ويُقيَّد نجاحًا.
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "out.zip");
        std::fs::write(&a.temp, b"").unwrap();

        let r = ArtifactGuard::new(&a).commit();

        assert!(matches!(r, Err(CoreError::PathMissing)), "got {r:?}");
        assert!(!a.final_path.exists(), "a zero-byte archive must never reach the final name");
        assert!(
            !a.temp.exists(),
            "and the claimed placeholder must not be left in the destination"
        );
    }

    #[test]
    fn the_link_free_promotion_moves_the_output_to_its_final_name() {
        // مسار الأقراص التي لا تعرف الروابط الصلبة، مُختبَرًا وحده: على APFS
        // لا يمكن إفشال `hard_link` عمدًا، والاختبار الكامل على قرص exFAT
        // حقيقي في tests/compress_integration.rs.
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "ناتج.zip");
        std::fs::write(&a.temp, b"payload").unwrap();

        promote_without_links(&a.temp, &a.final_path).unwrap();

        assert_eq!(std::fs::read(&a.final_path).unwrap(), b"payload");
        assert!(!a.temp.exists(), "rename must have consumed the temporary file");
    }

    #[test]
    fn the_link_free_promotion_also_refuses_to_replace_an_existing_file() {
        // القيمة كلها في هذا: بديلٌ يستبدل بصمت أسوأ من الخطأ الذي يعالجه.
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "ناتج.zip");
        std::fs::write(&a.final_path, b"PRECIOUS EXISTING DATA").unwrap();
        std::fs::write(&a.temp, b"new output").unwrap();

        let r = promote_without_links(&a.temp, &a.final_path);

        assert!(matches!(r, Err(CoreError::DestinationExists)));
        assert_eq!(std::fs::read(&a.final_path).unwrap(), b"PRECIOUS EXISTING DATA");
    }

    #[test]
    fn the_link_free_promotion_refuses_a_dangling_symlink_at_the_final_name() {
        // رابط معلَّق ليس مكانًا شاغرًا: `O_EXCL` يفشل عليه كما يفشل
        // `hard_link`، وإلّا كتبنا عبر الرابط إلى موضع لم نخطّط له.
        let dir = tempfile::tempdir().unwrap();
        let a = artifact(dir.path(), "ناتج.zip");
        std::os::unix::fs::symlink(dir.path().join("nowhere"), &a.final_path).unwrap();
        std::fs::write(&a.temp, b"new output").unwrap();

        assert!(matches!(
            promote_without_links(&a.temp, &a.final_path),
            Err(CoreError::DestinationExists)
        ));
        assert!(!dir.path().join("nowhere").exists(), "nothing may be written through the link");
    }

    // ── الناتج المجلد ─────────────────────────────────────────────────

    #[test]
    fn a_directory_artifact_is_promoted_with_its_contents() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir_artifact(dir.path(), "المستخرَج");
        std::fs::write(a.temp.join("ملف.txt"), b"payload").unwrap();
        std::fs::create_dir(a.temp.join("داخل")).unwrap();

        let out = ArtifactGuard::new(&a).commit().unwrap();

        assert_eq!(out, a.final_path);
        assert_eq!(std::fs::read(a.final_path.join("ملف.txt")).unwrap(), b"payload");
        assert!(a.final_path.join("داخل").is_dir());
        assert!(!a.temp.exists(), "the temporary directory must not survive a commit");
    }

    #[test]
    fn an_empty_directory_artifact_is_not_promoted() {
        // أداةٌ خرجت بصفر ولم تستخرج شيئًا. ترقيةُ مجلدٍ فارغ تقول «نجح» عن
        // لا شيء، ويفتحه المستخدم فلا يجد فيه ما جاء من أجله.
        let dir = tempfile::tempdir().unwrap();
        let a = dir_artifact(dir.path(), "فارغ");

        let r = ArtifactGuard::new(&a).commit();

        assert!(matches!(r, Err(CoreError::PathMissing)), "got {r:?}");
        assert!(!a.final_path.exists());
        assert!(!a.temp.exists(), "and the claimed placeholder must be cleaned up");
    }

    #[test]
    fn an_existing_directory_at_the_final_name_is_never_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir_artifact(dir.path(), "موجود");
        std::fs::write(a.temp.join("جديد.txt"), b"new").unwrap();
        std::fs::create_dir(&a.final_path).unwrap();
        std::fs::write(a.final_path.join("قديم.txt"), b"PRECIOUS").unwrap();

        let r = ArtifactGuard::new(&a).commit();

        assert!(matches!(r, Err(CoreError::DestinationExists)), "got {r:?}");
        assert_eq!(std::fs::read(a.final_path.join("قديم.txt")).unwrap(), b"PRECIOUS");
        assert!(!a.final_path.join("جديد.txt").exists(), "nothing may be merged in");
        assert!(!a.temp.exists(), "and the temp must still be cleaned up");
    }

    #[test]
    fn an_existing_file_at_the_final_name_also_blocks_a_directory_promotion() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir_artifact(dir.path(), "مأخوذ");
        std::fs::write(a.temp.join("x"), b"x").unwrap();
        std::fs::write(&a.final_path, b"A FILE IS HERE").unwrap();

        assert!(matches!(ArtifactGuard::new(&a).commit(), Err(CoreError::DestinationExists)));
        assert_eq!(std::fs::read(&a.final_path).unwrap(), b"A FILE IS HERE");
    }

    #[test]
    fn dropping_a_directory_guard_removes_a_half_extracted_tree() {
        // مسار الإلغاء: `ditto -x` أو `tar -x` قُتلت في المنتصف، فبقيت شجرةٌ
        // ناقصة. لا يجوز أن تصل إلى الاسم النهائي ولا أن تبقى في الوجهة.
        let dir = tempfile::tempdir().unwrap();
        let a = dir_artifact(dir.path(), "ناقص");
        std::fs::create_dir_all(a.temp.join("أ/ب")).unwrap();
        std::fs::write(a.temp.join("أ/ب/جزء"), b"half").unwrap();

        {
            let _guard = ArtifactGuard::new(&a);
        }

        assert!(!a.temp.exists(), "a half-extracted tree must not be left behind");
        assert!(!a.final_path.exists());
    }

    #[test]
    fn cleaning_a_directory_artifact_does_not_follow_a_symlink_out_of_it() {
        // شجرةٌ مستخرَجة قد تحوي روابط رمزية. التنظيف يحذف الرابط لا هدفه.
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().join("خارج.txt");
        std::fs::write(&outside, b"PRECIOUS").unwrap();

        let a = dir_artifact(dir.path(), "شجرة");
        std::os::unix::fs::symlink(&outside, a.temp.join("رابط")).unwrap();

        ArtifactGuard::new(&a).abort();

        assert!(!a.temp.exists());
        assert_eq!(
            std::fs::read(&outside).unwrap(),
            b"PRECIOUS",
            "cleanup must never walk out through a link"
        );
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
