//! إصدار النظام: الاسم والرقم ورقم البناء، باستخدام `sw_vers`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/sw_vers
//! ```
//!
//! أداةٌ بلا وسائط. ليس في هذا الأمر موضعٌ واحد يمكن أن يصير فيه نصٌّ قادمٌ من
//! الواجهة وسيطًا — لا لأن فحصًا يمنعه، بل لأن الوسائط كلها غير موجودة.
//!
//! ## لماذا `sw_vers` لا `system_profiler SPSoftwareDataType`
//!
//! السؤال هنا صغير: أي نسخةٍ من macOS هذه؟ و`system_profiler` تجيبه بعد ثوانٍ
//! وفي فقرةٍ تحمل معه ما لم يُسأل عنه — زمن التشغيل، ونسخة النواة، واسم
//! الجهاز، واسم المستخدم. من ينسخ ذلك إلى منتدًى أو تذكرة دعم ينسخ معه ما
//! يعرّفه. `sw_vers` تجيب في أجزاء من الثانية بثلاثة أسطر، وليس فيها سطرٌ
//! يقول من صاحب الجهاز.
//!
//! والتقرير المسهب ليس ممنوعًا في هذا المنتج: هو عمليةٌ أخرى قائمةٌ بذاتها
//! («تقرير النظام») يختار فيها المستخدم القسم ويعرف مقدّمًا أنها تتمهّل. الفرق
//! بين العمليتين هو الفرق بين السؤالين، لا اختصارٌ لإحداهما.
//!
//! ## ولماذا لا `uname -a`
//!
//! `uname` تطبع نسخة نواة Darwin — رقمٌ كـ `25.6.0` — وهو ليس الرقم الذي
//! يسأل عنه أحد ولا الذي تطلبه صفحات التنزيل. تحويله إلى رقم macOS يحتاج
//! جدول مقابلةٍ يُبحث عنه، وجوابٌ يحتاج بحثًا ثانيًا ليس جوابًا.
//!
//! ## ولماذا بلا رايات
//!
//! `sw_vers` تقبل `-productName` و`-productVersion` و`-buildVersion`، وكلٌّ
//! منها يطبع سطرًا واحدًا **بلا عنوان**: رقمٌ عارٍ لا يقول أي شيءٍ هو. واختيار
//! إحداها كان يعني أن التطبيق يقرّر أي ثلثٍ من الجواب يستحقه المستخدم. الشكل
//! الافتراضي معنون ومكتمل، وهو ثلاثة أسطر — أي أن «الاختصار» كان سيوفّر سطرين
//! ويأخذ المعنى.
//!
//! ## رقمٌ لا يكذب هنا
//!
//! ‏`sw_vers` تتأثر بمتغيّر البيئة `SYSTEM_VERSION_COMPAT`: النظام يعيد به رقمًا
//! متوافقًا مع الماضي لبرامج بُنيت على أدواتٍ قديمة. النواة تمسح البيئة قبل
//! الإطلاق ولا تُبقي إلا `HOME` و`LC_ALL` (انظر `executor.rs`)، فالرقم الظاهر
//! هنا هو رقم النظام لا رقمًا مُتنازلًا عنه لأحد.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تكتب شيئًا ولا تتصل بشبكة ولا تسأل أبل عن تحديث. تقرأ ما يعلنه النظام
//! عن نفسه محليًّا، وتتوقّف عند ذلك — فليس فيها جواب عن «هل يوجد تحديث؟».

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.info";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.info.title",
    description_key: "op.system.info.description",
    category: Category::System,
    // ثلاثة أسطر من قراءة. لا تُنشئ ولا تعدّل ولا تتلف.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::SW_VERS,
    conflict: Conflict::NoArtifact,
    inputs: &[],
    sort_order: 20,
    search_terms: &[
        "sw_vers",
        "إصدار",
        "version",
        "نظام",
        "system",
        "macos",
        "ماك",
        "بناء",
        "build",
        "أبوت",
        "about",
        "uname",
    ],
    plan,
};

fn plan(_inputs: &Inputs) -> Result<PlannedCommand> {
    // أمرٌ من رمزٍ واحد: اسم الأداة وحده. الشرح كلّه في مفتاحٍ واحد لأن
    // الأمر كلّه رمزٌ واحد — ولا يوجد هنا «وسيطٌ بلا شرح» ولا يمكن أن يوجد.
    Argv::tool(tools::SW_VERS, "explain.sw_vers.tool").read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

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
        let found = listed.iter().find(|o| o.id == ID).expect("system.info must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert!(found.inputs.is_empty(), "this screen has no form");
    }

    #[test]
    fn the_argv_is_the_tool_alone() {
        let cmd = plan_it().unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/sw_vers"));
        assert!(cmd.args.is_empty(), "no flag is chosen, so no third of the answer is dropped");
        assert!(cmd.artifact.is_none(), "reading a version produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "the version of the system does not depend on `here`");
        assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
        assert!(cmd.warnings.is_empty(), "a three-line answer needs no caveat");
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
    fn the_single_token_is_the_tool_and_it_is_explained() {
        // الأمر رمزٌ واحد. لو خلا من مفتاح شرحٍ لكانت شاشة «سَطْر» فارغةً
        // تمامًا في هذه العملية — لا مختصرةً، بل خاليةً.
        let cmd = plan_it().unwrap();
        assert_eq!(cmd.explain.len(), 1);
        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert_eq!(cmd.explain[0].key, Some("explain.sw_vers.tool"));
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        // لا وسيط أصلًا. الاختبار يبقى لأن غيابه اليوم لا يمنع إضافته غدًا،
        // وحينها يجب أن يسقط هنا لا على جهاز مستخدم.
        let cmd = plan_it().unwrap();
        for a in &cmd.args {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        for smuggled in ["field", "productVersion", "-buildVersion", "verbose"] {
            let raw = BTreeMap::from([(smuggled.to_owned(), RawValue::Text("1".to_owned()))]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_cannot_reach_the_argv_because_there_is_nowhere_to_put_it() {
        let baseline = plan_it().unwrap();
        for shellish in ["; rm -rf ~", "$(whoami)", "`id`", "a & b", "> /etc/hosts"] {
            let raw = BTreeMap::from([("what".to_owned(), RawValue::Text(shellish.to_owned()))]);
            assert!(crate::value::validate(&SPEC, &raw).is_err(), "{shellish:?} was accepted");
        }
        assert_eq!(plan_it().unwrap().args, baseline.args);
        assert!(baseline.args.is_empty(), "sw_vers takes no arguments at all");
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        assert_eq!(plan_it().unwrap(), plan_it().unwrap());
    }
}
