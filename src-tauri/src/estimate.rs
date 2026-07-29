//! تقدير حجم المصدر والمساحة المتاحة في الوجهة.
//!
//! **هذا تقدير، لا ضمان.** ثلاثة أسباب تجعله كذلك، وكلها مقصودة ومعلَنة:
//!
//! 1. المسح **محدود** بعدد مدخلات وبزمن. مجلد فيه مليون ملف لا يجوز أن يجمّد
//!    الواجهة عند كل ضغطة مفتاح، فنتوقّف ونقول «التقدير ناقص» بدل أن نكذب أو
//!    نتجمّد.
//! 2. ZIP **يضغط**، فحجم الأرشيف أصغر من الشجرة عادةً — ونحن نقارن حجم الشجرة
//!    بالمساحة الحرة، فالتحذير متحفّظ.
//! 3. المساحة الحرة **تتغيّر** بين التخطيط والتنفيذ.
//!
//! لذلك ناتج هذا الملف يصل الواجهة تحذيرًا يقرؤه المستخدم، ولا يمنع التنفيذ.
//! ما يمنع التنفيذ هو فشل `ditto` الفعلي إن نفدت المساحة.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// سقف المدخلات الممسوحة. أعلى بكثير من مجلد مستخدم معتاد، وأقلّ بكثير من
/// شجرة تجمّد الواجهة.
const MAX_ENTRIES: usize = 50_000;
/// سقف زمن المسح. التخطيط يجري عند كل تغيير في النموذج، فالتأخير محسوس.
const MAX_TIME: Duration = Duration::from_millis(400);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeEstimate {
    /// مجموع أحجام الملفات العادية بالبايت.
    pub bytes: u64,
    pub entries: usize,
    /// `false` إن أوقفنا المسح عند حدّ، أو تعذّرت قراءة جزء من الشجرة.
    pub complete: bool,
}

/// يمسح شجرة بلا استدعاء ذاتي (‏مكدّس صريح): شجرة عميقة جدًا كانت ستُفجّر
/// مكدّس البرنامج، وهي مدخل يتحكّم به المستخدم.
///
/// لا يتبع الروابط الرمزية ولا يحسب أحجامها، تمامًا كما تفعل `ditto` حين
/// تجتاز الشجرة: هي تحفظ الرابط نفسه ولا تنسخ هدفه.
pub fn tree_size(root: &Path) -> SizeEstimate {
    let start = Instant::now();
    let mut bytes: u64 = 0;
    let mut entries: usize = 0;
    let mut complete = true;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];

    'walk: while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else {
            // مجلد لا نستطيع قراءته: التقدير ناقص، والضغط قد يفشل عليه لاحقًا.
            complete = false;
            continue;
        };
        for entry in read {
            let Ok(entry) = entry else {
                complete = false;
                continue;
            };
            entries += 1;
            if entries >= MAX_ENTRIES || start.elapsed() >= MAX_TIME {
                complete = false;
                break 'walk;
            }
            // `file_type` على `DirEntry` لا يتبع الروابط.
            let Ok(kind) = entry.file_type() else {
                complete = false;
                continue;
            };
            if kind.is_dir() {
                stack.push(entry.path());
            } else if kind.is_file() {
                match entry.metadata() {
                    Ok(m) => bytes = bytes.saturating_add(m.len()),
                    Err(_) => complete = false,
                }
            }
        }
    }

    SizeEstimate { bytes, entries, complete }
}

/// المساحة المتاحة **للمستخدم العادي** على نظام الملفات الذي يحوي `dir`.
///
/// `f_bavail` لا `f_bfree`: الثاني يشمل كتلًا محجوزة للجذر لا نستطيع الكتابة
/// فيها، فاستعماله يعِد بمساحة لا توجد.
pub fn available_bytes(dir: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(dir.as_os_str().as_bytes()).ok()?;
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    // SAFETY: المؤشّر صالح ومنتهٍ بصفر، والبنية مملوكة لنا ومصفّرة.
    if unsafe { libc::statvfs(c_path.as_ptr(), &mut st) } != 0 {
        return None;
    }
    Some((st.f_bavail as u64).saturating_mul(st.f_frsize as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flat_tree_is_measured_exactly() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a"), vec![0u8; 100]).unwrap();
        std::fs::write(dir.path().join("ملف عربي"), vec![0u8; 250]).unwrap();

        let e = tree_size(dir.path());
        assert_eq!(e.bytes, 350);
        assert_eq!(e.entries, 2);
        assert!(e.complete);
    }

    #[test]
    fn nested_directories_are_included() {
        let dir = tempfile::tempdir().unwrap();
        let deep = dir.path().join("مجلد/فرعي");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("f"), vec![0u8; 500]).unwrap();

        assert_eq!(tree_size(dir.path()).bytes, 500);
    }

    #[test]
    fn symlinks_are_not_followed_and_not_counted() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        std::fs::write(&real, vec![0u8; 1000]).unwrap();
        std::os::unix::fs::symlink(&real, dir.path().join("link")).unwrap();

        // الملف الحقيقي مرّة واحدة، والرابط لا يضيف حجمًا.
        assert_eq!(tree_size(dir.path()).bytes, 1000);
    }

    #[test]
    fn a_symlink_loop_terminates_instead_of_hanging() {
        let dir = tempfile::tempdir().unwrap();
        let inner = dir.path().join("inner");
        std::fs::create_dir(&inner).unwrap();
        // رابط يشير إلى جدّه: اجتيازٌ يتبع الروابط كان سيدور إلى الأبد.
        std::os::unix::fs::symlink(dir.path(), inner.join("loop")).unwrap();

        let e = tree_size(dir.path());
        assert!(e.complete, "a symlink loop must not exhaust the scan budget");
    }

    #[test]
    fn an_empty_directory_measures_zero() {
        let dir = tempfile::tempdir().unwrap();
        let e = tree_size(dir.path());
        assert_eq!(e.bytes, 0);
        assert!(e.complete);
    }

    #[test]
    fn an_unreadable_path_is_reported_as_incomplete_not_as_zero() {
        let e = tree_size(Path::new("/definitely/not/a/real/path/xyz"));
        assert_eq!(e.bytes, 0);
        assert!(!e.complete, "a failed scan must not masquerade as an empty folder");
    }

    #[test]
    fn free_space_is_readable_and_plausible() {
        let dir = tempfile::tempdir().unwrap();
        let free = available_bytes(dir.path()).expect("statvfs must work on a temp dir");
        assert!(free > 0, "a writable temp dir on a full disk is not a case we can test around");
    }

    #[test]
    fn free_space_of_a_missing_path_is_unknown_not_zero() {
        assert_eq!(available_bytes(Path::new("/definitely/not/a/real/path/xyz")), None);
    }
}
