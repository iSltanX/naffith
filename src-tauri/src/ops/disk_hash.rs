//! بصمة SHA-256 لملف واحد، باستخدام `shasum`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/shasum -a 256 -- <الملف>
//! ```
//!
//! تطبع الأداة سطرًا واحدًا: البصمة ثم مسار الملف. وهي نفس الصيغة التي
//! يقرؤها `shasum -c` لاحقًا، فما يُنسخ من الشاشة اليوم يبقى صالحًا للتحقّق
//! غدًا بأداةٍ لا علاقة لها بهذا التطبيق.
//!
//! ## لماذا SHA-256 لا MD5 ولا SHA-1
//!
//! الاثنتان أسرع، والاثنتان ما زالتا في كل مكان — وكلتاهما **مكسورة** حين
//! يكون السؤال عن السلامة لا عن التلف العابر:
//!
//! * MD5: بناء تصادمٍ بسابقةٍ مختارة صار عمليًّا منذ ٢٠٠٩، ويقع على حاسوب
//!   عادي في دقائق.
//! * SHA-1: أُعلن أول تصادم فعلي عام ٢٠١٧، وتصادم السابقة المختارة عام ٢٠٢٠.
//!
//! والفرق الذي يهمّ المستخدم أن الأداتين تصلحان للسؤال الأول — «هل تلف الملف
//! في النقل؟» — ولا تصلحان للثاني: «هل هذا هو الملف نفسه الذي نُشر؟». وحين
//! تطبع الشاشةُ بصمةً بلا قيدٍ يفرّق بين السؤالين، **يُقرأ الجواب على أنه
//! الثاني**. فالاختيار ليس تفضيل خوارزميةٍ أحدث، بل رفضُ عرضِ رقمٍ يُفهم أقوى
//! ممّا هو.
//!
//! الثمن أن SHA-256 أبطأ على الملفات الكبيرة، وهو ثمنٌ يُدفع مرّةً لكل ملف
//! ويُعلَن قبل التنفيذ بتحذير `warn.hash.slow`.
//!
//! ## لماذا `shasum` لا `openssl dgst` ولا `md5`
//!
//! * `/usr/bin/md5` موجودة على macOS ولا تفعل غير MD5. مستبعدة لما سبق.
//! * `openssl dgst -sha256` متاحة، لكن `openssl` على macOS هي LibreSSL، وصيغة
//!   خرجها بادئةٌ ثم قوسان (`SHA2-256(ملف)= …`) تغيّرت بين الإصدارات. بصمةٌ
//!   يريد المستخدم نسخها ومقارنتها بما نُشر على صفحة تنزيل يجب أن تظهر بصيغةٍ
//!   واحدة لا تتبع إصدار مكتبة.
//! * `sha256sum` من GNU **ليست** على macOS أصلًا، وهي التي يعرفها أكثر من
//!   يبحث عن هذه العملية. `shasum -a 256` تطبع نفس الصيغة تمامًا.
//!
//! ## ما لا تراه هذه البصمة
//!
//! تُحسب على **بيانات الملف وحدها**. السمات الممتدّة و resource fork ووسوم
//! Finder والأذونات لا تدخل فيها بحال. أي أن نسخةً فقدت وسومها في النقل
//! (‏`cp` تُسقطها، ولهذا تستعمل عملية النسخ هنا `ditto`) تعطي **نفس البصمة**
//! تمامًا. البصمة تقول «البايتات هي هي»، ولا تقول «الملف هو هو» — والوصف
//! يقول ذلك بدل أن يترك المستخدم يستنتج الأقوى.
//!
//! وهي كذلك بصمةُ الملف **لحظة القراءة**: لا تُقفل الأداةُ الملفَ، فملفٌ يُكتب
//! فيه أثناء الحساب تخرج له بصمةٌ لا تطابق أي نسخةٍ كاملة منه.
//!
//! ## سلامة الوسائط
//!
//! المسار مطلق ومُحلّ الروابط قبل الوصول إلى هنا، فيبدأ بـ `/` ولا يُقرأ راية.
//! و`--` قبله حزامُ أمانٍ فوق ذلك: `shasum` سكربت Perl يفكّ رايات بـ
//! `Getopt::Long`، و`--` هو ما تفهمه هذه المكتبة نهايةً للرايات. تُضاف هنا
//! لأنها **تعمل** مع هذه الأداة تحديدًا، لا لأنها تبدو مرتّبة.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "disk.hash.sha256";

