//! قراءة ترويسات عنوان عبر `curl`.
//!
//! ## لماذا `curl`
//!
//! لأنها في النظام، ولأنها الأداة التي تفصل الطلب عن الجسد: `-I` تطلب
//! الترويسات وحدها فلا يُنزَّل شيء. البديل — فتح العنوان في المتصفّح وقراءة
//! أدوات المطوّر — يجيب السؤال نفسه، لكنه يشغّل السكربتات ويرسل الكعكات
//! ويسجّل الزيارة. هذه العملية تسأل الخادم سؤالًا واحدًا وتعرض جوابه.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/curl -sS -I -L --max-time 20 --max-redirs 5 <العنوان>
//! ```
//!
//! * `-sS` — أطفئ عدّاد التقدّم، وأبقِ رسائل الخطأ. الحرفان معًا لا أحدهما:
//!   `-s` وحدها تُصمت الفشل أيضًا، فيعود أمرٌ فاشل بلا سطرٍ يقول لماذا.
//! * `-I` — اطلب الترويسات وحدها (‏`HEAD`). لا يُنزَّل جسد الردّ.
//! * `-L` — اتبع إعادة التوجيه.
//! * `--max-time 20` — سقف زمني للعملية كلها.
//! * `--max-redirs 5` — سقفٌ لعدد مرات التتبّع.
//!
//! ## لماذا سقفان لا سقف واحد
//!
//! السقفان يحرسان عطبين مختلفين. `--max-time` تحرس من خادمٍ يقبل الاتصال ثم
//! يصمت — وهو ما يترك الأمر معلّقًا بلا خطأ. و`--max-redirs` تحرس من حلقة
//! إعادة توجيه: خادمان يحيل كلٌّ منهما إلى الآخر يستهلكان المهلة كاملةً في
//! طلباتٍ صحيحة الشكل. `curl` تتبع حتى خمسين مرة افتراضيًا، وخفضُها إلى خمسٍ
//! قرارُ منتج: سلسلةٌ أطول من ذلك سوء إعدادٍ لا وجهةٌ صحيحة.
//!
//! ## حدُّ `HEAD`، ولا نسكت عنه
//!
//! ليس كل خادم يعامل `HEAD` كما يعامل `GET`. بعضها يردّ `405`، وبعضها يردّ
//! ترويسات تختلف في `Content-Length` أو في الكعكات. فالمعروض هنا ترويسات
//! **هذا الطلب**، لا ترويسات تنزيلٍ كامل. تحذيرٌ يقول ذلك يسبق كل تشغيل: هو
//! صفةٌ ثابتة في الطريقة لا حالةٌ تطرأ، فحضوره الدائم هو الصدق.
//!
//! ## العنوان: نقّته النواة، ولا نعيد تنقيته
//!
//! `InputKind::Url` يمرّ بـ`value::sanitize_url` قبل أن يصل إلى هنا: `http`
//! و`https` وحدهما (فلا `file:` ولا `scp:` تتجاوز `paths.rs`)، ولا محارف
//! تحكّم ولا فراغ، ولا بدايةٌ بشرطة، ولا طولٌ يتجاوز الحدّ. إعادةُ الفحص هنا
//! كانت ستُنشئ نسختين من قاعدةٍ واحدة تفترقان يوم تتغيّر إحداهما — وهو أسوأ
//! من فحصٍ واحد في موضعٍ واحد. ما نضيفه فوقها ليس فحصًا بل تحذيرًا: عنوان
//! `http` بلا تشفير يُنفَّذ ويُعلَن، لا يُرفض.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "net.headers";

/// السقف الزمني للطلب كله بالثواني.
///
/// عشرون تكفي ترويسات أي خادم حيّ. وهي **ليست** حقلًا في النموذج للسبب نفسه
/// الذي في `net_ping`: رقمٌ لا سبب لأحدٍ أن يغيّره، وضبطُه على قيمةٍ كبيرة
/// يعيد المشكلة التي وُضع لأجلها.
const MAX_TIME_SECONDS: &str = "20";

