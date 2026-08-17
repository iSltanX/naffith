//! البحث عن عمليةٍ جارية باسمها، باستخدام `pgrep`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/pgrep -l <الاسم>
//! ```
//!
//! ## `-l` وحدها، ولا `-f` أبدًا — قرارُ خصوصيةٍ لا اختصارُ كتابة
//!
//! هذه العملية أختُ `system.processes`، والقرار الذي يحكمها اتُّخذ هناك: رأس
//! ذلك الملف يشرح لماذا اختير العمود `comm` — الملف التنفيذي وحده — بدل
//! `command` الذي يطبع سطر الأوامر كاملًا، لأن سطر الأوامر «موضعٌ قد يحمل
//! مسارات ملفات المستخدم، فيصير جدولُ عملياتٍ سجلًّا لما يفتحه».
//!
//! و`pgrep -f` تفعل بالضبط ما مُنع هناك: تطابق **سطر الأوامر الكامل** بكل
//! وسائطه، ثم تطبعه مع `-l`. فمن يبحث عن `Preview` بها يرى أسماء المستندات
//! المفتوحة، ومن يبحث عن `ssh` يرى المضيف والمستخدم. أي أن نصف سطرٍ في هذا
//! الملف كان سينقض قرارًا مكتوبًا في ملفٍ آخر — بلا أن يمرّ أحدٌ على ذلك
//! القرار ليراجعه.
//!
//! فهذا الملف **يحترم القرار ولا يعيد فتحه**: `-l` تطابق اسم العملية وحده
//! وتطبع «رقم اسم» لا أكثر، وهو نفس القدر الذي تعرضه أختُها في عمودها. ومن
//! أراد تغيير الحدّ فموضعه هناك أولًا، لا هنا.
//!
//! ## ولماذا `pgrep` أصلًا لا `ps` ثم ترشيح
//!
//! الجواب في `executor.rs`: لا صدفة ولا أنابيب ولا تسلسل أوامر. فـ`ps | grep`
//! ليست صيغةً «أقصر» هنا، بل صيغةٌ لا يملك هذا المنتج طريقًا للتعبير عنها
//! أصلًا. و`pgrep` تجيب عن السؤال ببرنامجٍ واحد ووسائط ثابتة، وهو الشكل
//! الوحيد الذي تعرفه `PlannedCommand`.
//!
//! و`pgrep` قارئةٌ بحتة: لا رايةَ فيها تُنهي شيئًا. أختُها `pkill` تحمل الاسم
//! نفسه تقريبًا وتقتل ما تجده، وهي **غير معلَنة في `tools.rs`** — فلا سبيل
//! إليها من المنتج كلّه، لا من هنا ولا من غيره.
//!
//! ## الاسم: لا فراغ، ولا محارف تحكّم، ولا تشذيب
//!
//! الاسم الفارغ يُرفض في الحقل لا يُمرَّر: `pgrep -l ""` — مقيسةً على macOS —
//! تطبع `Cannot compile regular expression … (empty (sub)expression)` وتخرج
//! بـ`2`. أي أن الثمن ليس نتيجةً خاطئة، بل رسالةً تتحدّث عن «تعبيرٍ نمطي» إلى
//! مستخدمٍ لم يطلب أن يكتب واحدًا. والرفض هنا يحمل اسم الحقل، فيُقرأ في مكانه.
//!
//! ومحارف التحكّم تُرفض للسبب الذي يرفضها به `text_search`: السطر الجديد يجعل
//! الأمر يُعرض على سطرين في «سَطْر» وفي السجلّ، فيُقرأ غير ما يُنفَّذ.
//!
//! ولا يُشذَّب الاسم. المسافة في طرفه جزءٌ ممّا يطابقه المستخدم، وقصُّها يجعل
//! الأمر يبحث عن غير ما كُتب في الحقل — وهو نفس قرار `text_search` في نمطه.
//!
//! ## ولا `--` في هذا الأمر
//!
//! `common.rs` يقول إن فاصل نهاية الرايات يُضاف «حيث يعمل لا حيث يبدو
//! مرتّبًا». وهنا معاملٌ واحدٌ لا غير، وهو محروسٌ من أن يبدأ بشرطة مرّتين:
//! `validate_name` أدناه ترفضه برسالةٍ منسوبة إلى حقله، و`Argv::explained_value`
//! ترفضه ثانيةً بلا استثناء. فالفاصل كان سيضيف رمزًا ثالثًا يقرؤه المستخدم في
//! كل تشغيل حمايةً من حالةٍ لا يمكن التعبير عنها من الشاشة أصلًا.
//!
//! ## حدٌّ معروف: المعامل تعبيرٌ نمطي، ولا `-F` في `pgrep`
//!
//! `grep` تقبل `-F` فيصير نمطها نصًّا حرفيًا، ولذلك يفرضها `text_search`.
//! و`pgrep` لا تملك نظيرًا لها: معاملها تعبيرٌ نمطي ممتد دائمًا — مقيسًا،
//! `pgrep -l 'Find.r'` تطابق `Finder` — واسمٌ فيه قوسٌ غير متوازن يخرج بـ`2`
//! برسالة صياغة. وهذا يُقال ولا يُخفى.
//!
//! والخطر الذي دفع `text_search` إلى `-F` لا يقع هنا بنفس الحجم: المطابقة
//! تجري على أسماء عملياتٍ قصيرة معدودة، لا على شجرة ملفاتٍ بالميغابايتات،
//! فزمن أسوأ نمطٍ محكومٌ بطول المُدخَل نفسه وهو محدودٌ سلفًا.
//!
//! ## رمز الخروج دلالةُ بحثٍ لا دلالةُ فشل
//!
//! مقيسًا على macOS: `0` حين تجد، و`1` حين لا تجد — وهو جوابٌ مكتمل لا عطب،
//! نظير `grep` تمامًا. (و`2` لخطأ صياغةٍ أو استعمال، وهو ما يمنعه الحقل أعلاه
//! في حالته الغالبة.) ربطُ ذلك بعقد النتائج يقع مركزيًا في `result.rs`، لا في
//! هذا الملف: العملية تبني أمرًا، ولا تفسّر ما يعود منه.
//!
//! ## لا تكتب شيئًا ولا تُظهر موضعًا
//!
//! قراءةٌ محضة: لا ناتج ولا مؤقّت ولا مسار. ولا `reveal` كذلك — لا مسار على
//! القرص في هذه العملية أصلًا، وزرُّ إظهارٍ يفتح مجلدًا لم يُذكر في الأمر جوابٌ
//! عن سؤالٍ لم يُطرح.