/// الحجم الذي يصير بعده الصمت طويلًا بما يشبه التعليق.
///
/// رقمُ قرارٍ لا قياسٍ: `shasum` لا تطبع شيئًا حتى تنتهي — لا نسبة ولا نقطة —
/// و«سَطْر» يعلن ذلك أصلًا في حالة التشغيل. الحدّ مضبوطٌ عاليًا عمدًا كي يبقى
/// التحذير نادرًا: تحذيرٌ يظهر مع كل ملف يصير جزءًا من الأثاث ولا يُقرأ.
const SLOW_ABOVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.disk.hash.sha256.title",
    description_key: "op.disk.hash.sha256.description",
    category: Category::Disk,
    // تقرأ الملف من أوّله إلى آخره ولا تكتب فيه ولا بجانبه.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::SHASUM,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("source", InputKind::ExistingFile)],
    sort_order: 20,
    search_terms: &[
        "shasum",
        "sha256",
        "sha",
        "بصمة",
        "تجزئة",
        "hash",
        "checksum",
        "سلامة",
        "تحقق",
        "integrity",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let source = inputs.file("source")?;

    // مجلد الملف لا الملف: «إظهار في Finder» سؤالٌ عن **أين**، وجوابه موضع
    // الملف. والمسار مُحلّ الروابط، فأبوه موضعٌ حقيقي لا اسمٌ عابر.
    let folder = source.parent().unwrap_or(source);

    let mut argv = Argv::tool(tools::SHASUM, "explain.shasum.tool")
        .flag("-a", "explain.shasum.algorithm")
        .explained_value("256", "explain.shasum.sha256")
        .end_of_flags()
        .explained_path(source, "explain.role.hashed_file")
        .reveal(folder);

    for key in warnings_for(inputs, source) {
        argv = argv.warn(key);
    }
    argv.read_only()
}

fn warnings_for(inputs: &Inputs, source: &Path) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    warnings.extend(warn_if_resolved(inputs, "source", source, "warn.hash.source_resolved"));
    if file_bytes(source).is_some_and(slow_to_hash) {
        warnings.push("warn.hash.slow");
    }
    warnings
}

/// حجم الملف، أو `None` إن تعذّر قياسه.
///
/// `metadata` لا `estimate::tree_size`: تلك تمشي شجرةً وتعود بصفرٍ عن ملفٍ
/// مفرد، وهذا نداءٌ واحد يعطي الرقم الصحيح.
fn file_bytes(source: &Path) -> Option<u64> {
    std::fs::metadata(source).ok().map(|m| m.len())
}

/// معلَنة دالّةً نقيّة كي يُختبر حدّها عند حافّته بلا إنشاء ملفٍ بحجمها.
fn slow_to_hash(bytes: u64) -> bool {
    bytes > SLOW_ABOVE_BYTES
}

