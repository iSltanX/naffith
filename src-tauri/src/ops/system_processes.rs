//! لقطةٌ عن العمليات الجارية مرتّبةً بحمل المعالج، باستخدام `ps`.
//!
//! ## لماذا `ps` لا `top`
//!
//! `top` برنامجٌ تفاعلي يرسم شاشةً كاملة بمحارف تحكّم ثم يعيد رسمها ولا ينتهي
//! من تلقاء نفسه. وهذا التطبيق يشغّل أمرًا واحدًا ويلتقط خرجه عبر أنبوب ثم
//! يعرضه كما هو: لا طرفية حقيقية، ولا شاشة تُعاد كتابتها. `top -l 1` كانت
//! تُنهي نفسها، لكنها تطبع قبل الجدول كتلةَ ملخّصٍ للنظام ثم تكتب جدولًا
//! مصمَّمًا لعرض طرفيةٍ لا لأنبوب. `ps` تطبع مرّةً وتخرج، وخرجها جدولٌ نصيّ
//! عادي — وهو بالضبط ما تعرضه هذه الشاشة.
//!
//! ## الصيغة
//!
//! ```text
//! /bin/ps -Ao pid,ppid,%cpu,%mem,comm -r
//! ```
//!
//! ## لماذا `-A`، ولماذا لا تصلح `ps` وحدها
//!
//! `ps` بلا رايات تعرض عمليات المستخدم الحالي **المرتبطة بطرفية التحكّم**.
//! وهذا التطبيق يشغّل الأمر بلا طرفية تحكّم أصلًا، فالجواب كان سيصير قائمةً
//! شبه فارغة — لا خطأً يُنبّه، بل جدولًا صحيح الشكل يقول شيئًا غير المقصود.
//! `-A` تلغي الشرطين معًا: كل عملية، لكل مستخدم، بطرفية أو بغيرها.
//!
//! ## ولماذا أعمدةٌ مُملاة لا الافتراضي
//!
//! `-o` تقول: هذه الأعمدة بعينها لا غيرها. بدونها يتبع الشكل ما تراه `ps`
//! مناسبًا — وفيه `TT` و`STAT` و`TIME` — وليس فيه `%cpu` ولا `%mem`، أي أن
//! السؤال الذي فُتحت الشاشة لأجله («ما الذي يستهلك الجهاز؟») لا جواب له في
//! الخرج الافتراضي.
//!
//! والأعمدة الخمسة قرارٌ لا عادة:
//!
//! * `pid` — رقم العملية، وهو ما يُنسخ ويُبحث به.
//! * `ppid` — رقم أبيها. نسختان من البرنامج نفسه تظهران بنفس الاسم تمامًا،
//!   ولا يفرّق بينهما إلا من أطلقهما.
//! * `%cpu` و`%mem` — الرقمان المقصودان.
//! * `comm` — مسار الملف التنفيذي وحده، لا `command` التي تطبع سطر الأوامر
//!   كاملًا بكل وسائطه. سطر أوامر واحد قد يبلغ مئات المحارف — وهو كذلك موضعٌ
//!   قد يحمل مسارات ملفات المستخدم، فيصير جدولُ عملياتٍ سجلًّا لما يفتحه.
//!
//! والترتيب مقصود: الأعمدة القصيرة أولًا والطويل آخرًا. `ps` تقصّ السطر عند
//! عرض الخرج حين لا تكتب إلى طرفية، فما يضيع من القصّ هو ذيل مسارٍ لا رقمٌ.
//!
//! ## ولماذا `-r`
//!
//! ترتيبٌ تنازلي بحمل المعالج. بدونها يأتي الجدول بترتيب `pid` — أي بترتيب
//! زمن الإطلاق تقريبًا — فيُدفن جواب السؤال في وسط مئتَي سطر.
//!
//! ## الحدّ الذي يستحق تحذيرًا في كل تشغيل
//!
//! `%cpu` في `ps` ليست «الآن». هي متوسّطٌ متلاشٍ على ما يبلغ الدقيقة السابقة
//! من الزمن الحقيقي، ولعمليةٍ وُلدت قبل ثانيتين يُحسب المتوسّط على ثانيتين —
//! ولهذا قد يتجاوز مجموع العمود ١٠٠٪ ولا يكون في ذلك عطب.
//!
//! أثر ذلك عمليّ: عمليةٌ بدأت تلتهم المعالج قبل لحظات تظهر منخفضة، وعمليةٌ
//! هدأت للتوّ تبقى في الأعلى حينًا. و«مراقب النشاط» يقيس فترةً بين لقطتين
//! فيعطي ترتيبًا آخر — والاختلاف بين الشاشتين ليس خطأً في إحداهما، بل قياسان
//! مختلفان. تحذير `warn.ps.snapshot` يقول ذلك في كل تشغيل، وحضوره الدائم
//! مقصود: هو صفةٌ ثابتة في كل نتيجة لا حالةٌ تطرأ.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تُنهي عمليةً ولا تغيّر أولويتها. `ps` لا تملك رايةً تفعل ذلك أصلًا —
//! الإنهاء `kill` وأداةٌ أخرى — والفهرس لا يحمل تلك الأداة. ولا مدخل واحد في
//! هذه العملية: الوسائط الثلاثة ثوابت في هذا الملف، فلا رقم عمليةٍ من الشاشة
//! يستطيع أن يصير وسيطًا.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.processes";

