//! قراءة فهرس الأرشيف **قبل** استخراجه.
//!
//! ## لماذا نقرأ الأرشيف بأنفسنا بدل أن نثق بالأداة
//!
//! «‏Zip Slip» هجومٌ بسيط: مدخلةٌ في الأرشيف اسمها `../../../../etc/x` أو
//! `/Users/…/x`. أداةُ استخراجٍ ساذجة تضمّ الاسم إلى جذر الاستخراج وتكتب خارجه.
//! أدوات macOS الحديثة تدافع عن نفسها — و`ditto -x` منها — لكن بناء منتجٍ على
//! **افتراضِ** ذلك الدفاع يعني أن كل تغيّرٍ في سلوك أداة النظام غدًا يصير ثغرة
//! في هذا التطبيق، ولن يكسر شيءٌ حتى يقع الضرر.
//!
//! هنا نقرأ الفهرس المركزي بأنفسنا ونرفض قبل أن يُطلق شيء. الرفض **قبل**
//! التشغيل لا بعده هو الفرق كلّه: أداةٌ تكتب ثم نكتشف الخروج تكون قد كتبت.
//!
//! ## ولماذا لا مكتبة ZIP
//!
//! لسنا نفكّ الضغط — لا نحتاج DEFLATE ولا CRC ولا شيئًا من ذلك. نحتاج أسماء
//! المدخلات وحدها، وهي في الفهرس المركزي نصًّا صريحًا. اعتمادٌ كامل مقابل
//! مئتي سطر تقرأ بنيةً موثَّقة منذ ١٩٨٩ لا يستحقّ — والاعتماد نفسه سطحُ هجوم.
//!
//! ## ما يقرؤه هذا الملف وما لا يقرؤه
//!
//! يقرأ: الفهرس المركزي (‏ZIP و Zip64)، أسماء المدخلات، أحجامها غير المضغوطة،
//! وبتات النوع في سمات يونكس الخارجية.
//!
//! لا يقرأ: بيانات المدخلات. أي أن **هدف** الرابط الرمزي غير معروف هنا — هو
//! في جسم المدخلة لا في فهرسها. لذلك الروابط تُعلَن تحذيرًا لا رفضًا، ويبقى
//! الحارس الحقيقي أن الاستخراج يقع في مجلدٍ مؤقّت نملكه ثم يُرقّى كتلةً واحدة.

use crate::error::{CoreError, Result};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// أقصى عدد مدخلات نقرأ أسماءها. أرشيفٌ أكبر يُفحص جزئيًا ويُعلَن ذلك.
///
/// السقف يحمي من أرشيفٍ مُصاغ ليستهلك الذاكرة وقت التخطيط — والتخطيط يقع مع
/// كل ضغطة مفتاح في النموذج.
pub const MAX_SCANNED_ENTRIES: usize = 20_000;

/// أقصى طول اسم مدخلة نقبله. أطول من أي مسارٍ مشروع.
const MAX_ENTRY_NAME: usize = 4_096;

/// أقصى حجمٍ نقرؤه بحثًا عن نهاية الفهرس المركزي: ٦٤ ك.ب للتعليق + ترويسته.
const EOCD_SEARCH_WINDOW: usize = 66_000;

const EOCD_SIG: u32 = 0x0605_4b50;
const EOCD64_LOCATOR_SIG: u32 = 0x0706_4b50;
const EOCD64_SIG: u32 = 0x0606_4b50;
const CENTRAL_SIG: u32 = 0x0201_4b50;

/// ما استُخرج من فهرس الأرشيف.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ArchiveScan {
    pub entries: usize,
    /// مدخلةٌ تخرج من جذر الاستخراج: مسار مطلق، أو `..` في أحد مكوّناته.
    pub escaping: bool,
    /// مدخلةٌ سمات يونكس فيها تقول إنها رابط رمزي.
    pub symlinks: bool,
    /// مجموع الأحجام غير المضغوطة كما يعلنها الفهرس.
    ///
    /// **ما يعلنه الأرشيف، لا ما سيُكتب فعلًا.** أرشيفٌ يكذب في هذا الحقل
    /// موجود (‏«zip bomb»)، فالرقم تقديرٌ يُعرض ولا يُبنى عليه قرار أمني.
    pub uncompressed_bytes: u64,
    /// بلغ المسحُ سقفه، فالأرقام أعلاه حدٌّ أدنى لا مجموع.
    pub truncated: bool,
}

