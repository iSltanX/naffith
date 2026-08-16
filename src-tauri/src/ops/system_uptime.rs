//! منذ متى يعمل الجهاز، وكم من العمل ينتظر المعالج — باستخدام `uptime`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/uptime
//! ```
//!
//! سطرٌ واحد يحمل أربعة أشياء: الساعة الآن، والمدّة منذ الإقلاع، وعدد جلسات
//! الدخول المسجّلة، وثلاثة أرقام هي متوسّطات الحمل.
//!
//! ## الأرقام الثلاثة، وما تعنيه فعلًا
//!
//! `load averages: 2.13 2.45 2.60` — متوسّط الدقيقة، ثم الخمس، ثم الخمس عشرة.
//! ثلاثة أرقام لا رقم واحد لأن المفيد فيها **اتجاهها**: الأول أصغر من الثالث
//! يعني أن الضغط ينحسر، والعكس يعني أنه يتصاعد. رقمٌ واحد كان سيقول «الآن»
//! ولا يقول «إلى أين».
//!
//! **وليست نسبةً مئوية.** هي طول طابور: متوسّط عدد ما كان يعمل أو ينتظر دورًا
//! على معالج. ولذلك لا سقف لها عند ١٠٠، ولا معنى لقراءتها بلا معرفة عدد
//! الأنوية: حملٌ قدره ٨ على جهازٍ بثمانية أنوية يعني أنها مشغولة تمامًا ولا
//! شيء ينتظر، ونفسُه على جهازٍ بأربعة يعني أن نصف العمل واقفٌ في الطابور.
//! فالحدّ الذي يُقرأ عنده الرقم هو عدد الأنوية، لا رقمٌ مطلق.
//!
//! والعدّ لِما ينتظر **معالجًا**. جهازٌ يتلكّأ على قرصٍ بطيء أو على شبكةٍ
//! متوقّفة قد يعطي حملًا متواضعًا وهو يبدو جامدًا — فانخفاض الرقم ليس شهادةً
//! بأن كل شيء على ما يرام، وارتفاعُه لا يقول **ما** الذي يزدحم. جواب «ما»
//! في عملية «العمليات الجارية» بجوارها.
//!
//! ## لماذا `uptime` لا `sysctl -n vm.loadavg`
//!
//! الثانية تطبع `{ 2.13 2.45 2.60 }` — قوسان وثلاثة أرقام بلا عنوان، فلا
//! يعرف قارئها أيّها الدقيقة وأيّها الربع ساعة، ولا تقول شيئًا عن زمن
//! الإقلاع. و`w` تطبع نفس السطر ثم تُتبعه بجدول الجلسات المفتوحة، وهو سؤالٌ
//! آخر لم يُطرح هنا.
//!
//! ## ولماذا بلا رايات
//!
//! `uptime` على macOS ‏— وهي `w` نفسها بوجهٍ آخر — لا تقبل ما يغيّر شكل
//! السطر. والرايات التي تُنسخ عن لينكس (`-p` للصياغة المقروءة، و`-s` لتاريخ
//! الإقلاع) لا وجود لها هنا، وكتابتُها تُخرج خطأً لا صياغةً ألطف. فالأمر
//! يبقى بلا وسائط، ويبقى ذلك قرارًا مذكورًا لا سهوًا.
//!
//! ## ولماذا بلا تحذير في كل تشغيل
//!
//! سوء قراءة الحمل احتمالٌ حقيقي، لكن الأمر هنا رمزٌ واحد وشرحُه هو الشاشة
//! كلها: تحذيرٌ يكرّر ما تحته بسطرين يعلّم المستخدم أن يتخطّى التحذيرات. حيث
//! يكون التحذير هو **الشيء الوحيد** الذي لا يقوله الأمر — كما في تفريغ ذاكرة
//! DNS — يُكتب؛ وهنا الشرح يكفي.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تكتب شيئًا، ولا تعيد ضبط عدّاد، ولا تُنهي جلسة. وعدد الجلسات في السطر
//! عدّادُ ما سجّله النظام من دخول — كل نافذة طرفية واحدةٌ منها — لا عددُ
//! الأشخاص أمام الجهاز.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.uptime";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.uptime.title",
    description_key: "op.system.uptime.description",
    category: Category::System,
    // سطرٌ واحد من قراءة. لا تكتب ولا تعدّل ولا تعيد ضبط شيء.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::UPTIME,
    conflict: Conflict::NoArtifact,
    inputs: &[],
    sort_order: 30,
    search_terms: &[
        "uptime",
        "تشغيل",
        "مدة",
        "إقلاع",
        "boot",
        "حمل",
        "load",
        "load average",
        "متوسط",
        "بطء",
        "slow",
        "w",
    ],
    plan,
};

fn plan(_inputs: &Inputs) -> Result<PlannedCommand> {
    // رمزٌ واحد، وشرحٌ واحد يحمل كل ما يحتاجه قارئ السطر: الأرقام الثلاثة
    // وما تعنيه. لا `.reveal(…)` — لم يُنتج شيءٌ يُفتح.
    Argv::tool(tools::UPTIME, "explain.uptime.tool").read_only()
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
        let found = listed.iter().find(|o| o.id == ID).expect("system.uptime must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert!(found.inputs.is_empty(), "this screen has no form");
    }

    #[test]
    fn the_argv_is_the_tool_alone() {
        let cmd = plan_it().unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/uptime"));
        assert!(cmd.args.is_empty(), "the Linux `-p` and `-s` do not exist here");
        assert!(cmd.artifact.is_none(), "reading a load average produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "the machine's uptime does not depend on `here`");
        assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
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
    fn the_single_token_carries_the_whole_explanation() {
        // الأمر رمزٌ واحد، فمفتاحه هو كل ما سيقرؤه المستخدم عن الأرقام الثلاثة.
        // خلوّه من مفتاح يجعل شاشة «سَطْر» فارغةً لا مختصرة.
        let cmd = plan_it().unwrap();
        assert_eq!(cmd.explain.len(), 1);
        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert_eq!(cmd.explain[0].key, Some("explain.uptime.tool"));
    }

    #[test]
    fn the_explanation_replaces_a_warning_rather_than_being_doubled_by_one() {
        // قرارٌ معلَن: لا تحذير هنا. لو أُضيف يومًا فليكن عن قصدٍ لا بالعدوى
        // من عمليةٍ مجاورة، وهذا الاختبار هو الموضع الذي يُراجَع فيه القرار.
        assert!(plan_it().unwrap().warnings.is_empty());
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let cmd = plan_it().unwrap();
        for a in &cmd.args {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        for smuggled in ["since", "-p", "pretty", "format"] {
            let raw = BTreeMap::from([(smuggled.to_owned(), RawValue::Text("1".to_owned()))]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_cannot_reach_the_argv_because_there_is_nowhere_to_put_it() {
        let baseline = plan_it().unwrap();
        for shellish in ["; shutdown -r now", "$(whoami)", "`id`", "a && b", "| tee /tmp/x"] {
            let raw = BTreeMap::from([("extra".to_owned(), RawValue::Text(shellish.to_owned()))]);
            assert!(crate::value::validate(&SPEC, &raw).is_err(), "{shellish:?} was accepted");
        }
        assert_eq!(plan_it().unwrap().args, baseline.args);
        assert!(baseline.args.is_empty(), "uptime takes no arguments at all");
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        assert_eq!(plan_it().unwrap(), plan_it().unwrap());
    }
}
