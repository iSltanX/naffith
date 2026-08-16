//! قسمٌ واحد من تقرير النظام، باستخدام `system_profiler`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/sbin/system_profiler -detailLevel mini <القسم>
//! ```
//!
//! ## لماذا قسمٌ واحد لا التقرير كلّه
//!
//! `system_profiler` بلا معاملات تمرّ على كل مُبلِّغ تعرفه — العتاد، والبرمجيات،
//! والتخزين، والشاشات، والشبكة، وUSB، وThunderbolt، والخطوط، والتطبيقات
//! المثبَّتة، والسجلّات — فتتمهّل عشرات الثواني ثم تطبع آلاف الأسطر.
//!
//! وما كان التطبيق ليعرض ذلك كما هو: كان سيقصّه. وقصُّ جوابٍ أسوأ من عدم
//! طباعته، لأن القارئ لا يعرف أوقع ما يبحث عنه في المحذوف أم لم يكن هناك
//! أصلًا. فاختيار القسم يقع **قبل** التشغيل، من قائمةٍ مغلقة، ويأتي الجواب
//! كاملًا كما طبعته الأداة.
//!
//! ## ولماذا قائمةٌ مغلقة لا حقل نصّ
//!
//! سببان. الأول معماريّ: نصٌّ حرّ هنا يعني أن الواجهة تكتب جزءًا من الأمر، وهو
//! ما بُنيت هذه النواة كي يبقى غير قابلٍ للتعبير عنه — القيمة تخرج من
//! `&'static str` في هذا الملف، والواجهة لا تفعل إلا أن تختار بينها.
//!
//! والثاني سلوكيّ: `system_profiler` تُعطى نوعًا لا تعرفه فتطبع لا شيء وتخرج
//! بنجاح. أي أن خطأً مطبعيًّا واحدًا كان سيظهر «قسمًا فارغًا» لا «اسمًا خاطئًا»،
//! فيستنتج المستخدم أن جهازه بلا شاشات بدل أن يعرف أنه أخطأ حرفًا.
//!
//! ## ولماذا `-detailLevel mini`
//!
//! المستويات ثلاثة: `mini` و`basic` و`full`. و`full` هي المسح المستقصي الذي
//! يجمع كل ما يقدر عليه كل مُبلِّغ، وهو مصدر البطء.
//!
//! و`mini` ليست «أقلّ تفصيلًا» وحسب: هي المستوى المعرَّف بأنه **بلا معلومات
//! شخصية**، فيُسقط ما يعرّف الجهاز بعينه — كالرقم التسلسلي ومعرّف العتاد.
//! وهذا يجمع فائدتين في راية: أسرع مستوًى، وأصلحُ مخرجٍ يُلصق في منتدًى أو
//! تذكرة دعم. و`basic` في الوسط ولا تعطي أيًّا من الضمانتين كاملة.
//!
//! ومع ذلك لا يَعِد هذا التطبيق بأن ما يظهر خالٍ تمامًا مما يدلّ عليك: الوعد
//! وعدُ الأداة لا وعدُنا، والوصف يقول «اقرأ قبل أن تلصق» بدل أن يضمن نيابةً
//! عن غيره.
//!
//! ## ولماذا لا `-json` ولا `-xml`
//!
//! الاثنتان تنتجان صيغةً للآلات: مفاتيح متداخلة وأقواس، تُقرأ ببرنامجٍ لا
//! بالعين. وهذه الشاشة تعرض ما طبعته الأداة لإنسان يقرؤه، فالشكل الافتراضي —
//! عناوين وقيم — هو الشكل المقصود. ومن أراد بيانات للآلة فأمرُه معروضٌ أمامه
//! ينسخه ويضيف ما شاء في صدفته.
//!
//! ## البطء المعلَن قبل وقوعه
//!
//! حتى بقسمٍ واحد وبمستوى `mini`، تجمع الأداة من النظام حيًّا: التخزين
//! والشاشات قد يستغرقان ثوانيَ لا تُطبع فيها كلمة. تحذير `warn.profiler.slow`
//! يقول ذلك قبل الضغط، لأن انتظارًا صامتًا بلا إنذار يُقرأ تعليقًا لا عملًا.
//! والإلغاء في أي لحظة بلا أثر: لا شيء يُكتب.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تغيّر إعدادًا ولا تكتب ملفًا. `system_profiler` مُبلِّغة لا مُهيِّئة، وهذا
//! الأمر يقرأ ويطبع على الشاشة. وحفظ التقرير في ملف عمليةٌ أخرى لا مكسبٌ
//! جانبي هنا.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.report";

