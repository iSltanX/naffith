//! قياس المساحة التي يشغلها مجلد ومجلداته الفرعية، باستخدام `du`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/du -h -d <العمق> <المجلد>
//! ```
//!
//! ## `du` تقيس **ما يشغله القرص** لا مجموع أحجام الملفات
//!
//! هذان رقمان مختلفان، والخلط بينهما هو أول ما يجعل المستخدم يظنّ أن الأداة
//! تكذب:
//!
//! * ملفٌ من بايتٍ واحد يشغل كتلةً كاملة، فألفُ ملفٍ صغير تظهر أكبر بكثير من
//!   مجموع أحجامها.
//! * وملفٌ مضغوطٌ على مستوى نظام الملفات (‏APFS تفعل ذلك لكثيرٍ من ملفات
//!   النظام) يظهر أصغر من حجمه المعلَن.
//! * ونسخة APFS (‏clone) تشترك في كتلها مع أصلها، فلا تُحسب مرتين.
//!
//! وهذا هو **الرقم الصحيح** لسؤال «كم يأكل هذا المجلد من قرصي؟»، وهو السؤال
//! المقصود هنا. أمّا التقدير المعروض في الشاشة قبل التنفيذ فمصدره
//! `estimate::tree_size` — مجموع أحجام الملفات — وقد يختلف عن ناتج `du`
//! لهذه الأسباب نفسها.
//!
//! ## `-d` والعمق صفرًا
//!
//! `-d 0` تعني سطرًا واحدًا: المجموع الكلّي للمجلد. و`-d 1` — وهي المبدئية —
//! تعني سطرًا لكل مجلدٍ فرعيّ مباشر، وهي الصورة التي يبحث عنها من يريد أن يعرف
//! *أين* ذهبت المساحة. والسقف ستّة: `du` تطبع سطرًا لكل مجلد حتى العمق
//! المطلوب، وشجرةُ مشروعٍ فيها `node_modules` تُخرج عند العمق العاشر عشرات
//! آلاف الأسطر — أي جدارَ نصٍّ لا جوابًا.
//!
//! ## **بلا** `--`
//!
//! `du` في BSD لا توثّق فاصل نهاية الرايات، ووسيطٌ لا تفهمه الأداة ليس حزام
//! أمانٍ بل عطبٌ محتمل. والمسار هنا مطلق ومُتحقَّق منه فيبدأ بـ`/` حتمًا، فلا
//! يمكن أن يُقرأ راية. القاعدة في هذا المنتج أن يُضاف `--` حيث يعمل لا حيث
//! يبدو مرتّبًا.
//!
//! ## ما لا تفعله
//!
//! تقرأ ولا تكتب. وما لا تملك قراءته تذكره على خرج الخطأ وتُكمل، فالمجموع قد
//! يكون أقلّ من الحقيقة على شجرةٍ فيها ما لا يُقرأ.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "files.tree.size";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.tree.size.title",
    description_key: "op.files.tree.size.description",
    category: Category::Files,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::DU,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("folder", InputKind::ExistingDir),
        InputSpec::new("depth", InputKind::Number { min: 0, max: 6, default: 1 }),
    ],
    sort_order: 70,
    search_terms: &["du", "حجم", "size", "مساحة", "disk usage", "قياس", "مجلد", "usage"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let folder = inputs.dir("folder")?;
    let depth = inputs.number("depth")?;

    // مسحةٌ واحدة تخدم أمرين: الرقم المعروض في الشاشة، والحكم على تمامه.
    // مسحةٌ ثانية كانت ستضاعف كلفة كل ضغطة مفتاح في النموذج بلا أن تعرف شيئًا
    // جديدًا — والتخطيط يقع عند كل تغيير في الحقول.
    let size = crate::estimate::tree_size(folder);

    let mut argv = Argv::tool(tools::DU, "explain.du.tool")
        .flag("-h", "explain.du.human")
        .flag("-d", "explain.du.depth")
        // `0..=6` من `Number`، فلا يمكن أن يحمل شرطة. و`Argv::value` تفحص ذلك
        // ثانيةً ولا تعتمد على المتصل بها.
        .value(depth.to_string())
        .path(folder)
        // التقدير يُحمل مع الأمر لا يُعاد حسابه في `planner`.
        .with_estimate(size)
        .reveal(folder);

    if let Some(key) = warn_if_resolved(inputs, "folder", folder, "warn.source.resolved") {
        argv = argv.warn(key);
    }
    // مسحٌ توقّف عند حدّه يجعل الرقم المعروض حدًّا أدنى لا مجموعًا. التحذير عن
    // **التقدير** لا عن ناتج `du`: الأداة نفسها لا تتوقّف عند حدّ.
    if !size.complete {
        argv = argv.warn("warn.size.partial");
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

    fn raw(folder: &Path, depth: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("folder".to_owned(), RawValue::Path(folder.display().to_string())),
            ("depth".to_owned(), RawValue::Text(depth.to_owned())),
        ])
    }

    fn plan_with(folder: &Path, depth: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(folder, depth))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("files.tree.size must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_writes_nothing() {
        let s = Scratch::new("du-argv").unwrap();
        let folder = s.dir("المجلد");
        std::fs::write(folder.join("ملف"), vec![0u8; 64]).unwrap();

        let cmd = plan_with(&folder, "2").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/du"));
        assert_eq!(args[0], "-h");
        assert_eq!(args[1], "-d");
        assert_eq!(args[2], "2");
        assert_eq!(Path::new(&args[3]), folder.as_path());
        assert_eq!(args.len(), 4);

        assert!(cmd.artifact.is_none(), "du measures; it never produces a file");
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
    }

    #[test]
    fn the_end_of_flags_separator_is_deliberately_absent() {
        // ‏`du` في BSD لا توثّقه. وسيطٌ لا تفهمه الأداة ليس حزام أمان.
        let s = Scratch::new("du-noflagsep").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder, "1").unwrap();
        assert!(!cmd.args.iter().any(|a| a == "--"), "du does not document `--`");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("du-explain").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder, "1").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("du-dashes").unwrap();
        let folder = s.dir("-rf");

        let cmd = plan_with(&folder, "0").unwrap();
        // ‎0 و‎1 رايتان معلَنتان؛ ما بعدهما بيانات.
        for a in cmd.args.iter().skip(2) {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn a_depth_of_zero_is_a_legitimate_answer_not_a_missing_one() {
        let s = Scratch::new("du-zero").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder, "0").unwrap();
        assert_eq!(cmd.args[2].to_string_lossy(), "0");
    }

    #[test]
    fn a_depth_outside_the_declared_range_is_refused_before_it_becomes_an_argument() {
        let s = Scratch::new("du-range").unwrap();
        let folder = s.dir("م");

        for bad in ["-1", "7", "100"] {
            assert_eq!(
                refusal(plan_with(&folder, bad)),
                ("err.input.range", Some("depth")),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_depth_that_is_not_a_number_is_refused_not_coerced() {
        let s = Scratch::new("du-nan").unwrap();
        let folder = s.dir("م");
        assert_eq!(refusal(plan_with(&folder, "1 -x")), ("err.input.type", Some("depth")));
    }

    #[test]
    fn the_estimate_is_measured_at_plan_time_and_carried_with_the_command() {
        let s = Scratch::new("du-estimate").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("أ"), vec![0u8; 100]).unwrap();
        std::fs::write(folder.join("ب"), vec![0u8; 250]).unwrap();

        let e = plan_with(&folder, "1").unwrap().estimate.expect("du must carry an estimate");
        assert_eq!(e.bytes, 350);
        assert_eq!(e.entries, 2);
        assert!(e.complete);
    }

    #[test]
    fn an_ordinary_folder_raises_no_noisy_warning() {
        let s = Scratch::new("du-quiet").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("ملف"), vec![0u8; 64]).unwrap();

        let cmd = plan_with(&folder, "1").unwrap();
        assert!(cmd.warnings.is_empty(), "unexpected warnings: {:?}", cmd.warnings);
    }

    #[test]
    fn shell_syntax_in_the_folder_name_stays_literal_and_adds_no_argument() {
        let s = Scratch::new("du-shellish").unwrap();

        for name in ["مجلد 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let folder = s.dir(name);
            let cmd = plan_with(&folder, "1").unwrap();
            assert_eq!(Path::new(&cmd.args[3]), folder.as_path());
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_symlinked_folder_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("du-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "1").unwrap();
        assert_eq!(Path::new(&cmd.args[3]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_the_folder_untouched() {
        let s = Scratch::new("du-clean").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        for _ in 0..10 {
            plan_with(&folder, "1").unwrap();
        }
        assert_eq!(s.names(&folder), vec!["ملف".to_string()]);
    }
}
