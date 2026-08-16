//! خريطة الأقراص والأقسام ووحدات APFS، باستخدام `diskutil list`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/sbin/diskutil list
//! ```
//!
//! لا مدخلات، ولا مسارات، ولا اسم جهاز.
//!
//! ## أخطر أداةٍ في هذا الفهرس، وكيف صارت آمنة
//!
//! `diskutil` تحمل أفعالًا تمحو أقراصًا: `eraseDisk` و`eraseVolume` و
//! `partitionDisk` و`apfs deleteContainer`. ولا شيء في هذه العملية يمنعها
//! **بالفحص** — لا قائمة أفعالٍ ممنوعة، ولا مرشِّح نصوص، ولا تحقّق يقول «هذا
//! ليس `eraseDisk`». الفحوص من هذا النوع تُكتب ثم تُنسى ثغرةٌ فيها.
//!
//! المنع بنيويّ: الفعل `list` سلسلةٌ ثابتة في هذا الملف تدخل الثنائيّة عند
//! الترجمة. والعملية لا تعلن مدخلًا واحدًا، فلا يوجد في المنتج كلّه موضعٌ
//! يستطيع نصٌّ قادم من الواجهة أن يصير فيه وسيطًا لهذه الأداة. ليس لأن أحدًا
//! يمنعه، بل لأن الطريق غير موجود: `RawValue` لا تملك صيغةً تعبّر عن «أمر»
//! أو «وسيط»، وهذا الملف لا يقرأ منها شيئًا أصلًا.
//!
//! ولهذا تحديدًا لا يوجد هنا حقل «القرص»: `diskutil list disk0` صيغةٌ صالحة
//! وقصيرة ومغرية، وثمنها أن يصير في العملية حقلٌ نصّي تلتصق قيمته بأداةٍ
//! أفعالُها الأخرى تمحو. الفائدة — تضييق جدولٍ لا يتجاوز عادةً بضعة أسطر —
//! لا تساوي فتح ذلك الباب. واختبارٌ في هذا الملف يثبّت أن `argv` لا تحمل
//! يومًا فعلًا غير `list`.
//!
//! ## لماذا `diskutil list` لا `df` ولا `system_profiler`
//!
//! * `df` تجيب عن سؤالٍ آخر: كم بقي في كل نظام ملفات **مركَّب**. وحدةٌ غير
//!   مركَّبة، أو قسمٌ بلا نظام ملفات، أو قرصٌ لم يُهيَّأ بعد — لا يظهر فيها
//!   بحال. العمليتان جاران في القسم لا بديلان.
//! * `system_profiler SPStorageDataType` تعطي شيئًا قريبًا، وتستغرق ثوانيَ
//!   وتطبع كتلةً مسهبة. `diskutil list` تطبع جدولًا في جزءٍ من ثانية.
//!
//! ## ما تعرضه ولا نصفّيه
//!
//! سطورٌ لا تخصّ المستخدم مباشرةً: قسم `EFI` على كل قرص مهيَّأ بـ GUID،
//! وحاويات `Apple_APFS_Recovery`، وأقراص اصطناعية (‏`synthesized`) هي وجهُ
//! APFS للحاوية لا جهازًا فعليًّا، وصور أقراص مركَّبة. لا تُحذف: الشاشة تعرض
//! ما طبعته الأداة كما طبعته، وتصفيةُ سطورٍ منها كانت تعني أن التطبيق يفسّر
//! خرج الأداة ويعيد كتابته — وهو ما لا يفعله في أي موضع.
//!
//! ## حدٌّ يُقال ولا يُخفى
//!
//! تعمل بلا صلاحيات مدير، ولذلك قد يمتنع بعض التفصيل عن وحداتٍ مشفّرة لا
//! يملكها المستخدم الحالي. هذا المنتج لا يطلب صلاحيات المدير ولا يعرض على
//! المستخدم أن يمنحها، فالنقص يُعلَن ولا يُعالَج.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "disk.list";

/// الفعل الوحيد الذي تعرفه هذه العملية. سلسلةٌ في الثنائيّة لا قيمةٌ من حقل.
const SUBCOMMAND: &str = "list";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.disk.list.title",
    description_key: "op.disk.list.description",
    category: Category::Disk,
    // تقرأ جداول الأقسام ولا تكتب بايتًا واحدًا على أي قرص.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::DISKUTIL,
    conflict: Conflict::NoArtifact,
    // بلا مدخلات عمدًا. انظر التعليق أعلى الملف: هذا هو ما يجعل الأفعال
    // المدمِّرة في `diskutil` غير قابلةٍ للتعبير عنها من هنا.
    inputs: &[],
    sort_order: 40,
    search_terms: &[
        "diskutil",
        "أقراص",
        "قرص",
        "أقسام",
        "وحدات",
        "disks",
        "partition",
        "volumes",
        "apfs",
        "list",
    ],
    plan,
};

