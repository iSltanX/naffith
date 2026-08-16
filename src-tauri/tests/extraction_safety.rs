//! الخروج من جذر الاستخراج — مقيسًا على أرشيفاتٍ خبيثة حقيقية.
//!
//! ## لماذا هذا الملف موجود
//!
//! `compress_tar_extract.rs` يقول في رأسه إن `bsdtar` «تُسقط الشرطة الأولى من
//! كل مسار مطلق وترفض أي مدخلة يحوي مسارها `..`»، وإن هذا **مقيسٌ لا منقولٌ
//! عن التوثيق». تعليقٌ يقول ذلك بلا اختبارٍ يسنده هو ادّعاء. هنا يُبنى الأرشيف
//! الخبيث فعلًا، ويُستخرج فعلًا، ويُفحص القرص بعده.
//!
//! والقيمة الحقيقية في المستقبل لا في اليوم: إن غيّرت أداةُ نظامٍ سلوكَها في
//! تحديثٍ قادم، سقط هذا الاختبار قبل أن يقع الضرر على ملفات أحد.
//!
//! ## وما يفحصه إلى جانب ذلك
//!
//! أن الحارس الثاني يعمل مستقلًّا عن الأول: الاستخراج يقع داخل مجلدٍ مؤقّت
//! نملكه، فحتى لو أفلتت مدخلةٌ من الأداة فجذرُها مجلدٌ لا شيء فيه للمستخدم.

use naffith_core::archive;
use naffith_core::planner;
use naffith_core::plans::PlanStore;
use naffith_core::policy::Policy;
use naffith_core::value::RawValue;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// مساحة اختبار داخل المنزل: `/var` خارج الجذور المسموحة، فاختبارٌ يريد أن
/// يمرّ بالسياسة الحقيقية يحتاج موضعًا مسموحًا.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Option<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let base = home.join(format!(".naffith-escape-{tag}-{}", std::process::id()));
        std::fs::remove_dir_all(&base).ok();
        std::fs::create_dir_all(&base).ok()?;
        Some(Scratch(base.canonicalize().ok()?))
    }
    fn path(&self) -> &Path {
        &self.0
    }
    fn dir(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// يبني أرشيف TAR بترويسات `ustar` مكتوبة بيدنا.
///
/// **بيدنا لا بـ`tar`**: أداةٌ سليمة ترفض أن تكتب مدخلةً اسمها `../../x`، وهي
/// بالضبط المدخلة التي نريد اختبار الاستخراج عليها. الترويسة ٥١٢ بايتًا
/// موثَّقة منذ POSIX.1-1988، وكتابتها هنا أصدق من افتراض وجود أداةٍ خبيثة.
fn malicious_tar(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, body) in entries {
        let mut header = [0u8; 512];
        let n = name.as_bytes();
        header[..n.len()].copy_from_slice(n);
        // الوضع، والمالك، والمجموعة — قيمٌ ثمانية بصيغة نصّية.
        header[100..107].copy_from_slice(b"0000644");
        header[108..115].copy_from_slice(b"0000000");
        header[116..123].copy_from_slice(b"0000000");
        let size = format!("{:011o}", body.len());
        header[124..135].copy_from_slice(size.as_bytes());
        header[136..147].copy_from_slice(b"00000000000");
        header[156] = b'0'; // ملف عادي
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");

        // مجموع التحقّق يُحسب والحقل مملوء فراغات، ثم يُكتب مكانها.
        header[148..156].copy_from_slice(b"        ");
        let sum: u32 = header.iter().map(|b| *b as u32).sum();
        let checksum = format!("{sum:06o}\0 ");
        header[148..156].copy_from_slice(checksum.as_bytes());

        out.extend_from_slice(&header);
        out.extend_from_slice(body);
        let padding = (512 - body.len() % 512) % 512;
        out.extend(std::iter::repeat(0u8).take(padding));
    }
    // كتلتان صفريتان تُنهيان الأرشيف.
    out.extend(std::iter::repeat(0u8).take(1024));
    out
}

