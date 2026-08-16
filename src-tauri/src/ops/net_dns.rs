//! استعلام سجلّات DNS باستخدام `dig`.
//!
//! ## لماذا `dig` لا `nslookup` ولا `host`
//!
//! الثلاثة في النظام. `nslookup` مهجورة منذ سنوات وتخلط في خرجها بين ما جاء
//! من الخادم وما استنتجته هي، و`host` تختصر الجواب في سطرٍ مصوغ لا في السجلّ
//! كما هو. `dig` تطبع السجلّات بصيغتها الأصلية — نفس الصيغة التي تُكتب بها
//! ملفات المناطق — فما تراه هو ما ردّ به الخادم حرفيًا. وهذا هو الفرق الذي
//! يهمّ منتجًا أطروحته أن يُري المستخدم ما جرى فعلًا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/dig +noall +answer <النطاق> <نوع السجلّ>
//! ```
//!
//! * `+noall` — أطفئ كل أقسام الخرج: الترويسة، والسؤال، والإحصاءات، والتعليقات.
//! * `+answer` — ثم أعِد قسم الإجابة وحده.
//!
//! الاثنتان معًا لا إحداهما: `+noall` وحدها تُخرج أمرًا صامتًا، و`+answer`
//! وحدها لا تُطفئ شيئًا. والبديل — تركُ الخرج كاملًا — يعني عشرة أسطر من
//! التعليقات حول سطرٍ واحد هو الجواب، وقارئٌ يبحث عن جوابه داخل ضجيج.
//!
//! ## ما لا تراه هنا
//!
//! `dig` تسأل خوادم الأسماء المُعلَنة في إعداد النظام مباشرة، ولا تمرّ بذاكرة
//! macOS المؤقّتة (‏mDNSResponder). فما تعرضه هذه العملية هو ما يقوله DNS
//! **الآن**، وقد يختلف عمّا يستعمله متصفّحك في هذه اللحظة لأنه يقرأ من ذاكرةٍ
//! محفوظة. هذا فرقٌ مقصود لا عيب: من يفحص تغييرًا في سجلّ يريد الحقيقة الحالية
//! لا المحفوظة — لكنه يُقال في الوصف كي لا يُكتشف بالمفاجأة.
//!
//! ولا تُظهر هذه العملية أسماء Bonjour المحلية (‏`.local`): تلك تُحلّ بـ mDNS
//! على الشبكة المحلية لا بـ DNS، ولا تعرفها `dig` بحال. تحذيرٌ يقول ذلك يسبق
//! التنفيذ بدل أن يعود المستخدم بجوابٍ فارغ لا يفهم سببه.
//!
//! ## سلامة الوسائط
//!
//! النطاق نصٌّ يكتبه المستخدم ولا يمرّ بـ`paths.rs`. `checked_domain` أدناه
//! يفحصه قبل بناء الأمر، ونوعُ السجلّ قائمةٌ مغلقة تخرج قيمتها من الشيفرة
//! المُترجَمة لا من الواجهة.

use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "net.dns";

/// أطول اسم نطاق يقبله DNS.
const MAX_DOMAIN_LEN: usize = 253;

/// أنواع السجلّات المعروضة.
///
/// ستّة لا كلّ ما يعرفه DNS. القائمة المغلقة قرارُ منتج: هذه هي الأنواع التي
/// يسأل عنها من ينقل موقعًا أو يضبط بريدًا، وإضافةُ ثلاثين نوعًا نادرًا تجعل
/// القائمة تُقرأ ولا تُختار. ومن احتاج نوعًا آخر فسطر `dig` أمامه في «سَطْر».
const RECORD_TYPES: &[ChoiceOption] = &[
    ChoiceOption::new("A", "choice.dns.a"),
    ChoiceOption::new("AAAA", "choice.dns.aaaa"),
    ChoiceOption::new("MX", "choice.dns.mx"),
    ChoiceOption::new("TXT", "choice.dns.txt"),
    ChoiceOption::new("NS", "choice.dns.ns"),
    ChoiceOption::new("CNAME", "choice.dns.cname"),
];

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.net.dns.title",
    description_key: "op.net.dns.description",
    category: Category::Network,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::DIG,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("domain", InputKind::Text { max_len: MAX_DOMAIN_LEN }),
        InputSpec::new("record", InputKind::Choice { options: RECORD_TYPES }),
    ],
    sort_order: 20,
    search_terms: &[
        "dig",
        "dns",
        "نطاق",
        "domain",
        "سجل",
        "record",
        "mx",
        "txt",
        "cname",
        "أسماء",
        "استعلام",
        "lookup",
        "nslookup",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let domain = checked_domain(inputs, "domain")?;
    let record = inputs.choice("record")?;

    // `+noall` و`+answer` صيغة رايات `dig` نفسها. تُضاف بـ`flag` لا بـ`value`
    // كي تحمل دور «راية» في الشرح فتُلوَّن رايةً — الدور معلَن لا مخمَّن من
    // النصّ، ولذلك لا تخدع بدايتُها بـ`+` بدل `-` التلوينَ.
    let mut argv = Argv::tool(tools::DIG, "explain.dig.tool")
        .flag("+noall", "explain.dig.noall")
        .flag("+answer", "explain.dig.answer")
        .explained_value(domain, "explain.dig.domain")
        .explained_value(record, "explain.dig.record");

    argv = argv.warn_all(warnings_for(domain));
    argv.read_only()
}