fn plan(_inputs: &Inputs) -> Result<PlannedCommand> {
    // لا `.reveal(…)`: الأقراص ليست ملفات، ونافذة Finder ليست جوابًا عن
    // «أين أنظر؟» هنا. الجواب كلّه في السطور التي تطبعها الأداة.
    Argv::tool(tools::DISKUTIL, "explain.diskutil.tool")
        .explained_value(SUBCOMMAND, "explain.diskutil.list")
        .read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    /// أفعال `diskutil` التي تتلف بيانات. الاختبار يثبّت أن `argv` لا تحمل
    /// واحدًا منها — لا بمدخل، ولا بخطأ مطبعي في هذا الملف يومًا.
    const DESTRUCTIVE_VERBS: &[&str] = &[
        "eraseDisk",
        "eraseVolume",
        "eraseOptical",
        "zeroDisk",
        "randomDisk",
        "secureErase",
        "partitionDisk",
        "reformat",
        "splitPartition",
        "mergePartitions",
        "resizeVolume",
        "deleteVolume",
        "deleteContainer",
        "addVolume",
        "createContainer",
        "unmountDisk",
        "eject",
        "repairDisk",
        "repairVolume",
        "enableOwnership",
        "apfs",
        "coreStorage",
        "appleRAID",
    ];

    fn plan_it() -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &BTreeMap::new())?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("disk.list must be listed");
        assert_eq!(found.category, Category::Disk);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_more() {
        let cmd = plan_it().unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/sbin/diskutil"));
        assert_eq!(args, vec!["list".to_string()]);
        assert!(cmd.artifact.is_none(), "listing produces no file");
        assert!(cmd.stdout_to.is_none(), "the table is printed, not written to disk");
        assert!(cmd.cwd.is_none(), "the answer does not depend on `here`");
        assert!(cmd.reveal_target.is_none(), "disks are not files, so there is nothing to open");
        assert_eq!(cmd.warnings, Vec::<&'static str>::new());
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let cmd = plan_it().unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        // ولا راية معلَنة هنا أصلًا: الوسيط الوحيد فعلٌ قارئ.
        let cmd = plan_it().unwrap();
        for a in &cmd.args {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn the_argv_can_never_carry_a_destructive_subcommand() {
        // الحارس الذي يجعل هذه العملية آمنةً بالبناء لا بالمراجعة: أيّ تعديلٍ
        // مستقبلي يضيف فعلًا يتلف بيانات يسقط هنا، لا في يد مستخدم.
        let cmd = plan_it().unwrap();
        for a in &cmd.args {
            let text = a.to_string_lossy().into_owned();
            assert!(
                !DESTRUCTIVE_VERBS.contains(&text.as_str()),
                "{text:?} is a diskutil verb that writes to a disk"
            );
        }
        assert_eq!(cmd.args.len(), 1, "one reading verb, and no room for a second");
        assert_eq!(SUBCOMMAND, "list");
    }

    #[test]
    fn the_operation_declares_no_inputs_so_no_field_can_reach_the_argv() {
        assert!(SPEC.inputs.is_empty(), "disk.list must stay input-free");
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        // هذا هو الاختبار الذي يحرس الأطروحة كلها على هذه الأداة: مفتاحٌ لا
        // تعرفه المواصفة يُرفض، فلا يصل «eraseDisk» إلى `argv` من أي طريق.
        for smuggled in ["device", "disk", "subcommand", "verb", "args"] {
            let raw = BTreeMap::from([(
                smuggled.to_owned(),
                RawValue::Text("eraseDisk APFS x disk0".to_owned()),
            )]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_cannot_reach_the_argv_because_there_is_nowhere_to_put_it() {
        let baseline = plan_it().unwrap();
        for shellish in ["; diskutil eraseDisk", "$(id)", "`whoami`", "list && rm -rf /", "|| true"]
        {
            let raw = BTreeMap::from([("text".to_owned(), RawValue::Text(shellish.to_owned()))]);
            assert!(crate::value::validate(&SPEC, &raw).is_err(), "{shellish:?} was accepted");
        }
        assert_eq!(plan_it().unwrap().args, baseline.args);
        assert_eq!(baseline.args.len(), 1);
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        assert_eq!(plan_it().unwrap(), plan_it().unwrap());
    }
}
