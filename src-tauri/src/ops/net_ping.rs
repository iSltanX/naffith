//! فحص وصول مضيف عبر `ping`.
//!
//! ## لماذا `ping` لا `curl` ولا `nc`
//!
//! الثلاثة تُسأل «هل الجهاز هناك؟» وتجيب عن أسئلة مختلفة. `curl` تسأل خادم
//! تطبيقات أن يجيب طلبًا، و`nc` تسأل عن منفذٍ بعينه — وكلاهما يعود بـ«لا»
//! عن مضيفٍ حيٍّ تمامًا لا يشغّل ما سألت عنه. `ping` تسأل طبقة الشبكة نفسها
//! (‏ICMP echo)، وهو السؤال الذي يقصده من يكتب «هل الاتصال يصل؟». وهي جزء من
//! النظام في `/sbin/ping`، فلا تحتاج تثبيتًا ولا تُقرأ من `PATH`.
//!
//! ## الصيغة
//!
//! ```text
//! /sbin/ping -c <العدد> -t 10 <المضيف>
//! ```
//!
//! * `-c` — عدد الرزم ثم توقّف. بدونها تعمل `ping` على macOS إلى ما لا نهاية.
//! * `-t 10` — سقف زمني للأمر كله بالثواني.
//!
//! ## لماذا المهلة مثبّتة ولا يملكها المستخدم
//!
//! المهلة ليست حقلًا في النموذج. عددُ الرزم وحده لا يحدّ زمن التشغيل: مضيفٌ
//! لا يردّ يجعل `ping` تنتظر كل رزمةٍ حتى تيأس، فتشغيلٌ من عشرين رزمة على
//! عنوانٍ ميت يبقى معلّقًا دقائق. البديل — حقلٌ ثالث اسمه «المهلة» — يضيف إلى
//! النموذج رقمًا لا سبب لأحدٍ أن يغيّره، ويسمح بضبطه على قيمةٍ تعيد المشكلة
//! نفسها. عشر ثوانٍ سقفٌ يكفي كل فحصٍ معقول، والإلغاء متاح لمن أراد أقلّ.
//!
//! وثمن هذا التثبيت مُعلَن لا مسكوتٌ عنه: `ping` ترسل رزمةً في الثانية، فعددٌ
//! يقارب العشر يبلغ المهلة قبل أن يكتمل. لا نقصّ مدى العدد لأجل ذلك — من أراد
//! عشرين رزمة يرى ما جُمع منها — لكن تحذيرًا يقول ذلك يسبق التنفيذ.
//!
//! ## ما لا تقوله هذه العملية
//!
//! صمتُ المضيف ليس موته. كثير من الشبكات والجدران النارية تُسقط ICMP بينما
//! يعمل الموقع على المنفذ ٤٤٣ تمامًا، فـ«لا ردّ» جوابٌ عن ICMP لا عن الجهاز.
//! ولا تقيس هذه العملية سعةَ الاتصال ولا سرعته، بل زمن ذهابٍ وإياب لرزمةٍ
//! صغيرة. وهذا مكتوبٌ في وصف العملية كي يُقرأ قبل التشغيل لا بعده.
//!
//! ## سلامة الوسائط
//!
//! اسم المضيف نصٌّ يكتبه المستخدم، فلا يمرّ بـ`paths.rs` ولا يحرسه شيء قبل
//! أن يصير وسيطًا. `checked_host` أدناه هو ذلك الحارس، ويقع **قبل** بناء
//! الأمر: ما لا يشبه اسم مضيف لا يصل إلى `argv` أصلًا.

use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "net.ping";

/// السقف الزمني للأمر كله بالثواني. انظر رأس الملف لسبب تثبيته.
const DEADLINE_SECONDS: i64 = 10;

/// أكبر عدد رزم يكتمل يقينًا داخل المهلة.
///
/// `ping` ترسل رزمةً في الثانية: الرزمة رقم *ن* تخرج عند الثانية *ن-١*، فما
/// يتجاوز المهلة يُقطع. الطرح واحدٌ لا صفر لأن ردّ الرزمة الأخيرة يحتاج
/// لحظةً بعد إرسالها، والحدّ المتحفّظ أصدق من حدٍّ يصحّ على الورق وحده.
const COUNT_THAT_FITS: i64 = DEADLINE_SECONDS - 1;

