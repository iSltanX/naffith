//! التعامل مع المسارات.
//!
//! ثلاث قواعد تحكم هذا الملف:
//!
//! 1. **المسار `Path` لا `String`.** أسماء الملفات على macOS بايتات لا محارف؛
//!    المرور بـ `String` يفسد الأسماء العربية والمحارف غير المعتادة ويفتح باب
//!    تحويلات ضائعة. لا يوجد `to_string_lossy` في مسار تنفيذ في هذا الملف.
//!
//! 2. **لا `canonicalize` على شيء لم يُنشأ بعد.** تفشل على غير الموجود. لذلك
//!    نحلّ *أقرب أصل موجود*، ثم نبني بقيّة المسار فوقه بعد رفض `..`.
//!
//! 3. **الجذور المسموحة تُفحص بعد حلّ الروابط الرمزية،** لأن رابطًا داخل
//!    المنزل قد يشير خارجه.

use crate::error::{CoreError, NameRejection, Result};
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// أطول اسم ملف تقبله APFS/HFS+ بالبايتات.
const MAX_NAME_BYTES: usize = 255;

fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).filter(|p| p.is_absolute())
}

/// الجذور التي يُسمح للمنتج بالعمل داخلها في v1.
///
/// المنزل والأقراص المركّبة فقط. عمليات خارجهما (‏`/usr/local` مثلًا) تحتاج
/// صلاحيات مدير، وقد أُجّلت — فلا داعي لفتح الباب قبل وجود آليّة التأكيد.
fn allowed_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(h) = home() {
        // الجذر نفسه يُحلّ، لأن /Users قد يكون رابطًا على بعض الإعدادات.
        roots.push(h.canonicalize().unwrap_or(h));
    }
    let volumes = PathBuf::from("/Volumes");
    if volumes.is_dir() {
        roots.push(volumes);
    }
    roots
}

/// مواضع داخل المنزل لا يلمسها المنتج حتى لو كانت ضمن جذر مسموح.
/// قراءتها وضغطها تسريب، لا خدمة.
const PROTECTED_SUFFIXES: &[&str] = &[
    ".ssh",
    ".gnupg",
    ".aws",
    "Library/Keychains",
    "Library/Application Support/com.apple.TCC",
    "Library/Cookies",
    "Library/Containers/com.apple.Safari",
];

fn is_protected(canonical: &Path) -> bool {
    let Some(h) = home() else { return false };
    let h = h.canonicalize().unwrap_or(h);
    PROTECTED_SUFFIXES.iter().any(|suffix| {
        let guarded = h.join(suffix);
        canonical == guarded || canonical.starts_with(&guarded)
    })
}

fn under_allowed_root(canonical: &Path) -> bool {
    allowed_roots().iter().any(|root| canonical.starts_with(root))
}

fn check_policy(canonical: &Path) -> Result<()> {
    if !under_allowed_root(canonical) {
        return Err(CoreError::PathOutsideAllowedRoots);
    }
    if is_protected(canonical) {
        return Err(CoreError::PathProtected);
    }
    Ok(())
}

/// مجلد قائم، محلول الروابط، ومسموح.
pub fn existing_dir(raw: &Path) -> Result<PathBuf> {
    if !raw.is_absolute() {
        return Err(CoreError::PathNotAbsolute);
    }
    reject_dotdot(raw)?;
    let canonical = raw.canonicalize().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => CoreError::PathMissing,
        _ => CoreError::Io(e),
    })?;
    if !canonical.is_dir() {
        return Err(CoreError::NotADirectory);
    }
    check_policy(&canonical)?;
    Ok(canonical)
}

/// ملف قائم، محلول الروابط، ومسموح.
pub fn existing_file(raw: &Path) -> Result<PathBuf> {
    if !raw.is_absolute() {
        return Err(CoreError::PathNotAbsolute);
    }
    reject_dotdot(raw)?;
    let canonical = raw.canonicalize().map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => CoreError::PathMissing,
        _ => CoreError::Io(e),
    })?;
    if !canonical.is_file() {
        return Err(CoreError::PathMissing);
    }
    check_policy(&canonical)?;
    Ok(canonical)
}

