//! تنزيل ملف من عنوان إلى مجلد تختاره، باستخدام `curl`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/curl -fL --progress-bar --max-time 900 --max-redirs 5 -o <المؤقّت> <العنوان>
//! ```
//!
//! * `-fL` — افشل عند ردٍّ ‎4xx/5xx‎ بدل كتابته، واتبع إعادة التوجيه.
//! * `--progress-bar` — شريط تقدّم بدل جدول العدّادات.
//! * `--max-time 900` — سقف زمني للتنزيل كله.
//! * `--max-redirs 5` — سقفٌ لعدد مرات التتبّع.
//! * `-o <المؤقّت>` — اكتب الجسد إلى ملف، والملف **مؤقّت** لا نهائي.
//!
//! ## `-f` هي التي تجعل الضمان ضمانًا
//!
//! بلا `-f` تكتب `curl` جسد الردّ أيًّا كان رمزه: صفحة «‏404 غير موجود» تُحفظ
//! ملفًا بحجم أربعة كيلوبايت وامتدادٍ صحيح، ويخرج الأمر بنجاح، فيُرقّى المؤقّت
//! إلى اسمه النهائي ويقول التطبيق «تمّ». هذا بالضبط ما تمنعه `-f`: رمزٌ ‎‏≥ 400‎
//! يصير خروجًا فاشلًا، والفشل يحذف المؤقّت ولا يُرقّي شيئًا. أي أن وعد
//! «لا يظهر الملف باسمه النهائي إلا بعد نجاح كامل» يبقى وعدًا صادقًا بدل أن
//! يصير وعدًا عن *نجاح الأمر* لا عن *صحّة الملف*.
//!
//! وحدُّ `-f` معلَن: خادمٌ يرسل صفحة خطأ برمز ‎200‎ لا تميّزه أداة. ولا يفحص هذا
//! التطبيق بصمةً ولا توقيعًا، فالتحقّق مما نُزِّل يبقى على المستخدم — وتحذير
//! `warn.download.unverified` يقول ذلك قبل التنفيذ لا بعده.
//!
//! ## `--progress-bar` لا صمت
//!
//! `curl` تكتب تقدّمها إلى الخرج القياسي للخطأ، والتطبيق يبثّ هذا المجرى إلى
//! الشاشة سطرًا سطرًا. فالشريط ليس زينة: هو الفرق بين تنزيلٍ يبدو معلّقًا
//! وتنزيلٍ يُرى وهو يتقدّم. والبديل — `-s` كما في `net.headers` — كان يجعل
//! ملفًا بحجم غيغابايت ينزل خلف شاشةٍ صامتة.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تستبدل. إن كان الاسم مأخوذًا في الوجهة تتوقّف وتخبر، ولا تضيف رقمًا إلى
//! الاسم ولا تنحّي القائم جانبًا. ولا تستأنف تنزيلًا منقطعًا (‏`-C -`): استئنافٌ
//! يعني الكتابة فوق ملفٍ قائم، وهو ما تقوم عليه الترقية الذرّية كي لا يحدث.
//! ولا تقيس المساحة الحرة قبل البدء لأن حجم الملف غير معلوم قبل طلبه — وإن
//! امتلأ القرص فشل الأمر وحُذف المؤقّت، فلا يبقى ملفٌ ناقص.
//!
//! ## العنوان سرٌّ في السجلّ
//!
//! `secret` على حقل العنوان ليس تزيّنًا: استعلامُ العنوان يحمل رمز وصولٍ في
//! روابط التنزيل الموقّعة (‏S3 و Drive وروابط الشراء)، وسجلُّ التشغيل يُحفظ
//! على القرص كي يُعاد التشغيل منه. القيد يبقى ويبقى الحقل مرئيًا بأنه كان،
//! ولا تُكتب قيمته. الثمن أن «أعِد التشغيل» لا يملأ هذا الحقل — وهو ثمنٌ أرخص
//! من رمزٍ صالح ينام في ملفٍ نصّي.

use crate::atomic;
use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;

pub const ID: &str = "net.download";

