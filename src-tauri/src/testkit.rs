//! مساحة اختبارٍ داخل المنزل.
//!
//! `tempfile` يضع مجلداته تحت `/var`، وهو **خارج الجذور المسموحة** التي
//! يعلنها `paths.rs`. فاختبارٌ يريد أن يمرّ بالسياسة الحقيقية — لا أن
//! يتجاوزها — يحتاج موضعًا مسموحًا فعلًا، وهو المنزل.
//!
//! والاسم يحمل لاحقةً عشوائية لا اسمًا ثابتًا: الاختبارات تتوازى، ومشغّلان
//! متزامنان للطقم (أو `cargo test` مرتين في آن) كانا يريان أحدهما بقايا
//! الآخر فيسقط اختبارٌ على شيء ليس عيبًا في الشيفرة.
//!
//! `#[cfg(test)]` على مستوى الوحدة في `lib.rs`: لا يدخل البناء الموزَّع.

use std::path::{Path, PathBuf};

pub struct Scratch(PathBuf);

impl Scratch {
    /// مساحةٌ جديدة، أو `None` إن لم يكن `HOME` مضبوطًا.
    pub fn new(tag: &str) -> Option<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let base = home.join(format!(".naffith-test-{tag}-{}", crate::plans::random_suffix()));
        std::fs::create_dir_all(&base).ok()?;
        Some(Scratch(base.canonicalize().ok()?))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// مجلدٌ فرعي، يُنشأ بكل آبائه.
    pub fn dir(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    /// ملفٌ بمحتوًى، ويُنشأ مجلده الحاوي إن لزم.
    pub fn file(&self, name: &str, contents: &[u8]) -> PathBuf {
        let p = self.0.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, contents).unwrap();
        p
    }

    /// أسماء ما داخل مجلد، مرتّبة. لإثبات أن التخطيط لم يخلّف شيئًا.
    pub fn names(&self, dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        names.sort();
        names
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// يبني أرشيف ZIP حقيقيًا بأسماء نختارها، بلا اعتمادٍ خارجي.
///
/// المدخلات مخزَّنة بلا ضغط (‏method 0) ومحتواها بايتٌ واحد: الغرض اختبارُ ما
/// يقرأ **الفهرس** — حارس Zip Slip، وعمليات الفحص والقائمة — لا فكُّ ضغطٍ.
///
/// وبناؤه هنا لا بـ`/usr/bin/zip`: أرشيفٌ بمدخلةٍ اسمها `../../etc/passwd` لا
/// تصنعه أداةٌ سليمة أصلًا، وهو بالضبط ما نحتاج اختباره.
pub fn zip_with(names: &[&str]) -> Vec<u8> {
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
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
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