/// مجلد وجهة: قائم، مسموح، وقابل للكتابة فعليًا لا اسميًا.
pub fn target_dir(raw: &Path) -> Result<PathBuf> {
    let canonical = existing_dir(raw)?;
    if !is_writable_dir(&canonical) {
        return Err(CoreError::DestinationNotWritable);
    }
    Ok(canonical)
}

/// البادئة الثابتة لملف فحص الكتابة. الجزء الباقي عشوائي في كل مرة.
/// الاختبارات تبحث بها عن بقايا، فهي ليست تفصيلًا داخليًا بحتًا.
pub const WRITE_PROBE_PREFIX: &str = ".naffith-write-probe-";

/// يحرس ملف الفحص كي يُحذف في كل مسار خروج — بما فيه الذعر.
struct Probe(PathBuf);

impl Drop for Probe {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// يفحص الكتابة بمحاولة فعلية، لا بقراءة بتات الصلاحيات: قد يمنع ACL أو
/// SIP أو قرص للقراءة فقط الكتابةَ رغم أن الوضع يبدو مسموحًا.
///
/// ثلاث خصائص مقصودة:
///
/// * **الاسم عشوائي** (‏64 بت) لا ثابت، فلا يمكن لطرف آخر أن يتوقّعه ويسبقنا
///   إليه، ولا يتصادم فحصان متزامنان.
/// * **`create_new`** يعني أننا لا نكتب فوق شيء قائم أبدًا، ويعني كذلك أن
///   الفشل يساوي «لم نُنشئ ملفًا» — فلا نحذف ملف غيرنا.
/// * **`mode 0o600`** — لا يقرؤه أحد سوى مالكه في اللحظة القصيرة التي يعيشها.
///
/// والتنظيف في `Drop` لا في نهاية الدالة، كي يقع على كل مسار خروج.
///
/// نجاحه **ليس ضمانًا دائمًا**: القرص قد يُفصل أو يمتلئ أو تتغيّر صلاحياته
/// بعد لحظة. لذلك يُعاد الفحص قبل التنفيذ (‏`plans::Preconditions`) وتُعالَج
/// أخطاء الكتابة الفعلية من `ditto` عند وقوعها.
pub(crate) fn is_writable_dir(dir: &Path) -> bool {
    use std::os::unix::fs::OpenOptionsExt;
    let path = dir.join(format!("{WRITE_PROBE_PREFIX}{}", crate::plans::random_suffix()));
    match std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path) {
        Ok(file) => {
            let _probe = Probe(path);
            drop(file);
            true
        }
        // لم يُنشأ شيء، فلا شيء يُحذف.
        Err(_) => false,
    }
}

/// مسار نهائي داخل مجلد وجهة قائم، لملف **لم يُنشأ بعد**.
///
/// هذه هي الحالة التي يخطئ فيها `canonicalize`: لا يمكن حلّ ما لا يوجد. نحلّ
/// المجلد الحاوي (وهو موجود) ثم نضيف الاسم — والاسم مُنقّى ولا يحمل فاصلًا،
/// فلا يمكن أن يخرج بنا من المجلد.
pub fn new_file_in(dir: &Path, name: &OsStr) -> Result<PathBuf> {
    let canonical_dir = target_dir(dir)?;
    // إعادة تأكيد: الاسم المُنقّى لا يحوي فاصلًا، لكن لا نعتمد على المتصل بنا.
    if Path::new(name).components().count() != 1 {
        return Err(CoreError::PathTraversal);
    }
    Ok(canonical_dir.join(name))
}

fn reject_dotdot(p: &Path) -> Result<()> {
    if p.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(CoreError::PathTraversal);
    }
    Ok(())
}

