//! تفريغ ذاكرة DNS المؤقتة — نصفَ التفريغ، ومعه قولُ ذلك صراحةً.
//!
//! ## الوصفة الشائعة، وما ينفّذه هذا التطبيق منها
//!
//! الوصفة المنسوخة في كل مكان سطران:
//!
//! ```text
//! sudo dscacheutil -flushcache; sudo killall -HUP mDNSResponder
//! ```
//!
//! وهذا التطبيق ينفّذ الأول وحده:
//!
//! ```text
//! /usr/bin/dscacheutil -flushcache
//! ```
//!
//! ## لماذا النصف، ولماذا هذا هو بيت القصيد
//!
//! ثلاثة أسباب، كلٌّ منها يكفي وحده:
//!
//! **أولًا: لا صلاحيات مدير.** هذا التطبيق لا يطلب كلمة المرور ولن يطلبها.
//! برنامجٌ يسأل صلاحيات المدير كي يفرّغ ذاكرةً مؤقتة إنما يسأل صلاحية أن يفعل
//! **كل شيء** لأجل عملٍ لا يحتاج شيئًا تقريبًا، ومن يمنحها مرّةً يمنحها لكل
//! ما يأتي بعدها. الرفض هنا قرار معماري لا نقصٌ في الإنجاز.
//!
//! **ثانيًا: لا صدفة.** الوصفة سطرٌ فيه فاصلة منقوطة، والفاصلة المنقوطة صيغةُ
//! صدفةٍ لا وسيطُ أمر. النواة تطلق برنامجًا مطلق المسار بمتّجه وسائط، وليس في
//! المسار كلّه مفسّرٌ يفهم `;` — فالوصفة كما تُنسخ غير قابلة للتعبير عنها في
//! هذا المنتج أصلًا، لا ممنوعةً بفحصٍ يمكن أن يُنسى.
//!
//! **ثالثًا: `killall -HUP` إشارةٌ إلى عمليةٍ بالاسم.** فعلٌ يصيب أي عملية
//! تحمل ذلك الاسم، وهو من جنسٍ آخر غير قراءة الملفات وكتابتها الذي بُني عليه
//! هذا التطبيق.
//!
//! ## أثر ذلك، بلا تجميل
//!
//! `dscacheutil -flushcache` تفرّغ ذاكرة خدمة الدليل (Directory Service).
//! و`mDNSResponder` — وهي العملية التي تجيب استعلامات DNS لأكثر التطبيقات على
//! macOS الحديثة — تحتفظ بذاكرتها الخاصة، ولا تُرسَل إليها إشارة هنا. فما
//! تحفظه يبقى محفوظًا، والتفريغ **جزئيّ**.
//!
//! أي أن الأثر الفعلي قد يكون لا شيء يُلحظ: اسمٌ عالق قد يظلّ عالقًا. وتحذير
//! `warn.dns.partial_flush` يقول ذلك في كل تشغيل، قبل الضغط لا بعده.
//!
//! **ولماذا نعرضها أصلًا إن كانت ناقصة؟** لأن البديل ليس عمليةً كاملة، بل
//! أحد اثنين: أن نطلب كلمة مرور — وقد رُفض — أو أن نصمت فيبقى المستخدم ينسخ
//! سطرًا بـ`sudo` من منتدًى لا يعرف من كتبه. شرحُ الحدّ خيرٌ من ادّعاءٍ
//! ومن سؤالِ كلمة مرور، وهذه العملية موجودة لتقول الحدّ بقدر ما هي موجودة
//! لتفرّغ الذاكرة.
//!
//! ## لماذا `Modifies` لا `Safe`
//!
//! لا يُكتب ملف ولا يُتلف شيء، لكن حالة الجهاز بعد التشغيل غير حالته قبله:
//! ذاكرةٌ كانت ممتلئة صارت فارغة، والاستعلامات التالية تذهب إلى الشبكة.
//! و`Safe` في هذا المنتج تعني «لا يتغيّر شيء»، وإعلانها هنا كان كذبًا صغيرًا
//! في الحقل الذي تبني عليه الواجهة سؤال التأكيد.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تغيّر خوادم DNS ولا تلمس `/etc/hosts` ولا تعيد تشغيل الشبكة ولا تفصل
//! واجهةً وتعيد وصلها. ولا تمسّ ذاكرة المتصفّح: لكروم ذاكرة DNS خاصة به داخله،
//! وتفريغُها من داخله لا من هنا. وبلا مدخلات: لا اسم نطاقٍ من الشاشة يستطيع
//! أن يصير وسيطًا، لأن الوسيط الوحيد رايةٌ مكتوبة في هذا الملف.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.dns.flush";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.dns.flush.title",
    description_key: "op.system.dns.flush.description",
    category: Category::System,
    // تغيّر حالةً على الجهاز ولا تتلف شيئًا: ذاكرةٌ مؤقتة تُفرَّغ، والنظام
    // يعيد ملأها من الشبكة عند أول استعلام.
    danger: Danger::Modifies,
    visibility: Visibility::Production,
    tool: tools::DSCACHEUTIL,
    conflict: Conflict::NoArtifact,
    inputs: &[],
    sort_order: 40,
    search_terms: &[
        "dscacheutil",
        "dns",
        "تفريغ",
        "flush",
        "ذاكرة",
        "cache",
        "نطاق",
        "domain",
        "شبكة",
        "network",
        "mdnsresponder",
        "موقع لا يفتح",
    ],
    plan,
};

