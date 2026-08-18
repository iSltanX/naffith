//! من يشغل منفذًا بعينه الآن، باستخدام `lsof`.
//!
//! ## بمَ تختلف عن `net.ports`
//!
//! تلك تسأل «ما الذي يستمع على هذا الجهاز؟» فترشّح الحالة `LISTEN`
//! (‏`-sTCP:LISTEN`) وتسرد المستمعين وحدهم على كل المنافذ. وهذه تسأل عن
//! **منفذٍ واحد بعينه**، وتعرض ما عليه من مقابس بكل حالاتها — المستمعة
//! والمُنشأة (‏ESTABLISHED) معًا.
//!
//! والفرق قدرةٌ أخرى لا تضييقُ قائمة. السؤال الذي يفتح هذه الشاشة غالبًا
//! «لماذا لا أستطيع الاستماع على ٣٠٠٠؟»، وجوابُه قد لا يكون مستمعًا أصلًا: قد
//! يكون اتصالًا قائمًا على المنفذ نفسه، أو مقبسًا لم ينصرم بعدُ من تشغيلٍ
//! سابق. وشيءٌ من ذلك لا يظهر في `net.ports` بحال، لأنها ترشّح `LISTEN` قبل أن
//! تطبع — فتردّ قائمةً فارغة عن منفذٍ مشغولٍ فعلًا، وهي أسوأ إجابةٍ ممكنة على
//! هذا السؤال بالذات. فلو كانت هذه العملية ترشيحًا فوق تلك لكان الأصحّ ألّا
//! توجد؛ وإنما وُجدت لأنها ترى ما لا تراه تلك.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/sbin/lsof -nP -i :<المنفذ>
//! ```
//!
//! * `-nP` — لا تحوّل العناوين إلى أسماء مضيفات ولا أرقام المنافذ إلى أسماء
//!   خدمات. السبب مشروحٌ بتمامه في `net_ports.rs` ولا يُعاد هنا. ويكفي أنه هنا
//!   أوكد: من كتب في الحقل `3000` ينتظر أن يقرأ `3000` في المخرج، لا اسم خدمةٍ
//!   مسجَّلٍ لهذا الرقم في جدولٍ قديم.
//! * `-i` — اقتصر على ملفات الإنترنت، أي المقابس دون سائر ما تفتحه العمليات.
//! * `:<المنفذ>` — ومنها ما كان على هذا المنفذ. النقطتان بلا مضيفٍ قبلهما
//!   تعنيان في صيغة `lsof` «أيّ عنوان، هذا المنفذ».
//!
//! ولاحظ أنها `-i` مجرّدة لا `-iTCP` كما في `net.ports`: البروتوكول غير محدَّد،
//! فمقابس UDP على الرقم نفسه داخلةٌ في الجواب. وذلك مقصود — من يسأل «ما الذي
//! يحتلّ ٥٣٥٣؟» لا يعرف سلفًا أن ما يحتلّه TCP، وحصرُ السؤال في بروتوكولٍ
//! يفترض في السائل معرفةَ ما جاء يسأل عنه. وهو أيضًا ما يجعل هذه العملية
//! أوسع من أختها لا أضيق منها.
//!
//! ## لماذا `:<المنفذ>` رمزٌ منفصل لا ملتصقٌ بـ`-i`
//!
//! `lsof` تقبل الشكلين — `-i:3000` ملتصقًا، و`-i` ثم `:3000` منفصلين — ومقيسٌ
//! تجريبيًا أن ناتجهما واحد. فالاختيار ليس عن أثرٍ في الأداة بل عن بنية بانِي
//! الوسائط:
//!
//! رقم المنفذ قيمةٌ يكتبها المستخدم، وكل قيمةٍ منه تدخل الأمر عبر
//! `Argv::explained_value` التي ترفض ما يبدأ بشرطة. أمّا الشكل الملتصق فكان
//! سيحتاج رايةً تُبنى وقت التشغيل وتحمل قيمة المستخدم في جوفها، و`Argv::flag`
//! لا تقبل إلا `&'static str` — نصًّا ثابتًا في الثنائيّة عند الترجمة. أي أن
//! الشكل الملتصق ليس أصعب هنا، بل **غير قابلٍ للتعبير عنه** ببانِي الوسائط
//! أصلًا. وذلك هو المقصود: لا سبيل في هذه المعمارية إلى أن تُهرَّب قيمة مستخدمٍ
//! في موضع راية، ولو كانت الأداة تقبلها هناك.
//!
//! والفصل يكسب شيئًا ثانيًا: رمزان في «سَطْر» لكلٍّ منهما شرحه — «اقتصر على
//! المقابس» ثم «وعلى هذا المنفذ». والملتصق كان سيصير رمزًا واحدًا يشرح فكرتين.
//!
//! ## القائمة جزئية دائمًا
//!
//! `warn.lsof.user_scope` ثابتٌ في كل خطة لا مشروط، لنفس السبب المشروح في
//! `net_ports.rs`: بلا صلاحيات مدير لا ترى `lsof` إلا عمليات المستخدم الحالي،
//! وهذا المنتج لا يطلب تلك الصلاحيات ولا يعرض منحها. وحضور التحذير الدائم
//! مقصود — هو صفةٌ في كل نتيجة لا حالةٌ تطرأ أحيانًا.
//!
//! وأثره هنا أثقل منه هناك: مخرجٌ خالٍ عن منفذٍ تملكه خدمةٌ للنظام يُقرأ
//! «المنفذ حرّ»، وهو استنتاجٌ لا يصحّ. فالعملية تجيب «لا شيء **لك** على هذا
//! المنفذ»، والفرق بين الجملتين هو ما يقوله التحذير قبل التشغيل.
//!
//! ## ملاحظةٌ مقيسة لا يُبنى عليها كود
//!
//! هذا الشكل يخرج بالرمز `0` حين يجد شيئًا و`1` حين لا يجد شيئًا على المنفذ.
//! مذكورةٌ للعلم فحسب: لا تفسير هنا لرمز الخروج ولا شيء في هذا الملف يفرّق
//! بينهما — التفريق يقع مركزيًا خارج التخطيط، وعمليةٌ تقرأ رمز خروج أداتها
//! بنفسها كانت ستصير نسخةً ثانية من قرارٍ يخصّ النواة.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "net.port.owner";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.net.port.owner.title",
    description_key: "op.net.port.owner.description",
    category: Category::Network,
    // قراءةٌ محضة: لا تفتح منفذًا ولا تغلقه ولا تمسّ عمليةً تملكه.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::LSOF,
    conflict: Conflict::NoArtifact,
    // المدى هو مدى المنافذ نفسه: ‏١ أدناها و‏٦٥٥٣٥ أقصاها. والافتراضي ٣٠٠٠
    // لأنه المنفذ الذي يُسأل عنه فعلًا في هذا السياق أكثر من غيره.
    inputs: &[InputSpec::new("port", InputKind::Number { min: 1, max: 65535, default: 3000 })],
    sort_order: 35,
    search_terms: &["lsof", "port", "منفذ", "مالك", "owner", "يستعمل", "مشغول", "شبكة"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let port = inputs.number("port")?;

    Argv::tool(tools::LSOF, "explain.lsof.tool")
        .flag("-nP", "explain.lsof.no_lookups")
        .flag("-i", "explain.lsof.internet_files")
        // النقطتان بادئةٌ من صيغة `lsof` لا شرطة، فتقبلها `explained_value`.
        // والقيمة عددٌ تحقّقت النواة من مداه، فلا شيء فيها إلا أرقام.
        .explained_value(format!(":{port}"), "explain.lsof.port")
        // ثابتٌ لا مشروط: حدُّ القائمة صفةٌ في كل نتيجة. انظر رأس الملف.
        .warn("warn.lsof.user_scope")
        .read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;

    fn plan_raw(port: &str) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([("port".to_owned(), RawValue::Text(port.to_owned()))]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    fn plan_with(port: i64) -> Result<PlannedCommand> {
        plan_raw(&port.to_string())
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
        let found = listed.iter().find(|o| o.id == ID).expect("net.port.owner must be listed");
        assert_eq!(found.category, Category::Network);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.inputs.len(), 1, "one field: the port");
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let cmd = plan_with(3000).unwrap();
        assert_eq!(cmd.program, std::path::Path::new("/usr/sbin/lsof"));
        assert_eq!(args_of(&cmd), vec!["-nP", "-i", ":3000"]);
        assert!(cmd.artifact.is_none(), "asking who holds a port produces no file");
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.cwd.is_none(), "every argument is literal");
        assert!(cmd.reveal_target.is_none(), "there is no place on disk to open");
    }

    #[test]
    fn the_port_is_a_separate_token_never_glued_to_the_flag() {
        // الوعد المعماري في رأس الملف: قيمة المستخدم رمزٌ مستقلّ، ولا رمز في
        // الأمر يحمل الرقم داخل رايةٍ. `-i:3000` لو ظهر يومًا لكان يعني أن
        // رايةً بُنيت وقت التشغيل من مدخلٍ — وهو ما لا يجوز أن يوجد سبيلٌ إليه.
        for port in [1, 80, 3000, 65535] {
            let args = args_of(&plan_with(port).unwrap());
            assert_eq!(args[1], "-i", "the flag carries no value");
            assert_eq!(args[2], format!(":{port}"));
            assert_eq!(args.len(), 3, "no port may add or drop an argument");
        }
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        for port in [1, 8080, 65535] {
            let cmd = plan_with(port).unwrap();
            let mut expected = vec![cmd.program.display().to_string()];
            expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
            let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
            assert_eq!(shown, expected);
        }
    }

    #[test]
    fn every_argument_is_explained_and_none_is_left_bare() {
        // `-nP` و`-i` و`:3000` ثلاثةٌ لا يفهمها أحد بالنظر. رمزٌ بلا مفتاح شرح
        // هنا يعني سطرًا في «سَطْر» بلا معنى.
        let cmd = plan_with(3000).unwrap();
        for token in &cmd.explain {
            assert!(token.key.is_some(), "{:?} carries no explanation", token.token);
        }
    }

    #[test]
    fn the_port_is_declared_a_value_not_a_flag_so_the_screen_colours_it_right() {
        let cmd = plan_with(3000).unwrap();
        let roles: Vec<TokenRole> = cmd.explain.iter().map(|t| t.role).collect();
        assert_eq!(
            roles,
            vec![TokenRole::Tool, TokenRole::Flag, TokenRole::Flag, TokenRole::Value]
        );
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        for port in [1, 22, 3000, 65535] {
            let cmd = plan_with(port).unwrap();
            for (arg, token) in cmd.args.iter().zip(cmd.explain.iter().skip(1)) {
                match token.role {
                    // رايةٌ معلَنة يجب أن تبدو رايةً: ما عداه يعني أن بيانات
                    // عُرضت في موضع راية.
                    TokenRole::Flag => assert!(
                        !cannot_be_read_as_a_flag(arg),
                        "{arg:?} is announced as a flag but is not one"
                    ),
                    // وقيمةُ المستخدم يجب ألّا تبدو رايةً بحال.
                    _ => assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag"),
                }
            }
        }
    }

    #[test]
    fn nothing_but_a_colon_and_digits_ever_reaches_the_command() {
        for port in [1, 7, 443, 5432, 65535] {
            let last = args_of(&plan_with(port).unwrap()).pop().unwrap();
            assert_eq!(&last[..1], ":");
            assert!(
                last[1..].chars().all(|c| c.is_ascii_digit()),
                "{last:?} carries something that is not a digit"
            );
        }
    }

    #[test]
    fn the_partial_view_is_announced_on_every_single_run() {
        for port in [1, 3000, 65535] {
            let cmd = plan_with(port).unwrap();
            assert_eq!(cmd.warnings, vec!["warn.lsof.user_scope"]);
        }
    }

    #[test]
    fn a_port_outside_the_declared_range_is_refused_and_blamed_on_its_field() {
        for bad in ["0", "-1", "65536", "99999"] {
            assert_eq!(
                refusal(plan_raw(bad)),
                ("err.input.range", Some("port")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn a_port_that_is_not_a_number_is_refused_not_coerced() {
        for bad in ["", " ", "3000 -i", "3000,443", "٣٠٠٠", "0x50", "3000/tcp", "abc"] {
            assert_eq!(
                refusal(plan_raw(bad)),
                ("err.input.type", Some("port")),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn shell_syntax_in_the_port_field_is_refused_before_it_can_become_anything() {
        for shellish in ["$(whoami)", "3000; rm -rf ~", "3000 | cat", "`id`", "3000 && ls"] {
            assert_eq!(
                refusal(plan_raw(shellish)),
                ("err.input.type", Some("port")),
                "{shellish:?} must not reach the command"
            );
        }
    }

    #[test]
    fn a_field_this_screen_does_not_declare_is_refused() {
        // حقلٌ واحد معلَن، فأي مفتاحٍ آخر يُرسَل تردّه النواة باسمه. هذا ما يجعل
        // شكل الأمر ثابتًا: لا مدخل ثانٍ يستطيع أن يضيف رمزًا.
        for smuggled in ["host", "state", "extra"] {
            let raw = BTreeMap::from([
                ("port".to_owned(), RawValue::Text("3000".to_owned())),
                (smuggled.to_owned(), RawValue::Text("-sTCP:LISTEN".to_owned())),
            ]);
            let r = crate::value::validate(&SPEC, &raw);
            assert!(
                matches!(r, Err(CoreError::UnexpectedInput(ref k)) if k == smuggled),
                "{smuggled:?} must be refused, got {r:?}"
            );
        }
    }

    #[test]
    fn the_command_is_identical_on_every_plan_for_the_same_port() {
        let first = plan_with(3000).unwrap();
        let second = plan_with(3000).unwrap();
        assert_eq!(first.program, second.program);
        assert_eq!(first.args, second.args);
        assert_eq!(first.explain, second.explain);
        assert_eq!(first.warnings, second.warnings);
    }

    #[test]
    fn planning_writes_nothing_to_disk() {
        // لا مسار في هذه العملية، فلا موضع «طبيعي» تكتب فيه. والاختبار يثبّت
        // ذلك بدل أن يفترضه: مساحةٌ فارغة تبقى فارغة بعد عشرات التخطيطات.
        let s = Scratch::new("port-owner-clean").unwrap();
        for port in 1..=50 {
            plan_with(port).unwrap();
        }
        assert_eq!(s.names(s.path()), Vec::<String>::new());
    }
}
