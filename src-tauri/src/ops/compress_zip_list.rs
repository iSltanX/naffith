//! فحص محتويات أرشيف ZIP **قبل** استخراجه.
//!
//! ## لماذا عمليةٌ مستقلّة لا خطوةٌ داخل الاستخراج
//!
//! «ماذا في هذا الأرشيف؟» سؤالٌ يُطرح قبل أن يُقرَّر الاستخراج أصلًا: أرشيفٌ
//! نُزّل من الشبكة، أو وصل في بريد، ولا يريد المستخدم أن ينثر محتواه في مجلده
//! ليعرف ما فيه. جعلُها خطوةً داخل الاستخراج كان يعني أن الجواب لا يُقرأ إلا
//! بعد أن يقع الفعل.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/unzip -l <الأرشيف>
//! ```
//!
//! `unzip` لا `ditto` هنا: `ditto` ليس فيها «اعرض الفهرس» — تستخرج أو لا
//! تفعل. و`unzip -l` تقرأ الفهرس المركزي وتطبعه بأحجامه وتواريخه، ولا تكتب
//! بايتًا واحدًا على القرص.
//!
//! ## وفحصٌ ثانٍ لا يراه المستخدم
//!
//! `archive::scan_zip` تقرأ الفهرس نفسه في النواة قبل التخطيط، فتُعلن الأرشيف
//! التالف خطأً واضحًا بدل أن يخرج `unzip` برمزٍ غامض، وتُظهر التحذيرات — روابط
//! رمزية، أو مدخلات تخرج من جذرها — على شاشة المعاينة قبل أن يفكّر المستخدم
//! في الاستخراج. أي أن هذه العملية تُجيب عن سؤالها وتُحضِّر للتي بعدها.

use crate::archive;
use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "compress.zip.list";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.compress.zip.list.title",
    description_key: "op.compress.zip.list.description",
    category: Category::Compress,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::UNZIP,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("archive", InputKind::ExistingFile)],
    sort_order: 20,
    search_terms: &["unzip", "zip", "قائمة", "محتويات", "list", "فحص", "أرشيف", "inspect"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let archive_path = inputs.file("archive")?;

    // القراءة في النواة قبل التخطيط: أرشيفٌ لا يُقرأ فهرسه يُرفض هنا برسالةٍ
    // تخصّ الحقل، لا برمز خروجٍ من `unzip` بعد أن يكون المستخدم قد ضغط «نفّذ».
    let scan = archive::scan_zip(archive_path).map_err(|e| e.on_input("archive"))?;

    let mut argv = Argv::tool(tools::UNZIP, "explain.unzip.tool")
        .flag("-l", "explain.unzip.list")
        .path(archive_path);

    for key in scan.warnings() {
        argv = argv.warn(key);
    }
    // الخروج من الجذر لا يمنع **العرض** — بل هو أهمّ ما يستحقّ أن يُعرض. المنع
    // موضعه عملية الاستخراج، وهناك يقع رفضًا لا تحذيرًا.
    if scan.escaping {
        argv = argv.warn("warn.archive.escapes");
    }
    if scan.truncated {
        argv = argv.warn("warn.archive.partial_scan");
    }

    if let Some(parent) = archive_path.parent() {
        argv = argv.reveal(parent);
    }
    argv.read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::{zip_with, Scratch};
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn plan_with(archive_path: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([(
            "archive".to_owned(),
            RawValue::Path(archive_path.display().to_string()),
        )]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    #[test]
    fn the_operation_is_listed_as_a_read_only_member_of_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.category, Category::Compress);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let s = Scratch::new("zip-list").unwrap();
        let z = s.file("أرشيف.zip", &zip_with(&["a.txt", "مجلد/ب.txt"]));

        let cmd = plan_with(&z).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/unzip"));
        assert_eq!(args, vec!["-l".to_string(), z.display().to_string()]);
        assert!(cmd.artifact.is_none(), "listing must produce nothing");
        assert_eq!(cmd.reveal_target.as_deref(), Some(s.path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("zip-list-explain").unwrap();
        let z = s.file("a.zip", &zip_with(&["a.txt"]));
        let cmd = plan_with(&z).unwrap();

        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("zip-list-dash").unwrap();
        let z = s.file("-weird.zip", &zip_with(&["a.txt"]));
        let cmd = plan_with(&z).unwrap();
        // المسار مطلق فيبدأ بـ `/` مهما كان اسم الملف.
        assert!(cannot_be_read_as_a_flag(&cmd.args[1]));
    }

    #[test]
    fn a_file_that_is_not_an_archive_is_refused_with_its_field_named() {
        let s = Scratch::new("zip-list-bad").unwrap();
        let f = s.file("ليس-أرشيفًا.zip", b"just some text, definitely not a zip file");

        let e = plan_with(&f).unwrap_err();
        assert_eq!((e.key(), e.input()), ("err.archive.unreadable", Some("archive")));
    }

    #[test]
    fn an_archive_that_escapes_its_root_is_shown_with_a_warning_not_hidden() {
        // العرض هو بالضبط ما يجب أن يقع هنا: المستخدم يريد أن يرى ما في
        // الأرشيف، وأن يعرف أن فيه ما يخرج من جذره. المنع موضعه الاستخراج.
        let s = Scratch::new("zip-list-slip").unwrap();
        let z = s.file("خبيث.zip", &zip_with(&["ok.txt", "../../../../etc/passwd"]));

        let cmd = plan_with(&z).unwrap();
        assert!(cmd.warnings.contains(&"warn.archive.escapes"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_empty_archive_is_announced_as_empty() {
        let s = Scratch::new("zip-list-empty").unwrap();
        let z = s.file("فارغ.zip", &zip_with(&[]));
        assert!(plan_with(&z).unwrap().warnings.contains(&"warn.archive.empty"));
    }

    #[test]
    fn a_healthy_archive_raises_no_noisy_warning() {
        let s = Scratch::new("zip-list-quiet").unwrap();
        let z = s.file("سليم.zip", &zip_with(&["a.txt", "b.txt"]));
        assert_eq!(plan_with(&z).unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn a_directory_cannot_be_passed_where_a_file_is_declared() {
        let s = Scratch::new("zip-list-dir").unwrap();
        let d = s.dir("مجلد");
        let e = plan_with(&d).unwrap_err();
        assert!(matches!(e, CoreError::OnInput { id: "archive", .. }), "got {e:?}");
    }
}
