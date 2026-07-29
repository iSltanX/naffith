//! ضغط مجلد في أرشيف ZIP باستخدام `ditto`.
//!
//! ## لماذا `ditto` لا `zip`
//!
//! كلاهما ينتج ZIP. الفرق أن `ditto` جزء من النظام وتحفظ بيانات macOS الوصفية
//! (‏resource forks و extended attributes) التي يُسقطها `zip` صامتًا. مستخدمٌ
//! يضغط مجلد تصميم ثم يجد الألوان أو الوسوم قد ضاعت لن يعرف أين ضاعت. وهذا
//! أيضًا ما يفعله Finder حين تختار «ضغط».
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/ditto -c -k --sequesterRsrc --keepParent <المصدر> <المؤقّت>
//! ```
//!
//! * `-c` — أنشئ أرشيفًا في مسار الوجهة.
//! * `-k` — اجعله PKZip لا CPIO. بدونها ينتج `.cpio` باسم ينتهي بـ `.zip`.
//! * `--sequesterRsrc` — احفظ البيانات الوصفية في `__MACOSX` داخل الأرشيف،
//!   وهي الطريقة القياسية التي تفهمها أدوات فكّ ZIP على كل النظم.
//! * `--keepParent` — ضع اسم المجلد المصدر مجلدًا أعلى داخل الأرشيف، فلا
//!   ينثر الفكُّ محتوياتِه في المجلد الحالي.
//!
//! الوسيط الأخير هو **الاسم المؤقّت** لا النهائي، لأن الترقية إلى الاسم
//! النهائي لا تقع إلا بعد خروج ناجح. «سَطْر» يعرض هذا كما هو ويشرحه.
//!
//! ## ما لا تستطيعه `ditto`، ولماذا نقوله بدل أن نسكت
//!
//! ثلاثة حدود قِيست على هذا الجهاز بأرشيفات حقيقية، ولا راية في `ditto` تعالج
//! أيًّا منها (راجع `man ditto`: لا Zip64 ولا ترميز أسماء):
//!
//! 1. `--sequesterRsrc` تعامل **كل** ملف اسمه يبدأ بـ `._` سِجلًّا مصاحبًا لا
//!    ملفًا. ملف المستخدم `._notes` لا يظهر في الأرشيف بأي صورة — لا في شجرته
//!    ولا في `__MACOSX` — ويعود الفكّ بشجرة ناقصة. البديل `--norsrc` كان
//!    سيُنقذه ويُسقط بدلًا منه البيانات الوصفية كلها، وهي سبب اختيار `ditto`
//!    على `zip` من الأصل. فالمقايضة مرفوضة، والخسارة تُعلَن قبل التنفيذ.
//! 2. لا Zip64: حجم مصدرٍ يتجاوز ٤ غيغابايت يُكتب مقصوصًا على ٣٢ بت. قيس:
//!    ملف ٥ غ.ب سُجّل حجمه ١٫٠٧٣٫٧٤١٫٨٢٤ (‏٥ غ.ب مقسومة على ٢^٣٢)، و`unzip`
//!    يستخرجه مبتورًا. `ditto -x` وحدها تقرؤه صحيحًا.
//! 3. راية UTF-8 (‏البت ١١) لا تُضبط أبدًا مع أن الأسماء تُكتب UTF-8 خامًا.
//!    فأداةٌ تتبع المواصفة تقرؤها CP437: `unzip -l` يعرض `??????`.
//!
//! لا تمنع أيٌّ من هذه التنفيذ — لكن أرشيفًا يخسر ملفًا أو يُقرأ مشوّهًا عند
//! المستلم أسوأ من تحذيرٍ يُقرأ قبل الضغط.
//!
//! ## سلامة الوسائط
//!
//! المسارات مطلقة ومُحلّة الروابط قبل الوصول إلى هنا، فلا يبدأ أي منها بـ `-`
//! ولا يمكن أن يُقرأ رايةً. اختبارٌ في هذا الملف يثبّت ذلك بدل الاعتماد عليه.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::estimate;
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::time::{Duration, Instant};

pub const ID: &str = "compress.folder.zip";