/// أقصى عدد مرات تتبّع إعادة التوجيه.
const MAX_REDIRECTS: &str = "5";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.net.headers.title",
    description_key: "op.net.headers.description",
    category: Category::Network,
    // طلبُ ترويسات لا يكتب على القرص شيئًا.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::CURL,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("url", InputKind::Url)],
    sort_order: 40,
    search_terms: &[
        "curl",
        "ترويسات",
        "headers",
        "header",
        "عنوان",
        "url",
        "http",
        "https",
        "رابط",
        "شبكة",
        "network",
        "redirect",
        "تحويل",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let url = inputs.url("url")?;

    let mut argv = Argv::tool(tools::CURL, "explain.curl.tool")
        .flag("-sS", "explain.curl.silent_but_loud")
        .flag("-I", "explain.curl.head")
        .flag("-L", "explain.curl.follow")
        .flag("--max-time", "explain.curl.max_time")
        .explained_value(MAX_TIME_SECONDS, "explain.curl.max_time.headers")
        .flag("--max-redirs", "explain.curl.max_redirs")
        .explained_value(MAX_REDIRECTS, "explain.curl.max_redirs.value")
        .explained_value(url, "explain.curl.url")
        // ثابتٌ لا مشروط: `HEAD` طريقةٌ لها حدٌّ في كل ردّ، لا في بعضها.
        .warn("warn.curl.head_may_differ");

    if is_plaintext(url) {
        argv = argv.warn("warn.url.plaintext");
    }

    argv.read_only()
}