/// الأعمدة المُملاة، نصًّا واحدًا كما تنتظره `ps` بعد `-o`.
///
/// ثابتٌ مسمّى لا نصٌّ مبثوث في السطر: الاختبار يقارن الوسيط به، فلو غُيّر
/// العمود يومًا تغيّر الاثنان معًا ولم يتفرّقا.
const COLUMNS: &str = "pid,ppid,%cpu,%mem,comm";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.processes.title",
    description_key: "op.system.processes.description",
    category: Category::System,
    // قراءةٌ محضة لجدول العمليات: لا تكتب، ولا تُنهي، ولا تغيّر أولوية.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::PS,
    conflict: Conflict::NoArtifact,
    // بلا مدخلات عن قصد: السؤال عن الجهاز كلّه لا عن عمليةٍ بعينها.
    inputs: &[],
    sort_order: 10,
    search_terms: &[
        "ps",
        "عمليات",
        "processes",
        "عملية",
        "process",
        "معالج",
        "cpu",
        "ذاكرة",
        "memory",
        "top",
        "بطء",
        "slow",
        "مراقب النشاط",
        "activity monitor",
    ],
    plan,
};

fn plan(_inputs: &Inputs) -> Result<PlannedCommand> {
    // لا `.reveal(…)`: لم تُنتج هذه العملية موضعًا يُفتح، وزرُّ إظهارٍ يفتح
    // مجلدًا لم نكتب فيه شيئًا جوابٌ عن سؤالٍ لم يُطرح.
    Argv::tool(tools::PS, "explain.ps.tool")
        .flag("-Ao", "explain.ps.all_and_format")
        .explained_value(COLUMNS, "explain.ps.columns")
        .flag("-r", "explain.ps.sort_by_cpu")
        // ثابتٌ لا مشروط: طبيعة القياس صفةٌ في كل نتيجة.
        .warn("warn.ps.snapshot")
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
    const DECLARED_FLAGS: &[&str] = &["-Ao", "-r"];

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
        let found = listed.iter().find(|o| o.id == ID).expect("system.processes must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert!(found.inputs.is_empty(), "this screen has no form");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_more() {
        let cmd = plan_it().unwrap();

        assert_eq!(cmd.program, Path::new("/bin/ps"));
        assert_eq!(args_of(&cmd), vec!["-Ao", "pid,ppid,%cpu,%mem,comm", "-r"]);
        assert!(cmd.artifact.is_none(), "reading the process table produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "ps answers about the machine, not about `here`");
        assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
        assert!(cmd.estimate.is_none(), "no tree is measured");
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
    fn the_column_list_is_one_argument_not_five() {
        // `-o` تنتظر نصًّا واحدًا مفصولًا بفواصل. تقسيمُه على الفواصل كان
        // سيعطي `ps` أربعة معاملات لا تعنيها، وهو الشكل الذي لا تملك `Argv`
        // صيغةً تعبّر عنه أصلًا: كل وسيط يُضاف بندائه.
        let cmd = plan_it().unwrap();
        assert_eq!(cmd.args[1], std::ffi::OsString::from(COLUMNS));
        assert_eq!(cmd.args[1].to_string_lossy().matches(',').count(), 4);
        assert_eq!(cmd.args.len(), 3, "the column list must not become five arguments");
    }

    #[test]
    fn every_flag_carries_an_explanation_and_so_does_the_column_list() {
        // `-Ao` و`pid,ppid,%cpu,%mem,comm` لا يفهمهما أحد بالنظر. رمزٌ بلا
        // مفتاح شرحٍ هنا يعني سطرًا في «سَطْر» بلا معنى.
        let cmd = plan_it().unwrap();
        for token in cmd.explain.iter() {
            assert!(token.key.is_some(), "{:?} carries no explanation", token.token);
        }
        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert_eq!(cmd.explain[1].role, TokenRole::Flag);
        assert_eq!(cmd.explain[3].role, TokenRole::Flag);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        // الاختبار يثبّت أمرين: أن كل ما يبدأ بشرطة رايةٌ **معلَنة في الشيفرة**،
        // وأن ما عداها — قائمة الأعمدة — لا يمكن أن يُقرأ راية.
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
        // لا حقل في الشاشة يرسل هذا. لكن الحدّ لا يثق بالشاشة: مفتاحٌ لا تعرفه
        // المواصفة يعني إمّا واجهةً قديمة وإمّا محاولة تهريب، وكلاهما رفض.
        for smuggled in ["pid", "user", "-o", "format"] {
            let raw = BTreeMap::from([(smuggled.to_owned(), RawValue::Text("1".to_owned()))]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_cannot_reach_the_argv_because_there_is_nowhere_to_put_it() {
        // نظير اختبار «المحارف الحرفية» في العمليات ذات المدخلات: هناك يمرّ
        // النصّ وسيطًا واحدًا، وهنا لا يمرّ أصلًا — والأمر يبقى كما هو مهما
        // حاولت الواجهة.
        let baseline = plan_it().unwrap();
        for shellish in ["; kill -9 1", "$(whoami)", "`id`", "a & b", "| grep x", "&& reboot"] {
            let raw = BTreeMap::from([("filter".to_owned(), RawValue::Text(shellish.to_owned()))]);
            assert!(crate::value::validate(&SPEC, &raw).is_err(), "{shellish:?} was accepted");
        }
        assert_eq!(plan_it().unwrap().args, baseline.args);
        assert_eq!(baseline.args.len(), 3, "ps takes exactly three declared arguments");
    }

    #[test]
    fn the_measurement_caveat_is_announced_on_every_single_run() {
        for _ in 0..3 {
            assert_eq!(plan_it().unwrap().warnings, vec!["warn.ps.snapshot"]);
        }
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        // لا حالة ولا عشوائية ولا اسم مؤقّت: عمليةُ قراءةٍ بلا مدخلات يجب أن
        // تكون دالّةً ثابتة، وإلا كان في المسار شيءٌ لم نقصده.
        assert_eq!(plan_it().unwrap(), plan_it().unwrap());
    }
}
