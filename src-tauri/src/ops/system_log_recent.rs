//! أخطاء النظام وأعطاله في نافذةٍ زمنية قصيرة، باستخدام `log show`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/log show --last <ن>m --style compact --predicate <المُرشِّح>
//! ```
//!
//! ## لماذا خمس عشرة دقيقة سقفٌ مقيسٌ لا مذاقٌ مختار
//!
//! السجلّ الموحَّد في macOS ليس ملفًّا يُقرأ سطرًا سطرًا: هو مجرًى تكتب فيه
//! مئاتُ العمليات في كل ثانية. والرقم الذي يحكم هذه العملية ليس تقديرًا، بل
//! قياسٌ على جهازٍ حقيقي قوبل بسقفٍ معلَن في هذه الشيفرة نفسها:
//!
//! * بلا مُرشِّح، `--last 2m` وحدها أنتجت **٢٥٬٧٩٦ سطرًا**.
//! * وسقف البثّ في `executor.rs` هو `MAX_OUTPUT_LINES = 5_000` سطرًا.
//!
//! ## والحجم لا يُعتمد عليه: قياسان متباعدان على الجهاز نفسه
//!
//! مع مُرشِّح الأخطاء والأعطال، وعلى هذا الجهاز، في وقتين مختلفين من اليوم:
//!
//! | النافذة | قياسٌ أول | قياسٌ ثانٍ |
//! |---|---|---|
//! | `5m` | ٨١٣ سطرًا | ١٬٢٨٢ |
//! | `15m` | ٣٬٤٦٥ | **١٧٬٤٤٢** |
//!
//! خمسة أضعافٍ بين قياسين لنافذةٍ واحدة. السبب أن «الخطأ» في سجلّ macOS حدثٌ
//! نوبيّ لا معدّلٌ ثابت: دقيقةٌ واحدة قد تحمل سبعةً وسبعين سطرًا، وربعُ ساعةٍ
//! فيه نوبةٌ واحدة يحمل سبعة عشر ألفًا. فأيّ حدٍّ زمنيّ يُشتقّ من قياسٍ **واحد**
//! يكون معايرةً على لحظةٍ لا على سلوك.
//!
//! ولهذا الحدّ **خمس** لا خمس عشرة: أقلّ ما قيس عندها ٨١٣ وأكثرُه ١٬٢٨٢، وكلاهما
//! دون السقف بهامشٍ واسع — بينما خمس عشرة تجاوزته بثلاثة أضعافٍ ونصف في القياس
//! الثاني. ونسخةٌ سابقة من هذا الملفّ حملت الحدّ خمس عشرة مستندةً إلى القياس
//! الأول وحده، ووصفته بأنه «هامشٌ يحتمل جهازًا أثقل» — وهو ما لم يصمد.
//!
//! ## والقصّ مُعلَن لا مُتجنَّب
//!
//! خمس دقائق لا تضمن اكتمال الجواب في كل حال: نوبةٌ كثيفة قد تتجاوز السقف
//! فيُقصّ الخرج. والقصّ في هذا المنتج **يُقال** — يُرسل سطر `Truncated` بعدد
//! ما سقط، وتعرضه الشاشة — فالوعد هنا ليس «لن يُقصّ» بل «إن قُصّ عرفتَ».
//! وتحذير `warn.log.bounded_window` يقول ذلك قبل التشغيل.
//!
//! ومن أراد نافذةً أطول فأمرُه معروضٌ أمامه ينسخه ويغيّر الرقم في صدفته.
//!
//! ## ولماذا المُرشِّح ثابتٌ في الشيفرة لا حقلٌ في الشاشة
//!
//! القرار من شقّين، وكلٌّ منهما وحده كافٍ.
//!
//! **الأول معماريّ.** `--predicate` لا تقبل كلمةً ولا كلمتين: تقبل لغة
//! `NSPredicate` كاملة — مقارناتٍ ومعاملاتٍ منطقية و`CONTAINS` و`BEGINSWITH`
//! و`MATCHES` بتعبيرٍ نمطيّ داخلها. أي أنها لغةٌ ثانية تسكن حقلًا واحدًا. وهذا
//! هو بعينه ما رُفض في رأس `text_search.rs` حين رُفض التعبير النمطي: «التعبير
//! النمطي لغةٌ ثانية داخل الحقل، وشرحُها في سَطْر شرحُ لغةٍ لا شرحُ أمر». ومنتجٌ
//! دعواه أنه يشرح ما ينفّذ لا يفتح حقلًا لا يستطيع شرح ما يُكتب فيه.
//!
//! **والثاني قياسيّ.** بلا ترشيحٍ لا توجد نافذةٌ زمنية مفيدة تحت السقف أصلًا:
//! دقيقتان تعنيان ٢٥٬٧٩٦ سطرًا، أي خمسة أضعاف السقف. فالترشيح هنا ليس تفضيلًا
//! في العرض ولا تضييقًا اختياريًا، بل شرطُ أن يكون لهذه العملية جوابٌ كامل غير
//! مقصوص. وحقلٌ يسمح بإزالته كان سيسمح بإلغاء الشرط الذي تقوم عليه.
//!
//! والثمن مُعلَن: من أراد أخطاء عمليةٍ بعينها، أو رسائل نظامٍ فرعيّ محدَّد، لا
//! يستطيع التعبير عن ذلك من هنا.
//!
//! ## ولماذا `--style compact`
//!
//! الشكل الافتراضي يطبع لكل قيدٍ حقولًا مسهبة على أكثر من سطر. و`compact`
//! تطبع القيد سطرًا واحدًا: وقتٌ، ونوعٌ، وعمليةٌ، ونصّ الرسالة. والفرق ليس
//! تجميلًا: عدد الأسطر هو ما يُقاس عليه السقف أعلاه، فالشكل الذي يُطبع به
//! القيد جزءٌ من الحساب لا زينةٌ فوقه.
//!
//! ## أخطر ما في هذه الأداة، ولماذا لا سبيل إليه
//!
//! `log` تحمل `log erase` التي تمحو أرشيف السجلّات كلّه، و`log config` التي
//! تغيّر إعداد التسجيل في النظام. ولا شيء هنا يمنع أيًّا منهما **بالفحص** — لا
//! قائمة أفعالٍ ممنوعة ولا مرشِّح نصوص؛ فحوصٌ من هذا النوع تُكتب ثم تُنسى ثغرةٌ
//! فيها.
//!
//! المنع بنيويّ: الفعل `show` سلسلةٌ ثابتة في هذا الملف تدخل الثنائيّة عند
//! الترجمة، والمُرشِّح والشكل مثله. والمدخل الوحيد الذي تعلنه هذه العملية
//! **رقمٌ** ضمن مدًى مغلق تتحقّق منه النواة قبل `plan`، ثم يصير الوسيط
//! `<ن>m` — فلا يوجد في المنتج كلّه موضعٌ يستطيع نصٌّ قادم من الواجهة أن يصير
//! فيه فعلًا لهذه الأداة. ليس لأن أحدًا يمنعه، بل لأن الطريق غير موجود.
//! واختبارٌ في هذا الملف يثبّت أن `argv` لا تحمل يومًا فعلًا غير `show`، نظير
//! اختبار `diskutil` في `disk_list.rs`.
//!
//! ## الخصوصية: وعدٌ نَنقله ولا نُصدره
//!
//! macOS تحجب في السجلّ ما تعدّه بياناتٍ حسّاسة فتطبع مكانها `<private>`،
//! ويُظهرها المطوّر بإعلانٍ صريح في شيفرته. لكن هذا **وعد النظام لا وعدُنا**:
//! ما يُعدّ حسّاسًا يقرّره من كتب رسالة السجلّ، لا هذا التطبيق. وقد يمرّ في
//! السطور اسمُ ملفٍ أو مضيفٍ أو حسابٍ لأن كاتب الرسالة لم يعدّه سرًّا. فالوصف
//! يقول «اقرأ قبل أن تلصق» ولا يضمن نيابةً عن غيره.
//!
//! ## وسيطٌ واحد فيه فراغات — مقيسٌ لا مفترض
//!
//! المُرشِّح جملةٌ فيها فراغات، ويمرّ رمزًا **واحدًا** كاملًا. وهذا مُثبَتٌ
//! بتشغيلٍ حقيقي عبر `execve` مباشرةً بلا صدفة: `log` تقبله وسيطًا واحدًا فيه
//! فراغات وتخرج بـ`0`. الفراغ هنا محرفٌ داخل بيانات وسيطٍ واحد لا فاصلٌ بين
//! وسيطين، لأن الأمر يُبنى مصفوفةً لا نصًّا يُقسَّم — وهذه هي القاعدة الأولى في
//! `executor.rs`، وهذا قياسٌ يثبّتها على هذه الأداة بعينها.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تكتب ملفًا ولا تغيّر إعدادًا ولا تمحو قيدًا. تقرأ وتطبع، والإلغاء في أي
//! لحظة بلا أثر. وحفظُ السجلّ في ملف عمليةٌ أخرى لا مكسبٌ جانبي هنا.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.log.recent";