/// أقصى ما تسعه حقول ZIP الكلاسيكية: أربعة بايتات للحجم والإزاحة.
const ZIP_FIELD_LIMIT: u64 = u32::MAX as u64;

/// أقصى عدد مدخلات في شجرة المصدر قبل أن يبلغ الأرشيف سقف عدّاد ZIP.
///
/// السقف نفسه ٦٥٥٣٥، والنصف لأن `--sequesterRsrc` تضيف مدخلة في `__MACOSX`
/// لكل مدخلة تحمل بيانات وصفية، فعدد مدخلات الأرشيف يبلغ ضعف الشجرة.
const ZIP_ENTRY_LIMIT: usize = u16::MAX as usize / 2;

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.compress.folder.zip.title",
    description_key: "op.compress.folder.zip.description",
    category: Category::Compress,
    // تُنشئ ملفًا جديدًا ولا تمسّ المصدر ولا تستبدل موجودًا.
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::DITTO,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec { id: "source", kind: InputKind::ExistingDir, required: true },
        InputSpec { id: "destination", kind: InputKind::TargetDir, required: true },
        InputSpec {
            id: "archive_name",
            kind: InputKind::NewName { ext: Some("zip") },
            required: true,
        },
    ],
    plan,
};

/// وسيط واحد ومعناه. مصدرٌ واحد تُشتقّ منه `argv` و«سَطْر» معًا، فلا يمكن
/// لأحدهما أن ينحرف عن الآخر بإضافة راية في مكان ونسيانها في الآخر.
struct Arg {
    value: OsString,
    key: Option<&'static str>,
    role: TokenRole,
}

fn flag(text: &'static str, key: &'static str) -> Arg {
    Arg { value: OsString::from(text), key: Some(key), role: TokenRole::Flag }
}

fn path_arg(p: &Path) -> Arg {
    Arg { value: p.as_os_str().to_os_string(), key: None, role: TokenRole::Path }
}

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    // تُحلّ عند كل تخطيط لا مرة واحدة عند الإقلاع: تحديث نظام قد يزيلها.
    let program = tools::DITTO.resolve()?;

    let source = inputs.dir("source")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("archive_name")?;

    // كلا المسارين مُحلّ الروابط هنا، فالمقارنة بينهما مقارنة مواضع حقيقية لا
    // مقارنة نصوص.
    if destination == source || destination.starts_with(source) {
        return Err(CoreError::DestinationInsideSource.on_input("destination"));
    }

    // `new_file_in` يفحص شيئين في نداء واحد: مجلد الوجهة، والاسم النهائي بعد
    // أن أُضيف امتداده. فالنسبة لا يمكن أن تكون واحدة لكليهما.
    //
    // `InvalidName` تخصّ الاسم وحدها — وهي تصل هنا في الحالة التي لا يلتقطها
    // `sanitize_name`: اسمٌ من ٢٥٥ وحدة يُقبل، ثم `.zip` تجعله ٢٥٩ فيرفضه نظام
    // الملفات. نسبتُها إلى `destination` كانت تُبرز مربّع الوجهة، والمستخدم
    // يبدّل مجلدًا سليمًا مرة بعد مرة بينما الحقل الذي يصلح العطب هو الاسم.
    // وما عداها — غير موجود، ليس مجلدًا، خارج الجذور، غير قابل للكتابة — يخصّ
    // الوجهة فعلًا.
    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "archive_name" } else { "destination" };
        e.on_input(field)
    })?;

    // `symlink_metadata` لا يتبع الروابط: رابط معلَّق بالاسم النهائي ما زال
    // تضاربًا. ولا نستبدل ولا ننحّي جانبًا — القرار للمستخدم لا لنا.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("archive_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    // مسحة الأحجام تُغذّي التقدير المعروض وتحذيرات المساحة وحدود ZIP معًا.
    let size = estimate::tree_size(source);
    // ومسحة الأسماء منفصلة عنها لأنها تسأل سؤالًا آخر — أسماءٌ لا أحجام —
    // فلا تستدعي `metadata` لأي مدخلة وتتوقّف فور أن تعرف جوابيها. دمجها في
    // `estimate::tree_size` أنظف، لكنه يوسّع `SizeEstimate` لمن لا يعنيه.
    let names = scan_names(source);

    let plan_args = vec![
        flag("-c", "explain.ditto.create"),
        flag("-k", "explain.ditto.pkzip"),
        flag("--sequesterRsrc", "explain.ditto.sequester"),
        flag("--keepParent", "explain.ditto.keep_parent"),
        path_arg(source),
        path_arg(&temp),
    ];

    debug_assert!(
        plan_args.iter().all(|a| !a.value.as_encoded_bytes().starts_with(b"-") || a.key.is_some()),
        "an argument that is not a declared flag must never begin with a dash"
    );

    let args: Vec<OsString> = plan_args.iter().map(|a| a.value.clone()).collect();

    let mut explain = Vec::with_capacity(plan_args.len() + 1);
    explain.push(
        ExplainToken::new(program.display().to_string(), "explain.ditto.tool")
            .with_role(TokenRole::Tool),
    );
    for a in &plan_args {
        explain.push(ExplainToken {
            token: a.value.to_string_lossy().into_owned(),
            key: a.key,
            role: a.role,
        });
    }

    Ok(PlannedCommand {
        program,
        args,
        // لا مجلد عمل: كل المسارات مطلقة، فلا شيء يُحلّ نسبةً إلى مكان.
        cwd: None,
        explain,
        warnings: warnings_for(&size, &names, inputs, source, destination),
        artifact: Some(Artifact { temp, final_path }),
        estimate: Some(size),
    })
}