fn warnings_for(domain: &str) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    // اسمٌ بلا نقطة ليس اسمًا كاملًا. و`dig` لا تضيف لواحق البحث افتراضيًا
    // (‏`+search` مطفأة)، فتسأل عن الاسم كما هو ويعود الجواب فارغًا غالبًا.
    if !domain.contains('.') {
        warnings.push("warn.dns.single_label");
    }
    // أسماء Bonjour تُحلّ بـ mDNS لا بـ DNS. `dig` لا تعرفها بحال.
    if domain.to_ascii_lowercase().ends_with(".local") {
        warnings.push("warn.dns.mdns_not_dns");
    }
    warnings
}

/// يتحقّق أن نصّ المستخدم يشبه اسم نطاق قبل أن يصير وسيطًا.
///
/// ## لماذا هنا لا في النواة
///
/// `InputKind::Text` تعني «نصٌّ بطولٍ محدود» ولا تعني أكثر: النواة لا تعرف أن
/// هذا الحقل سيصير وسيطًا لأداة شبكة. فالحارس يقع في العملية التي تعرف، ويقع
/// **قبل** بناء الأمر لا بعده.
///
/// ## ما يُرفض، ولماذا
///
/// * **الفارغ** — `dig` بلا وسائط تسأل عن الجذر وتخرج بنجاح، فتشغيلٌ ناجح لا
///   يفحص ما أراده المستخدم. الرفض في الحقل أوضح.
/// * **ما يتجاوز ٢٥٣ محرفًا** — حدّ DNS نفسه.
/// * **ما يبدأ بشرطة** — نطاقٌ اسمه `+short` أو `-x` يُقرأ رايةً من رايات
///   `dig` فيغيّر الأمر معنًى كاملًا. `Argv::value` تمسك الشرطة أيضًا، وهذا
///   الفحص يسبقها كي يحمل الرفضُ اسمَ الحقل.
/// * **ما يبدأ أو ينتهي بنقطة** — تسميةٌ فارغة في الطرف.
/// * **ما فيه محرفٌ خارج ‏`[A-Za-z0-9.-]`** — أشهرها عنوانٌ كامل مُلصق في حقل
///   النطاق. أسماء النطاقات الدولية (‏العربية وغيرها) تُكتب على السلك بصيغة
///   Punycode (`xn--…`) وهي داخل المجموعة؛ أمّا النصّ العربي نفسه فيحتاج تحويلًا
///   لا تفعله هذه العملية ولا تدّعيه.
///
/// ## نسبة الرفض إلى سببٍ من مجموعة مغلقة
///
/// `NameRejection` في `error.rs` كُتبت لأسماء الملفات، وليس فيها عضوٌ يقول
/// «محرفٌ لا يصلح في اسم نطاق». وإضافة عضوٍ تعني تعديل ملفٍّ مشترك لا تملكه
/// عملية. فمخالفة مجموعة المحارف تُنسب إلى `ContainsSeparator`، ونصُّها
/// المعروض «لا يجوز أن يحتوي الاسم على /» يصلح الحالة الغالبة فعلًا: عنوانٌ
/// كامل في حقل نطاق. وما عداها منسوبٌ إلى سببه الحقيقي.
fn checked_domain<'a>(inputs: &'a Inputs, id: &'static str) -> Result<&'a str> {
    let raw = inputs.text(id)?;
    let domain = raw.trim();

    if domain.is_empty() {
        return Err(rejected(id, NameRejection::Empty));
    }
    // بالبايتات: المجموعة المسموحة كلها ASCII، فالبايت محرفٌ هنا.
    if domain.len() > MAX_DOMAIN_LEN {
        return Err(rejected(id, NameRejection::TooLong));
    }
    if domain.contains('\0') {
        return Err(rejected(id, NameRejection::ContainsNul));
    }
    if domain.chars().any(char::is_control) {
        return Err(rejected(id, NameRejection::ContainsControl));
    }
    if domain.starts_with('.') {
        return Err(rejected(id, NameRejection::LeadingDot));
    }
    // النقطة الأخيرة صيغةٌ صحيحة تعني «الجذر صراحةً»، ونرفضها رغم ذلك لأنها في
    // الحقل خطأ لصقٍ في الغالب. والثمن معلَن: من أراد تثبيت الاسم عند الجذر
    // لا يستطيع التعبير عنه من هنا.
    if domain.ends_with('.') {
        return Err(rejected(id, NameRejection::TrailingSpaceOrDot));
    }
    if domain.starts_with('-') || !domain.chars().all(is_domain_char) {
        return Err(rejected(id, NameRejection::ContainsSeparator));
    }
    Ok(domain)
}

