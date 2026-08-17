//! ما تفتحه عمليةٌ واحدة من ملفّاتٍ ومقابس، باستخدام `lsof`.
//!
//! ## السؤال، ومقابله في الفهرس
//!
//! في هذا الفهرس سؤالان متقابلان عن مقابض الملفّات، ولكلٍّ منهما شاشةٌ خاصّة:
//! `disk.directory.open_handles` تسأل «من يفتح ما في هذا المجلد؟» فتأخذ موضعًا
//! وتردّ بقائمة عمليات، وهذه تسأل «ماذا تفتح هذه العملية؟» فتأخذ رقمًا وتردّ
//! بقائمة ملفّاتٍ ومقابس. الأداة واحدة والاتجاه معكوس.
//!
//! وفصلُهما قرارٌ لا تكرار. شاشةٌ واحدة بحقلين — مجلدٌ **أو** رقم — كانت تعني
//! نموذجًا يُملأ نصفه ويُهمَل نصفه، ونتيجةً لا يعرف قارئها أيّ السؤالين
//! أجابت. وحين يقف السؤال في عنوان الشاشة، يعرف من يفتحها ما سيحصل عليه قبل
//! أن يكتب شيئًا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/sbin/lsof -nP -p <رقم العملية>
//! ```
//!
//! * `-nP` — لا تحوّل العناوين إلى أسماء مضيفات، ولا أرقام المنافذ إلى أسماء
//!   خدمات. رايتان مدمجتان في رمزٍ واحد كما تكتبهما `lsof` نفسها.
//! * `-p` — اقتصر على العملية ذات هذا الرقم وحدها.
//!
//! ## ولماذا `-nP` هنا أيضًا، لا في فحص المنافذ وحده
//!
//! ما تفتحه عمليةٌ ليس ملفّاتٍ فقط: كل اتصال شبكةٍ لها يظهر سطرًا في القائمة
//! نفسها. فبدون `-n` تسأل `lsof` عن اسم كل عنوانٍ تراه في تلك الأسطر، فأمرٌ
//! يفترض أن يكون فوريًا يتعلّق ثوانيَ على استعلامات DNS — وهي كلفةٌ تُدفع على
//! متصفّحٍ يفتح مئة مقبس، لا على سطرٍ أو سطرين. وبدون `-P` يُعرض `443` بالاسم
//! `https`، فيبحث المستخدم عن رقمٍ لا يجده.
//!
//! ## لماذا رقمٌ لا اسمُ برنامج
//!
//! `lsof -c <اسم>` صيغةٌ قائمة، ورفضُها مقصود لسببين. الأول أن الاسم يطابق
//! أكثر من عملية: نسختان من البرنامج نفسه تعطيان قائمةً واحدة مدموجة لا
//! يفرّق قارئها بين مقابض هذه ومقابض تلك. والثاني أن الاسم نصٌّ حرّ يكتبه
//! المستخدم فيحتاج حارسًا خاصًّا به قبل أن يصير وسيطًا، أمّا الرقم فتحرسه
//! النواة بمداه المعلَن قبل أن يبلغ `plan` أصلًا.
//!
//! والرقم ليس شيئًا يُخترع: شاشة «العمليات الأعلى استهلاكًا»
//! (`system.processes`) تعرض عمود `pid` أوّلًا لهذا الغرض بعينه — يُنسخ منها
//! ويُلصق هنا.
//!
//! ## مدى الرقم معلَن، والافتراضي ليس توصية
//!
//! من `1` إلى `99998`، والحدّ الأعلى هو `PID_MAX` على macOS: رقمٌ فوقه لا
//! يمكن أن يكون لعمليةٍ على هذا النظام بحال، ورفضُه في الحقل بسببٍ يُقرأ أصدق
//! من تشغيلٍ يعود صامتًا. و`lsof` نفسها لا تحرس هذا: قياسٌ على هذا الجهاز
//! يعطي رقمًا من ثمانية أرقام خرجًا فارغًا لا رسالةَ خطأ.
//!
//! والافتراضي `1` أصغر قيمةٍ مشروعة لا اقتراحُ عمليةٍ يُنظر فيها: العملية
//! رقم واحد هي `launchd` وهي لِـroot، فقراءتها بلا صلاحيات مدير تعطي قائمةً
//! فارغة — انظر القسمين التاليين. حقلٌ يبدأ بقيمةٍ صالحة الشكل أهدأ من حقلٍ
//! فارغ يرفض عند أول ضغطة، والرقم المقصود يُلصق فوقها.
//!
//! ## الرقم بلا مفتاح شرحٍ ثانٍ
//!
//! `.value()` لا `.explained_value()`: رايةُ `-p` التي تسبقه تقول ما هو،
//! ومفتاحٌ ثانٍ كان سيقول «هذا هو الرقم الذي كتبته» — جملةٌ تشغل سطرًا في
//! «سَطْر» ولا تضيف معرفة. نظير `-c <العدد>` في `net_ping.rs`.
//!
//! ## ما لا يظهر هنا
//!
//! **ما لا يملكه المستخدم.** بلا صلاحيات مدير لا ترى `lsof` إلا عمليات
//! المستخدم الحالي، فرقمُ عمليةٍ لمستخدمٍ آخر أو للنظام يعطي قائمةً فارغة لا
//! قائمةً ناقصة. وهذا المنتج لا يطلب صلاحيات المدير ولا يعرض على المستخدم أن
//! يمنحها: رفعُ الصلاحيات لعرض قائمةٍ مقايضةٌ خاسرة، وبديلُها الصادق أن تُعلَن
//! حدود القائمة قبل قراءتها. تحذير `warn.lsof.user_scope` يقول ذلك في كل
//! تشغيل — وحضوره الدائم مقصود: هو صفةٌ ثابتة في كل نتيجة لا حالةٌ تطرأ.
//!
//! ## و«لا خرج» جوابٌ صامت، وغامضٌ كذلك
//!
//! صفحة `lsof` تقول نصًّا إنها تعود بـ`1` حين تفشل في العثور على ما طُلب منها
//! سرده — وأرقام العمليات مذكورةٌ في تلك القائمة صراحةً — وقياسٌ على هذا
//! الجهاز يوافقها: `-p` برقمٍ لا عمليةَ له تطبع لا شيء وتعود بـ`1`، وكذلك
//! `-i` على منفذٍ لا أحد عليه.
//!
//! والأهمّ من رقم الخروج أن الخرج الفارغ **لا يفرّق بين حالتين**: رقمٌ لا
//! عمليةَ له، وعمليةٌ قائمةٌ تمامًا لكنها لمستخدمٍ آخر فلا تُرى. وليست هناك
//! حالةٌ ثالثة عمليًّا — كل عملية تفتح دليلها الحالي وملفّها التنفيذي على
//! الأقل — فالقائمة الفارغة تعني «لا أراها»، لا «لا تفتح شيئًا». حدٌّ يُقال
//! في وصف العملية قبل التشغيل بدل أن يُكتشف بعده.
//!
//! ## بلا إظهارٍ في Finder
//!
//! لا `.reveal(…)`: لم تُنتج هذه العملية موضعًا، وزرُّ إظهارٍ يفتح مجلدًا لم
//! نكتب فيه شيئًا جوابٌ عن سؤالٍ لم يُطرح. والمسارات التي تظهر في الخرج
//! مسارات ملفّاتٍ يفتحها غيرنا — وبعضها ليس ملفًّا في نظام الملفّات أصلًا
//! (مقابس، أنابيب، `KQUEUE`).

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.process.open_files";

