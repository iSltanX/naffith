//! حلّ الأدوات التنفيذية.
//!
//! لا يُقرأ `PATH` أبدًا. كل أداة تُعرَّف بمسارها المطلق داخل النظام، ويُتحقَّق
//! منها عند كل تخطيط. السبب مباشر: `PATH` قابل للتأثير من البيئة، ومنتجٌ
//! يشغّل أوامر على ملفات المستخدم لا يجوز أن يسأل البيئة «أين `ditto`؟».

use crate::error::{CoreError, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tool {
    pub id: &'static str,
    /// مسار مطلق داخل نظام التشغيل. لا يُشتق من البيئة.
    pub absolute: &'static str,
}

impl Tool {
    pub const fn new(id: &'static str, absolute: &'static str) -> Self {
        Self { id, absolute }
    }

    /// يتحقّق أن الأداة موجودة وأنها ملف تنفيذي عادي، ويعيد مسارها.
    ///
    /// يُستدعى عند كل تخطيط لا مرة واحدة عند الإقلاع: الغياب بين الإقلاع
    /// والتنفيذ حالة واقعية (تحديث نظام، قرص مفصول).
    pub fn resolve(&self) -> Result<PathBuf> {
        let p = Path::new(self.absolute);
        let meta = std::fs::metadata(p).map_err(|_| CoreError::ToolMissing { id: self.id })?;
        if !meta.is_file() {
            return Err(CoreError::ToolNotExecutable { id: self.id });
        }
        if !is_executable(&meta) {
            return Err(CoreError::ToolNotExecutable { id: self.id });
        }
        Ok(p.to_path_buf())
    }
}

fn is_executable(meta: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

// ── الأدوات المعتمدة ────────────────────────────────────────────────────
//
// كل أداة هنا قرار منتج، لا اختيار عشوائي.

/// أداة النسخ والأرشفة الأصلية على macOS.
///
/// اختيرت على `zip` لأنها جزء من النظام، وتحفظ بيانات macOS الوصفية
/// (‏resource forks و extended attributes) التي يُسقطها `zip`، وتنتج أرشيفًا
/// متوافقًا مع صيغة ZIP القياسية عبر `-c -k --sequesterRsrc`.
/// كونها أقل شهرة لا يجعلها أقل صحّة — ودور «سَطْر» أن يشرح، لا أن يجاري الشائع.
pub const DITTO: Tool = Tool::new("ditto", "/usr/bin/ditto");

/// إظهار ملف في Finder.
///
/// ليست عمليةً في الفهرس ولا تُخطَّط: لا تكتب شيئًا ولا تلمس ملفًا، وكل ما
/// تفعله فتحُ نافذة على مسارٍ **أخرجته النواة بنفسها** من سجلّها. الواجهة لا
/// تستطيع تمرير مسار إليها. انظر `reveal.rs`.
pub const OPEN: Tool = Tool::new("open", "/usr/bin/open");

/// للاختبار الداخلي للسلسلة فقط. لا تدخل فهرس الإنتاج.
pub const ECHO: Tool = Tool::new("echo", "/bin/echo");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_tools_resolve_to_absolute_paths() {
        assert_eq!(DITTO.resolve().unwrap(), Path::new("/usr/bin/ditto"));
        assert_eq!(OPEN.resolve().unwrap(), Path::new("/usr/bin/open"));
        assert_eq!(ECHO.resolve().unwrap(), Path::new("/bin/echo"));
    }

    #[test]
    fn a_missing_tool_is_reported_not_guessed() {
        let ghost = Tool::new("ghost", "/usr/bin/definitely-not-installed-xyz");
        assert!(matches!(ghost.resolve(), Err(CoreError::ToolMissing { id: "ghost" })));
    }

    #[test]
    fn a_directory_is_not_accepted_as_a_tool() {
        let dir = Tool::new("dir", "/usr/bin");
        assert!(matches!(dir.resolve(), Err(CoreError::ToolNotExecutable { id: "dir" })));
    }

    #[test]
    fn no_tool_is_resolved_through_path() {
        // لو اعتمدنا PATH لكان "echo" وحده كافيًا. هذا الاختبار يثبّت القاعدة.
        for t in [DITTO, OPEN, ECHO] {
            assert!(
                Path::new(t.absolute).is_absolute(),
                "tool {} must be declared by absolute path",
                t.id
            );
        }
    }
}