fn is_domain_char(c: char) -> bool {
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

    fn plan_with(domain: &str, record: &str) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("domain".to_owned(), RawValue::Text(domain.to_owned())),
            ("record".to_owned(), RawValue::Text(record.to_owned())),
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
        let found = listed.iter().find(|o| o.id == ID).expect("net.dns must be listed");
        assert_eq!(found.category, Category::Network);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let cmd = plan_with("example.com", "A").unwrap();
        assert_eq!(cmd.program, std::path::Path::new("/usr/bin/dig"));
        assert_eq!(args_of(&cmd), vec!["+noall", "+answer", "example.com", "A"]);
        assert!(cmd.artifact.is_none(), "a query produces no file");
        assert!(cmd.stdout_to.is_none());
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let cmd = plan_with("example.com", "MX").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn both_output_switches_are_present_and_explained_as_flags() {
        // ‏`+noall` وحدها أمرٌ صامت، و`+answer` وحدها لا تُطفئ شيئًا. سقوطُ
        // إحداهما يغيّر الخرج كلّه، فالاختبار يثبّت الاثنتين معًا وترتيبهما.
        let cmd = plan_with("example.com", "A").unwrap();
        let switches: Vec<(&str, Option<&str>, TokenRole)> =
            cmd.explain.iter().skip(1).take(2).map(|t| (t.token.as_str(), t.key, t.role)).collect();
        assert_eq!(
            switches,
            vec![
                ("+noall", Some("explain.dig.noall"), TokenRole::Flag),
                ("+answer", Some("explain.dig.answer"), TokenRole::Flag),
            ]
        );
    }

    #[test]
    fn every_declared_record_type_reaches_the_command_unchanged() {
        for option in RECORD_TYPES {
            let cmd = plan_with("example.com", option.value).unwrap();
            let args = args_of(&cmd);
            assert_eq!(args.last().unwrap(), option.value);
            assert_eq!(args.len(), 4, "{} must not add an argument", option.value);
        }
    }

    #[test]
    fn a_record_type_outside_the_closed_list_is_refused_by_the_core() {
        // القيمة الداخلة إلى الأمر تخرج من `RECORD_TYPES` المُترجَمة، والواجهة
        // لا تفعل إلا أن تختار بينها. هذا ما يجعل الحقل غير قابل للتهريب.
        for smuggled in ["ANY", "-x", "AXFR", "a"] {
            assert_eq!(
                refusal(plan_with("example.com", smuggled)),
                ("err.input.type", Some("record")),
                "{smuggled:?} must not become an argument"
            );
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let cmd = plan_with("example.com", "TXT").unwrap();
        for arg in &cmd.args {
            assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag");
        }
    }

    #[test]
    fn an_empty_or_blank_domain_is_refused_and_blamed_on_its_field() {
        for blank in ["", "   ", "\t"] {
            assert_eq!(
                refusal(plan_with(blank, "A")),
                ("err.name.invalid", Some("domain")),
                "{blank:?} must be refused"
            );
        }
    }

    #[test]
    fn a_domain_that_would_be_read_as_a_dig_switch_is_refused() {
        // `-x` و`--help` تسقطان بقاعدة الشرطة البادئة، و`+short` بقاعدة
        // المجموعة: `+` خارجها. الثلاث رايات حقيقية لـ`dig` تغيّر معنى الأمر.
        for switchy in ["-x", "+short", "--help"] {
            assert_eq!(refusal(plan_with(switchy, "A")), ("err.name.invalid", Some("domain")));
        }
    }

    #[test]
    fn a_domain_with_a_dot_at_either_end_is_refused() {
        for edged in [".example.com", "example.com.", "."] {
            assert_eq!(refusal(plan_with(edged, "A")), ("err.name.invalid", Some("domain")));
        }
    }

    #[test]
    fn a_character_that_does_not_belong_in_a_domain_is_refused() {
        for bad in [
            "https://example.com",
            "example.com/path",
            "exa mple.com",
            "example.com:53",
            "نطاق.السعودية",
            "under_score.com",
            "user@example.com",
        ] {
            assert_eq!(
                refusal(plan_with(bad, "A")),
                ("err.name.invalid", Some("domain")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn a_control_character_never_reaches_the_argument_list() {
        // محرفٌ **داخل** الاسم يُرفض.
        for bad in ["exa\nmple.com", "a\0b", "a\tb"] {
            assert_eq!(refusal(plan_with(bad, "A")), ("err.name.invalid", Some("domain")));
        }

        // ومحرفٌ في الطرف يُقصّ لا يُرفض. الوعد في اسم هذا الاختبار هو أن شيئًا
        // من ذلك لا يبلغ الأمر، والقصّ يفي به كما يفي الرفض.
        for trimmed in ["example.com\r", " example.com ", "example.com\n"] {
            let cmd = plan_with(trimmed, "A").expect("trailing whitespace is not a rejection");
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
    fn shell_syntax_in_a_domain_is_refused_before_it_can_become_anything() {
        for shellish in ["a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "a|b"] {
            assert_eq!(
                refusal(plan_with(shellish, "A")),
                ("err.name.invalid", Some("domain")),
                "{shellish:?} must not reach the command"
            );
        }
    }

    #[test]
    fn an_over_long_domain_is_refused_by_both_guards() {
        let long = format!("{}.com", "a".repeat(MAX_DOMAIN_LEN));
        assert_eq!(refusal(plan_with(&long, "A")), ("err.input.type", Some("domain")));

        let inputs =
            Inputs::from_pairs(vec![("domain", Value::Text(long)), ("record", Value::Choice("A"))]);
        assert_eq!(refusal(plan(&inputs)), ("err.name.invalid", Some("domain")));
    }

    #[test]
    fn a_name_with_no_dot_is_planned_but_the_empty_answer_is_predicted() {
        let cmd = plan_with("localhost", "A").unwrap();
        assert!(cmd.warnings.contains(&"warn.dns.single_label"), "{:?}", cmd.warnings);
        assert_eq!(args_of(&cmd)[2], "localhost", "the warning does not change the command");
    }

    #[test]
    fn a_bonjour_name_is_planned_but_announced_as_outside_dns() {
        for name in ["mac-mini.local", "MAC-MINI.LOCAL"] {
            let cmd = plan_with(name, "A").unwrap();
            assert!(cmd.warnings.contains(&"warn.dns.mdns_not_dns"), "{:?}", cmd.warnings);
        }
    }

    #[test]
    fn an_ordinary_query_raises_no_noisy_warning() {
        let cmd = plan_with("example.com", "A").unwrap();
        assert_eq!(cmd.warnings, Vec::<&str>::new());
    }

    #[test]
    fn surrounding_whitespace_is_trimmed_and_never_becomes_a_second_argument() {
        let cmd = plan_with("  example.com  ", "NS").unwrap();
        assert_eq!(args_of(&cmd), vec!["+noall", "+answer", "example.com", "NS"]);
    }

    #[test]
    fn a_punycode_domain_passes_through_untouched() {
        // النطاقات الدولية تُكتب على السلك هكذا، وهي داخل المجموعة المسموحة.
        let cmd = plan_with("xn--mgbaam7a8h.xn--ngbc5azd", "A").unwrap();
        assert_eq!(args_of(&cmd)[2], "xn--mgbaam7a8h.xn--ngbc5azd");
    }
}