// ملاحظة على ما **لا** يُعلَن هنا: لا `with_estimate`. الحقل موجود ويقبل
// حجم الملف، لكن الشاشة تعرض تحته نصًّا واحدًا لكل العمليات يقول إن الرقم
// «تقدير قبل الضغط لا حجم الأرشيف الناتج» — وهي جملةٌ صادقة في الضغط وكاذبة
// هنا. عرضُ رقمٍ صحيح تحت شرحٍ خاطئ أسوأ من ألّا يُعرض، فالحجم يصل المستخدم
// تحذيرًا حين يكون له معنى (‏`warn.hash.slow`) ولا يصل عدّادًا مضلِّلًا.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::ffi::OsStr;

    const DECLARED_FLAGS: &[&str] = &["-a", "--"];

    fn plan_with(source: &Path) -> Result<PlannedCommand> {
        let raw =
            BTreeMap::from([("source".to_owned(), RawValue::Path(source.display().to_string()))]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    /// كل وسيطٍ بعد `--`. هذه هي وسائط البيانات، وما قبلها رايات معلَنة.
    fn data_args(cmd: &PlannedCommand) -> Vec<&OsStr> {
        let end =
            cmd.args.iter().position(|a| a.as_os_str() == "--").expect("`--` must be in the argv");
        cmd.args[end + 1..].iter().map(|a| a.as_os_str()).collect()
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("disk.hash.sha256 must be listed");
        assert_eq!(found.category, Category::Disk);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_file_is_the_last_argument() {
        let s = Scratch::new("hash-argv").unwrap();
        let file = s.file("مستند.txt", b"data");

        let cmd = plan_with(&file).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/shasum"));
        assert_eq!(args[0], "-a");
        assert_eq!(args[1], "256", "the algorithm is chosen in Rust, never in the form");
        assert_eq!(args[2], "--");
        assert_eq!(Path::new(&args[3]), file.as_path());
        assert_eq!(args.len(), 4);
        assert!(cmd.artifact.is_none(), "hashing produces no file");
        assert!(cmd.stdout_to.is_none(), "the hash is printed, not written to disk");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("hash-explain").unwrap();
        let file = s.file("م", b"x");

        let cmd = plan_with(&file).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn revealing_opens_the_folder_that_holds_the_hashed_file() {
        let s = Scratch::new("hash-reveal").unwrap();
        let file = s.file("مجلد/ملف.bin", b"x");
        let folder = s.path().join("مجلد");

        let cmd = plan_with(&file).unwrap();
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
        assert!(cmd.artifact.is_none(), "reveal only answers when there is no artifact");
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("hash-dashes").unwrap();
        let file = s.file("-a", b"x");

        let cmd = plan_with(&file).unwrap();
        for a in data_args(&cmd) {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
        // وما قبل `--` رايات معلَنة في الشيفرة لا قيمٌ من الواجهة.
        for a in cmd.args.iter().take_while(|a| a.as_os_str() != "--") {
            let text = a.to_string_lossy().into_owned();
            assert!(
                DECLARED_FLAGS.contains(&text.as_str()) || cannot_be_read_as_a_flag(a.as_os_str()),
                "{text:?} is neither a declared flag nor safe data"
            );
        }
    }

    #[test]
    fn shell_syntax_in_a_file_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("hash-shellish").unwrap();
        for name in ["a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "ملف 'اليوم'", "x|y"]
        {
            let file = s.file(name, b"x");
            let cmd = plan_with(&file).unwrap();
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[3]), file.as_path());
        }
    }

    #[test]
    fn a_path_that_is_not_a_file_is_refused_and_the_field_is_named() {
        let s = Scratch::new("hash-refusals").unwrap();
        let dir = s.dir("مجلد");

        // مجلد: `shasum` عليه تطبع خطأً بعد أن تكون العملية قد أُطلقت.
        // الرفض هنا يقع قبل الإطلاق، ومنسوبًا إلى الحقل الذي يصلحه.
        assert_eq!(refusal(plan_with(&dir)), ("err.path.missing", Some("source")));

        // مسار غير موجود.
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"))),
            ("err.path.missing", Some("source"))
        );

        // مسار نسبي: لا يصل إلى `Argv` أصلًا.
        let raw = BTreeMap::from([("source".to_owned(), RawValue::Path("نسبي/هنا".to_owned()))]);
        let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
        assert_eq!(refusal(r), ("err.path.relative", Some("source")));

        // حقل ناقص.
        let r = crate::value::validate(&SPEC, &BTreeMap::new()).map(|i| plan(&i).unwrap());
        assert_eq!(refusal(r), ("err.input.missing", Some("source")));
    }

    #[test]
    fn a_symlinked_file_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("hash-symlink").unwrap();
        let real = s.file("الحقيقي.bin", b"data");
        let link = s.path().join("رابط.bin");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(&cmd.args[3]), real.as_path(), "the command must name what it reads");
        assert!(cmd.warnings.contains(&"warn.hash.source_resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_file_raises_no_warning_at_all() {
        let s = Scratch::new("hash-quiet").unwrap();
        let file = s.file("عادي.txt", b"some bytes");
        assert_eq!(plan_with(&file).unwrap().warnings, Vec::<&'static str>::new());
    }

    #[test]
    fn the_slow_threshold_is_tested_at_its_edge_not_near_it() {
        assert!(!slow_to_hash(0));
        assert!(!slow_to_hash(SLOW_ABOVE_BYTES), "the limit itself is still fast enough");
        assert!(slow_to_hash(SLOW_ABOVE_BYTES + 1));
    }

    #[test]
    fn a_file_past_the_threshold_is_announced_as_a_long_silent_wait() {
        let s = Scratch::new("hash-slow").unwrap();
        let file = s.file("كبير.bin", b"");
        // `set_len` يمدّ الملف بلا كتابة بايتات: على APFS يبقى متفرّقًا فلا
        // يستهلك مساحة. الاختبار يخطّط ولا يشغّل، فلا شيء يقرأ هذه البايتات.
        let Ok(handle) = std::fs::OpenOptions::new().write(true).open(&file) else {
            return;
        };
        if handle.set_len(SLOW_ABOVE_BYTES + 1).is_err() {
            return; // نظام ملفات لا يمدّ الملفات بلا كتابة — لا نُسقط الاختبار عليه.
        }
        drop(handle);

        let cmd = plan_with(&file).unwrap();
        assert!(cmd.warnings.contains(&"warn.hash.slow"), "{:?}", cmd.warnings);
        assert_eq!(cmd.args.len(), 4, "a warning must not change the command");
    }

    #[test]
    fn planning_reads_the_file_metadata_and_writes_nothing_beside_it() {
        let s = Scratch::new("hash-clean").unwrap();
        let file = s.file("مجلد/ملف", b"data");
        let dir = s.path().join("مجلد");

        for _ in 0..10 {
            plan_with(&file).unwrap();
        }
        assert_eq!(s.names(&dir), vec!["ملف".to_string()]);
        assert_eq!(std::fs::read(&file).unwrap(), b"data");
    }
}
