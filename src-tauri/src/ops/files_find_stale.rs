//! العثور على الملفات التي لم تُقرأ منذ مدّة، باستخدام `find`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/find <المجلد> -type f -atime +<الأيام>
//! ```
//!
//! ## هذه العملية تعطي **مؤشّرًا لا حُكمًا**، والتحذير دائم
//!
//! `-atime` تقرأ زمن آخر *وصول* إلى الملف (‏`st_atime`)، وهو على macOS الحديث
//! رقمٌ لا يُوثق به قياسًا على ما يظنّه المستخدم:
//!
//! * **النظام يقلّل تحديثه عمدًا.** كتابة `atime` عند كل قراءة تعني كتابةً على
//!   القرص عند كل قراءة، فالأنظمة الحديثة تؤجّلها أو تُهملها. فملفٌ قرأه
//!   المستخدم أمس قد يحمل `atime` من الشهر الماضي.
//! * **وبرامج لا يراها المستخدم تلمسه.** فهرسة Spotlight، وماسحات مكافحة
//!   الفيروسات، وأدوات النسخ الاحتياطي، وحتى معاينة Finder — كلها تفتح ملفاتٍ
//!   لم يفتحها أحد. فملفٌ منسيّ منذ سنوات قد يحمل `atime` من هذا الصباح.
//!
//! أي أن الخطأ يقع في الاتجاهين: نتائج ناقصة ونتائج زائدة معًا. ولذلك
//! `warn.atime.unreliable` تُعلَن **دائمًا**، لا عند حالةٍ بعينها: ليست تحذيرًا
//! من ظرفٍ طارئ بل وصفًا لما تقيسه الأداة أصلًا. وإخفاؤها كان سيجعل المستخدم
//! يقرأ القائمة حقيقةً ويحذف على أساسها.
//!
//! ### ولماذا `-atime` رغم ذلك
//!
//! البديلان أسوأ لهذا السؤال بعينه. `-mtime` تقيس آخر **تعديل**، فتُعلن
//! «قديمًا» كلَّ أرشيفٍ أو صورةٍ لم تتغيّر منذ حُفظت وإن فُتحت أمس — وهو عكس
//! المطلوب تمامًا. و`mdls` تقرأ ما سجّلته Spotlight من زمن آخر استعمال، وهي
//! أدقّ فعلًا لكنها تعتمد فهرسًا قد يكون مطفأً أو ناقصًا على القرص المقصود،
//! وتحتاج استدعاءً لكل ملف. `-atime` مؤشّرٌ خشن يُعلن خشونته.
//!
//! ## `find` تخبر ولا تحذف
//!
//! لا `-delete` ولا `-exec`. الأمر يطبع مسارات، والقرار للمستخدم — وهذا هنا
//! أوكد منه في أي عملية بحثٍ أخرى، لأن المعيار نفسه غير موثوق.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "files.find.stale";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.find.stale.title",
    description_key: "op.files.find.stale.description",
    category: Category::Files,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::FIND,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("folder", InputKind::ExistingDir),
        InputSpec::new("days", InputKind::Number { min: 1, max: 3650, default: 180 }),
    ],
    sort_order: 50,
    search_terms: &["find", "قديم", "old", "stale", "غير مستعمل", "unused", "أيام", "بحث"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let folder = inputs.dir("folder")?;
    let days = inputs.number("days")?;

    let mut argv = Argv::tool(tools::FIND, "explain.find.tool")
        .path(folder)
        .flag("-type", "explain.find.type_f")
        .value("f")
        .flag("-atime", "explain.find.atime")
        // `+` لا `-`: البادئة هنا جزءٌ من صيغة `find` («أكثر من») لا راية،
        // ولذلك تقبلها `Argv::value` بينما كانت سترفض `-{days}`.
        .value(format!("+{days}"))
        .reveal(folder)
        // تحذيرٌ عن طبيعة المقياس لا عن حالةٍ طارئة، فلا شرط عليه.
        .warn("warn.atime.unreliable");

    if let Some(key) = warn_if_resolved(inputs, "folder", folder, "warn.source.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn raw(folder: &Path, days: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("folder".to_owned(), RawValue::Path(folder.display().to_string())),
            ("days".to_owned(), RawValue::Text(days.to_owned())),
        ])
    }

    fn plan_with(folder: &Path, days: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(folder, days))?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("files.find.stale must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_writes_nothing() {
        let s = Scratch::new("stale-argv").unwrap();
        let folder = s.dir("المجلد");

        let cmd = plan_with(&folder, "365").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/find"));
        assert_eq!(Path::new(&args[0]), folder.as_path());
        assert_eq!(args[1], "-type");
        assert_eq!(args[2], "f");
        assert_eq!(args[3], "-atime");
        assert_eq!(args[4], "+365");
        assert_eq!(args.len(), 5);

        assert!(cmd.artifact.is_none(), "find reports; it never produces a file");
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
        assert!(!args.iter().any(|a| a == "-delete" || a == "-exec"), "find must only report");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("stale-explain").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder, "180").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn the_unreliability_of_atime_is_always_announced_never_conditionally() {
        let s = Scratch::new("stale-warn").unwrap();
        let folder = s.dir("م");

        // مجلدٌ فارغ، ومجلدٌ فيه ملف، وكل قيمةٍ على حدّي المدى: التحذير واحد.
        std::fs::write(folder.join("ملف"), b"x").unwrap();
        let empty = s.dir("فارغ");

        for target in [&folder, &empty] {
            for days in ["1", "180", "3650"] {
                let cmd = plan_with(target, days).unwrap();
                assert!(
                    cmd.warnings.contains(&"warn.atime.unreliable"),
                    "atime is a hint, not a verdict: {:?}",
                    cmd.warnings
                );
            }
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("stale-dashes").unwrap();
        let folder = s.dir("-rf");

        let cmd = plan_with(&folder, "1").unwrap();
        for i in [0usize, 2, 4] {
            assert!(
                cannot_be_read_as_a_flag(&cmd.args[i]),
                "{:?} would be read as a flag",
                cmd.args[i]
            );
        }
    }

    #[test]
    fn a_number_outside_the_declared_range_is_refused_before_it_becomes_an_argument() {
        let s = Scratch::new("stale-range").unwrap();
        let folder = s.dir("م");

        for bad in ["0", "-1", "3651"] {
            assert_eq!(
                refusal(plan_with(&folder, bad)),
                ("err.input.range", Some("days")),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_number_that_is_not_a_number_is_refused_not_coerced() {
        let s = Scratch::new("stale-nan").unwrap();
        let folder = s.dir("م");
        assert_eq!(refusal(plan_with(&folder, "180 -delete")), ("err.input.type", Some("days")));
    }

    #[test]
    fn shell_syntax_in_the_folder_name_stays_literal_and_adds_no_argument() {
        let s = Scratch::new("stale-shellish").unwrap();

        for name in ["مجلد 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let folder = s.dir(name);
            let cmd = plan_with(&folder, "180").unwrap();
            assert_eq!(Path::new(&cmd.args[0]), folder.as_path());
            assert_eq!(cmd.args.len(), 5, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_symlinked_folder_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("stale-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "180").unwrap();
        assert_eq!(Path::new(&cmd.args[0]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_the_folder_untouched() {
        let s = Scratch::new("stale-clean").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        for _ in 0..10 {
            plan_with(&folder, "180").unwrap();
        }
        assert_eq!(s.names(&folder), vec!["ملف".to_string()]);
    }
}