/// مستوى التفصيل. ثابتٌ في الشيفرة لا خيارٌ في الشاشة: `full` تعني الانتظار
/// الطويل وتسريب المعرّفات معًا، و`basic` لا تضمن أيًّا من الاثنين. حقلٌ
/// يعرضهما كان سيجعل الشاشة تسأل سؤالًا جوابه معروف.
const DETAIL_LEVEL: &str = "mini";

/// الأقسام المعروضة. أربعةٌ تجيب الأسئلة التي تُطرح فعلًا، لا كل ما تعرفه
/// الأداة: قائمةٌ من عشرين نوعًا تصير بحثًا داخل قائمةٍ لا اختيارًا منها.
const DETAILS: &[ChoiceOption] = &[
    ChoiceOption::new("SPHardwareDataType", "choice.report.hardware"),
    ChoiceOption::new("SPSoftwareDataType", "choice.report.software"),
    ChoiceOption::new("SPStorageDataType", "choice.report.storage"),
    ChoiceOption::new("SPDisplaysDataType", "choice.report.displays"),
];

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.report.title",
    description_key: "op.system.report.description",
    category: Category::System,
    // مُبلِّغة لا مُهيِّئة: تقرأ وتطبع، ولا تكتب على الجهاز شيئًا.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::SYSTEM_PROFILER,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("detail", InputKind::Choice { options: DETAILS })],
    sort_order: 50,
    search_terms: &[
        "system_profiler",
        "تقرير",
        "report",
        "عتاد",
        "hardware",
        "معلومات",
        "info",
        "شاشة",
        "display",
        "تخزين",
        "storage",
        "ذاكرة",
        "memory",
        "مواصفات",
        "specs",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    // `&'static str` من قائمة المواصفة لا نسخةٌ من نصّ الواجهة: ما يدخل الأمر
    // يخرج من الشيفرة المُترجَمة، والواجهة لا تفعل إلا أن تختار بينها.
    let detail = inputs.choice("detail")?;

    Argv::tool(tools::SYSTEM_PROFILER, "explain.profiler.tool")
        .flag("-detailLevel", "explain.profiler.detail_level")
        .explained_value(DETAIL_LEVEL, "explain.profiler.mini")
        .explained_value(detail, "explain.profiler.data_type")
        // ثابتٌ لا مشروط: الانتظار صفةُ الأداة في كل قسم، ولو تفاوت طوله.
        .warn("warn.profiler.slow")
        .read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    /// الرايات المعلَنة في هذا الأمر. ما عداها في `argv` بيانات.
    const DECLARED_FLAGS: &[&str] = &["-detailLevel"];

    fn raw(detail: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([("detail".to_owned(), RawValue::Text(detail.to_owned()))])
    }

    fn plan_with(detail: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(detail))?)
    }

    fn args_of(cmd: &PlannedCommand) -> Vec<String> {
        cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
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
        let found = listed.iter().find(|o| o.id == ID).expect("system.report must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.inputs.len(), 1);
        assert!(found.inputs[0].required, "a report of nothing is not a report");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_more() {
        let cmd = plan_with("SPHardwareDataType").unwrap();

        assert_eq!(cmd.program, Path::new("/usr/sbin/system_profiler"));
        assert_eq!(args_of(&cmd), vec!["-detailLevel", "mini", "SPHardwareDataType"]);
        assert!(cmd.artifact.is_none(), "the report is printed, not saved");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none());
        assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let cmd = plan_with("SPSoftwareDataType").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_declared_section_reaches_the_command_verbatim() {
        // القيمة تعبر بلا إعادة كتابة: ما يُعرض في «سَطْر» هو ما تتلقّاه الأداة.
        for option in DETAILS {
            let cmd = plan_with(option.value).unwrap();
            assert_eq!(cmd.args[2], std::ffi::OsString::from(option.value));
            assert_eq!(cmd.args.len(), 3, "{} must not add arguments", option.value);
        }
    }

    #[test]
    fn the_detail_level_is_never_taken_from_the_screen() {
        // المستوى ثابتٌ في الشيفرة. لو صار حقلًا يومًا، هذا هو الاختبار الذي
        // يسقط ويُراجَع عنده القرار — لا شاشةٌ تعرض `full` وتُعلّق دقيقة.
        for option in DETAILS {
            let cmd = plan_with(option.value).unwrap();
            assert_eq!(cmd.args[1], std::ffi::OsString::from("mini"));
        }
    }

    #[test]
    fn a_section_the_specification_does_not_declare_is_refused() {
        // القائمة مغلقة: الأداة نفسها تقبل نوعًا مجهولًا وتطبع فراغًا وتنجح،
        // فالرفض هنا هو ما يحوّل خطأً صامتًا إلى رسالةٍ على الحقل.
        for bogus in
            ["SPUSBDataType", "SPApplicationsDataType", "sphardwaredatatype", "-detailLevel", ""]
        {
            assert_eq!(
                refusal(plan_with(bogus)),
                ("err.input.type", Some("detail")),
                "{bogus:?} must be refused"
            );
        }
    }

    #[test]
    fn the_section_is_required_and_its_absence_names_its_own_field() {
        let r = crate::value::validate(&SPEC, &BTreeMap::new()).map(|i| plan(&i).unwrap());
        assert_eq!(refusal(r), ("err.input.missing", Some("detail")));
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        let mut smuggled = raw("SPHardwareDataType");
        smuggled.insert("detailLevel".to_owned(), RawValue::Text("full".to_owned()));
        let r = crate::value::validate(&SPEC, &smuggled).map(|i| plan(&i).unwrap());
        assert_eq!(refusal(r), ("err.input.unexpected", None));
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        for option in DETAILS {
            let cmd = plan_with(option.value).unwrap();
            for a in &cmd.args {
                if DECLARED_FLAGS.contains(&a.to_string_lossy().as_ref()) {
                    continue;
                }
                assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
            }
        }
    }

    #[test]
    fn shell_syntax_in_the_chosen_section_never_becomes_an_argument() {
        // الحقل قائمةٌ مغلقة، فالنصّ الشِبه-صدفيّ لا يُقتبس ولا يُهرَّب: يُرفض
        // قبل أن يبلغ باني الوسائط أصلًا، والأمر يبقى كما هو.
        let baseline = plan_with("SPStorageDataType").unwrap();
        for shellish in
            ["SPHardwareDataType; rm -rf ~", "$(whoami)", "`id`", "a & b", "SPHardwareDataType x"]
        {
            assert_eq!(
                refusal(plan_with(shellish)),
                ("err.input.type", Some("detail")),
                "{shellish:?} was accepted"
            );
        }
        assert_eq!(plan_with("SPStorageDataType").unwrap().args, baseline.args);
        assert_eq!(baseline.args.len(), 3, "three arguments, whatever the screen sends");
    }

    #[test]
    fn every_token_carries_an_explanation() {
        // أربعة رموز، ولا واحد منها مفهومٌ بالنظر: اسم الأداة، ومستوى تفصيل،
        // وكلمة `mini`، ومعرّفٌ بصيغة `SP…DataType`.
        let cmd = plan_with("SPDisplaysDataType").unwrap();
        for token in cmd.explain.iter() {
            assert!(token.key.is_some(), "{:?} carries no explanation", token.token);
        }
        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert_eq!(cmd.explain[1].role, TokenRole::Flag);
    }

    #[test]
    fn the_wait_is_announced_on_every_single_run() {
        for option in DETAILS {
            assert_eq!(plan_with(option.value).unwrap().warnings, vec!["warn.profiler.slow"]);
        }
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        let (first, second) = (plan_with("SPHardwareDataType"), plan_with("SPHardwareDataType"));
        assert_eq!(first.unwrap(), second.unwrap());
    }
}