impl ArchiveScan {
    /// يرفض الاستخراج إن كان في الأرشيف ما يخرج من جذره.
    ///
    /// يُستدعى في `plan` لا في المنفّذ: الرفض قبل أن يُطلق شيء.
    pub fn guard_extraction(&self, field: &'static str) -> Result<()> {
        // الشرطان يشتركان في الجواب عمدًا: «رأيتُ خروجًا» و«لم أرَ كل شيء»
        // كلاهما «لا أستطيع أن أضمن الاحتواء»، والفشل مغلقٌ في الحالين. مسحٌ
        // ناقص لا يجوز أن يُقرأ «نظيف»: ما لم يُفحص قد يكون هو الخبيث.
        if self.escaping || self.truncated {
            return Err(CoreError::ArchiveEscapes.on_input(field));
        }
        Ok(())
    }

    /// تحذيراتٌ تُعرض قبل التنفيذ ولا تمنعه.
    pub fn warnings(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.symlinks {
            out.push("warn.archive.symlinks");
        }
        if self.entries == 0 {
            out.push("warn.archive.empty");
        }
        out
    }
}

/// هل يخرج هذا الاسم من جذر الاستخراج؟
///
/// ثلاث حالات، وكلٌّ منها يقع فعلًا في أرشيفاتٍ في البريّة:
///
/// * **مسار مطلق** (`/etc/x`) — يتجاوز الجذر كليًا.
/// * **`..` في أي مكوّن** — يصعد فوقه. الفحص على المكوّنات لا على النصّ:
///   `..foo` اسمٌ مشروع، و`a/../../b` ليس كذلك، ومطابقةُ النصّ تخلط بينهما.
/// * **بادئة قرص ويندوز** (`C:\x`) — أرشيفاتٌ صُنعت على ويندوز تحملها، وهي
///   مطلقةٌ هناك وإن بدت نسبيةً هنا.
///
/// و`\` تُعامل فاصلًا **إلى جانب** `/`: على macOS هي محرفٌ مشروع في الاسم، لكن
/// أرشيفًا كُتب على ويندوز يستعملها فاصلًا، وأداةَ استخراجٍ قد تفعل الشيء نفسه.
/// المعاملةُ على أنها فاصل تجعلنا نرفض `..\..\x` — وأسوأ ما تكلّفه رفضُ اسمٍ
/// غريبٍ يحوي `\..\` حرفيًا، وهو ثمنٌ زهيد أمام كتابةٍ خارج الجذر.
pub fn name_escapes(name: &str) -> bool {
    if name.is_empty() {
        // اسمٌ فارغ ليس مدخلةً صالحة؛ نرفض بدل أن نخمّن ما يعنيه.
        return true;
    }
    if name.starts_with('/') || name.starts_with('\\') {
        return true;
    }
    if name.contains('\0') {
        return true;
    }
    // `C:` أو `c:/…`
    let bytes = name.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return true;
    }
    name.split(['/', '\\']).any(|part| part == "..")
}