fn extract_with_tar(archive: &Path, into: &Path) -> std::process::Output {
    std::process::Command::new("/usr/bin/tar")
        .arg("-x")
        .arg("-f")
        .arg(archive)
        .arg("-C")
        .arg(into)
        .output()
        .expect("tar must be runnable")
}

// ── TAR: الحارس من الأداة، مقيسًا ──────────────────────────────────────

#[test]
fn tar_refuses_an_entry_that_climbs_above_the_extraction_root() {
    let Some(s) = Scratch::new("tar-climb") else { return };
    let victim = s.path().join("ثمين.txt");
    std::fs::write(&victim, b"PRECIOUS USER DATA").unwrap();
    let root = s.dir("جذر");

    // `جذر/../ثمين.txt` تصعد درجةً واحدة بالضبط إلى الضحية.
    let archive = s.path().join("خبيث.tar");
    std::fs::write(&archive, malicious_tar(&[("../ثمين.txt", b"OVERWRITTEN")])).unwrap();

    extract_with_tar(&archive, &root);

    assert_eq!(
        std::fs::read(&victim).unwrap(),
        b"PRECIOUS USER DATA",
        "bsdtar must refuse `..` — if this fails, the tool changed and \
         compress_tar_extract's stated guarantee is no longer true"
    );
    assert!(!root.join("ثمين.txt").exists(), "and nothing may land inside the root either");
}

#[test]
fn tar_strips_the_leading_slash_of_an_absolute_entry() {
    let Some(s) = Scratch::new("tar-abs") else { return };
    let root = s.dir("جذر");

    // مسارٌ مطلق داخل مساحتنا: لو لم تُسقط الشرطة لكُتب هناك.
    let outside = s.path().join("مطلق.txt");
    let name = outside.to_string_lossy().into_owned();
    let archive = s.path().join("مطلق.tar");
    std::fs::write(&archive, malicious_tar(&[(&name, b"ABSOLUTE")])).unwrap();

    extract_with_tar(&archive, &root);

    assert!(
        !outside.exists(),
        "bsdtar must not honour an absolute path without -P; the guarantee has changed"
    );
    // والمسار يُعاد نسبيًا **داخل** الجذر: هذا ما يعنيه إسقاط الشرطة.
    let landed = root.join(name.trim_start_matches('/'));
    assert!(landed.exists(), "the entry should land under the root instead: {landed:?}");
}

#[test]
fn an_ordinary_tar_still_extracts_completely() {
    // حارسٌ على الحارسين: لو صار `tar` يرفض كل شيء لمرّ الاختباران أعلاه
    // ومرّت معهما عمليةٌ لا تعمل.
    let Some(s) = Scratch::new("tar-ok") else { return };
    let root = s.dir("جذر");
    let archive = s.path().join("سليم.tar");
    std::fs::write(&archive, malicious_tar(&[("مجلد/ملف.txt", b"hello"), ("آخر.txt", b"bye")]))
        .unwrap();

    extract_with_tar(&archive, &root);

    assert_eq!(std::fs::read(root.join("مجلد/ملف.txt")).unwrap(), b"hello");
    assert_eq!(std::fs::read(root.join("آخر.txt")).unwrap(), b"bye");
}

// ── ZIP: الحارس منّا، قبل التشغيل ──────────────────────────────────────