/// أكبر رقم عمليةٍ يمنحه النظام: `PID_MAX` على macOS.
///
/// ثابتٌ مسمّى لا رقمٌ مبثوث في المواصفة والاختبار معًا: لو تغيّر يومًا تغيّر
/// الاثنان معه ولم يتفرّقا.
const MAX_PID: i64 = 99_998;

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.process.open_files.title",
    description_key: "op.system.process.open_files.description",
    category: Category::System,
    // قراءةٌ محضة: تسرد مقابض عمليةٍ ولا تكتب، ولا تُنهيها، ولا تغلق مقبضًا.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::LSOF,
    conflict: Conflict::NoArtifact,
    // حقلٌ واحد، وهو رقمٌ لا نصّ: النواة تحرس مداه قبل أن يبلغ `plan`.
    inputs: &[InputSpec::new("pid", InputKind::Number { min: 1, max: MAX_PID, default: 1 })],
    sort_order: 17,
    search_terms: &["lsof", "process", "files", "عملية", "ملفات", "مفتوحة", "مقابض", "pid"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let pid = inputs.number("pid")?;

    Argv::tool(tools::LSOF, "explain.lsof.tool")
        .flag("-nP", "explain.lsof.no_lookups")
        .flag("-p", "explain.lsof.process")
        // رقمٌ تشرحه رايتُه: انظر رأس الملف. والمدى المعلَن يجعله لا يبدأ
        // بشرطة أصلًا، و`value` ترفض ما يبدأ بها على أي حال.
        .value(pid.to_string())
        // ثابتٌ لا مشروط: حدُّ القائمة صفةٌ في كل نتيجة لا حالةٌ تطرأ أحيانًا.
        .warn("warn.lsof.user_scope")
        .read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::{RawValue, Value};
    use std::collections::BTreeMap;
    use std::path::Path;

    /// الرايات المعلَنة في هذا الأمر. ما عداها في `argv` بيانات.
    const DECLARED_FLAGS: &[&str] = &["-nP", "-p"];

    fn plan_raw(pid: &str) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([("pid".to_owned(), RawValue::Text(pid.to_owned()))]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    fn plan_with(pid: i64) -> Result<PlannedCommand> {
        plan_raw(&pid.to_string())
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
        let found =
            listed.iter().find(|o| o.id == ID).expect("system.process.open_files must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.inputs.len(), 1, "one field: the process number");
        assert_eq!(found.inputs[0].id, "pid");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_more() {
        let cmd = plan_with(42).unwrap();

        assert_eq!(cmd.program, Path::new("/usr/sbin/lsof"));
        assert_eq!(args_of(&cmd), vec!["-nP", "-p", "42"]);
        assert!(cmd.artifact.is_none(), "listing open handles produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "the question is about a process, not about `here`");
        assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
        assert!(cmd.estimate.is_none(), "no tree is measured");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let cmd = plan_with(1234).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn the_chosen_number_is_the_only_thing_the_form_changes() {
        for pid in [1, 2, 42, 65_535, MAX_PID] {
            let cmd = plan_with(pid).unwrap();
            assert_eq!(
                args_of(&cmd),
                vec!["-nP".to_owned(), "-p".to_owned(), pid.to_string()],
                "pid {pid} must change the number and nothing else"
            );
        }
    }

    #[test]
    fn every_flag_carries_an_explanation_and_the_number_leans_on_its_flag() {
        // `-nP` رمزٌ غامض الشكل لا يفهمه أحد بالنظر، و`-p` تقول ما هو الرقم
        // الذي يليها. أمّا الرقم نفسه فمفتاحٌ له كان سيكرّر رايته.
        let cmd = plan_with(7).unwrap();
        assert_eq!(cmd.explain.len(), 4);

        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert_eq!(cmd.explain[0].key, Some("explain.lsof.tool"));
        assert_eq!(cmd.explain[1].role, TokenRole::Flag);
        assert_eq!(cmd.explain[1].key, Some("explain.lsof.no_lookups"));
        assert_eq!(cmd.explain[2].role, TokenRole::Flag);
        assert_eq!(cmd.explain[2].key, Some("explain.lsof.process"));

        assert_eq!(cmd.explain[3].token, "7");
        assert_eq!(cmd.explain[3].role, TokenRole::Value);
        assert_eq!(cmd.explain[3].key, None, "the number is explained by its flag");
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        // كل ما يبدأ بشرطة هنا رايةٌ **معلَنة في هذا الملفّ**، وما عداها —
        // الرقم — لا يمكن أن يُقرأ راية.
        for pid in [1, 999, MAX_PID] {
            let cmd = plan_with(pid).unwrap();
            for (arg, token) in cmd.args.iter().zip(cmd.explain.iter().skip(1)) {
                if DECLARED_FLAGS.contains(&arg.to_string_lossy().as_ref()) {
                    assert_eq!(token.role, TokenRole::Flag);
                    continue;
                }
                assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag");
            }
        }
    }

    #[test]
    fn a_number_outside_the_declared_range_is_refused_and_blamed_on_its_field() {
        for out in [0, -1, -99, MAX_PID + 1, 12_345_678, i64::MAX] {
            assert_eq!(
                refusal(plan_with(out)),
                ("err.input.range", Some("pid")),
                "{out} must be refused by the core, not sent to lsof"
            );
        }
    }

    #[test]
    fn text_that_is_not_a_number_is_refused_and_blamed_on_its_field() {
        // ‏`lsof` نفسها تقبل صيغًا أخرى بعد `-p` (قوائم بفواصل، ومحارف
        // أخرى)، والنواة لا تسمح بأن يبلغها إلا عددٌ صحيح واحد.
        for bad in ["", "   ", "abc", "1,2", "1 2", "٧", "1.0", "0x10", "1_000"] {
            assert_eq!(
                refusal(plan_raw(bad)),
                ("err.input.type", Some("pid")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn surrounding_whitespace_is_trimmed_and_never_becomes_a_second_argument() {
        let cmd = plan_raw("  7\n").unwrap();
        assert_eq!(args_of(&cmd), vec!["-nP", "-p", "7"]);
    }

    #[test]
    fn the_argument_is_the_parsed_number_rendered_again_not_the_text_that_was_typed() {
        // ‏`"+7"` عددٌ صحيح عند Rust، فتقبله النواة وتعطي `7`. والوسيط يُبنى من
        // العدد لا من نصّ المستخدم، فالإشارة لا تعبر إلى الأمر أصلًا — وهذا هو
        // ما يجعل «لا وسيط يبدأ بشرطة» خاصيةً في البناء لا صدفةً في المدى.
        let cmd = plan_raw("+7").unwrap();
        assert_eq!(args_of(&cmd), vec!["-nP", "-p", "7"]);
        assert_eq!(cmd.explain[3].token, "7");
    }

    #[test]
    fn a_field_of_the_wrong_kind_is_refused_not_converted() {
        // مسارٌ في حقلٍ رقمي يعني واجهةً تُرسل غير ما تعلنه المواصفة. رفضٌ لا
        // تحويلٌ ضمني.
        let raw = BTreeMap::from([("pid".to_owned(), RawValue::Path("/tmp".to_owned()))]);
        assert_eq!(
            refusal(crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap())),
            ("err.input.type", Some("pid"))
        );
    }

    #[test]
    fn a_missing_number_is_refused_rather_than_defaulted_in_the_core() {
        // القيمة الافتراضية تخصّ الحقل في الواجهة، ولا تجعل الحقل اختياريًا:
        // طلبٌ بلا `pid` نقصٌ يُسمّى حقله، لا أمرٌ يُبنى على تخمين.
        assert_eq!(
            refusal(crate::value::validate(&SPEC, &BTreeMap::new()).map(|i| plan(&i).unwrap())),
            ("err.input.missing", Some("pid"))
        );
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        for smuggled in ["command", "user", "-p", "path", "folder"] {
            let raw = BTreeMap::from([
                ("pid".to_owned(), RawValue::Text("1".to_owned())),
                (smuggled.to_owned(), RawValue::Text("1".to_owned())),
            ]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_in_the_number_field_never_reaches_the_argument_list() {
        let baseline = plan_with(1).unwrap();
        for shellish in ["1; rm -rf ~", "$(whoami)", "`id`", "1 && reboot", "1 | grep", "1>/etc"] {
            assert_eq!(
                refusal(plan_raw(shellish)),
                ("err.input.type", Some("pid")),
                "{shellish:?} must not reach the command"
            );
        }
        assert_eq!(plan_with(1).unwrap().args, baseline.args);
        assert_eq!(baseline.args.len(), 3, "lsof takes exactly three arguments here");
    }

    #[test]
    fn a_number_that_bypassed_the_core_still_cannot_become_a_flag() {
        // ‏`plan` قد تُدعى من طريقٍ لا يمرّ بالنواة يومًا. المدى المعلَن يمنع
        // السالب، و`Argv::value` هي الحدّ الأخير حين لا يمنعه شيء قبله.
        let inputs = Inputs::from_pairs(vec![("pid", Value::Number(-1))]);
        assert_eq!(refusal(plan(&inputs)), ("err.arg.flag", None));
    }

    #[test]
    fn the_partial_view_is_announced_on_every_single_run() {
        for pid in [1, 42, MAX_PID] {
            assert_eq!(plan_with(pid).unwrap().warnings, vec!["warn.lsof.user_scope"]);
        }
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        // لا حالة ولا عشوائية ولا اسم مؤقّت: عمليةُ قراءةٍ بمدخلٍ واحد يجب أن
        // تكون دالّةً ثابتة، وإلا كان في المسار شيءٌ لم نقصده.
        assert_eq!(plan_with(42).unwrap(), plan_with(42).unwrap());
    }

    #[test]
    fn planning_writes_nothing_to_disk() {
        // لا مصدر ولا وجهة ولا ناتج في هذه العملية: لا موضع واحد كان يُخطَّط
        // له كتابة. الاختبار يثبّت ذلك على أرضٍ حقيقية — مجلدٌ مؤقّت لا علاقة
        // له بالعملية يبقى فارغًا بعد تخطيطاتٍ متكرّرة.
        let s = Scratch::new("lsof-open-files").unwrap();
        let untouched = s.dir("لا علاقة له بالعملية");
        let before = s.names(&untouched);

        for pid in 1..=10 {
            plan_with(pid).unwrap();
        }
        assert_eq!(s.names(&untouched), before, "planning must not touch the filesystem at all");
    }
}