fn plan(_inputs: &Inputs) -> Result<PlannedCommand> {
    Argv::tool(tools::DSCACHEUTIL, "explain.dscacheutil.tool")
        .flag("-flushcache", "explain.dscacheutil.flushcache")
        // ثابتٌ لا مشروط: نقصان الأثر صفةٌ في كل تشغيل لا حالةٌ تطرأ أحيانًا.
        // وهو التحذير الوحيد في هذا المنتج الذي يشرح ما **لم** يُكتب في الأمر.
        .warn("warn.dns.partial_flush")
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
    const DECLARED_FLAGS: &[&str] = &["-flushcache"];

    fn plan_it() -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &BTreeMap::new())?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("system.dns.flush must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Modifies, "it changes machine state, so it is not Safe");
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert!(found.inputs.is_empty(), "no domain name can reach this command");
    }

    #[test]
    fn the_argv_is_the_documented_half_and_nothing_more() {
        let cmd = plan_it().unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/dscacheutil"));
        assert_eq!(args_of(&cmd), vec!["-flushcache"]);
        assert!(cmd.artifact.is_none(), "flushing a cache produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none());
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
    fn the_second_half_of_the_recipe_is_absent_from_the_command() {
        // الوصفة الشائعة تُتبع هذا الأمر بـ`killall -HUP mDNSResponder`. الغياب
        // هنا قرارٌ لا سهو، وهذا الاختبار يمنع أن يتسلّل يومًا: لا اسم عملية،
        // ولا إشارة، ولا `sudo`، ولا فاصلة منقوطة.
        let cmd = plan_it().unwrap();
        let whole = args_of(&cmd).join(" ");
        for forbidden in ["killall", "mDNSResponder", "-HUP", "sudo", ";"] {
            assert!(!whole.contains(forbidden), "{forbidden:?} must never appear in this argv");
            assert!(!cmd.program.display().to_string().contains(forbidden));
        }
        assert_eq!(cmd.args.len(), 1, "one flag: the whole command is the first half");
    }

    #[test]
    fn the_partial_flush_is_announced_on_every_single_run() {
        // التحذير هو نصف قيمة هذه العملية. غيابه يجعلها تعِد بما لا تفي به.
        for _ in 0..3 {
            assert_eq!(plan_it().unwrap().warnings, vec!["warn.dns.partial_flush"]);
        }
    }

    #[test]
    fn every_flag_carries_an_explanation() {
        let cmd = plan_it().unwrap();
        for token in cmd.explain.iter() {
            assert!(token.key.is_some(), "{:?} carries no explanation", token.token);
        }
        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert_eq!(cmd.explain[1].role, TokenRole::Flag);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let cmd = plan_it().unwrap();
        for a in &cmd.args {
            if DECLARED_FLAGS.contains(&a.to_string_lossy().as_ref()) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        for smuggled in ["host", "domain", "-q", "extra"] {
            let value = RawValue::Text("example.com".to_owned());
            let raw = BTreeMap::from([(smuggled.to_owned(), value)]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_cannot_reach_the_argv_because_there_is_nowhere_to_put_it() {
        // الحالة الأخصّ بهذه العملية: نصف الوصفة الثاني نفسه. حتى لو أرسلته
        // الواجهة نصًّا، لا حقل يقبله ولا مفسّر ينفّذه.
        let baseline = plan_it().unwrap();
        for shellish in
            ["; sudo killall -HUP mDNSResponder", "$(sudo -v)", "`id`", "&& reboot", "| sh"]
        {
            let raw = BTreeMap::from([("then".to_owned(), RawValue::Text(shellish.to_owned()))]);
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