/// يبني أرشيف ZIP بأسماء نختارها. نسخةٌ من الباني في `testkit`، لأن ذاك
/// `#[cfg(test)]` داخل المكتبة ولا يبلغه اختبارُ تكامل.
fn zip_with(names: &[&str]) -> Vec<u8> {
    const CENTRAL_SIG: u32 = 0x0201_4b50;
    const LOCAL_SIG: u32 = 0x0403_4b50;
    const EOCD_SIG: u32 = 0x0605_4b50;

    let mut out = Vec::new();
    let mut central = Vec::new();
    let payload = b"x";

    for name in names {
        let local_offset = out.len() as u32;
        let n = name.as_bytes();
        out.extend_from_slice(&LOCAL_SIG.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&(n.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(n);
        out.extend_from_slice(payload);

        central.extend_from_slice(&CENTRAL_SIG.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u32.to_le_bytes());
        central.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        central.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        central.extend_from_slice(&(n.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u32.to_le_bytes());
        central.extend_from_slice(&local_offset.to_le_bytes());
        central.extend_from_slice(n);
    }

    let cd_offset = out.len() as u32;
    let cd_size = central.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&EOCD_SIG.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(names.len() as u16).to_le_bytes());
    out.extend_from_slice(&(names.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

fn plan_zip_extract(archive: &Path, destination: &Path, name: &str) -> Result<(), &'static str> {
    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let inputs = BTreeMap::from([
        ("archive".to_owned(), RawValue::Path(archive.display().to_string())),
        ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
        ("folder_name".to_owned(), RawValue::Text(name.to_owned())),
    ]);
    planner::plan(&mut store, &session, Policy::production(), "compress.zip.extract", &inputs)
        .map(|_| ())
        .map_err(|e| e.key())
}

#[test]
fn a_zip_that_escapes_is_refused_at_plan_time_so_nothing_ever_runs() {
    // الفرق عن TAR كلّه هنا: الرفض يقع **قبل** أن تُطلق أداة. لا تُكتب بايتة
    // واحدة، ولا يُنشأ مجلد مؤقّت، ولا يرى المستخدم شاشة تشغيل.
    let Some(s) = Scratch::new("zip-slip") else { return };
    let dest = s.dir("وجهة");
    let archive = s.path().join("خبيث.zip");
    std::fs::write(&archive, zip_with(&["ok.txt", "../../../../etc/passwd"])).unwrap();

    assert_eq!(plan_zip_extract(&archive, &dest, "مستخرَج"), Err("err.archive.escapes"));
    assert_eq!(
        std::fs::read_dir(&dest).unwrap().count(),
        0,
        "a refused plan must leave the destination untouched"
    );
}

#[test]
fn an_absolute_zip_entry_is_refused_the_same_way() {
    let Some(s) = Scratch::new("zip-abs") else { return };
    let dest = s.dir("وجهة");
    let archive = s.path().join("مطلق.zip");
    std::fs::write(&archive, zip_with(&["/etc/passwd"])).unwrap();

    assert_eq!(plan_zip_extract(&archive, &dest, "مستخرَج"), Err("err.archive.escapes"));
}

#[test]
fn a_wholesome_zip_plans_normally() {
    // الحارس على الحارس مرّةً أخرى: لو رفض الماسح كل شيء لمرّ ما فوقه.
    let Some(s) = Scratch::new("zip-ok") else { return };
    let dest = s.dir("وجهة");
    let archive = s.path().join("سليم.zip");
    std::fs::write(&archive, zip_with(&["a.txt", "مجلد/ب.txt"])).unwrap();

    assert_eq!(plan_zip_extract(&archive, &dest, "مستخرَج"), Ok(()));
}

#[test]
fn the_scanner_and_the_planner_agree_on_what_escapes() {
    // الماسح دالّةٌ نقيّة يمكن اختبارها وحدها، والمخطِّط هو من يستدعيها.
    // الاتفاق بينهما ليس مفروغًا منه: عمليةٌ تنسى `guard_extraction` تمرّ.
    let Some(s) = Scratch::new("zip-agree") else { return };
    let dest = s.dir("وجهة");
    for (names, escapes) in [
        (vec!["a.txt"], false),
        (vec!["../a.txt"], true),
        (vec!["ok/../../a.txt"], true),
        (vec!["..hidden.txt"], false),
    ] {
        let archive = s.path().join("عيّنة.zip");
        std::fs::write(&archive, zip_with(&names)).unwrap();

        let scan = archive::scan_zip(&archive).unwrap();
        assert_eq!(scan.escaping, escapes, "scanner disagrees about {names:?}");

        let planned = plan_zip_extract(&archive, &dest, "مستخرَج");
        assert_eq!(
            planned.is_err(),
            escapes,
            "the planner must refuse exactly what the scanner flags: {names:?}"
        );
    }
}