/// ما تحتاج تحذيراتُ التوافق أن تعرفه عن أسماء الشجرة.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct NameScan {
    /// وُجد ملف اسمه يبدأ بـ `._`، وهو ما تُسقطه `--sequesterRsrc`.
    appledouble: bool,
    /// وُجد اسم فيه بايت خارج ASCII — أي كل اسم عربي.
    non_ascii: bool,
}

/// سقفا المسح. أقلّ من سقفي `estimate` لأن هذه المسحة تجري بعدها، ولأن
/// التوقّف المبكر يجعل بلوغهما نادرًا أصلًا.
const MAX_SCANNED_NAMES: usize = 50_000;
const MAX_SCAN_TIME: Duration = Duration::from_millis(200);

/// يمسح أسماء الشجرة بحثًا عمّا يفسد الأرشيف عند المستلم.
///
/// بمكدّس صريح لا باستدعاء ذاتي، تمامًا كـ `estimate::tree_size`: عمق الشجرة
/// مدخلٌ يتحكّم به المستخدم. ويتوقّف فور أن يعرف الجوابين، فالشجرة العربية —
/// وهي الغالبة هنا — تُحسم عادةً عند أول مدخلة.
///
/// اسم المجلد المصدر نفسه محسوب: `--keepParent` تجعله جذر الأرشيف، فترميزه
/// يخصّ المستلم كما يخصّه ما تحته.
fn scan_names(root: &Path) -> NameScan {
    let start = Instant::now();
    let mut found = NameScan {
        appledouble: false,
        non_ascii: root.file_name().is_some_and(|n| !n.as_bytes().is_ascii()),
    };
    let mut seen: usize = 0;
    let mut stack = vec![root.to_path_buf()];

    'walk: while let Some(dir) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else { continue };
        for entry in read.flatten() {
            seen += 1;
            if seen >= MAX_SCANNED_NAMES || start.elapsed() >= MAX_SCAN_TIME {
                break 'walk;
            }
            let name = entry.file_name();
            let bytes = name.as_bytes();
            if !bytes.is_ascii() {
                found.non_ascii = true;
            }
            // `file_type` على `DirEntry` لا يتبع الروابط.
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_dir() {
                stack.push(entry.path());
            } else if kind.is_file() && bytes.starts_with(b"._") {
                found.appledouble = true;
            }
            if found.appledouble && found.non_ascii {
                break 'walk;
            }
        }
    }

    found
}