/// الفعل الوحيد الذي تعرفه هذه العملية. سلسلةٌ في الثنائيّة لا قيمةٌ من حقل:
/// أختاه `erase` و`config` تمحوان وتُهيّئان، ولا طريق إليهما من هنا.
const VERB: &str = "show";

/// المُرشِّح. ثابتٌ في الشيفرة لا حقلٌ في الشاشة — انظر رأس الملف: `NSPredicate`
/// لغةٌ ثانية داخل حقل، وبلا ترشيحٍ لا نافذة زمنية تحت السقف أصلًا.
const PREDICATE: &str = "messageType == error OR messageType == fault";

/// شكل الطباعة: الشكل المضغوط بدل المسهب، فهو أقلّهما أسطرًا.
///
/// و**لا يعني «سطرًا واحدًا لكل قيد»** — ادّعتْه نسخةٌ سابقة من هذا التعليق
/// وأُبطل بالقياس: قيدٌ واحد امتدّ على خمسة وثلاثين سطرًا ماديًّا حين حمل
/// حمولةً متعدّدة الأسطر. وفي قياسٍ لربع ساعة كان ٥٬٣٤٩ قيدًا يقابلها ١٧٬٤١٠
/// أسطر مادّية — نحو ثلاثة أضعاف. والسقف في `executor.rs` يعدّ **الأسطر
/// المادّية** لا القيود، فهذا هو العدد الذي يُقاس عليه.
const STYLE: &str = "compact";

