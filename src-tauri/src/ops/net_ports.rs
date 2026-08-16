//! المنافذ التي يستمع عليها هذا الجهاز، باستخدام `lsof`.
//!
//! ## لماذا `lsof` لا `netstat`
//!
//! `netstat` على macOS تعرض المقابس ولا تقول لأيّ برنامج تنتمي — والسؤال الذي
//! يطرحه من يفتح هذه العملية ليس «أيّ منفذ مفتوح؟» بل «ما الذي يستمع عليه؟».
//! `lsof` تربط كل مقبس بالعملية التي تملكه وباسمها ورقمها، فتجيب السؤال
//! المقصود. وهي في `/usr/sbin/lsof` جزءًا من النظام.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/sbin/lsof -nP -iTCP -sTCP:LISTEN
//! ```
//!
//! * `-nP` — لا تحوّل العناوين إلى أسماء مضيفات، ولا أرقام المنافذ إلى أسماء
//!   خدمات. رايتان مدمجتان في رمزٍ واحد كما تكتبهما `lsof` نفسها.
//! * `-iTCP` — اقتصر على مقابس TCP.
//! * `-sTCP:LISTEN` — ومنها ما حالته «يستمع» وحده.
//!
//! ## لماذا `-nP` وليست زينة
//!
//! بدون `-n` تسأل `lsof` عن اسم كل عنوان تراه، فأمرٌ يفترض أن يكون فوريًا
//! يتعلّق ثوانيَ على استعلامات DNS — وقد يتعلّق أطول إن كانت الشبكة مقطوعة،
//! وهي بالضبط الحالة التي يُفتح فيها فحص المنافذ. وبدون `-P` يُعرض `443`
//! بالاسم `https`، فيبحث المستخدم عن رقمٍ لا يجده. الرقم الخام هو ما يُبحث
//! عنه وما يُنسخ إلى إعداد جدار ناري.
//!
//! ## ما لا يظهر هنا
//!
//! **ما لا يملكه المستخدم.** بلا صلاحيات مدير لا ترى `lsof` إلا عمليات
//! المستخدم الحالي، فقائمةٌ خالية من خدمات النظام لا تعني أنها لا تستمع.
//! وهذا المنتج لا يطلب صلاحيات المدير ولا يعرض على المستخدم أن يمنحها: رفعُ
//! الصلاحيات لعرض قائمةٍ مقايضةٌ خاسرة، وبديلُها الصادق أن تُعلَن حدود القائمة
//! قبل قراءتها. تحذير `warn.lsof.user_scope` يقول ذلك في كل تشغيل — وحضوره
//! الدائم مقصود: هو ليس حالةً عارضة بل صفةٌ ثابتة في كل نتيجة.
//!
//! **وما ليس TCP.** المستمعون على UDP (‏Bonjour و DHCP ومزامنة الوقت) خارج
//! هذه العملية. جمعُهما في أمرٍ واحد يخلط قائمتين لهما معنيان مختلفان: مستمع
//! TCP يقبل اتصالًا، ومقبس UDP لا «يستمع» بالمعنى نفسه أصلًا.
//!
//! ## بلا مدخلات
//!
//! لا حقل في هذه الشاشة، فلا نصّ من المستخدم يصل إلى الأمر بحال. الوسائط
//! الثلاثة ثوابت مكتوبة في هذا الملف، واختبارٌ أدناه يثبّت أن الفهرس يرفض أي
//! مدخل يُرسَل إليها.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "net.ports";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.net.ports.title",
    description_key: "op.net.ports.description",
    category: Category::Network,
    // قراءةٌ محضة لحالة النظام: لا تكتب ولا تفتح ولا تغلق شيئًا.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::LSOF,
    conflict: Conflict::NoArtifact,
    inputs: &[],
    sort_order: 30,
    search_terms: &[
        "lsof",
        "منافذ",
        "ports",
        "منفذ",
        "port",
        "استماع",
        "listen",
        "خوادم",
        "server",
        "شبكة",
        "network",
        "tcp",
        "netstat",
    ],
    plan,
};