use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.process.find";

/// أطول اسمٍ يقبله الحقل.
///
/// مئة محرف: أطول بكثير من أي اسم عمليةٍ حقيقي على هذا النظام، وأقصر من أن
/// يصير الحقلُ موضعًا لحمولةٍ تُعرض في سطرٍ ويُقيَّد في السجلّ. ثابتٌ مسمّى لا
/// رقمٌ مبثوث: الاختبار يقيس عليه، فلو تغيّر يومًا تغيّر الاثنان معًا.
const MAX_NAME_LEN: usize = 100;

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.process.find.title",
    description_key: "op.system.process.find.description",
    category: Category::System,
    // قراءةٌ محضة: لا تكتب، ولا تُنهي عمليةً، ولا تغيّر أولوية.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::PGREP,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("name", InputKind::Text { max_len: MAX_NAME_LEN })],
    sort_order: 15,
    search_terms: &["pgrep", "process", "find", "عملية", "بحث", "اسم", "عمليات", "تشغيل"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let name = inputs.text("name")?;

    validate_name(name)?;

    // لا `.reveal(…)`: لا مسار في هذا الأمر ولا موضع يُفتح بعده.
    Argv::tool(tools::PGREP, "explain.pgrep.tool")
        .flag("-l", "explain.pgrep.list_name")
        .explained_value(name, "explain.pgrep.name")
        .read_only()
}

