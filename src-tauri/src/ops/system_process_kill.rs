//! إنهاء عمليةٍ واحدة برقمها، باستخدام `kill -TERM`.
//!
//! ## أخطر عملية في هذا الفهرس، ولماذا هي مع ذلك أضيق ممّا طُلب
//!
//! القائمة الأصلية طلبت `pkill` — إنهاءً **بالاسم**. وهذه العملية تُنهي
//! **برقمٍ واحد**، وهو تضييقٌ مقصود بعد نقاشٍ صريح مع صاحب المنتج، لا سهوًا
//! ولا نسيانًا لما طُلب.
//!
//! السبب أن `pkill firefox` أمرٌ لا يعرف أحدٌ — لا المستخدم ولا الشاشة — كم
//! عمليةً سيُنهي قبل أن يضغط: واحدة أم ثلاثين. والعدد لا يظهر في «سَطْر» لأنه
//! ليس في الأمر أصلًا، بل في نتيجة مطابقةٍ تقع داخل الأداة بعد الضغط. وهذا هو
//! بالضبط الشكل الذي يرفضه هذا المنتج في كل موضعٍ آخر: `git_branches.rs` تسرد
//! الفروع المدموجة ولا تحذفها، و`disk_list.rs` ترفض حقل «القرص» كي لا تلتصق
//! قيمةٌ من الشاشة بأداةٍ أفعالُها الأخرى تمحو.
//!
//! فالرقم الواحد هنا ليس تقييدًا للراحة، بل هو ما يجعل **ما يُعرض هو ما
//! يحدث**: سطرٌ واحد فيه رقمٌ واحد، وأثرٌ واحد على عمليةٍ واحدة.
//!
//! والثمن معلَن: من أراد إنهاء عدّة نسخ يكرّر العملية برقمٍ رقم. ومن لا يعرف
//! الرقم يجده في «البحث عن عملية» أو «العمليات الأعلى استهلاكًا» — وهاتان هما
//! الطريق المقصود إلى هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /bin/kill -TERM <رقم العملية>
//! ```
//!
//! ## `-TERM` مكتوبةً، ولماذا ليست `-KILL`
//!
//! `TERM` هي إشارة `kill` الافتراضية أصلًا، فكتابتها لا تغيّر الأمر. تُكتب
//! لأنها **تظهر**: من يقرأ السطر يرى أي إشارةٍ سُترسل، بدل أن يستنتج افتراضًا
//! لا يراه.
//!
//! والفرق بينها وبين `-KILL` (‏`-9`) ليس في الشدّة بل في الأمانة: `TERM` طلبٌ
//! يصل إلى البرنامج فيغلق ملفاته ويحفظ ما عنده ثم يخرج. و`KILL` لا تصل إليه
//! أصلًا — النواة تنزعه من الوجود بلا إخطار، فيبقى ما لم يُحفظ ضائعًا وما لم
//! يُغلق مفتوحًا. **ولا سبيل إلى `KILL` من هذا المنتج**: الإشارة سلسلةٌ ثابتة
//! في هذا الملفّ تدخل الثنائيّة عند الترجمة، ولا مدخل في هذه العملية يمكن أن
//! يصير رايةً.
//!
//! ## المدى المغلق حارسٌ بنيويّ لا فحصٌ يُنسى
//!
//! `pid` عددٌ في `2..=99998`، وهذا المدى هو أهمّ سطرٍ في الملفّ. ما يمنعه:
//!
//! * **الصفر.** `kill 0` لا تعني «لا أحد»: تعني — في `kill(2)` نفسها —
//!   إرسال الإشارة إلى **كل عمليات مجموعة المُرسِل**. أي أن التطبيق كان
//!   سيُنهي نفسه وكل ما وُلد معه.
//! * **السالب.** `kill -1` تعني كل عمليةٍ يملكها المستخدم على الجهاز. جلسةٌ
//!   كاملة تُقفل بضغطةٍ واحدة.
//! * **الواحد.** `launchd` — أصل كل عمليةٍ على macOS.
//!
//! والحارس ليس شرطًا مكتوبًا في `plan` يمكن أن يُنسى عند تعديلٍ لاحق: هو نوع
//! المدخل نفسه (`InputKind::Number`)، تفحصه النواة في `value.rs` قبل أن تصل
//! العملية إلى هذا الملفّ. القيم الثلاث القاتلة **لا يمكن التعبير عنها** من
//! الواجهة، لا أنها تُرفض بعد التعبير عنها.
//!
//! و`99998` هو `PID_MAX` على macOS — لا رقمٌ سخيّ اختير بالذوق. وقيس على هذا
//! الجهاز أن الأرقام المستعملة فعلًا تبلغ `99620`، فالمدى ليس نظريًا.
//!
//! ## القيمة المبدئية `2`، ولماذا هي أحرص ما يمكن
//!
//! النموذج يُعرض والحقل الرقمي مملوءٌ بقيمته المبدئية (‏`emptyValues` في
//! `operations.ts`)، فلا مفرّ من أن يحمل حقلُ رقمِ عمليةٍ رقمًا قبل أن يكتب
//! المستخدم. والمبدئي هنا هو **أدنى المدى**، لا رقمًا في وسطه:
//!
//! قيس على macOS أن الأرقام `2..=5` لا تقابلها عمليات — بعد `launchd` تبدأ
//! الأرقام المستعملة أعلى بكثير، خلافًا لِـLinux التي تعرض خيوط النواة
//! أرقامًا صغيرة. فـ`kill -TERM 2` تخرج بـ«‏No such process» بلا أثر. وهذا
//! **قياسٌ لا ضمان**: الأرقام تُعاد بعد الالتفاف، وجهازٌ آخر قد يخالف. لكنه
//! أحرص ما تستطيعه قيمةٌ مبدئية مضطرّة، ويبقى الحارس الحقيقي أن الرقم مكتوبٌ
//! في السطر المعروض قبل الضغط.
//!
//! ## أمانة: إرسال الإشارة ليس موت العملية
//!
//! `kill` تخرج بصفرٍ حين **تُرسَل** الإشارة، لا حين تنتهي العملية. وبينهما
//! فرقٌ حقيقي: برنامجٌ يلتقط `SIGTERM` ويتجاهلها يبقى حيًّا ورمزُ الخروج صفر،
//! وبرنامجٌ يحفظ عمله قبل أن يخرج يستغرق لحظات. فنصّ الإقرار في هذا المنتج
//! يقول «أُرسلت إشارة الإنهاء» لا «أُنهيت العملية» — والتحذير
//! `warn.kill.signal_not_guarantee` يقوله قبل الضغط لا بعده.
//!
//! ## ما لا تفعله
//!
//! لا ترفع الصلاحيات ولا تعرض ذلك. عمليةٌ يملكها مستخدمٌ آخر أو النظام تُرفض
//! من النواة نفسها بـ«‏Operation not permitted» (قيس على `launchd`)، ويصل ذلك
//! رمزَ خروجٍ غير صفري — أي فشلًا صادقًا لا صمتًا. ولا تلاحق العملية إن لم
//! تمت: لا مهلة ثم `-KILL`، فتلك سلسلةُ أمرين ونموذجُ فشلٍ جزئيّ لا تصوغه
//! هذه النواة اليوم.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.process.kill";

