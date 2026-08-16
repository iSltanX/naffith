//! العثور على الملفات الكبيرة داخل مجلد، باستخدام `find`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/find <المجلد> -type f -size +<العدد>M
//! ```
//!
//! ## `find` **تخبر ولا تحذف**
//!
//! لا `-delete` ولا `-exec` ولا `-print0` يُمرَّر إلى شيء. الأمر يطبع مساراتٍ
//! على الخرج القياسي، وتعرضها الشاشة، ويقرّر المستخدم بعدها بنفسه. وهذا قرارُ
//! منتجٍ لا نقصٌ في التنفيذ: «احذف كل ما تجاوز مئة ميغابايت» أمرٌ يبدو مفيدًا
//! حتى تكون فيه ملفات القرص الافتراضي أو أرشيف المشروع. الحذف عمليةٌ أخرى،
//! يختار فيها المستخدم ملفًا بعينه.
//!
//! ## `-size` وحداتها ليست بديهية
//!
//! `M` تعني ميغابايت (‏1048576 بايتًا)، و**`find` تقرّب حجم الملف إلى أعلى**
//! قبل المقارنة: ملفٌ من بايتٍ واحد يُحسب وحدةً كاملة. وبضمّ ذلك إلى `+` التي
//! تعني «أكبر تمامًا من»، تصير `+100M` قراءتُها الصحيحة: **كل ملفٍ يتجاوز مئة
//! ميغابايت**. ملفٌ حجمه مئة ميغابايت بالضبط لا يظهر، وملفٌ يزيد عنها ببايتٍ
//! يظهر. الصياغة الشائعة «مئة ميغابايت فأكثر» كانت ستكون كذبًا بمقدار ملفٍ
//! واحد على الحدّ تمامًا، وهي بالضبط الحالة التي يشكّ فيها المستخدم في الأداة.
//!
//! ## `-type f` — لماذا تُذكر أصلًا
//!
//! بدونها يطبع الأمر المجلدات والروابط الرمزية أيضًا، ولها أحجامٌ لا تعني
//! شيئًا للمستخدم (‏حجم المجلد هو حجم مدخلاته لا محتواه، وحجم الرابط طول
//! نصّه). قائمةٌ فيها «هذا المجلد ‎4 ك.ب» بين نتائج بحثٍ عن الكبير تجعل
//! الأرقام كلها موضع شكّ.
//!
//! ## الحدود المعلَنة
//!
//! `find` لا تعبر نقاط التركيب صعودًا ولا تتبع الروابط الرمزية افتراضيًا على
//! macOS (‏لا `-L`)، فرابطٌ داخل الشجرة يشير إلى مكانٍ آخر لا يجرّ الأمر إليه.
//! وما لا تملك قراءته يُذكر على خرج الخطأ ولا يوقف بقيّة المسح.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "files.find.large";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.find.large.title",
    description_key: "op.files.find.large.description",
    category: Category::Files,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::FIND,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("folder", InputKind::ExistingDir),
        InputSpec::new("min_megabytes", InputKind::Number { min: 1, max: 1_000_000, default: 100 }),
    ],
    sort_order: 40,
    search_terms: &["find", "بحث", "كبير", "large", "big", "حجم", "size", "ملفات كبيرة", "مساحة"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let folder = inputs.dir("folder")?;
    let megabytes = inputs.number("min_megabytes")?;

    let mut argv = Argv::tool(tools::FIND, "explain.find.tool")
        // المسار قبل الشروط: هذه صيغة `find` لا اختيارٌ جماليّ. شرطٌ يسبق
        // المسار يجعل الأداة تقرأ المسار شرطًا وتفشل.
        .path(folder)
        .flag("-type", "explain.find.type_f")
        .value("f")
        .flag("-size", "explain.find.size")
        // العدد من `Number` مُتحقَّق من مداه، والبادئة `+` لا `-` فلا يمكن أن
        // يُقرأ الوسيط راية. لو بدأ بشرطة لرفضته `Argv::value` قبل البناء.
        .value(format!("+{megabytes}M"))
        .reveal(folder);

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

    fn raw(folder: &Path, megabytes: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("folder".to_owned(), RawValue::Path(folder.display().to_string())),
            ("min_megabytes".to_owned(), RawValue::Text(megabytes.to_owned())),
        ])
    }

    fn plan_with(folder: &Path, megabytes: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(folder, megabytes))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("files.find.large must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_writes_nothing() {
        let s = Scratch::new("large-argv").unwrap();
        let folder = s.dir("المجلد");

        let cmd = plan_with(&folder, "250").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/find"));
        assert_eq!(Path::new(&args[0]), folder.as_path());
        assert_eq!(args[1], "-type");
        assert_eq!(args[2], "f");
        assert_eq!(args[3], "-size");
        assert_eq!(args[4], "+250M");
        assert_eq!(args.len(), 5);

        assert!(cmd.artifact.is_none(), "find reports; it never produces a file");
        assert!(cmd.stdout_to.is_none(), "the listing is streamed, not captured into a file");
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
        assert!(!args.iter().any(|a| a == "-delete" || a == "-exec"), "find must only report");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("large-explain").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder, "100").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("large-dashes").unwrap();
        let folder = s.dir("-rf");

        let cmd = plan_with(&folder, "1").unwrap();
        // ‎1 و‎3 رايتان معلَنتان؛ ما عداهما بيانات لا يجوز أن تُقرأ رايات.
        for i in [0usize, 2, 4] {
            assert!(
                cannot_be_read_as_a_flag(&cmd.args[i]),
                "{:?} would be read as a flag",
                cmd.args[i]
            );
        }
    }

    #[test]
    fn the_size_argument_is_built_with_a_plus_never_a_minus() {
        let s = Scratch::new("large-plus").unwrap();
        let folder = s.dir("م");

        for (given, expected) in [("1", "+1M"), ("100", "+100M"), ("1000000", "+1000000M")] {
            let cmd = plan_with(&folder, given).unwrap();
            assert_eq!(cmd.args[4].to_string_lossy(), expected);
        }
    }

    #[test]
    fn a_number_outside_the_declared_range_is_refused_before_it_becomes_an_argument() {
        let s = Scratch::new("large-range").unwrap();
        let folder = s.dir("م");

        for bad in ["0", "-5", "1000001"] {
            assert_eq!(
                refusal(plan_with(&folder, bad)),
                ("err.input.range", Some("min_megabytes")),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_number_that_is_not_a_number_is_refused_not_coerced() {
        let s = Scratch::new("large-nan").unwrap();
        let folder = s.dir("م");
        assert_eq!(
            refusal(plan_with(&folder, "100M -delete")),
            ("err.input.type", Some("min_megabytes"))
        );
    }

    #[test]
    fn shell_syntax_in_the_folder_name_stays_literal_and_adds_no_argument() {
        let s = Scratch::new("large-shellish").unwrap();

        for name in ["مجلد 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let folder = s.dir(name);
            let cmd = plan_with(&folder, "100").unwrap();
            assert_eq!(Path::new(&cmd.args[0]), folder.as_path());
            assert_eq!(cmd.args.len(), 5, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_symlinked_folder_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("large-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "100").unwrap();
        assert_eq!(Path::new(&cmd.args[0]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_the_folder_untouched() {
        let s = Scratch::new("large-clean").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        for _ in 0..10 {
            plan_with(&folder, "100").unwrap();
        }
        assert_eq!(s.names(&folder), vec!["ملف".to_string()]);
    }
}