/// يقرأ فهرس أرشيف ZIP.
pub fn scan_zip(path: &Path) -> Result<ArchiveScan> {
    let mut file = std::fs::File::open(path).map_err(|_| CoreError::ArchiveUnreadable)?;
    let len = file.metadata().map_err(|_| CoreError::ArchiveUnreadable)?.len();
    if len < 22 {
        return Err(CoreError::ArchiveUnreadable);
    }

    // نهاية الفهرس المركزي في آخر الملف، وقد يسبقها تعليقٌ حتى ٦٤ ك.ب.
    let window = EOCD_SEARCH_WINDOW.min(len as usize);
    let start = len - window as u64;
    file.seek(SeekFrom::Start(start)).map_err(|_| CoreError::ArchiveUnreadable)?;
    let mut tail = vec![0u8; window];
    file.read_exact(&mut tail).map_err(|_| CoreError::ArchiveUnreadable)?;

    let eocd = find_last(&tail, EOCD_SIG).ok_or(CoreError::ArchiveUnreadable)?;
    let mut total = u16::from_le_bytes([tail[eocd + 10], tail[eocd + 11]]) as u64;
    let mut cd_offset =
        u32::from_le_bytes([tail[eocd + 16], tail[eocd + 17], tail[eocd + 18], tail[eocd + 19]])
            as u64;

    // Zip64: القيم الحارسة تعني «انظر في السجلّ الموسَّع».
    if total == 0xFFFF || cd_offset == 0xFFFF_FFFF {
        let (t, o) = read_zip64(&tail, &mut file)?;
        total = t;
        cd_offset = o;
    }

    if cd_offset >= len {
        return Err(CoreError::ArchiveUnreadable);
    }

    let mut scan = ArchiveScan::default();
    let budget = total.min(MAX_SCANNED_ENTRIES as u64) as usize;
    scan.truncated = total > MAX_SCANNED_ENTRIES as u64;

    file.seek(SeekFrom::Start(cd_offset)).map_err(|_| CoreError::ArchiveUnreadable)?;
    let mut reader = std::io::BufReader::new(&mut file);
    let mut header = [0u8; 46];

    for _ in 0..budget {
        if reader.read_exact(&mut header).is_err() {
            // فهرسٌ أقصر ممّا يعلنه: أرشيف تالف. نفشل مغلقين.
            return Err(CoreError::ArchiveUnreadable);
        }
        if u32::from_le_bytes([header[0], header[1], header[2], header[3]]) != CENTRAL_SIG {
            return Err(CoreError::ArchiveUnreadable);
        }

        let uncompressed =
            u32::from_le_bytes([header[24], header[25], header[26], header[27]]) as u64;
        let name_len = u16::from_le_bytes([header[28], header[29]]) as usize;
        let extra_len = u16::from_le_bytes([header[30], header[31]]) as usize;
        let comment_len = u16::from_le_bytes([header[32], header[33]]) as usize;
        let external = u32::from_le_bytes([header[38], header[39], header[40], header[41]]);

        if name_len > MAX_ENTRY_NAME {
            return Err(CoreError::ArchiveUnreadable);
        }

        let mut name_bytes = vec![0u8; name_len];
        if reader.read_exact(&mut name_bytes).is_err() {
            return Err(CoreError::ArchiveUnreadable);
        }
        // الأسماء تُكتب UTF-8 أو CP437. التحويل المتساهل يكفي للفحص: ما يهمّنا
        // `/` و`..` و`\` وهي ASCII في الترميزين، ومحرفُ الاستبدال لا يخلقها.
        let name = String::from_utf8_lossy(&name_bytes);
        if name_escapes(&name) {
            scan.escaping = true;
        }

        // النوع في البايتات العليا من سمات يونكس الخارجية.
        const S_IFMT: u32 = 0o170_000;
        const S_IFLNK: u32 = 0o120_000;
        if (external >> 16) & S_IFMT == S_IFLNK {
            scan.symlinks = true;
        }

        scan.entries += 1;
        scan.uncompressed_bytes = scan.uncompressed_bytes.saturating_add(uncompressed);

        let skip = (extra_len + comment_len) as i64;
        if skip > 0 && reader.seek_relative(skip).is_err() {
            return Err(CoreError::ArchiveUnreadable);
        }
    }

    Ok(scan)
}

/// يقرأ سجلّي Zip64 ويعيد (عدد المدخلات، إزاحة الفهرس المركزي).
fn read_zip64(tail: &[u8], file: &mut std::fs::File) -> Result<(u64, u64)> {
    let loc = find_last(tail, EOCD64_LOCATOR_SIG).ok_or(CoreError::ArchiveUnreadable)?;
    if loc + 16 > tail.len() {
        return Err(CoreError::ArchiveUnreadable);
    }
    let eocd64_offset = u64::from_le_bytes(
        tail[loc + 8..loc + 16].try_into().map_err(|_| CoreError::ArchiveUnreadable)?,
    );

    file.seek(SeekFrom::Start(eocd64_offset)).map_err(|_| CoreError::ArchiveUnreadable)?;
    let mut rec = [0u8; 56];
    file.read_exact(&mut rec).map_err(|_| CoreError::ArchiveUnreadable)?;
    if u32::from_le_bytes([rec[0], rec[1], rec[2], rec[3]]) != EOCD64_SIG {
        return Err(CoreError::ArchiveUnreadable);
    }
    let total = u64::from_le_bytes(rec[32..40].try_into().unwrap());
    let cd_offset = u64::from_le_bytes(rec[48..56].try_into().unwrap());
    Ok((total, cd_offset))
}