/// الإشارة، سلسلةً ثابتة. ليست خيارًا في الشاشة: `-KILL` تُفقد ما لم يُحفظ
/// ولا تُعطي البرنامج فرصة إغلاق، وحقلٌ يعرض الاثنين كان سيجعل الشاشة تسأل
/// سؤالًا جوابه معروف. انظر رأس الملفّ.
const SIGNAL: &str = "-TERM";

/// أدنى رقمٍ مقبول. `1` هو `launchd`، و`0` والسالب لهما معنًى جماعيّ كارثيّ
/// في `kill(2)`. انظر رأس الملفّ — هذا الثابت هو الحارس البنيويّ.
const MIN_PID: i64 = 2;

/// `PID_MAX` على macOS.
const MAX_PID: i64 = 99_998;

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.process.kill.title",
    description_key: "op.system.process.kill.description",
    category: Category::System,
    // العملية الثانية وحدها في الفهرس بهذه الدرجة، وأشدّ من الأولى
    // (`dev.cargo.clean`): تلك تحذف ملفاتٍ تُولَّد من جديد، وهذه تُنهي عملًا
    // جاريًا قد يحمل ما لم يُحفظ.
    danger: Danger::Destructive,
    visibility: Visibility::Production,
    tool: tools::KILL,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new(
        "pid",
        InputKind::Number { min: MIN_PID, max: MAX_PID, default: MIN_PID },
    )],
    sort_order: 70,
    search_terms: &[
        "kill",
        "إنهاء",
        "أنهِ",
        "عملية",
        "process",
        "terminate",
        "quit",
        "إيقاف",
        "معلّق",
        "pid",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    // المدى فُحص في النواة قبل الوصول إلى هنا: `0` والسالب و`1` لا تصل.
    let pid = inputs.number("pid")?;

    Argv::tool(tools::KILL, "explain.kill.tool")
        .flag(SIGNAL, "explain.kill.term")
        // الرقم يستحقّ شرحه الخاص: `-TERM` تقول أي إشارة، ولا تقول أن ما
        // بعدها رقم عمليةٍ سيُصاب.
        .explained_value(pid.to_string(), "explain.kill.pid")
        // ثابتان في كل خطة لا مشروطان: كلاهما صفةٌ في كل تشغيلٍ لهذه العملية
        // لا حالةٌ تطرأ أحيانًا — نظير `warn.ps.snapshot` في `system_processes.rs`.
        .warn("warn.kill.signal_not_guarantee")
        .warn("warn.kill.no_undo")
        // لا ناتج على القرص، فلا ترقية ولا اسمٌ يتضارب.
        .read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn plan_with(pid: i64) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([("pid".to_owned(), RawValue::Text(pid.to_string()))]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    fn args_of(cmd: &PlannedCommand) -> Vec<String> {
        cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("system.process.kill must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Destructive, "this is not a safe operation");
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "kill");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_more() {
        let cmd = plan_with(4242).unwrap();

        assert_eq!(cmd.program, Path::new("/bin/kill"));
        assert_eq!(args_of(&cmd), vec!["-TERM", "4242"]);
        assert!(cmd.artifact.is_none(), "ending a process produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "a pid does not resolve against a directory");
        assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
    }

    /// الحارس البنيويّ: القيم الثلاث القاتلة لا تصل إلى `plan` أصلًا.
    ///
    /// `0` تعني مجموعة المُرسِل كاملةً، والسالب يعني كل ما يملكه المستخدم،
    /// و`1` هو `launchd`. الرفض يقع في `value.rs` لأن المدى جزءٌ من نوع
    /// المدخل — لا شرطًا في هذا الملفّ يمكن أن يسقط في تعديلٍ لاحق.
    #[test]
    fn the_catastrophic_pids_cannot_be_expressed_at_all() {
        for deadly in [0, -1, -4242, 1] {
            assert_eq!(
                refusal(plan_with(deadly)),
                ("err.input.range", Some("pid")),
                "{deadly} must be refused by the declared range"
            );
        }
    }

    #[test]
    fn a_pid_beyond_the_platform_maximum_is_refused() {
        for out in [MAX_PID + 1, 1_000_000] {
            assert_eq!(refusal(plan_with(out)), ("err.input.range", Some("pid")), "{out}");
        }
    }

    #[test]
    fn the_bounds_themselves_are_accepted() {
        // حدّا المدى داخل المقبول لا خارجه: مدًى يرفض طرفيه ليس هو المعلَن.
        for edge in [MIN_PID, MAX_PID] {
            let cmd = plan_with(edge).unwrap();
            assert_eq!(args_of(&cmd)[1], edge.to_string());
        }
    }

    /// المبدئي هو أدنى المدى لا رقمًا في وسطه. انظر رأس الملفّ: النموذج يملأ
    /// الحقل الرقمي بقيمته المبدئية، فأحرصُ ما يُملأ به أقلُّ ما يقبله المدى.
    #[test]
    fn the_declared_default_is_the_floor_of_the_range() {
        let InputKind::Number { min, max, default } = SPEC.inputs[0].kind else {
            panic!("the pid must stay a bounded number, not free text")
        };
        assert_eq!(min, MIN_PID);
        assert_eq!(max, MAX_PID);
        assert_eq!(default, MIN_PID, "the prefilled value must be the most conservative one");
    }

    /// `-KILL` لا سبيل إليها: الإشارة ثابتةٌ في الشيفرة، ولا مدخل نصّي في هذه
    /// العملية يمكن أن ينزلق إلى موضع الراية.
    #[test]
    fn no_signal_other_than_term_can_ever_enter_this_command() {
        for pid in [2, 999, 99_998] {
            let args = args_of(&plan_with(pid).unwrap());
            assert_eq!(args.len(), 2, "no input may add an argument");
            assert_eq!(args[0], "-TERM");
            assert!(!args.iter().any(|a| a == "-KILL" || a == "-9"));
        }
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let cmd = plan_with(4242).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_carries_its_own_explanation() {
        // ثلاثة رموز: الأداة، والإشارة، والرقم. لو خلا أحدها من مفتاح شرحٍ
        // لكانت أخطر شاشةٍ في المنتج أنقصها بيانًا.
        let cmd = plan_with(4242).unwrap();
        assert_eq!(cmd.explain.len(), 3);
        for token in &cmd.explain {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
        assert_eq!(cmd.explain[1].token, "-TERM");
        assert_eq!(cmd.explain[1].key, Some("explain.kill.term"));
    }

    #[test]
    fn the_pid_can_never_be_read_as_a_flag() {
        // عددٌ موجب لا يبدأ بشرطة بحال، والمدى يمنع السالب أصلًا — فحصان
        // متعاضدان لا واحد.
        for pid in [MIN_PID, 4242, MAX_PID] {
            let cmd = plan_with(pid).unwrap();
            assert!(cannot_be_read_as_a_flag(&cmd.args[1]), "{:?}", cmd.args[1]);
        }
    }

    #[test]
    fn both_standing_warnings_are_raised_on_every_single_plan() {
        // لا مشروطان: «الإشارة ليست ضمانًا» و«لا تراجع» صفتان في كل تشغيل.
        for pid in [MIN_PID, 4242, MAX_PID] {
            let warnings = plan_with(pid).unwrap().warnings;
            assert!(warnings.contains(&"warn.kill.signal_not_guarantee"), "{warnings:?}");
            assert!(warnings.contains(&"warn.kill.no_undo"), "{warnings:?}");
        }
    }

    #[test]
    fn a_pid_sent_as_prose_instead_of_a_number_is_refused_not_coerced() {
        for smuggled in ["0; rm -rf ~", "$(whoami)", "all", "", "9 9"] {
            let raw = BTreeMap::from([("pid".to_owned(), RawValue::Text(smuggled.to_owned()))]);
            let r = crate::value::validate(&SPEC, &raw).map(|_| ());
            assert!(
                matches!(r, Err(CoreError::WrongInputType { id: "pid" })),
                "{smuggled:?} got {r:?}"
            );
        }
    }

    /// `-9` نصًّا **يُحلَّل عددًا صحيحًا** (‏`-9`) فلا يسقط في فحص النوع —
    /// ويسقط في المدى. الحارسان يتعاضدان: النوع يمنع ما ليس عددًا، والمدى
    /// يمنع العدد الذي يحمل معنًى جماعيًّا. لو غاب الثاني لَمرّت `-9` هنا
    /// **وسيطًا سالبًا** تقرؤه `kill` مجموعةَ عمليات.
    #[test]
    fn a_negative_signal_looking_pid_is_stopped_by_the_range_not_by_the_type() {
        let raw = BTreeMap::from([("pid".to_owned(), RawValue::Text("-9".to_owned()))]);
        let r = crate::value::validate(&SPEC, &raw).map(|_| ());
        assert!(
            matches!(r, Err(CoreError::NumberOutOfRange { id: "pid", .. })),
            "`-9` must be refused by the declared range, got {r:?}"
        );
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        for smuggled in ["signal", "name", "pattern", "-9"] {
            let raw = BTreeMap::from([
                ("pid".to_owned(), RawValue::Text("4242".to_owned())),
                (smuggled.to_owned(), RawValue::Text("x".to_owned())),
            ]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn planning_twice_gives_the_same_command_and_signals_nothing() {
        // التخطيط لا يشغّل شيئًا: هذا الاختبار يمرّ لأن `plan` تبني وصفًا،
        // ولو أرسلت إشارةً يومًا لَما كان تكرارها عشر مرّات بلا أثر.
        for _ in 0..10 {
            assert_eq!(plan_with(4242).unwrap(), plan_with(4242).unwrap());
        }
    }
}
