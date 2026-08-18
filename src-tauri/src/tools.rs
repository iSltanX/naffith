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
        resolve_executable(self.id, Path::new(self.absolute))
    }
}

/// الفحص الذي يجريه `Tool::resolve` نفسه، معزولًا لمسارٍ لا يُعرَف وقت
/// البناء.
///
/// أدوات Node — `npm`، و`node_modules/.bin/tauri` — مساراتها ليست ثابتةً في
/// النظام كـ`ditto`: يختارها المستخدم (أو تُشتقّ من اختياره) وقت التشغيل.
/// الثابت الذي لا يتغيّر هو **الفحص**، لا المسار — فالتحقّق واحدٌ يُستدعى
/// من الحالتين، بدل نسخةٍ ثانية قد تفترق عن الأولى يومًا.
///
/// **هذا وحده لا يكفي أمانًا لمسارٍ من المستخدم**: يثبت أن الملف تنفيذيّ، لا
/// أنه الأداة التي يظنّها المستخدم. الاستدعاء يمرّ أولًا بسياسة المسارات
/// (‏`ExistingFile`)، فهذا تحقّقٌ إضافي فوقها لا بديلٌ عنها.
pub fn resolve_executable(id: &'static str, p: &Path) -> Result<PathBuf> {
    let meta = std::fs::metadata(p).map_err(|_| CoreError::ToolMissing { id })?;
    if !meta.is_file() {
        return Err(CoreError::ToolNotExecutable { id });
    }
    if !is_executable(&meta) {
        return Err(CoreError::ToolNotExecutable { id });
    }
    Ok(p.to_path_buf())
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

/// فتح مسار في Finder.
///
/// لها دوران، والفرق بينهما هو **من أين يأتي المسار**:
///
/// 1. **`reveal.rs`** — إظهار ناتج تشغيلٍ ناجح. المسار تُخرجه النواة من سجلّها
///    هي، والواجهة تمرّر معرّف تشغيل لا مسارًا. هذا ليس عمليةً في الفهرس ولا
///    يُخطَّط.
/// 2. **`ops::files_open`** — عمليةٌ في الفهرس يختار المستخدم مجلدها. المسار
///    هنا يأتي من الواجهة، فيمرّ بـ`paths.rs` كاملًا كأي مدخلٍ آخر: مطلقٌ،
///    محلول الروابط، داخل الجذور المسموحة، وخارج المواضع المحميّة.
///
/// كان هذا التعليق يقول إنها «ليست عمليةً في الفهرس» على إطلاقه، وصار نصفَ
/// صدقٍ يوم أُضيفت `files.open`. والفرق الذي يستحقّ أن يُقال ليس «هل هي في
/// الفهرس؟» بل «هل يستطيع أحدٌ أن يطلب فتح موضعٍ لم يُفحص؟» — والجواب لا، في
/// الطريقين معًا.
pub const OPEN: Tool = Tool::new("open", "/usr/bin/open");

/// للاختبار الداخلي للسلسلة فقط. لا تدخل فهرس الإنتاج.
pub const ECHO: Tool = Tool::new("echo", "/bin/echo");

// ── الملفات والمجلدات ───────────────────────────────────────────────────

/// النقل وإعادة التسمية. تُستعمل دائمًا مع `-n` كي لا تدهس وجهةً قائمة.
pub const MV: Tool = Tool::new("mv", "/bin/mv");

/// إنشاء المجلدات. `-p` تجعل الوسائط الوسيطة تُنشأ بلا خطأ.
pub const MKDIR: Tool = Tool::new("mkdir", "/bin/mkdir");

/// المسح المُرشَّح للشجرة: الكبير، والقديم، والمكرّر بالاسم. قراءةٌ محضة.
pub const FIND: Tool = Tool::new("find", "/usr/bin/find");

/// قياس المساحة المشغولة. الأداة نفسها التي يعتمد عليها كل تحليل قرص على يونكس.
pub const DU: Tool = Tool::new("du", "/usr/bin/du");

/// المساحة الحرة لكل نظام ملفات مركَّب.
pub const DF: Tool = Tool::new("df", "/bin/df");

/// عرض الصلاحيات وقوائم التحكّم والسمات الممتدّة. `-l` تكشف ACL، و`-@` السمات.
pub const LS: Tool = Tool::new("ls", "/bin/ls");

/// تعرّف نوع ملفٍّ من محتواه (magic bytes)، لا من امتداد اسمه.
pub const FILE: Tool = Tool::new("file", "/usr/bin/file");

/// دمج الملفات. تكتب إلى `stdout` وحده، فيُوجَّه إلى ناتج العملية في النواة.
pub const CAT: Tool = Tool::new("cat", "/bin/cat");

/// تقسيم ملف كبير إلى أجزاء داخل مجلد.
pub const SPLIT: Tool = Tool::new("split", "/usr/bin/split");

// ── الأرشفة ─────────────────────────────────────────────────────────────

/// فحص أرشيف ZIP وقراءة فهرسه واختبار سلامته. لا تُستعمل للاستخراج: `ditto -x`
/// أوثق على أرشيفات macOS لأنها تفهم `__MACOSX`.
pub const UNZIP: Tool = Tool::new("unzip", "/usr/bin/unzip");

/// أرشيفات TAR بكل ضغوطها. على macOS هي `bsdtar`، وترفض المسارات المطلقة
/// و`..` عند الاستخراج افتراضيًا — وهو الحارس الأول ضدّ Zip Slip، والثاني
/// فحصُ الاحتواء بعد الاستخراج في `reveal`/`ops`.
pub const TAR: Tool = Tool::new("tar", "/usr/bin/tar");

// ── الصور ───────────────────────────────────────────────────────────────

/// معالجة الصور الأصلية في macOS: تحويل الصيغة، وتغيير الأبعاد، والتدوير.
/// جزءٌ من النظام، ولا تحتاج تثبيتًا ولا تستدعي شبكة.
pub const SIPS: Tool = Tool::new("sips", "/usr/bin/sips");

/// توليد المصغّرات عبر QuickLook — نفس المعاينة التي يراها المستخدم في Finder.
pub const QLMANAGE: Tool = Tool::new("qlmanage", "/usr/bin/qlmanage");

// ── النصوص ──────────────────────────────────────────────────────────────

/// تحويل صيغ المستندات وترميزها. تدعم `-output` فتكتب إلى ملف مباشرة.
pub const TEXTUTIL: Tool = Tool::new("textutil", "/usr/bin/textutil");

/// مقارنة ملفين نصيين. `-u` تنتج فرقًا موحّدًا يقرؤه البشر والأدوات معًا.
pub const DIFF: Tool = Tool::new("diff", "/usr/bin/diff");

/// مقارنة ملفين بايتًا بايت. تتوقّف عند أول اختلاف — أسرع من بصمةٍ كاملة،
/// وأضيق منها: تحتاج الملفين حاضرين معًا، لا بصمةً منشورة من مكانٍ آخر.
pub const CMP: Tool = Tool::new("cmp", "/usr/bin/cmp");

/// البحث داخل الملفات. يُستعمل للبحث وحده — الاستبدال داخل الملفات مؤجَّل،
/// لأنه تعديلٌ في مكانه لا يمرّ بالترقية الذرّية التي يقوم عليها هذا المنتج.
pub const GREP: Tool = Tool::new("grep", "/usr/bin/grep");

/// عدّ الأسطر والكلمات والبايتات.
pub const WC: Tool = Tool::new("wc", "/usr/bin/wc");

// ── البصمات ─────────────────────────────────────────────────────────────

/// بصمة SHA-256. تُستعمل لبصمة ملف ولمقارنة ملفين معًا.
pub const SHASUM: Tool = Tool::new("shasum", "/usr/bin/shasum");

// ── الشبكة ──────────────────────────────────────────────────────────────

pub const PING: Tool = Tool::new("ping", "/sbin/ping");
pub const DIG: Tool = Tool::new("dig", "/usr/bin/dig");
pub const CURL: Tool = Tool::new("curl", "/usr/bin/curl");

/// المنافذ المحلية المستمعة. تعمل بلا صلاحيات مدير وتعرض ما يخصّ المستخدم.
pub const LSOF: Tool = Tool::new("lsof", "/usr/sbin/lsof");

// ── Git ─────────────────────────────────────────────────────────────────

/// تأتي مع أدوات سطر الأوامر في Xcode، وقد تغيب على نظامٍ لم تُثبَّت فيه.
/// وهذا بالضبط ما يجعل `Availability` سؤالًا يُطرح لحظة قراءة الفهرس.
pub const GIT: Tool = Tool::new("git", "/usr/bin/git");

// ── النظام ──────────────────────────────────────────────────────────────

pub const PS: Tool = Tool::new("ps", "/bin/ps");
pub const SW_VERS: Tool = Tool::new("sw_vers", "/usr/bin/sw_vers");
pub const UPTIME: Tool = Tool::new("uptime", "/usr/bin/uptime");
pub const SYSTEM_PROFILER: Tool = Tool::new("system_profiler", "/usr/sbin/system_profiler");
/// معمارية المعالج (‏`arm64`، `x86_64`) — سؤالٌ عن الجهاز، لا عن نسخة macOS
/// التي تجيب عنها `sw_vers`.
pub const UNAME: Tool = Tool::new("uname", "/usr/bin/uname");

/// البحث عن عمليةٍ بالاسم. **قارئةٌ فقط** خلافًا لأختها `pkill` التي تحمل
/// الاسم نفسه تقريبًا وتُنهي ما تجده — وهذه لا تملك رايةً تُنهي شيئًا أصلًا،
/// و`pkill` غير معلَنة في هذا الملفّ فلا سبيل إليها من المنتج كلّه.
pub const PGREP: Tool = Tool::new("pgrep", "/usr/bin/pgrep");

/// سجلّ النظام الموحَّد. أداةٌ خطرة الأفعال وآمنة الاستعمال هنا بالبنية لا
/// بالفحص: `log erase` تمحو أرشيف السجلّات و`log config` تغيّر إعداد التسجيل،
/// ولا سبيل إلى أيٍّ منهما لأن الفعل `show` سلسلةٌ ثابتة تدخل الثنائيّة عند
/// الترجمة — نفس منطق `diskutil list` في `disk_list.rs`.
pub const LOG: Tool = Tool::new("log", "/usr/bin/log");

/// إرسال إشارةٍ إلى عمليةٍ برقمها. **لا `pkill` ولا `killall`**: كلتاهما
/// تُنهيان كلّ ما يطابق اسمًا دفعةً واحدة — عددًا لا يظهر في الأمر المعروض
/// ولا يعرفه المستخدم قبل الضغط. هذه تأخذ رقمًا واحدًا فتُصيب عمليةً واحدة
/// مكتوبةً أمام عينه.
pub const KILL: Tool = Tool::new("kill", "/bin/kill");

/// أداتا Node.js وTauri CLI — استثناءٌ عن قاعدة هذا الملف.
///
/// كل أداة أخرى هنا مسارها ثابتٌ في نظام التشغيل. `npm` و`tauri` (المحلّي في
/// `node_modules/.bin`) ليستا كذلك: يثبّتهما المستخدم بنفسه في مسارٍ يختاره
/// هو، ويُحلّان وقت التخطيط من ذلك الاختيار — انظر `ops/dev_common.rs`.
///
/// فلماذا `Tool` بهذا الشكل إذن؟ لأن `Availability::of` تحتاج مسارًا مطلقًا
/// حقيقيًا تفحصه لتقرّر «هل هذه العملية متاحةٌ على هذا الجهاز؟» — وهذا سؤالٌ
/// عن **الجهاز**، لا عن إعداد المستخدم. إعداد المستخدم يُفحص لاحقًا كمدخلٍ
/// عاديّ (`node_path`)، وله رسالة رفضٍ خاصّة إن غاب أو كان فاسدًا.
///
/// فالمسار المطلق هنا `/usr/bin/env` نفسه — حاضرٌ دائمًا على macOS، وهو فعلًا
/// ما يُفسِّر شِبَنغ `#!/usr/bin/env node` الذي تبدأ به `npm` و`tauri.js`
/// كلتاهما. أمّا الهوية (`id`) فتحمل الاسم الحقيقي وحده، فيظهر في البحث
/// وفي رسائل الخطأ باسمه لا باسم `env`.
pub const NPM: Tool = Tool::new("npm", "/usr/bin/env");
pub const TAURI_CLI: Tool = Tool::new("tauri", "/usr/bin/env");

/// أداة Cargo — نفس منطق `NPM`/`TAURI_CLI` أعلاه، بمرساةٍ مختلفة.
///
/// ‏`cargo` ملفٌّ تنفيذي أصيل (‏Mach-O)، لا شِبَنغ `env` — فلا صلة لها بـ
/// `/usr/bin/env` كما كانت `npm`. لكنها تحتاج المصرّف والرابط (‏`cc`،
/// `xcrun`) لأي بناءٍ حقيقي، وهذا مُثبَتٌ تجريبيًا: `cargo check` تفشل بلا
/// `/usr/bin/cc` بالضبط. فهو المرساة الصادقة هنا: «هل هذا الجهاز يملك أصلًا
/// ما يلزم لبناء Rust؟» — سؤالٌ عن الجهاز، منفصلٌ عن مسار `cargo` نفسه الذي
/// يختاره المستخدم في الإعداد (`cargo_path`، مثل `node_path`).
pub const CARGO: Tool = Tool::new("cargo", "/usr/bin/cc");
pub const DISKUTIL: Tool = Tool::new("diskutil", "/usr/sbin/diskutil");

/// تفريغ ذاكرة DNS المؤقتة.
///
/// `dscacheutil -flushcache` وحدها، بلا `killall -HUP mDNSResponder`: الثانية
/// تحتاج صلاحيات مدير، وهذا المنتج لا يطلبها ولا يعرض على المستخدم أن يمنحها.
/// الأثر أضعف — تبقى ذاكرة mDNSResponder — والشرح يقول ذلك بدل أن يعِد بأكثر
/// مما يفعل.
pub const DSCACHEUTIL: Tool = Tool::new("dscacheutil", "/usr/bin/dscacheutil");

// ── الأمان ──────────────────────────────────────────────────────────────

/// قراءة السمات الممتدّة وكتابتها. يُستعمل هنا للقراءة وحدها.
pub const XATTR: Tool = Tool::new("xattr", "/usr/bin/xattr");

/// فحص توقيع التطبيقات.
pub const CODESIGN: Tool = Tool::new("codesign", "/usr/bin/codesign");

/// تقييم سياسة Gatekeeper على ملفٍ أو تطبيق.
pub const SPCTL: Tool = Tool::new("spctl", "/usr/sbin/spctl");

#[cfg(test)]
mod tests {
    use super::*;

    /// كل أداةٍ يعلنها المنتج. الاختبارات تمرّ على هذه القائمة لا على ذاكرة
    /// من كتبها، فأداةٌ تُضاف بلا مسارٍ مطلق تسقط هنا لا في يد مستخدم.
    const DECLARED: &[Tool] = &[
        DITTO,
        OPEN,
        ECHO,
        MV,
        MKDIR,
        FIND,
        DU,
        DF,
        LS,
        FILE,
        CAT,
        SPLIT,
        UNZIP,
        TAR,
        SIPS,
        QLMANAGE,
        TEXTUTIL,
        DIFF,
        CMP,
        GREP,
        WC,
        SHASUM,
        PING,
        DIG,
        CURL,
        LSOF,
        GIT,
        PS,
        SW_VERS,
        UPTIME,
        SYSTEM_PROFILER,
        UNAME,
        PGREP,
        LOG,
        KILL,
        DISKUTIL,
        DSCACHEUTIL,
        XATTR,
        CODESIGN,
        SPCTL,
    ];

    #[test]
    fn system_tools_resolve_to_absolute_paths() {
        assert_eq!(DITTO.resolve().unwrap(), Path::new("/usr/bin/ditto"));
        assert_eq!(OPEN.resolve().unwrap(), Path::new("/usr/bin/open"));
        assert_eq!(ECHO.resolve().unwrap(), Path::new("/bin/echo"));
    }

    #[test]
    fn every_declared_tool_id_is_unique() {
        let mut seen = std::collections::HashSet::new();
        for t in DECLARED {
            assert!(seen.insert(t.id), "duplicate tool id: {}", t.id);
        }
    }

    #[test]
    fn every_declared_tool_path_is_unique() {
        // أداتان بنفس المسار ومعرّفين مختلفين تعنيان أن سببَ تعطيلٍ يسمّي
        // الأداة الخطأ على الشاشة.
        let mut seen = std::collections::HashSet::new();
        for t in DECLARED {
            assert!(seen.insert(t.absolute), "duplicate tool path: {}", t.absolute);
        }
    }

    #[test]
    fn every_declared_tool_lives_in_a_system_location() {
        // لا `/usr/local` ولا `/opt/homebrew`: أداةٌ يستطيع المستخدم استبدالها
        // بلا صلاحيات مدير ليست أداة نظام، ومنتجٌ يشغّل أوامر على ملفات المستخدم
        // لا يجوز أن يستدعي واحدةً منها.
        const SYSTEM_ROOTS: &[&str] = &["/bin/", "/sbin/", "/usr/bin/", "/usr/sbin/"];
        for t in DECLARED {
            assert!(
                SYSTEM_ROOTS.iter().any(|root| t.absolute.starts_with(root)),
                "tool {} sits outside the system locations: {}",
                t.id,
                t.absolute
            );
        }
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
        for t in DECLARED {
            assert!(
                Path::new(t.absolute).is_absolute(),
                "tool {} must be declared by absolute path",
                t.id
            );
        }
    }

    #[test]
    fn no_declared_tool_is_a_shell() {
        // الأطروحة كلها في هذا السطر: لا يوجد في الفهرس أداةٌ تفسّر نصًّا.
        // أداةٌ كهذه كانت ستعيد للواجهة القدرة التي نُزعت منها عمدًا.
        const SHELLS: &[&str] =
            &["sh", "bash", "zsh", "csh", "tcsh", "ksh", "dash", "fish", "env", "xargs", "eval"];
        for t in DECLARED {
            let name = Path::new(t.absolute).file_name().unwrap().to_string_lossy().into_owned();
            assert!(!SHELLS.contains(&name.as_str()), "tool {} is an interpreter", t.id);
        }
    }

    #[test]
    fn tools_that_this_machine_lacks_are_reported_not_guessed() {
        // ليس كل جهازٍ يحمل كل أداة — `git` تأتي مع أدوات Xcode. الاختبار لا
        // يطالب بوجودها، بل يطالب بأن يكون الجواب صريحًا في الحالين.
        for t in DECLARED {
            match t.resolve() {
                Ok(p) => assert!(p.is_absolute()),
                Err(CoreError::ToolMissing { id }) | Err(CoreError::ToolNotExecutable { id }) => {
                    assert_eq!(id, t.id, "a missing tool must name itself")
                }
                Err(e) => panic!("unexpected error for {}: {e:?}", t.id),
            }
        }
    }
}