/// ينقّي اسم ملف يكتبه المستخدم.
///
/// يقبل العربية والمسافات ومحارف الصدفة (`;` `&` `$` …) لأن الأمر يُبنى
/// كمصفوفة وسائط ولا يمرّ بمفسّر، فهي محارف عادية لا خطر فيها. ويرفض ما
/// يغيّر *بنية المسار* أو يكسر نظام الملفات.
pub fn sanitize_name(raw: &str) -> Result<String> {
    let name = raw.trim();

    if name.is_empty() {
        return Err(CoreError::InvalidName { reason: NameRejection::Empty });
    }
    if name.len() > MAX_NAME_BYTES {
        return Err(CoreError::InvalidName { reason: NameRejection::TooLong });
    }
    if name.contains('/') {
        return Err(CoreError::InvalidName { reason: NameRejection::ContainsSeparator });
    }
    if name.contains('\0') {
        return Err(CoreError::InvalidName { reason: NameRejection::ContainsNul });
    }
    if name.chars().any(|c| c.is_control()) {
        return Err(CoreError::InvalidName { reason: NameRejection::ContainsControl });
    }
    if name == "." || name == ".." {
        return Err(CoreError::InvalidName { reason: NameRejection::DotOrDotDot });
    }
    if name.starts_with('.') {
        // ملف مخفيّ بالخطأ ناتجٌ لا يجده المستخدم بعد أن ينتظره.
        return Err(CoreError::InvalidName { reason: NameRejection::LeadingDot });
    }
    if raw.ends_with(' ') || raw.ends_with('.') {
        return Err(CoreError::InvalidName { reason: NameRejection::TrailingSpaceOrDot });
    }
    Ok(name.to_owned())
}