/// يبحث عن آخر ورودٍ لتوقيعٍ في مخزَن.
///
/// **الأخير** لا الأول: توقيع نهاية الفهرس قد يظهر بالصدفة داخل بياناتٍ مضغوطة
/// أو داخل تعليق، والسجلّ الحقيقي هو آخر ما في الملف.
fn find_last(haystack: &[u8], signature: u32) -> Option<usize> {
    let needle = signature.to_le_bytes();
    if haystack.len() < 4 {
        return None;
    }
    (0..=haystack.len() - 4).rev().find(|&i| haystack[i..i + 4] == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ordinary_relative_name_does_not_escape() {
        for good in [
            "a.txt",
            "folder/a.txt",
            "أ/ب/ج.txt",
            "..foo",
            "foo..",
            "a/..b/c",
            "./a.txt",
            "folder/",
        ] {
            assert!(!name_escapes(good), "{good:?} must be accepted");
        }
    }

    #[test]
    fn every_known_escape_shape_is_caught() {
        for bad in [
            "",
            "/etc/passwd",
            "../../../../etc/passwd",
            "a/../../b",
            "..",
            "a/..",
            "\\windows\\system32",
            "..\\..\\x",
            "C:/Windows/x",
            "c:\\Windows\\x",
            "a\0b",
        ] {
            assert!(name_escapes(bad), "{bad:?} must be refused");
        }
    }

    #[test]
    fn a_component_that_merely_contains_dots_is_not_a_traversal() {
        // مطابقةُ النصّ كانت سترفض هذه، وهي أسماء مشروعة تمامًا.
        assert!(!name_escapes("my..file.txt"));
        assert!(!name_escapes("dir/..hidden/file"));
    }

    /// يبني أرشيف ZIP حقيقيًا بأسماء نختارها، بلا اعتماد خارجي.
    ///
    /// المدخلات مخزَّنة بلا ضغط (method 0) — تكفي لاختبار قارئ الفهرس، وهو
    /// كل ما في هذا الملف.
    fn build_zip(names: &[&str]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        let payload = b"x";

        for name in names {
            let local_offset = out.len() as u32;
            let n = name.as_bytes();

            out.extend_from_slice(&0x0403_4b50u32.to_le_bytes()); // local sig
            out.extend_from_slice(&20u16.to_le_bytes()); // version
            out.extend_from_slice(&0u16.to_le_bytes()); // flags
            out.extend_from_slice(&0u16.to_le_bytes()); // method: stored
            out.extend_from_slice(&0u16.to_le_bytes()); // time
            out.extend_from_slice(&0u16.to_le_bytes()); // date
            out.extend_from_slice(&0u32.to_le_bytes()); // crc (unchecked here)
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            out.extend_from_slice(&(n.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes()); // extra
            out.extend_from_slice(n);
            out.extend_from_slice(payload);

            central.extend_from_slice(&CENTRAL_SIG.to_le_bytes());
            central.extend_from_slice(&20u16.to_le_bytes()); // made by
            central.extend_from_slice(&20u16.to_le_bytes()); // needed
            central.extend_from_slice(&0u16.to_le_bytes()); // flags
            central.extend_from_slice(&0u16.to_le_bytes()); // method
            central.extend_from_slice(&0u16.to_le_bytes()); // time
            central.extend_from_slice(&0u16.to_le_bytes()); // date
            central.extend_from_slice(&0u32.to_le_bytes()); // crc
            central.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            central.extend_from_slice(&(payload.len() as u32).to_le_bytes());
            central.extend_from_slice(&(n.len() as u16).to_le_bytes());
            central.extend_from_slice(&0u16.to_le_bytes()); // extra
            central.extend_from_slice(&0u16.to_le_bytes()); // comment
            central.extend_from_slice(&0u16.to_le_bytes()); // disk
            central.extend_from_slice(&0u16.to_le_bytes()); // internal
            central.extend_from_slice(&0u32.to_le_bytes()); // external
            central.extend_from_slice(&local_offset.to_le_bytes());
            central.extend_from_slice(n);
        }

        let cd_offset = out.len() as u32;
        let cd_size = central.len() as u32;
        out.extend_from_slice(&central);

        out.extend_from_slice(&EOCD_SIG.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // disk
        out.extend_from_slice(&0u16.to_le_bytes()); // cd disk
        out.extend_from_slice(&(names.len() as u16).to_le_bytes());
        out.extend_from_slice(&(names.len() as u16).to_le_bytes());
        out.extend_from_slice(&cd_size.to_le_bytes());
        out.extend_from_slice(&cd_offset.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // comment length
        out
    }

    fn scan_bytes(bytes: &[u8]) -> Result<ArchiveScan> {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.zip");
        std::fs::write(&p, bytes).unwrap();
        scan_zip(&p)
    }

    #[test]
    fn a_wholesome_archive_scans_clean() {
        let scan = scan_bytes(&build_zip(&["a.txt", "مجلد/ب.txt"])).unwrap();
        assert_eq!(scan.entries, 2);
        assert!(!scan.escaping);
        assert!(!scan.truncated);
        assert!(scan.guard_extraction("archive").is_ok());
    }

    #[test]
    fn a_zip_slip_archive_is_refused_before_anything_runs() {
        // الحالة كاملةً: أرشيفٌ سليم الشكل، ومدخلةٌ واحدة فيه تصعد فوق الجذر.
        let scan = scan_bytes(&build_zip(&["ok.txt", "../../../../etc/passwd"])).unwrap();
        assert!(scan.escaping, "the traversal must be seen");
        assert!(matches!(scan.guard_extraction("archive"), Err(CoreError::OnInput { .. })));
        assert_eq!(scan.guard_extraction("archive").unwrap_err().key(), "err.archive.escapes");
    }

    #[test]
    fn an_absolute_entry_is_refused_too() {
        let scan = scan_bytes(&build_zip(&["/etc/passwd"])).unwrap();
        assert!(scan.escaping);
        assert!(scan.guard_extraction("archive").is_err());
    }

    #[test]
    fn a_file_that_is_not_a_zip_is_reported_unreadable_not_clean() {
        // فشلٌ مغلق: «لا أستطيع قراءته» لا «لا شيء فيه».
        let r = scan_bytes(b"this is not a zip archive at all, not even close");
        assert!(matches!(r, Err(CoreError::ArchiveUnreadable)), "got {r:?}");
    }

    #[test]
    fn a_truncated_archive_is_reported_unreadable() {
        let mut bytes = build_zip(&["a.txt", "b.txt"]);
        bytes.truncate(bytes.len() / 2);
        assert!(matches!(scan_bytes(&bytes), Err(CoreError::ArchiveUnreadable)));
    }

    #[test]
    fn an_archive_whose_central_directory_is_a_lie_is_refused() {
        // الفهرس يعلن مدخلتين ولا يحمل إلا واحدة.
        let mut bytes = build_zip(&["a.txt"]);
        let n = bytes.len();
        // عدد المدخلات في EOCD يقع عند الإزاحة 10 و12 من نهايته (22 بايتًا).
        bytes[n - 12] = 2;
        bytes[n - 10] = 2;
        assert!(matches!(scan_bytes(&bytes), Err(CoreError::ArchiveUnreadable)));
    }

    #[test]
    fn an_empty_archive_is_readable_and_says_it_is_empty() {
        let scan = scan_bytes(&build_zip(&[])).unwrap();
        assert_eq!(scan.entries, 0);
        assert!(scan.guard_extraction("archive").is_ok());
        assert!(scan.warnings().contains(&"warn.archive.empty"));
    }

    #[test]
    fn an_archive_with_a_trailing_comment_is_still_found() {
        // التعليق يفصل EOCD عن نهاية الملف، وهو مشروع تمامًا.
        let mut bytes = build_zip(&["a.txt"]);
        let n = bytes.len();
        bytes[n - 2] = 5;
        bytes[n - 1] = 0;
        bytes.extend_from_slice(b"hello");
        let scan = scan_bytes(&bytes).unwrap();
        assert_eq!(scan.entries, 1);
    }

    #[test]
    fn a_scan_that_hit_its_ceiling_refuses_extraction_rather_than_passing_it() {
        // «لم أفحص كل شيء» ليست «كل شيء نظيف». الفشل مغلق.
        let scan = ArchiveScan { entries: 5, truncated: true, ..Default::default() };
        assert!(scan.guard_extraction("archive").is_err());
    }

    #[test]
    fn the_signature_search_takes_the_last_match_not_the_first() {
        // توقيعٌ يظهر بالصدفة داخل بيانات مدخلةٍ لا يجوز أن يُقرأ سجلًّا.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&EOCD_SIG.to_le_bytes());
        bytes.extend_from_slice(&[0u8; 40]);
        bytes.extend_from_slice(&EOCD_SIG.to_le_bytes());
        assert_eq!(find_last(&bytes, EOCD_SIG), Some(44));
    }
}