/// أطول اسم مضيف يقبله DNS.
const MAX_HOST_LEN: usize = 253;

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.net.ping.title",
    description_key: "op.net.ping.description",
    category: Category::Network,
    // لا تكتب شيئًا ولا تقرأ ملفًا: ترسل رزمًا وتطبع ما عاد.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::PING,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("host", InputKind::Text { max_len: MAX_HOST_LEN }),
        InputSpec::new("count", InputKind::Number { min: 1, max: 20, default: 5 }),
    ],
    sort_order: 10,
    search_terms: &[
        "ping",
        "بنق",
        "اتصال",
        "شبكة",
        "network",
        "مضيف",
        "host",
        "استجابة",
        "latency",
        "زمن",
        "وصول",
        "reachability",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let host = checked_host(inputs, "host")?;
    let count = inputs.number("count")?;

    let mut argv = Argv::tool(tools::PING, "explain.ping.tool")
        .flag("-c", "explain.ping.count")
        // العدد قيمةٌ تشرحها رايتُها: مفتاحٌ ثانٍ هنا كان سيقول «هذا هو العدد
        // الذي اخترته»، وهي جملةٌ تشغل سطرًا ولا تضيف معرفة.
        .value(count.to_string())
        .flag("-t", "explain.ping.deadline")
        .explained_value(DEADLINE_SECONDS.to_string(), "explain.ping.deadline_seconds")
        .explained_value(host, "explain.ping.host");

    if count > COUNT_THAT_FITS {
        argv = argv.warn("warn.ping.deadline_clips_count");
    }

    argv.read_only()
}

/// يتحقّق أن نصّ المستخدم يشبه اسم مضيف قبل أن يصير وسيطًا.
///
/// ## لماذا هنا لا في النواة
///
/// `InputKind::Text` تعني «نصٌّ بطولٍ محدود» ولا تعني أكثر: النواة لا تعرف أن
/// هذا الحقل بعينه سيصير وسيطًا لأداة شبكة. فالحارس يقع في العملية التي تعرف،
/// ويقع **قبل** بناء الأمر لا بعده.
///
/// ## ما يُرفض، ولماذا
///
/// * **الفارغ** — `ping` بلا مضيف تطبع صيغة الاستعمال، وتشغيلٌ ناجحٌ لا يفحص
///   شيئًا أسوأ من رفضٍ يُقرأ في الحقل.
/// * **ما يتجاوز ٢٥٣ محرفًا** — حدّ DNS نفسه.
/// * **ما يبدأ بشرطة** — `-c` مضيفًا تُقرأ رايةً. `Argv::value` تمسك هذا أيضًا،
///   وهذا الفحص يسبقه كي يحمل الرفضُ اسمَ الحقل الذي يصلحه المستخدم.
/// * **ما يبدأ أو ينتهي بنقطة** — نقطةٌ في الطرف تعني تسمية فارغة.
/// * **ما فيه محرفٌ خارج ‏`[A-Za-z0-9.-]`** — الفراغ وعلامات الصدفة والمحارف
///   العربية والشرطة المائلة. الأمر يُبنى مصفوفةً فلا خطر تفسير، لكن نصًّا كهذا
///   ليس اسم مضيف بحال، ورفضُه في الحقل أوضح من رسالة `ping` عن اسمٍ لا يُحلّ.
///
/// ## نسبة الرفض إلى سببٍ من مجموعة مغلقة
///
/// `NameRejection` في `error.rs` كُتبت لأسماء الملفات، وليس فيها عضوٌ يقول
/// «محرفٌ لا يصلح في اسم مضيف». وإضافة عضوٍ تعني تعديل `error.rs` — ملفٌّ
/// مشترك لا تملكه عملية. فالمخالفة في مجموعة المحارف تُنسب إلى
/// `ContainsSeparator`، ونصُّها المعروض «لا يجوز أن يحتوي الاسم على /» هو
/// بالضبط ما يصلح الحالة الغالبة عمليًا: عنوانٌ كامل مُلصق في حقل المضيف
/// (`https://example.com/x`). وما عدا ذلك منسوبٌ إلى سببه الحقيقي.
///
/// ## وما لا يفحصه
///
/// لا يسأل هل يُحلّ الاسم ولا هل المضيف موجود. ذاك سؤال `ping` نفسها، وجوابه
/// هو ناتج العملية لا شرطٌ لتشغيلها.
fn checked_host<'a>(inputs: &'a Inputs, id: &'static str) -> Result<&'a str> {
    let raw = inputs.text(id)?;
    // القصّ قبل الفحص: من يلصق اسمًا يلصق معه فراغًا غالبًا، ورفضُه على فراغٍ
    // طرفيّ تشدّدٌ لا يحمي شيئًا. وما بقي بعد القصّ هو ما يدخل الأمر حرفيًا.
    let host = raw.trim();

    if host.is_empty() {
        return Err(rejected(id, NameRejection::Empty));
    }
    // بالبايتات: مجموعة المحارف المسموحة كلها ASCII، فالبايت محرفٌ هنا.
    if host.len() > MAX_HOST_LEN {
        return Err(rejected(id, NameRejection::TooLong));
    }
    if host.contains('\0') {
        return Err(rejected(id, NameRejection::ContainsNul));
    }
    if host.chars().any(char::is_control) {
        return Err(rejected(id, NameRejection::ContainsControl));
    }
    if host.starts_with('.') {
        return Err(rejected(id, NameRejection::LeadingDot));
    }
    // النقطة الأخيرة صيغةٌ صحيحة في DNS تعني «الجذر صراحةً»، ونرفضها رغم ذلك:
    // هي في الحقل خطأ نسخٍ في كل مرةٍ رأيناها. والثمن معلَن — من أراد قطع
    // لواحق البحث لا يستطيع التعبير عنه من هنا.
    if host.ends_with('.') {
        return Err(rejected(id, NameRejection::TrailingSpaceOrDot));
    }
    if host.starts_with('-') || !host.chars().all(is_host_char) {
        return Err(rejected(id, NameRejection::ContainsSeparator));
    }
    Ok(host)
}