/// يضيف امتدادًا إن لم يكن موجودًا. المقارنة غير حسّاسة لحالة الأحرف لأن
/// نظام الملفات الافتراضي على macOS كذلك.
pub fn ensure_extension(name: &str, ext: &str) -> String {
    let suffix = format!(".{ext}");
    if name.to_lowercase().ends_with(&suffix.to_lowercase()) {
        name.to_owned()
    } else {
        format!("{name}{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_arabic_spaces_and_shell_metacharacters() {
        // هذه ليست تساهلًا: الأمر يُبنى argv، فالمحارف حرفية لا مفسَّرة.
        for name in [
            "مجلد المشروع",
            "نسخة احتياطية ٢٠٢٦",
            "a b; rm -rf ~",
            "$(whoami)",
            "back`tick`",
            "emoji 🎯 name",
        ] {
            assert!(sanitize_name(name).is_ok(), "should accept {name:?}");
        }
    }

    #[test]
    fn rejects_structure_breaking_names() {
        let cases: &[(&str, NameRejection)] = &[
            ("", NameRejection::Empty),
            ("   ", NameRejection::Empty),
            ("a/b", NameRejection::ContainsSeparator),
            ("../escape", NameRejection::ContainsSeparator),
            ("..", NameRejection::DotOrDotDot),
            (".", NameRejection::DotOrDotDot),
            (".hidden", NameRejection::LeadingDot),
            ("bad\0name", NameRejection::ContainsNul),
            ("bad\nname", NameRejection::ContainsControl),
        ];
        for (input, expected) in cases {
            match sanitize_name(input) {
                Err(CoreError::InvalidName { reason }) => {
                    assert_eq!(reason, *expected, "for {input:?}")
                }
                other => panic!("expected rejection for {input:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn rejects_name_longer_than_filesystem_allows() {
        let long = "ا".repeat(200); // 400 bytes in UTF-8
        assert!(matches!(
            sanitize_name(&long),
            Err(CoreError::InvalidName { reason: NameRejection::TooLong })
        ));
    }

    #[test]
    fn extension_is_added_once_and_case_insensitively() {
        assert_eq!(ensure_extension("مشروع", "zip"), "مشروع.zip");
        assert_eq!(ensure_extension("مشروع.zip", "zip"), "مشروع.zip");
        assert_eq!(ensure_extension("مشروع.ZIP", "zip"), "مشروع.ZIP");
    }

    #[test]
    fn relative_paths_are_refused() {
        assert!(matches!(
            existing_dir(Path::new("relative/path")),
            Err(CoreError::PathNotAbsolute)
        ));
    }

    #[test]
    fn parent_traversal_is_refused_before_touching_the_filesystem() {
        let p = PathBuf::from("/tmp/../etc");
        assert!(matches!(existing_dir(&p), Err(CoreError::PathTraversal)));
    }

    #[test]
    fn paths_outside_the_allowed_roots_are_refused() {
        // /etc موجود ومقروء، ورفضه سياسة لا حادث.
        let r = existing_dir(Path::new("/etc"));
        assert!(
            matches!(r, Err(CoreError::PathOutsideAllowedRoots)),
            "expected policy rejection, got {r:?}"
        );
    }

    #[test]
    fn new_file_path_is_built_without_canonicalising_the_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        // tempdir يقع تحت /var الذي ليس جذرًا مسموحًا — نتأكد أن السياسة تعمل
        // قبل أي محاولة لحلّ اسم غير موجود.
        let r = new_file_in(dir.path(), OsStr::new("out.zip"));
        assert!(matches!(r, Err(CoreError::PathOutsideAllowedRoots)), "got {r:?}");
    }

    /// يُعيد أسماء بقايا فحص الكتابة داخل مجلد.
    fn probe_leftovers(dir: &Path) -> Vec<String> {
        std::fs::read_dir(dir)
            .map(|rd| {
                rd.flatten()
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .filter(|n| n.starts_with(WRITE_PROBE_PREFIX))
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn the_write_probe_never_survives_a_successful_check() {
        let Some(h) = home() else { return };
        let base = h.join(".naffith-test-probe-success");
        std::fs::create_dir_all(&base).unwrap();

        for _ in 0..20 {
            assert!(is_writable_dir(&base));
        }

        assert_eq!(probe_leftovers(&base), Vec::<String>::new(), "no probe file may survive");
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn the_write_probe_leaves_nothing_when_the_directory_is_read_only() {
        use std::os::unix::fs::PermissionsExt;
        let Some(h) = home() else { return };
        let base = h.join(".naffith-test-probe-readonly");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o500)).unwrap();

        assert!(!is_writable_dir(&base), "a read-only directory must fail the check");

        // نعيد الصلاحيات كي نقرأ ما بداخله ثم ننظّف.
        std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(probe_leftovers(&base), Vec::<String>::new(), "a failed check leaves nothing");
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn probe_names_are_random_not_fixed() {
        // اسم ثابت يعني تصادمًا بين فحصين متزامنين، ويعني أن طرفًا آخر يستطيع
        // توقّعه. نثبت العشوائية بأن الفحص ينجح ألف مرة متتالية بلا تصادم.
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            assert!(seen.insert(crate::plans::random_suffix()), "probe suffix repeated");
        }
    }

    #[test]
    fn a_read_only_directory_is_refused_as_a_destination() {
        use std::os::unix::fs::PermissionsExt;
        let Some(h) = home() else { return };
        let base = h.join(".naffith-test-readonly-dest");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o500)).unwrap();

        let r = target_dir(&base);

        std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::remove_dir_all(&base).ok();
        assert!(matches!(r, Err(CoreError::DestinationNotWritable)), "got {r:?}");
    }

    #[test]
    fn new_file_in_home_resolves_without_the_file_existing() {
        let Some(h) = home() else { return };
        let base = h.join(".naffith-test-scratch");
        std::fs::create_dir_all(&base).unwrap();
        let p = new_file_in(&base, OsStr::new("ملف جديد.zip")).unwrap();
        assert_eq!(p.file_name().unwrap(), OsStr::new("ملف جديد.zip"));
        assert!(!p.exists(), "the planner must not create anything");
        std::fs::remove_dir_all(&base).ok();
    }
}