/// أطول نافذةٍ قيست دون السقف في **كل** قياساتها. انظر رأس الملفّ: القياس
/// الواحد لا يكفي هنا لأن الأخطاء نوبيّة — ‏`5m` أعطت ٨١٣ سطرًا مرّةً و١٬٢٨٢
/// أخرى، بينما `15m` أعطت ٣٬٤٦٥ مرّةً و١٧٬٤٤٢ أخرى.
const MAX_MINUTES: i64 = 5;

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.log.recent.title",
    description_key: "op.system.log.recent.description",
    category: Category::System,
    // قراءةٌ محضة: تقرأ من السجلّ الموحَّد وتطبع، ولا تكتب بايتًا واحدًا.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::LOG,
    conflict: Conflict::NoArtifact,
    // المدخل الوحيد رقمٌ في مدًى مغلق. لا حقل نصّ هنا، ولا يمكن أن يوجد: انظر
    // فقرتَي المُرشِّح والمنع البنيوي في رأس الملف.
    inputs: &[InputSpec::new(
        "minutes",
        InputKind::Number { min: 1, max: MAX_MINUTES, default: MAX_MINUTES },
    )],
    sort_order: 55,
    search_terms: &["log", "سجل", "أخطاء", "errors", "نظام", "تشخيص", "diagnose", "console"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    // رقمٌ تحقّقت النواة من مداه قبل أن يصل هنا، ثم يصير `<ن>m`. موجبٌ حتمًا
    // فلا يبدأ بشرطة، و`Argv::value` تثبّت ذلك بدل الاعتماد عليه.
    let minutes = inputs.number("minutes")?;

    // لا `.reveal(…)`: لم تُنتج هذه العملية موضعًا يُفتح، والجواب كلّه في
    // السطور المطبوعة.
    Argv::tool(tools::LOG, "explain.log.tool")
        .flag(VERB, "explain.log.show")
        .flag("--last", "explain.log.last")
        // النافذة قيمةٌ تشرحها رايتُها: مفتاحٌ ثانٍ هنا كان سيقول «هذه هي
        // المدّة التي اخترتها»، وهي جملةٌ تشغل سطرًا ولا تضيف معرفة.
        .value(format!("{minutes}m"))
        .flag("--style", "explain.log.style")
        .explained_value(STYLE, "explain.log.compact")
        .flag("--predicate", "explain.log.predicate")
        .explained_value(PREDICATE, "explain.log.errors_only")
        // ثابتٌ لا مشروط: قصرُ الجواب على الأخطاء والأعطال داخل نافذةٍ محدودة
        // صفةٌ في كل تشغيل، لا حالةٌ تطرأ عند رقمٍ دون رقم.
        .warn("warn.log.bounded_window")
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
    const DECLARED_FLAGS: &[&str] = &["--last", "--style", "--predicate"];

    /// كل نافذةٍ يقبلها المدى المعلَن. الاختبارات تمرّ عليها لا على ذاكرة من
    /// كتبها، فتوسيعُ المدى يومًا يُختبر كاملًا بلا تعديل اختبار.
    fn every_window() -> impl Iterator<Item = i64> {
        1..=MAX_MINUTES
    }

    /// أفعال `log` التي تكتب أو تمحو أو تغيّر إعدادًا. الاختبار يثبّت أن `argv`
    /// لا تحمل واحدًا منها — لا بمدخل، ولا بخطأ مطبعي في هذا الملف يومًا.
    const WRITING_VERBS: &[&str] =
        &["erase", "config", "collect", "stream", "note", "default", "info", "debug"];

    fn raw(minutes: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([("minutes".to_owned(), RawValue::Text(minutes.to_owned()))])
    }

    fn plan_with(minutes: i64) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(&minutes.to_string()))?)
    }

    fn plan_with_text(minutes: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(minutes))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("system.log.recent must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.inputs.len(), 1);
        assert!(found.inputs[0].required, "a window of nothing is not a window");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_more() {
        let cmd = plan_with(5).unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/log"));
        assert_eq!(
            args_of(&cmd),
            vec![
                "show",
                "--last",
                "5m",
                "--style",
                "compact",
                "--predicate",
                "messageType == error OR messageType == fault",
            ]
        );
        assert!(cmd.cwd.is_none(), "the answer does not depend on `here`");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        // `MAX_MINUTES` لا رقمًا حرفيًا: نسخةٌ سابقة كتبت `15` هنا، فلمّا ضاق
        // المدى صار الاختبار يخطّط قيمةً مرفوضة ويسقط لسببٍ غير الذي وُضع له.
        let cmd = plan_with(MAX_MINUTES).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn the_predicate_reaches_the_tool_as_one_argument_spaces_and_all() {
        // القياس التجريبي مثبَّتًا في اختبار: `log` تقبل الجملة وسيطًا واحدًا
        // وتخرج بـ`0`. لو انقسم الوسيط يومًا لصار الأمر غير الذي عُرض.
        let cmd = plan_with(5).unwrap();
        assert_eq!(cmd.args.last().unwrap(), std::ffi::OsStr::new(PREDICATE));
        assert!(PREDICATE.contains(' '), "the measurement is about an argument with spaces");
        assert_eq!(
            args_of(&cmd).iter().filter(|a| a.as_str() == PREDICATE).count(),
            1,
            "the predicate is one token, not four"
        );
    }

    #[test]
    fn the_window_is_the_only_thing_the_screen_decides() {
        for minutes in every_window() {
            let cmd = plan_with(minutes).unwrap();
            let args = args_of(&cmd);
            assert_eq!(args[2], format!("{minutes}m"), "the window is the chosen number");
            assert_eq!(args[0], "show", "the verb is fixed, not a field");
            assert_eq!(args[4], "compact", "the style is fixed, not a field");
            assert_eq!(args[6], PREDICATE, "the predicate is fixed, not a field");
            assert_eq!(args.len(), 7, "no window may add or drop an argument");
        }
    }

    #[test]
    fn the_argv_can_never_carry_a_verb_that_writes() {
        // الحارس الذي يجعل هذه العملية آمنةً بالبناء لا بالمراجعة: أيّ تعديلٍ
        // مستقبلي يضيف فعلًا يمحو أو يغيّر إعدادًا يسقط هنا، لا في يد مستخدم.
        for minutes in every_window() {
            let cmd = plan_with(minutes).unwrap();
            for a in &cmd.args {
                let text = a.to_string_lossy().into_owned();
                assert!(
                    !WRITING_VERBS.contains(&text.as_str()),
                    "{text:?} is a log verb that changes the machine"
                );
            }
            assert_eq!(cmd.args[0], std::ffi::OsString::from("show"));
        }
        assert_eq!(VERB, "show", "one reading verb, and no room for a second");
    }

    #[test]
    fn the_ceiling_the_window_was_measured_against_is_still_the_ceiling() {
        // الحدّ مشتقٌّ من هذا الثابت ومن قياساتٍ عليه. لو تغيّر السقف يومًا سقط
        // هذا الاختبار، فيُراجَع الحدّ بقياسٍ جديد لا بالتخمين.
        assert_eq!(crate::executor::MAX_OUTPUT_LINES, 5_000);
        assert_eq!(MAX_MINUTES, 5, "measured twice: 5m → 813 then 1,282 physical lines");
    }

    /// النافذة القصوى هي المبدئية أيضًا.
    ///
    /// ليس تبسيطًا: مدًى أقصاه خمس ومبدئيّه خمس يعني أن أسوأ حالةٍ يبلغها
    /// المستخدم هي التي قيست، فلا يوجد رقمٌ مقبولٌ لم يُقَس عليه. ونسخةٌ سابقة
    /// كان أقصاها خمس عشرة ومبدئيّها خمسًا — فكان الحدّ الأعلى المسموح ثلاثة
    /// أضعاف السقف بلا أن يمرّ أحدٌ عليه في اختبار.
    #[test]
    fn the_widest_window_the_form_allows_is_the_one_that_was_measured() {
        let InputKind::Number { min, max, default } = SPEC.inputs[0].kind else {
            panic!("the window must stay a bounded number")
        };
        assert_eq!(min, 1);
        assert_eq!(max, MAX_MINUTES);
        assert_eq!(default, MAX_MINUTES, "no reachable window may exceed the measured one");
    }

    #[test]
    fn a_window_outside_the_declared_range_is_refused_by_the_core() {
        for out in [0, -1, MAX_MINUTES + 1, 30, 1_440] {
            assert_eq!(
                refusal(plan_with(out)),
                ("err.input.range", Some("minutes")),
                "{out} must be refused"
            );
        }
    }

    #[test]
    fn a_window_that_is_not_a_number_is_refused_and_names_its_own_field() {
        for bogus in ["", "five", "5m", "٥", "5.5", "1e3"] {
            assert_eq!(
                refusal(plan_with_text(bogus)),
                ("err.input.type", Some("minutes")),
                "{bogus:?} must be refused"
            );
        }
    }

    #[test]
    fn the_window_is_required_and_its_absence_names_its_own_field() {
        let r = crate::value::validate(&SPEC, &BTreeMap::new()).map(|i| plan(&i).unwrap());
        assert_eq!(refusal(r), ("err.input.missing", Some("minutes")));
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        // هذا هو الاختبار الذي يحرس الأطروحة كلها على هذه الأداة: مفتاحٌ لا
        // تعرفه المواصفة يُرفض، فلا يصل مُرشِّحٌ ولا فعلٌ إلى `argv` من أي طريق.
        for smuggled in ["predicate", "style", "verb", "subcommand", "process", "args"] {
            let mut raw = raw("5");
            raw.insert(smuggled.to_owned(), RawValue::Text("erase --all".to_owned()));
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        for minutes in every_window() {
            let cmd = plan_with(minutes).unwrap();
            for a in &cmd.args {
                if DECLARED_FLAGS.contains(&a.to_string_lossy().as_ref()) {
                    continue;
                }
                assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
            }
        }
    }

    #[test]
    fn shell_syntax_in_the_window_never_becomes_an_argument() {
        // الحقل رقمٌ في مدًى مغلق، فالنصّ الشِبه-صدفيّ لا يُقتبس ولا يُهرَّب:
        // يُرفض قبل أن يبلغ باني الوسائط أصلًا، والأمر يبقى كما هو.
        let baseline = plan_with(5).unwrap();
        for shellish in ["5; log erase --all", "$(id)", "`whoami`", "5 && rm -rf ~", "5 | tee x"] {
            assert_eq!(
                refusal(plan_with_text(shellish)),
                ("err.input.type", Some("minutes")),
                "{shellish:?} was accepted"
            );
        }
        assert_eq!(plan_with(5).unwrap().args, baseline.args);
        assert_eq!(baseline.args.len(), 7, "seven arguments, whatever the screen sends");
    }

    #[test]
    fn every_token_but_the_number_carries_its_own_explanation() {
        // ولا رمزٌ هنا مفهومٌ بالنظر: اسم أداة، وفعلٌ، وثلاث رايات، وشكلٌ،
        // وجملة `NSPredicate`. الرقم وحده يشرحه ما قبله (`--last`).
        let cmd = plan_with(5).unwrap();
        for (i, token) in cmd.explain.iter().enumerate() {
            // 0 الأداة، 1 الفعل، 2 `--last`، 3 النافذة نفسها.
            if i == 3 {
                assert!(token.key.is_none(), "the number is explained by the flag before it");
                continue;
            }
            assert!(token.key.is_some(), "{:?} carries no explanation", token.token);
        }
        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert_eq!(cmd.explain[2].role, TokenRole::Flag);
    }

    #[test]
    fn the_bounded_window_is_announced_on_every_single_run() {
        for minutes in every_window() {
            assert_eq!(plan_with(minutes).unwrap().warnings, vec!["warn.log.bounded_window"]);
        }
    }

    #[test]
    fn planning_declares_nothing_that_would_touch_the_disk() {
        for minutes in every_window() {
            let cmd = plan_with(minutes).unwrap();
            assert!(cmd.artifact.is_none(), "the log is printed, not saved");
            assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
            assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
            assert!(cmd.estimate.is_none(), "no tree is scanned to plan this");
            assert!(cmd.extra_path.is_empty(), "the tool calls nothing else");
        }
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        assert_eq!(plan_with(5).unwrap(), plan_with(5).unwrap());
    }
}