/// يفحص الاسم قبل أن يصير وسيطًا. ثلاثة شروط، ولكلٍّ منها ما يمنعه بعينه.
fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(CoreError::InvalidName { reason: NameRejection::Empty }.on_input("name"));
    }
    if name.chars().any(|c| c.is_control()) {
        return Err(
            CoreError::InvalidName { reason: NameRejection::ContainsControl }.on_input("name")
        );
    }
    // ترفضه `Argv::explained_value` كذلك، والرفض هنا كي يُنسب إلى حقله ويُقرأ
    // في مكانه بدل أن يخرج من البانِي بلا حقلٍ يدلّ عليه.
    if name.starts_with('-') {
        return Err(CoreError::ArgumentLooksLikeFlag.on_input("name"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    /// الرايات المعلَنة في هذا الأمر. ما عداها في `argv` بيانات.
    const DECLARED_FLAGS: &[&str] = &["-l"];

    fn raw(name: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([("name".to_owned(), RawValue::Text(name.to_owned()))])
    }

    fn plan_with(name: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(name))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("system.process.find must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact, "a reader has no final name to protect");
        assert_eq!(found.inputs.len(), 1, "one field: the name searched for");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_more() {
        let cmd = plan_with("Finder").unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/pgrep"));
        assert_eq!(args_of(&cmd), vec!["-l", "Finder"]);
        assert!(cmd.artifact.is_none(), "searching for a process produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "pgrep answers about the machine, not about `here`");
        assert!(cmd.reveal_target.is_none(), "there is no path in this command to reveal");
        assert!(cmd.estimate.is_none(), "no tree is measured");
    }

    #[test]
    fn the_command_line_of_other_processes_is_never_matched_nor_printed() {
        // هذا هو القرار الذي وُرث من `system.processes`: `-f` تطابق سطر الأوامر
        // الكامل وتطبعه، فتكشف ما يفتحه المستخدم. سقوطُ هذا الاختبار يعني أن
        // قرارًا خصوصيًا نُقض من ملفٍّ لا يملكه.
        for name in ["Preview", "ssh", "Safari"] {
            let cmd = plan_with(name).unwrap();
            let args = args_of(&cmd);
            assert!(!args.iter().any(|a| a == "-f"), "{name:?}: {args:?}");
            assert_eq!(args, vec!["-l", name], "{name:?} must add nothing to the command");
        }
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let cmd = plan_with("Finder").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_carries_an_explanation_and_declares_its_role() {
        // `-l` لا يفهمها أحد بالنظر، والاسم يستحقّ كلمةً تقول إنه مطابَقٌ على
        // اسم العملية وحده. رمزٌ بلا مفتاح شرحٍ هنا يعني سطرًا في «سَطْر» بلا معنى.
        let cmd = plan_with("Finder").unwrap();
        for token in cmd.explain.iter() {
            assert!(token.key.is_some(), "{:?} carries no explanation", token.token);
        }
        let roles: Vec<TokenRole> = cmd.explain.iter().map(|t| t.role).collect();
        assert_eq!(roles, vec![TokenRole::Tool, TokenRole::Flag, TokenRole::Value]);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        // اسمٌ فيه شرطات في وسطه شائعٌ في أسماء المساعِدات على macOS، والاختبار
        // يثبّت أن كل ما يبدأ بشرطة رايةٌ **معلَنة في الشيفرة** لا قيمةُ مستخدم.
        let cmd = plan_with("com.apple.Safari-Helper").unwrap();
        for arg in &cmd.args {
            if DECLARED_FLAGS.iter().any(|flag| arg.as_os_str() == std::ffi::OsStr::new(flag)) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag");
        }
    }

    #[test]
    fn an_empty_name_is_refused_because_the_tool_would_answer_about_regular_expressions() {
        assert_eq!(refusal(plan_with("")), ("err.name.invalid", Some("name")));
    }

    #[test]
    fn a_name_carrying_a_control_character_is_refused() {
        // سطرٌ جديد = أمرٌ يُعرض على سطرين في «سَطْر» وفي السجلّ.
        for name in ["أ\nب", "أ\tب", "أ\rب", "أ\u{0}ب", "Fin\u{7}der"] {
            assert_eq!(
                refusal(plan_with(name)),
                ("err.name.invalid", Some("name")),
                "{name:?} must be refused"
            );
        }
    }

    #[test]
    fn a_name_that_begins_with_a_dash_is_refused_with_its_own_field() {
        // `-f` و`-x` و`--help` رايات `pgrep` حقيقية تغيّر معنى الأمر كلّه —
        // وأولاها هي بالضبط ما يمنعه هذا الملف. الرفض يحمل اسم الحقل.
        for name in ["-f", "-x", "--help", "-U 0"] {
            assert_eq!(
                refusal(plan_with(name)),
                ("err.arg.flag", Some("name")),
                "{name:?} must be refused"
            );
        }
    }

    #[test]
    fn an_over_long_name_is_refused_before_it_becomes_an_argument() {
        let long = "ا".repeat(MAX_NAME_LEN + 1);
        assert_eq!(refusal(plan_with(&long)), ("err.input.type", Some("name")));
    }

    #[test]
    fn a_name_is_never_trimmed_because_the_spaces_are_part_of_the_match() {
        let cmd = plan_with("  Finder  ").unwrap();
        assert_eq!(cmd.args[1], std::ffi::OsString::from("  Finder  "));
        assert_eq!(cmd.args.len(), 2, "a space must not split the name into two arguments");
    }

    #[test]
    fn shell_syntax_in_a_name_stays_one_literal_argument() {
        for name in ["a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "a | b", "\"مقتبس\""] {
            let cmd = plan_with(name).unwrap();
            assert_eq!(cmd.args[1], std::ffi::OsString::from(name), "{name:?} was rewritten");
            assert_eq!(cmd.args.len(), 2, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_regex_like_name_is_carried_literally_and_adds_no_argument() {
        // الاسم يصل الأداة كما كُتب. أن تقرأه `pgrep` تعبيرًا نمطيًا شأنُ
        // الأداة، وهو حدٌّ مذكور في رأس الملف — لا إعادةَ كتابةٍ من هنا.
        for name in ["Find.r", "^Finder$", "(a+)+b", "node|npm", "["] {
            let cmd = plan_with(name).unwrap();
            assert_eq!(args_of(&cmd), vec!["-l", name], "{name:?} was rewritten");
        }
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        // لا حقل في الشاشة يرسل هذا. لكن الحدّ لا يثق بالشاشة: مفتاحٌ لا تعرفه
        // المواصفة يعني إمّا واجهةً قديمة وإمّا محاولة تهريب، وكلاهما رفض.
        for smuggled in ["full", "pattern", "-f", "user", "signal"] {
            let mut raw = raw("Finder");
            raw.insert(smuggled.to_owned(), RawValue::Text("1".to_owned()));
            let r = crate::value::validate(&SPEC, &raw).and_then(|i| plan(&i));
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn the_search_verdict_lives_in_the_result_contract_not_in_a_warning() {
        // `1` تعني «لم أجد» لا «فشلت»، وترجمتها تقع في `result.rs`. تحذيرٌ هنا
        // كان سيقول للمستخدم شيئًا عن كل تشغيلٍ قبل أن يُعرف جوابه.
        let cmd = plan_with("Finder").unwrap();
        assert_eq!(cmd.warnings, Vec::<&str>::new());
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        // لا حالة ولا عشوائية ولا اسم مؤقّت: عمليةُ قراءةٍ يجب أن تكون دالّةً
        // ثابتة، وإلا كان في المسار شيءٌ لم نقصده.
        assert_eq!(plan_with("Finder").unwrap(), plan_with("Finder").unwrap());
    }

    #[test]
    fn planning_writes_nothing_to_disk() {
        let s = Scratch::new("pgrep-clean").unwrap();
        for _ in 0..10 {
            plan_with("Finder").unwrap();
        }
        assert_eq!(s.names(s.path()), Vec::<String>::new(), "planning must leave no trace");
    }
}
