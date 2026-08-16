//! اختبار سلامة أرشيف ZIP.
//!
//! ## ما تعنيه «سلامة» هنا بالضبط
//!
//! `unzip -t` تفكّ ضغط كل مدخلة في الذاكرة وتقارن CRC-32 المحسوب بالمخزَّن.
//! أي أنها تجيب عن سؤالٍ واحدٍ بدقّة: **هل وصل الأرشيف كما غادر؟** بايتٌ
//! انقلب في التنزيل، أو قرصٌ بدأ يفشل، أو نقلٌ عبر شبكةٍ قطعته — كلّها تظهر هنا.
//!
//! وما لا تعنيه: أن المحتوى صحيح، ولا أنه آمن، ولا أن الاستخراج سينجح على
//! هذا القرص. CRC مجموعُ تحقّقٍ لا توقيع: من يريد تزوير أرشيفٍ يزوّر معه CRC.
//! الوصف يقول ذلك، فلا يخرج المستخدم بثقةٍ أوسع ممّا تعطيه الأداة.
//!
//! ## ولماذا تقرأ كل بايت
//!
//! هي العملية الوحيدة هنا التي تفكّ الضغط فعلًا — في الذاكرة، بلا كتابة —
//! فزمنها زمنُ استخراجٍ كامل على أرشيفٍ كبير. لذلك تحذيرٌ يقول ذلك قبل أن
//! ينتظر المستخدم دقائق أمام شاشةٍ لا تعرض نسبة مئوية.

use crate::archive;
use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "compress.zip.test";

/// فوق هذا الحجم المعلَن يستحقّ الانتظار تحذيرًا. ٢٥٦ ميغابايت.
const SLOW_TEST_BYTES: u64 = 256 * 1024 * 1024;

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.compress.zip.test.title",
    description_key: "op.compress.zip.test.description",
    category: Category::Compress,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::UNZIP,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("archive", InputKind::ExistingFile)],
    sort_order: 30,
    search_terms: &["unzip", "zip", "اختبار", "سلامة", "test", "crc", "تحقق", "تالف", "verify"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let archive_path = inputs.file("archive")?;
    let scan = archive::scan_zip(archive_path).map_err(|e| e.on_input("archive"))?;

    let mut argv = Argv::tool(tools::UNZIP, "explain.unzip.tool")
        .flag("-t", "explain.unzip.test")
        .path(archive_path);

    for key in scan.warnings() {
        argv = argv.warn(key);
    }
    if scan.uncompressed_bytes > SLOW_TEST_BYTES {
        argv = argv.warn("warn.archive.slow_test");
    }

    if let Some(parent) = archive_path.parent() {
        argv = argv.reveal(parent);
    }
    argv.read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{zip_with, Scratch};
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn plan_with(p: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([("archive".to_owned(), RawValue::Path(p.display().to_string()))]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    #[test]
    fn the_operation_is_listed_as_read_only() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let s = Scratch::new("zipt-argv").unwrap();
        let z = s.file("أرشيف.zip", &zip_with(&["a.txt"]));
        let cmd = plan_with(&z).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/unzip"));
        assert_eq!(args, vec!["-t".to_string(), z.display().to_string()]);
        assert!(cmd.artifact.is_none());
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("zipt-explain").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt"]));
        let cmd = plan_with(&z).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        assert_eq!(cmd.explain.iter().map(|t| t.token.clone()).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn a_file_that_is_not_an_archive_is_refused_with_its_field_named() {
        let s = Scratch::new("zipt-bad").unwrap();
        let f = s.file("x.zip", b"definitely not a zip");
        let e = plan_with(&f).unwrap_err();
        assert_eq!((e.key(), e.input()), ("err.archive.unreadable", Some("archive")));
    }

    #[test]
    fn a_healthy_small_archive_raises_no_warning_about_waiting() {
        let s = Scratch::new("zipt-quiet").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt"]));
        assert!(!plan_with(&z).unwrap().warnings.contains(&"warn.archive.slow_test"));
    }
}