/// تحذيرات تُعرض قبل التنفيذ. **لا تمنع** — تُخبر.
fn warnings_for(
    size: &estimate::SizeEstimate,
    names: &NameScan,
    inputs: &Inputs,
    source: &Path,
    destination: &Path,
) -> Vec<&'static str> {
    let mut warnings = Vec::new();

    // ما اختاره المستخدم ليس ما سيُضغط: رابط رمزي حُلّ إلى موضع آخر. مسموح
    // (السياسة فحصت الموضع المُحلّ) لكن إخفاؤه مفاجأة.
    if let Some(given) = inputs.as_given("source") {
        if Path::new(given) != source {
            warnings.push("warn.source.resolved");
        }
    }
    if let Some(given) = inputs.as_given("destination") {
        if Path::new(given) != destination {
            warnings.push("warn.destination.resolved");
        }
    }

    if size.entries == 0 && size.complete {
        warnings.push("warn.source.empty");
    }
    if !size.complete {
        warnings.push("warn.size.partial");
    }
    match estimate::available_bytes(destination) {
        // مقارنة متحفّظة: ZIP يضغط، فالأرشيف عادةً أصغر من الشجرة. تجاوز
        // الحجم للمساحة الحرة لا يعني الفشل حتمًا — ولذلك هو تحذير.
        Some(free) if size.bytes > free => warnings.push("warn.space.low"),
        None => warnings.push("warn.space.unknown"),
        Some(_) => {}
    }

    // خسارة صامتة، وهي الأسوأ: الملف لا يظهر في الأرشيف ولا في `__MACOSX`،
    // والتشغيل يُقيَّد نجاحًا. من يحذف المصدر بعده لا يجد ما يستعيده.
    if names.appledouble {
        warnings.push("warn.source.appledouble");
    }

    // المقارنة بالشجرة لا بالأرشيف: الأرشيف أصغر، لكن حقل «الحجم قبل الضغط»
    // هو الذي يفيض، وهو حجم الشجرة نفسه. والمسح الناقص لا يُخرِج التحذير —
    // `warn.size.partial` يقول عندها إن الرقم إرشادي.
    if size.bytes > ZIP_FIELD_LIMIT || size.entries > ZIP_ENTRY_LIMIT {
        warnings.push("warn.size.zip_limit");
    }

    if names.non_ascii {
        warnings.push("warn.zip.name_encoding");
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{RawValue, Value};
    use std::collections::BTreeMap;

    /// مساحة اختبار داخل المنزل: `tempfile` يضع مجلداته تحت `/var` وهو خارج
    /// الجذور المسموحة، فاختبار السياسة الحقيقية يحتاج موضعًا مسموحًا.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(tag: &str) -> Option<Self> {
            let home = std::env::var_os("HOME").map(std::path::PathBuf::from)?;
            let base = home.join(format!(".naffith-test-{tag}-{}", crate::plans::random_suffix()));
            std::fs::create_dir_all(&base).ok()?;
            Some(Scratch(base.canonicalize().ok()?))
        }
        fn path(&self) -> &Path {
            &self.0
        }
        fn dir(&self, name: &str) -> std::path::PathBuf {
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

    fn raw(source: &Path, destination: &Path, name: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("source".to_owned(), RawValue::Path(source.display().to_string())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("archive_name".to_owned(), RawValue::Text(name.to_owned())),
        ])
    }

    fn plan_with(source: &Path, destination: &Path, name: &str) -> Result<PlannedCommand> {
        let inputs = crate::value::validate(&SPEC, &raw(source, destination, name))?;
        plan(&inputs)
    }

    /// الرفض كما تراه الواجهة: مفتاح الخطأ والحقل المسؤول عنه.
    ///
    /// أفضل من مطابقة شكل `enum`: هذا هو العقد الذي تعتمد عليه الشاشة فعلًا،
    /// وخطأٌ بلا حقل يظهر في أعلى الصفحة بدل أن يظهر تحت الحقل الذي يخصّه.
    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    #[test]
    fn the_operation_is_visible_in_the_production_catalogue() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("compress must be listed");
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.category, Category::Compress);
        assert_eq!(found.inputs.len(), 3);
    }

    #[test]
    fn the_argv_is_the_documented_ditto_form() {
        let s = Scratch::new("argv").expect("HOME must be set");
        let src = s.dir("المصدر");
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&src, &dst, "أرشيف").unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/ditto"));
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args[0], "-c");
        assert_eq!(args[1], "-k");
        assert_eq!(args[2], "--sequesterRsrc");
        assert_eq!(args[3], "--keepParent");
        assert_eq!(Path::new(&args[4]), src.as_path());
        assert_eq!(args.len(), 6);
        assert!(cmd.cwd.is_none(), "no working directory: every path is absolute");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("explain").expect("HOME must be set");
        let src = s.dir("مصدر");
        let dst = s.dir("وجهة");

        let cmd = plan_with(&src, &dst, "ملف").unwrap();

        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected, "سَطْر must render the executed argv, token for token");

        // كل راية مشروحة، والمسارات بيانات لا شرح لها.
        let explained = cmd.explain.iter().filter(|t| t.key.is_some()).count();
        assert_eq!(explained, 5, "the tool and its four flags each carry an explanation");
        assert_eq!(cmd.explain.iter().filter(|t| t.role == TokenRole::Path).count(), 2);
    }

    #[test]
    fn the_temp_file_lives_in_the_destination_and_the_final_name_is_not_used_yet() {
        let s = Scratch::new("atomic").expect("HOME must be set");
        let src = s.dir("مصدر");
        let dst = s.dir("وجهة");

        let cmd = plan_with(&src, &dst, "ناتج").unwrap();
        let artifact = cmd.artifact.as_ref().unwrap();

        assert_eq!(artifact.temp.parent().unwrap(), dst.as_path(), "must share the filesystem");
        assert_eq!(artifact.final_path, dst.join("ناتج.zip"));
        // الوسيط الأخير هو المؤقّت، لا النهائي.
        assert_eq!(Path::new(cmd.args.last().unwrap()), artifact.temp.as_path());
        assert!(!artifact.temp.exists(), "planning must create nothing");
        assert!(!artifact.final_path.exists());
    }

    #[test]
    fn the_zip_extension_is_added_once_and_never_doubled() {
        let s = Scratch::new("ext").expect("HOME must be set");
        let src = s.dir("م");
        let dst = s.dir("و");

        for (given, expected) in
            [("تقرير", "تقرير.zip"), ("تقرير.zip", "تقرير.zip"), ("تقرير.ZIP", "تقرير.ZIP")]
        {
            let cmd = plan_with(&src, &dst, given).unwrap();
            assert_eq!(
                cmd.artifact.unwrap().final_path.file_name().unwrap(),
                OsStr::new(expected),
                "for input {given:?}"
            );
        }
    }

    #[test]
    fn a_destination_inside_the_source_is_refused() {
        let s = Scratch::new("nested").expect("HOME must be set");
        let src = s.dir("مصدر");
        let dst = s.dir("مصدر/داخل");

        assert_eq!(
            refusal(plan_with(&src, &dst, "ناتج")),
            ("err.dest.inside_source", Some("destination"))
        );
    }

    #[test]
    fn a_destination_equal_to_the_source_is_refused() {
        let s = Scratch::new("same").expect("HOME must be set");
        let dir = s.dir("واحد");
        assert_eq!(
            refusal(plan_with(&dir, &dir, "ناتج")),
            ("err.dest.inside_source", Some("destination"))
        );
    }

    #[test]
    fn an_existing_final_file_stops_the_plan_before_anything_runs() {
        let s = Scratch::new("exists").expect("HOME must be set");
        let src = s.dir("م");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود.zip"), b"PRECIOUS").unwrap();

        // منسوب إلى اسم الأرشيف: هو الحقل الذي يغيّره المستخدم ليحلّ التضارب.
        assert_eq!(
            refusal(plan_with(&src, &dst, "موجود")),
            ("err.dest.exists", Some("archive_name"))
        );
        assert_eq!(
            std::fs::read(dst.join("موجود.zip")).unwrap(),
            b"PRECIOUS",
            "a refused plan must not touch the existing archive"
        );
    }

    #[test]
    fn a_dangling_symlink_at_the_final_name_also_stops_the_plan() {
        let s = Scratch::new("dangling").expect("HOME must be set");
        let src = s.dir("م");
        let dst = s.dir("و");
        std::os::unix::fs::symlink(dst.join("nowhere"), dst.join("رابط.zip")).unwrap();

        assert_eq!(
            refusal(plan_with(&src, &dst, "رابط")),
            ("err.dest.exists", Some("archive_name"))
        );
    }

    #[test]
    fn a_file_as_source_is_refused_because_the_spec_demands_a_directory() {
        let s = Scratch::new("file-src").expect("HOME must be set");
        let dst = s.dir("و");
        let file = s.path().join("ملف.txt");
        std::fs::write(&file, b"x").unwrap();

        assert_eq!(refusal(plan_with(&file, &dst, "ناتج")), ("err.path.not_dir", Some("source")));
    }

    #[test]
    fn an_invalid_archive_name_is_refused() {
        let s = Scratch::new("badname").expect("HOME must be set");
        let src = s.dir("م");
        let dst = s.dir("و");

        for bad in ["", "   ", "a/b", "../خارج", ".مخفي", "."] {
            assert_eq!(
                refusal(plan_with(&src, &dst, bad)),
                ("err.name.invalid", Some("archive_name")),
                "name {bad:?} must be refused, and blamed on the name field"
            );
        }
    }

    #[test]
    fn a_name_that_only_overflows_once_the_extension_is_added_blames_the_name_field() {
        // ٢٥٥ وحدة يقبلها `sanitize_name`، ثم `.zip` تجعلها ٢٥٩ فيرفضها نظام
        // الملفات. الوجهة هنا سليمة تمامًا، والحقل الذي يصلح العطب هو الاسم —
        // والواجهة تُبرز الحقل الذي يسمّيه الخطأ.
        let s = Scratch::new("toolong").expect("HOME must be set");
        let src = s.dir("م");
        let dst = s.dir("و");

        let long = "ا".repeat(255);
        assert_eq!(
            refusal(plan_with(&src, &dst, &long)),
            ("err.name.invalid", Some("archive_name")),
            "an over-long final name is not the destination's fault"
        );
    }

    #[test]
    fn a_destination_that_vanished_after_validation_is_still_blamed_on_the_destination() {
        // الحارس على إعادة النسبة أعلاه: `new_file_in` يفحص الوجهة والاسم
        // معًا، وما تخصّه الوجهة يجب أن يبقى منسوبًا إليها. الوجهة تُحقَّق في
        // `validate` قبل الوصول إلى هنا، فالحالة تُبنى مباشرة كما تقع فعلًا:
        // مجلدٌ اختفى بين التحقّق والتخطيط.
        let s = Scratch::new("destfield").expect("HOME must be set");
        let src = s.dir("م");
        let gone = s.path().join("لا-وجود-له");

        let inputs = Inputs::from_pairs(vec![
            ("source", Value::Dir(src)),
            ("destination", Value::TargetDir(gone)),
            ("archive_name", Value::Name("ناتج.zip".to_owned())),
        ]);
        let e = plan(&inputs).unwrap_err();
        assert_eq!(
            (e.key(), e.input()),
            ("err.path.missing", Some("destination")),
            "re-attributing the name error must not drag the destination errors with it"
        );
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("shellish").expect("HOME must be set");
        let src = s.dir("م");
        let dst = s.dir("و");

        for name in ["تقرير 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let cmd = plan_with(&src, &dst, name).unwrap();
            let expected = dst.join(format!("{name}.zip"));
            assert_eq!(cmd.artifact.unwrap().final_path, expected);
            assert_eq!(cmd.args.len(), 6, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_source_with_spaces_and_arabic_stays_a_single_argument() {
        let s = Scratch::new("spaces").expect("HOME must be set");
        let src = s.dir("مجلد المشروع ٢٠٢٦");
        let dst = s.dir("النسخ الاحتياطية");

        let cmd = plan_with(&src, &dst, "نسخة اليوم").unwrap();
        assert_eq!(cmd.args.len(), 6);
        assert_eq!(Path::new(&cmd.args[4]), src.as_path());
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("dashes").expect("HOME must be set");
        // مجلد اسمه يبدأ بشرطة: المسار المطلق يبدأ بـ `/` فلا يُقرأ رايةً.
        let src = s.dir("-rf");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "-x").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        for (i, a) in args.iter().enumerate().skip(4) {
            assert!(a.starts_with('/'), "argument {i} ({a:?}) must be an absolute path");
        }
    }

    #[test]
    fn a_symlinked_source_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("symlink").expect("HOME must be set");
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&link, &dst, "ناتج").unwrap();
        assert_eq!(
            Path::new(&cmd.args[4]),
            real.as_path(),
            "the command must run on the resolved location"
        );
        assert!(
            cmd.warnings.contains(&"warn.source.resolved"),
            "silently archiving somewhere else is a surprise: {:?}",
            cmd.warnings
        );
    }

    #[test]
    fn an_empty_source_is_flagged_but_not_refused() {
        let s = Scratch::new("empty").expect("HOME must be set");
        let src = s.dir("فارغ");
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ناتج").unwrap();
        assert!(cmd.warnings.contains(&"warn.source.empty"));
    }

    #[test]
    fn a_populated_source_raises_no_size_warning() {
        let s = Scratch::new("sized").expect("HOME must be set");
        let src = s.dir("ممتلئ");
        std::fs::write(src.join("ملف"), vec![0u8; 64]).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ناتج").unwrap();
        for noisy in ["warn.source.empty", "warn.size.partial", "warn.space.low"] {
            assert!(!cmd.warnings.contains(&noisy), "unexpected {noisy}: {:?}", cmd.warnings);
        }
    }

    #[test]
    fn planning_leaves_no_write_probe_behind() {
        let s = Scratch::new("probe").expect("HOME must be set");
        let src = s.dir("م");
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&src, &dst, "ناتج").unwrap();
        }

        let leftovers: Vec<String> = std::fs::read_dir(&dst)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(leftovers, Vec::<String>::new(), "planning must leave the destination clean");
    }

    #[test]
    fn planning_writes_nothing_into_the_source() {
        let s = Scratch::new("source-clean").expect("HOME must be set");
        let src = s.dir("م");
        std::fs::write(src.join("ملف"), b"data").unwrap();
        let dst = s.dir("و");

        plan_with(&src, &dst, "ناتج").unwrap();

        let mut names: Vec<String> = std::fs::read_dir(&src)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, vec!["ملف".to_string()]);
    }

    #[test]
    fn a_user_file_named_like_an_appledouble_sidecar_is_announced_not_lost_silently() {
        // `--sequesterRsrc` تُسقط `._notes` من الأرشيف تمامًا. قِيس بأرشيف
        // حقيقي: لا في الشجرة ولا في `__MACOSX`. السكوت عنه خسارة بيانات.
        let s = Scratch::new("appledouble").expect("HOME must be set");
        let src = s.dir("م");
        std::fs::write(src.join("._notes"), b"user data, not a sidecar").unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ناتج").unwrap();
        assert!(
            cmd.warnings.contains(&"warn.source.appledouble"),
            "a file ditto will drop must be named before execution: {:?}",
            cmd.warnings
        );
    }

    #[test]
    fn an_appledouble_file_nested_deep_in_the_tree_is_found_too() {
        let s = Scratch::new("appledeep").expect("HOME must be set");
        let src = s.dir("م");
        let deep = src.join("أ/ب/ج");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("._سجل"), b"x").unwrap();
        let dst = s.dir("و");

        assert!(plan_with(&src, &dst, "ناتج")
            .unwrap()
            .warnings
            .contains(&"warn.source.appledouble"));
    }

    #[test]
    fn an_ordinary_tree_raises_no_appledouble_warning() {
        // التحذير الذي يظهر دائمًا لا يُقرأ. هذا يثبت أنه لا يظهر دائمًا.
        let s = Scratch::new("noappledouble").expect("HOME must be set");
        let src = s.dir("م");
        std::fs::write(src.join("ملف.txt"), b"x").unwrap();
        std::fs::create_dir(src.join("مجلد")).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&src, &dst, "ناتج").unwrap();
        assert!(!cmd.warnings.contains(&"warn.source.appledouble"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_arabic_tree_is_told_its_names_may_be_mojibake_elsewhere() {
        // راية UTF-8 (‏البت ١١) لا تضبطها `ditto`. اختبار في
        // tests/compress_integration.rs يقرأ البت من أرشيف حقيقي.
        let s = Scratch::new("nonascii").expect("HOME must be set");
        let src = s.dir("مصدر");
        let dst = s.dir("و");

        assert!(plan_with(&src, &dst, "ناتج")
            .unwrap()
            .warnings
            .contains(&"warn.zip.name_encoding"));
    }

    #[test]
    fn a_pure_ascii_tree_carries_no_encoding_warning() {
        let s = Scratch::new("ascii").expect("HOME must be set");
        let src = s.dir("source");
        std::fs::write(src.join("file.txt"), b"x").unwrap();
        let dst = s.dir("dest");

        let cmd = plan_with(&src, &dst, "archive").unwrap();
        assert!(!cmd.warnings.contains(&"warn.zip.name_encoding"), "{:?}", cmd.warnings);
    }

    #[test]
    fn a_tree_past_the_classic_zip_limits_is_warned_about_before_the_wait() {
        // لا يمكن بناء شجرة ٤ غيغابايت في اختبار وحدة، والتحذير قرار خالص على
        // أرقام التقدير — فيُختبر عند حدّه مباشرة. الحدّ نفسه قِيس بأرشيف
        // حقيقي: ملف ٥ غ.ب سُجّل ١٫٠٧٣٫٧٤١٫٨٢٤.
        let names = NameScan::default();
        let inputs = Inputs::from_pairs(vec![]);
        let here = Path::new("/tmp");

        let over =
            estimate::SizeEstimate { bytes: ZIP_FIELD_LIMIT + 1, entries: 1, complete: true };
        assert!(warnings_for(&over, &names, &inputs, here, here).contains(&"warn.size.zip_limit"));

        let many =
            estimate::SizeEstimate { bytes: 1024, entries: ZIP_ENTRY_LIMIT + 1, complete: true };
        assert!(warnings_for(&many, &names, &inputs, here, here).contains(&"warn.size.zip_limit"));

        let fine = estimate::SizeEstimate { bytes: ZIP_FIELD_LIMIT, entries: 2, complete: true };
        assert!(!warnings_for(&fine, &names, &inputs, here, here).contains(&"warn.size.zip_limit"));
    }

    #[test]
    fn an_undeclared_input_is_refused() {
        let s = Scratch::new("extra").expect("HOME must be set");
        let src = s.dir("م");
        let dst = s.dir("و");
        let mut inputs = raw(&src, &dst, "ناتج");
        inputs.insert("flags".to_owned(), RawValue::Text("--norsrc".to_owned()));

        let r = crate::value::validate(&SPEC, &inputs);
        assert!(matches!(&r, Err(CoreError::UnexpectedInput(k)) if k == "flags"), "got {r:?}");
    }

    #[test]
    fn a_text_value_cannot_be_smuggled_where_a_path_is_declared() {
        let s = Scratch::new("smuggle").expect("HOME must be set");
        let dst = s.dir("و");
        let inputs = BTreeMap::from([
            ("source".to_owned(), RawValue::Text("/etc".to_owned())),
            ("destination".to_owned(), RawValue::Path(dst.display().to_string())),
            ("archive_name".to_owned(), RawValue::Text("ناتج".to_owned())),
        ]);
        let e = crate::value::validate(&SPEC, &inputs).unwrap_err();
        assert_eq!((e.key(), e.input()), ("err.input.type", Some("source")));
    }

    #[test]
    fn a_wrong_value_type_reaching_plan_directly_is_still_refused() {
        // حتى لو تجاوز أحدهم `validate`، المخطِّط لا يثق بما وصله.
        let inputs = Inputs::from_pairs(vec![
            ("source", Value::Text("/tmp".into())),
            ("destination", Value::Text("/tmp".into())),
            ("archive_name", Value::Name("x.zip".into())),
        ]);
        assert!(matches!(plan(&inputs), Err(CoreError::WrongInputType { id: "source" })));
    }
}