fn plan(_inputs: &Inputs) -> Result<PlannedCommand> {
    Argv::tool(tools::LSOF, "explain.lsof.tool")
        .flag("-nP", "explain.lsof.no_lookups")
        .flag("-iTCP", "explain.lsof.tcp_only")
        .flag("-sTCP:LISTEN", "explain.lsof.listening_only")
        // ثابتٌ لا مشروط: حدُّ القائمة صفةٌ في كل نتيجة لا حالةٌ تطرأ أحيانًا.
        .warn("warn.lsof.user_scope")
        .read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::value::RawValue;
    use std::collections::BTreeMap;

    fn plan_it() -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &BTreeMap::new())?)
    }

    fn args_of(cmd: &PlannedCommand) -> Vec<String> {
        cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("net.ports must be listed");
        assert_eq!(found.category, Category::Network);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert!(found.inputs.is_empty(), "this screen has no form");
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let cmd = plan_it().unwrap();
        assert_eq!(cmd.program, std::path::Path::new("/usr/sbin/lsof"));
        assert_eq!(args_of(&cmd), vec!["-nP", "-iTCP", "-sTCP:LISTEN"]);
        assert!(cmd.artifact.is_none(), "reading the port table produces no file");
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.cwd.is_none());
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
    fn every_argument_is_explained_and_none_is_left_bare() {
        // ثلاث رايات غامضة الشكل (`-nP`, `-iTCP`, `-sTCP:LISTEN`) لا يفهمها
        // أحد بالنظر. رايةٌ بلا مفتاح شرحٍ هنا تعني سطرًا في «سَطْر» بلا معنى.
        let cmd = plan_it().unwrap();
        for token in cmd.explain.iter().skip(1) {
            assert_eq!(token.role, TokenRole::Flag, "{:?} is not a flag", token.token);
            assert!(token.key.is_some(), "{:?} carries no explanation", token.token);
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        // كل وسيطٍ هنا رايةٌ معلَنة عمدًا، ولا وسيط بيانات بينها أصلًا. فالفحص
        // ليس «لا شيء يبدأ بشرطة» بل «ما بدأ بشرطة راية معلَنة في الشرح».
        let cmd = plan_it().unwrap();
        for (arg, token) in cmd.args.iter().zip(cmd.explain.iter().skip(1)) {
            if cannot_be_read_as_a_flag(arg) {
                panic!("{arg:?} is a data argument in an operation that has no data");
            }
            assert_eq!(token.role, TokenRole::Flag);
        }
    }

    #[test]
    fn the_partial_view_is_announced_on_every_single_run() {
        for _ in 0..3 {
            let cmd = plan_it().unwrap();
            assert_eq!(cmd.warnings, vec!["warn.lsof.user_scope"]);
        }
    }

    #[test]
    fn no_user_text_can_reach_the_argument_list() {
        // لا حقل في المواصفة، فالنواة ترفض أي مفتاح يُرسَل — بما فيه ما يحمل
        // صياغة صدفة. هذا هو ما يجعل الأمر ثابتًا لا مبنيًّا.
        for smuggled in ["host", "port", "extra"] {
            let raw = BTreeMap::from([(
                smuggled.to_owned(),
                RawValue::Text("a; rm -rf ~ | $(whoami)".to_owned()),
            )]);
            let r = crate::value::validate(&SPEC, &raw);
            assert!(
                matches!(r, Err(CoreError::UnexpectedInput(ref k)) if k == smuggled),
                "{smuggled:?} must be refused, got {r:?}"
            );
        }
    }

    #[test]
    fn the_command_is_identical_on_every_plan() {
        let first = plan_it().unwrap();
        let second = plan_it().unwrap();
        assert_eq!(first.program, second.program);
        assert_eq!(first.args, second.args);
        assert_eq!(first.explain, second.explain);
    }
}