/// السقف الزمني للتنزيل كله بالثواني: ربع ساعة.
///
/// أوسع بكثير من سقف `net.headers` لأن السؤال مختلف: هناك ننتظر ترويسةً،
/// وهنا ننتظر ملفًا. والثمن معلَن في شرح الرمز: ملفٌ ضخم على اتصالٍ بطيء
/// يُقطع عند هذا الحدّ، ولا يُرقّى المؤقّت.
const MAX_TIME_SECONDS: &str = "900";

/// أقصى عدد مرات تتبّع إعادة التوجيه.
///
/// روابط التنزيل تمرّ بتحويلٍ أو تحويلين عادةً (مختصِر، ثم شبكة توزيع). خمسٌ
/// تسع ذلك بسعة، وما زاد حلقةٌ لا وجهة.
const MAX_REDIRECTS: &str = "5";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.net.download.title",
    description_key: "op.net.download.description",
    category: Category::Network,
    // تُنشئ ملفًا جديدًا ولا تمسّ شيئًا قائمًا.
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::CURL,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("url", InputKind::Url).secret(),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("out_name", InputKind::NewName { ext: None }),
    ],
    sort_order: 50,
    search_terms: &[
        "curl",
        "تنزيل",
        "download",
        "تحميل",
        "جلب",
        "fetch",
        "wget",
        "عنوان",
        "url",
        "http",
        "https",
        "ملف",
        "شبكة",
        "network",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let url = inputs.url("url")?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("out_name")?;

    // لا امتداد يُضاف تلقائيًا (`ext: None`): اسم الملف على السلك قد يكون
    // `.tar.gz` أو بلا امتداد أصلًا، وتخمينُه من العنوان يعني اسمًا يخالف ما
    // كتبه المستخدم في الحقل الذي يراه.
    //
    // `new_file_in` يفحص شيئين في نداءٍ واحد — مجلد الوجهة، والاسم النهائي —
    // فالنسبة لا تكون واحدة لكليهما: `InvalidName` تخصّ الاسم، وما عداها
    // (غير موجود، ليس مجلدًا، خارج الجذور، غير قابل للكتابة) يخصّ الوجهة.
    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "out_name" } else { "destination" };
        e.on_input(field)
    })?;

    // `symlink_metadata` لا يتبع الروابط: رابطٌ معلَّق بالاسم النهائي تضاربٌ
    // كذلك — ولولا الفحص لكتبت `curl` عبر الرابط إلى حيث يشير.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("out_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    let argv = Argv::tool(tools::CURL, "explain.curl.tool")
        .flag("-fL", "explain.curl.fail_and_follow")
        .flag("--progress-bar", "explain.curl.progress_bar")
        .flag("--max-time", "explain.curl.max_time")
        .explained_value(MAX_TIME_SECONDS, "explain.curl.max_time.download")
        .flag("--max-redirs", "explain.curl.max_redirs")
        .explained_value(MAX_REDIRECTS, "explain.curl.max_redirs.value")
        .flag("-o", "explain.curl.output")
        .explained_path(&temp, "explain.role.temp")
        .explained_value(url, "explain.curl.url")
        .warn_all(warnings_for(inputs, url, destination));

    argv.producing(Artifact::file(temp, final_path))
}