/// هل العنوان بلا تشفير؟
///
/// المقارنة غير حسّاسة لحالة الأحرف لأن `sanitize_url` تقبل `HTTP://` كما
/// تقبل `http://` ولا تعيد كتابة ما تقبله: العنوان يدخل الأمر كما كُتب.
fn is_plaintext(url: &str) -> bool {
    url.to_ascii_lowercase().starts_with("http://")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::value::RawValue;
    use std::collections::BTreeMap;

    fn plan_with(url: &str) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([("url".to_owned(), RawValue::Text(url.to_owned()))]);
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
        let found = listed.iter().find(|o| o.id == ID).expect("net.headers must be listed");
        assert_eq!(found.category, Category::Network);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let cmd = plan_with("https://example.com/a").unwrap();
        assert_eq!(cmd.program, std::path::Path::new("/usr/bin/curl"));
        assert_eq!(
            args_of(&cmd),
            vec![
                "-sS",
                "-I",
                "-L",
                "--max-time",
                "20",
                "--max-redirs",
                "5",
                "https://example.com/a",
            ]
        );
        assert!(cmd.artifact.is_none(), "reading headers writes no file");
        assert!(cmd.stdout_to.is_none(), "the answer is streamed, not captured");
        assert!(cmd.cwd.is_none());
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let cmd = plan_with("https://example.com").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let cmd = plan_with("https://example.com/x").unwrap();
        // المواضع ٠ و١ و٢ و٣ و٥ رايات معلَنة؛ وما عداها قيم.
        for i in [4usize, 6, 7] {
            assert!(
                cannot_be_read_as_a_flag(&cmd.args[i]),
                "{:?} would be read as a flag",
                cmd.args[i]
            );
        }
    }

    #[test]
    fn both_ceilings_are_present_because_they_guard_different_failures() {
        // مهلةٌ بلا سقفِ تحويلٍ تُستهلك في حلقة إعادة توجيه صحيحة الشكل، وسقفُ
        // تحويلٍ بلا مهلة لا ينقذ من خادمٍ يقبل الاتصال ثم يصمت.
        let cmd = plan_with("https://example.com").unwrap();
        let args = args_of(&cmd);
        let at = |flag: &str| args.iter().position(|a| a == flag).expect("flag must be present");
        assert_eq!(args[at("--max-time") + 1], "20");
        assert_eq!(args[at("--max-redirs") + 1], "5");
    }

    #[test]
    fn the_url_is_the_last_argument_and_is_never_rewritten() {
        for url in [
            "https://example.com",
            "https://example.com/",
            "https://example.com/a/b?c=1&d=2#frag",
            "HTTPS://EXAMPLE.COM/Mixed",
            "https://example.com/%D9%85%D9%84%D9%81",
        ] {
            let cmd = plan_with(url).unwrap();
            assert_eq!(args_of(&cmd).last().unwrap(), url, "{url} must pass through untouched");
        }
    }

    #[test]
    fn shell_syntax_inside_a_url_stays_one_literal_argument() {
        // هذه المحارف صالحة في مسار العنوان واستعلامه، والنواة لا ترفضها.
        // الأمر يُبنى مصفوفةً ولا يمرّ بمفسّر، فتبقى محارف عادية في وسيطٍ واحد.
        for url in [
            "https://example.com/a;b",
            "https://example.com/?q=$(whoami)",
            "https://example.com/`tick`",
            "https://example.com/?a=1&b=2",
            "https://example.com/a|b",
            "https://example.com/'quoted'",
        ] {
            let cmd = plan_with(url).unwrap();
            assert_eq!(cmd.args.len(), 8, "{url} must not add an argument");
            assert_eq!(args_of(&cmd).last().unwrap(), url);
        }
    }

    #[test]
    fn a_scheme_the_core_refuses_never_reaches_curl() {
        // الفحص في `value.rs` لا هنا، والاختبار يثبّت أنه يسري على هذه العملية:
        // `curl` تفهم `file:` و`scp:`، فمخطّطٌ غير مفحوص كان قراءةً لقرص محلي
        // تتجاوز سياسة المسارات كاملةً.
        for bad in [
            "file:///etc/passwd",
            "ftp://example.com/x",
            "scp://host/x",
            "smb://host/share",
            "example.com",
            "//example.com",
            "https://",
            "",
            "   ",
        ] {
            assert_eq!(
                refusal(plan_with(bad)),
                ("err.input.url", Some("url")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn a_url_that_would_be_read_as_a_flag_is_refused_by_the_core() {
        for flagish in ["-o/etc/passwd", "--output=/tmp/x", "-I"] {
            assert_eq!(refusal(plan_with(flagish)), ("err.input.url", Some("url")));
        }
    }

    #[test]
    fn a_newline_inside_a_url_never_becomes_a_second_line_anywhere() {
        // سطرٌ جديد داخل العنوان يفصله إلى شيئين في الشرح وفي السجلّ معًا،
        // فيُقرأ غير ما يُنفَّذ.
        for bad in ["https://example.com/\nHost: evil", "https://example.com/a b"] {
            assert_eq!(refusal(plan_with(bad)), ("err.input.url", Some("url")));
        }
    }

    #[test]
    fn the_limit_of_the_head_method_is_announced_on_every_run() {
        for url in ["https://example.com", "https://other.example/x"] {
            let cmd = plan_with(url).unwrap();
            assert!(cmd.warnings.contains(&"warn.curl.head_may_differ"), "{:?}", cmd.warnings);
        }
    }

    #[test]
    fn an_unencrypted_url_is_planned_but_announced() {
        for url in ["http://example.com", "HTTP://EXAMPLE.COM/x"] {
            let cmd = plan_with(url).unwrap();
            assert!(cmd.warnings.contains(&"warn.url.plaintext"), "{url}: {:?}", cmd.warnings);
            assert_eq!(args_of(&cmd).last().unwrap(), url, "the warning changes nothing");
        }
    }

    #[test]
    fn an_encrypted_url_raises_no_plaintext_warning() {
        let cmd = plan_with("https://example.com/http://not-a-scheme").unwrap();
        assert!(!cmd.warnings.contains(&"warn.url.plaintext"), "{:?}", cmd.warnings);
    }
}