fn is_host_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '.' || c == '-'
}

fn rejected(id: &'static str, reason: NameRejection) -> CoreError {
    CoreError::InvalidName { reason }.on_input(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::value::{RawValue, Value};
    use std::collections::BTreeMap;

    fn plan_with(host: &str, count: i64) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("host".to_owned(), RawValue::Text(host.to_owned())),
            ("count".to_owned(), RawValue::Text(count.to_string())),
        ]);
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
        let found = listed.iter().find(|o| o.id == ID).expect("net.ping must be listed");
        assert_eq!(found.category, Category::Network);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let cmd = plan_with("example.com", 5).unwrap();
        assert_eq!(cmd.program, std::path::Path::new("/sbin/ping"));
        assert_eq!(args_of(&cmd), vec!["-c", "5", "-t", "10", "example.com"]);
        assert!(cmd.artifact.is_none(), "a reachability check produces no file");
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.cwd.is_none(), "every argument is absolute or literal");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let cmd = plan_with("example.com", 3).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn the_chosen_count_is_the_only_thing_the_form_changes() {
        for count in [1, 5, 20] {
            let cmd = plan_with("example.com", count).unwrap();
            let args = args_of(&cmd);
            assert_eq!(args[1], count.to_string());
            assert_eq!(args[3], "10", "the deadline is fixed, not a field");
            assert_eq!(args.len(), 5, "no count may add or drop an argument");
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let cmd = plan_with("example.com", 5).unwrap();
        for (i, arg) in cmd.args.iter().enumerate() {
            // الوسائط الفردية هنا رايات معلَنة (`-c`, `-t`)، وما عداها بيانات.
            if i == 0 || i == 2 {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag");
        }
    }

    #[test]
    fn a_count_that_the_deadline_would_clip_is_announced_before_it_runs() {
        let cmd = plan_with("example.com", 20).unwrap();
        assert!(cmd.warnings.contains(&"warn.ping.deadline_clips_count"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_check_raises_no_noisy_warning() {
        let cmd = plan_with("example.com", 5).unwrap();
        assert_eq!(cmd.warnings, Vec::<&str>::new());
    }

    #[test]
    fn an_empty_or_blank_host_is_refused_and_blamed_on_its_field() {
        for blank in ["", "   ", "\t"] {
            assert_eq!(
                refusal(plan_with(blank, 5)),
                ("err.name.invalid", Some("host")),
                "{blank:?} must be refused"
            );
        }
    }

    #[test]
    fn a_host_that_would_be_read_as_a_flag_is_refused() {
        for flagish in ["-c", "-rf", "--help"] {
            assert_eq!(refusal(plan_with(flagish, 5)), ("err.name.invalid", Some("host")));
        }
    }

    #[test]
    fn a_host_with_a_dot_at_either_end_is_refused() {
        for edged in [".example.com", "example.com.", "."] {
            assert_eq!(refusal(plan_with(edged, 5)), ("err.name.invalid", Some("host")));
        }
    }

    #[test]
    fn a_character_that_does_not_belong_in_a_host_is_refused() {
        for bad in [
            "https://example.com/x", // الحالة الغالبة: عنوانٌ كامل في حقل مضيف
            "example.com/path",
            "exa mple.com",
            "example.com:443",
            "مضيف.com",
            "example_com",
            "user@example.com",
        ] {
            assert_eq!(
                refusal(plan_with(bad, 5)),
                ("err.name.invalid", Some("host")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn a_control_character_never_reaches_the_argument_list() {
        // سطرٌ جديد داخل الوسيط يفصل ما يُعرض عمّا يُنفَّذ في كل سياقٍ يُقرأ فيه.
        // محرفٌ **داخل** الاسم يُرفض.
        for bad in ["exa\nmple.com", "a\0b", "a\tb"] {
            assert_eq!(refusal(plan_with(bad, 5)), ("err.name.invalid", Some("host")));
        }

        // ومحرفٌ في الطرف يُقصّ لا يُرفض — واللصق من متصفّح يجرّه غالبًا. الوعد
        // في اسم هذا الاختبار هو أن شيئًا من ذلك لا يبلغ الأمر، والقصّ يفي به
        // كما يفي الرفض. فالحارس على الوسيط لا على مسار الوصول إليه.
        for trimmed in ["example.com\r", " example.com ", "example.com\n"] {
            let cmd = plan_with(trimmed, 5).expect("trailing whitespace is not a rejection");
            for arg in args_of(&cmd) {
                assert!(
                    !arg.chars().any(char::is_control),
                    "a control character reached the argument list: {arg:?}"
                );
            }
            assert!(args_of(&cmd).contains(&"example.com".to_string()));
        }
    }

    #[test]
    fn shell_syntax_in_a_host_is_refused_before_it_can_become_anything() {
        for shellish in ["a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "a|b", "a>b"] {
            assert_eq!(
                refusal(plan_with(shellish, 5)),
                ("err.name.invalid", Some("host")),
                "{shellish:?} must not reach the command"
            );
        }
    }

    #[test]
    fn an_over_long_host_is_refused_by_both_guards() {
        // النواة تحدّ النصّ بـ٢٥٣ بايتًا، فما يتجاوزها يُرفض قبل `plan`.
        let long = "a".repeat(MAX_HOST_LEN + 1);
        assert_eq!(refusal(plan_with(&long, 5)), ("err.input.type", Some("host")));

        // وحارس العملية نفسه هو الفاصل حين يُدعى `plan` من طريقٍ آخر.
        let inputs =
            Inputs::from_pairs(vec![("host", Value::Text(long)), ("count", Value::Number(5))]);
        assert_eq!(refusal(plan(&inputs)), ("err.name.invalid", Some("host")));
    }

    #[test]
    fn a_count_outside_the_declared_range_is_refused_by_the_core() {
        for out in [0, 21, -1] {
            assert_eq!(refusal(plan_with("example.com", out)), ("err.input.range", Some("count")));
        }
    }

    #[test]
    fn surrounding_whitespace_is_trimmed_and_never_becomes_a_second_argument() {
        let cmd = plan_with("  example.com  ", 5).unwrap();
        assert_eq!(args_of(&cmd), vec!["-c", "5", "-t", "10", "example.com"]);
    }

    #[test]
    fn a_hyphen_inside_the_name_is_kept_because_real_hosts_carry_it() {
        let cmd = plan_with("my-host.example.co.uk", 5).unwrap();
        assert_eq!(args_of(&cmd).last().unwrap(), "my-host.example.co.uk");
    }
}