fn warnings_for(inputs: &Inputs, url: &str, destination: &std::path::Path) -> Vec<&'static str> {
    // ثابتٌ لا مشروط: لا بصمة ولا توقيع في هذا المنتج، وهي صفةٌ في كل تنزيل.
    let mut warnings = vec!["warn.download.unverified"];
    if url.to_ascii_lowercase().starts_with("http://") {
        warnings.push("warn.url.plaintext");
    }
    // الصيغة العامة `warn.destination.resolved` تتحدّث عن الأرشيف نصًّا، فهي
    // تقول هنا شيئًا لا يحدث. مفتاحٌ خاصّ أصدق من مفتاحٍ مشترك يكذب.
    warnings.extend(warn_if_resolved(
        inputs,
        "destination",
        destination,
        "warn.download.destination_resolved",
    ));
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    const URL: &str = "https://example.com/files/archive.tar.gz";

    fn raw(url: &str, destination: &Path, name: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("url".to_owned(), RawValue::Text(url.to_owned())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("out_name".to_owned(), RawValue::Text(name.to_owned())),
        ])
    }

    fn plan_with(url: &str, destination: &Path, name: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(url, destination, name))?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    fn args_of(cmd: &PlannedCommand) -> Vec<String> {
        cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("net.download must be listed");
        assert_eq!(found.category, Category::Network);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_output_is_the_temp_not_the_final_name() {
        let s = Scratch::new("download-argv").unwrap();
        let dst = s.dir("الوجهة");

        let cmd = plan_with(URL, &dst, "أرشيف.tar.gz").unwrap();
        let args = args_of(&cmd);
        let artifact = cmd.artifact.as_ref().unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/curl"));
        assert_eq!(args[0], "-fL");
        assert_eq!(args[1], "--progress-bar");
        assert_eq!(args[2], "--max-time");
        assert_eq!(args[3], "900");
        assert_eq!(args[4], "--max-redirs");
        assert_eq!(args[5], "5");
        assert_eq!(args[6], "-o");
        assert_eq!(Path::new(&args[7]), artifact.temp.as_path());
        assert_eq!(args[8], URL);
        assert_eq!(args.len(), 9);

        assert_eq!(artifact.final_path, dst.join("أرشيف.tar.gz"));
        assert_eq!(artifact.kind, ArtifactKind::File);
        assert_eq!(artifact.temp.parent(), artifact.final_path.parent(), "same filesystem");
        assert!(!artifact.temp.exists(), "planning must create nothing");
        assert!(cmd.stdout_to.is_none(), "curl writes the body itself through -o");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("download-explain").unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(URL, &dst, "ملف").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("download-dashes").unwrap();
        let dst = s.dir("-rf");

        // اسم الوجهة يبدأ بشرطة، والاسم النهائي كذلك: كلاهما ينتهي داخل مسارٍ
        // مطلق يبدأ بـ`/`، فلا يمكن أن يُقرأ رايةً مهما كتب المستخدم.
        let cmd = plan_with(URL, &dst, "-o").unwrap();
        for i in [3usize, 5, 7, 8] {
            assert!(
                cannot_be_read_as_a_flag(&cmd.args[i]),
                "{:?} would be read as a flag",
                cmd.args[i]
            );
        }
    }

    #[test]
    fn an_existing_name_in_the_destination_stops_the_plan_before_anything_runs() {
        let s = Scratch::new("download-exists").unwrap();
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود"), b"PRECIOUS").unwrap();

        assert_eq!(refusal(plan_with(URL, &dst, "موجود")), ("err.dest.exists", Some("out_name")));
        assert_eq!(std::fs::read(dst.join("موجود")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn a_dangling_symlink_at_the_final_name_is_a_conflict_not_a_free_slot() {
        // لولا الفحص لكتبت `curl` عبر الرابط إلى حيث يشير، خارج الوجهة كلها.
        let s = Scratch::new("download-dangling").unwrap();
        let dst = s.dir("و");
        std::os::unix::fs::symlink(s.path().join("لا-وجود-له"), dst.join("رابط")).unwrap();

        assert_eq!(refusal(plan_with(URL, &dst, "رابط")), ("err.dest.exists", Some("out_name")));
    }

    #[test]
    fn a_name_the_filesystem_would_refuse_is_blamed_on_the_name_field() {
        let s = Scratch::new("download-badname").unwrap();
        let dst = s.dir("و");

        for bad in ["a/b", ".مخفي", "..", "", "   "] {
            assert_eq!(
                refusal(plan_with(URL, &dst, bad)),
                ("err.name.invalid", Some("out_name")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn a_destination_that_is_not_a_directory_is_blamed_on_the_destination_field() {
        let s = Scratch::new("download-notdir").unwrap();
        let file = s.file("ليس-مجلدًا", b"x");
        assert_eq!(
            refusal(plan_with(URL, &file, "ملف")),
            ("err.path.not_dir", Some("destination"))
        );
    }

    #[test]
    fn a_scheme_the_core_refuses_never_reaches_curl() {
        let s = Scratch::new("download-scheme").unwrap();
        let dst = s.dir("و");
        for bad in ["file:///etc/passwd", "ftp://example.com/x", "example.com", "-o/etc/passwd", ""]
        {
            assert_eq!(
                refusal(plan_with(bad, &dst, "ملف")),
                ("err.input.url", Some("url")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn shell_syntax_in_a_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("download-shellish").unwrap();
        let dst = s.dir("و");

        for name in ["ملف 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let cmd = plan_with(URL, &dst, name).unwrap();
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(name));
            assert_eq!(cmd.args.len(), 9, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn shell_syntax_inside_a_url_stays_one_literal_argument() {
        let s = Scratch::new("download-url-shellish").unwrap();
        let dst = s.dir("و");

        for url in [
            "https://example.com/a;b",
            "https://example.com/?q=$(whoami)&t=1",
            "https://example.com/`tick`",
        ] {
            let cmd = plan_with(url, &dst, "ملف").unwrap();
            assert_eq!(cmd.args.len(), 9, "{url} must not add an argument");
            assert_eq!(args_of(&cmd).last().unwrap(), url);
        }
    }

    #[test]
    fn the_url_is_never_written_into_the_run_journal() {
        // استعلامُ رابط تنزيلٍ موقّع يحمل رمز وصولٍ صالحًا. السجلّ يبقى، والرمز
        // لا يبقى معه — ويبقى الحقل مرئيًا بأنه كان، فرقًا بين «لا قيمة» و«قيمةٌ
        // لا تُكتب».
        let s = Scratch::new("download-secret").unwrap();
        let dst = s.dir("و");
        let secret = "https://example.com/f?token=SECRET-DO-NOT-LOG";

        assert!(SPEC.input("url").unwrap().secret, "the url field must be declared secret");

        let inputs = crate::value::validate(&SPEC, &raw(secret, &dst, "ملف")).unwrap();
        let records = inputs.journal_form(&SPEC);

        let url_record =
            records.iter().find(|r| r.id == "url").expect("the field must be recorded");
        assert_eq!(url_record.value, None, "the url must be redacted, not stored");

        let name_record = records.iter().find(|r| r.id == "out_name").unwrap();
        assert_eq!(name_record.value.as_deref(), Some("ملف"), "other fields are still recorded");

        let written = serde_json::to_string(&records).unwrap();
        assert!(!written.contains("SECRET-DO-NOT-LOG"), "the token leaked into the journal");
    }

    #[test]
    fn every_download_says_that_nothing_was_verified() {
        let s = Scratch::new("download-unverified").unwrap();
        let dst = s.dir("و");
        let cmd = plan_with(URL, &dst, "ملف").unwrap();
        assert!(cmd.warnings.contains(&"warn.download.unverified"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_unencrypted_url_is_planned_but_announced() {
        let s = Scratch::new("download-plaintext").unwrap();
        let dst = s.dir("و");

        let cmd = plan_with("http://example.com/f.bin", &dst, "ملف").unwrap();
        assert!(cmd.warnings.contains(&"warn.url.plaintext"), "{:?}", cmd.warnings);

        let secure = plan_with(URL, &dst, "ملف2").unwrap();
        assert!(!secure.warnings.contains(&"warn.url.plaintext"), "{:?}", secure.warnings);
    }

    #[test]
    fn a_symlinked_destination_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("download-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(URL, &link, "ملف").unwrap();
        assert_eq!(cmd.artifact.unwrap().final_path, real.join("ملف"));
        assert!(cmd.warnings.contains(&"warn.download.destination_resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_nothing_behind_in_the_destination() {
        let s = Scratch::new("download-clean").unwrap();
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(URL, &dst, "ملف").unwrap();
        }

        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
    }

    #[test]
    fn the_temporary_name_is_hidden_and_carries_no_final_meaning() {
        let s = Scratch::new("download-temp").unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(URL, &dst, "ملف.bin").unwrap();
        let artifact = cmd.artifact.unwrap();
        let temp_name = artifact.temp.file_name().unwrap().to_string_lossy().into_owned();
        assert!(temp_name.starts_with('.'), "{temp_name} would show up in Finder while running");
        assert!(temp_name.ends_with(".part"), "{temp_name} must be recognisable as a leftover");
        assert_ne!(artifact.temp, artifact.final_path);
    }
}
