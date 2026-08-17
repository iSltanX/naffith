//! إصدار أداة Git نفسها، باستخدام `git --version`.
//!
//! غياب `git` عن الجهاز مشروحٌ في رأس `git_init.rs`، ولا يُعاد هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git --version
//! ```
//!
//! ## لماذا بلا `-C <المستودع>` — أول عملية Git في هذا القسم بلا مستودع
//!
//! كل عملية أخرى في هذا القسم تسأل عن **مستودعٍ بعينه**: ما تعديلاته، ما
//! فروعه، سجّل فيه شيئًا. وهذه تسأل عن **الأداة نفسها**: أي نسخةٍ من `git`
//! مثبَّتةٌ على هذا الجهاز؟ سؤالٌ لا يتغيّر جوابه من مستودعٍ إلى آخر، بل من
//! جهازٍ إلى آخر — فلا معنى لأن تحمل حقلاً لمجلدٍ لن يُقرأ منه شيء. ولذلك
//! `inputs` هنا فارغة، وهذا أول ملفّ Git لا يطلب مستودعًا على الإطلاق.
//!
//! ## لماذا تستحقّ عمليةً قائمة بذاتها
//!
//! ‏`Availability` (في `spec.rs`) يُخفي بقيّة عمليات هذا القسم خلف رسالة
//! «الأداة غائبة» حين لا يوجد `git` على الجهاز، فمن يواجه ذلك لا يحتاج هذه
//! العملية ليعرف أن شيئًا ناقص. لكن من *يملك* `git` وتباغته عملية أخرى برسالة
//! لا يفهمها — أو من يقرأ تقرير خطأ ويحتاج أن ينسخ رقم نسخته بدقّة — يحتاج
//! جوابًا لا يُستنتَج من غياب رسالةٍ، بل يُقرأ بأربعة أسطرٍ صريحة. وهي نظير
//! `system.info` تمامًا: سؤالٌ صغير عن أداةٍ واحدة، لا تقريرٌ شامل عن الجهاز.
//!
//! ## لماذا `--version` لا `git version`
//!
//! الصيغتان تطبعان السطر نفسه — `git` تفهم الاثنتين. اختيرت الراية لأنها
//! **صريحةٌ في العرض**: من يقرأ الأمر في «سَطْر» يرى رمزًا يبدأ بشرطتين
//! فيعرف فورًا أنه رايةٌ اختارها التطبيق، لا كلمةً قد يظنّها اسمَ أمرٍ فرعيّ
//! آخر كـ`status` أو `diff` يقبل حقولاً. وهي نفس التفرقة التي يشرحها
//! `git_init.rs` بين `init` كرمزٍ من رموز الأمر و`value` كحقلٍ من المستخدم —
//! هنا الفرق أدقّ لأن `version` نفسها ليست حقلاً، لكن الشرطتان يزيلان أي لبس
//! محتمل بلا كلفة.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تتصل بالشبكة، ولا تقارن الرقم بأحدث إصدارٍ منشور، ولا تقترح تحديثًا.
//! تطبع ما يعرفه الملف التنفيذي المثبَّت عن نفسه محليًّا، وتتوقّف عند ذلك.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "git.version";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.version.title",
    description_key: "op.git.version.description",
    category: Category::Git,
    // قراءةٌ محضة: لا تكتب بايتًا واحدًا، ولا تمسّ مستودعًا.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::NoArtifact,
    inputs: &[],
    sort_order: 120,
    search_terms: &["git", "version", "إصدار", "نسخة", "تثبيت", "أداة", "توفّر"],
    plan,
};

fn plan(_inputs: &Inputs) -> Result<PlannedCommand> {
    // لا `-C` ولا مسار: لا مستودع يُخاطَب، والأمر كلّه رايةٌ واحدة بعد اسم
    // الأداة. ولا `reveal`: ليس هناك مسارٌ على القرص تُنتجه هذه العملية أو
    // تفحصه لتُعرض «إظهار في Finder» بعده.
    Argv::tool(tools::GIT, "explain.git.tool").flag("--version", "explain.git.version").read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn plan_it() -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &BTreeMap::new())?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("git.version must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
        assert!(found.inputs.is_empty(), "this screen has no form");
    }

    #[test]
    fn the_argv_is_the_documented_form_with_no_repository_named_anywhere() {
        let cmd = plan_it().unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args, vec!["--version"]);
        assert!(!args.iter().any(|a| a == "-C"), "this operation names no repository");

        assert!(cmd.artifact.is_none(), "reading a version produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "the version of the tool does not depend on `here`");
        assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
        assert!(cmd.warnings.is_empty(), "a one-line answer needs no caveat");
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
    fn both_tokens_are_the_tool_and_its_flag_and_both_are_explained() {
        // الأمر رمزان لا أكثر: اسم الأداة، ثم الراية. لو خلا أحدهما من مفتاح
        // شرحٍ لكانت شاشة «سَطْر» ناقصة لا مختصرة.
        let cmd = plan_it().unwrap();
        assert_eq!(cmd.explain.len(), 2);
        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert_eq!(cmd.explain[0].key, Some("explain.git.tool"));
        assert_eq!(cmd.explain[1].role, TokenRole::Flag);
        assert_eq!(cmd.explain[1].key, Some("explain.git.version"));
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag_by_accident() {
        // الوسيط الوحيد هنا رايةٌ معلَنة بنيّة: `--version` نفسها. هذا الاختبار
        // يُثبت أنه لا يُوجد وسيطٌ **ثانٍ** غير مقصود يمكن أن يُقرأ رايةً — إذ
        // لا يوجد بيانات مستخدمٍ في هذا الأمر أصلًا يمكن أن تنزلق إلى هذا الموضع.
        let cmd = plan_it().unwrap();
        assert_eq!(cmd.args.len(), 1);
        assert!(
            !cannot_be_read_as_a_flag(&cmd.args[0]),
            "the sole argument is the flag chosen by the app itself"
        );
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        for smuggled in ["repo", "revision", "--version", "verbose"] {
            let raw = BTreeMap::from([(smuggled.to_owned(), RawValue::Text("1".to_owned()))]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_cannot_reach_the_argv_because_there_is_nowhere_to_put_it() {
        let baseline = plan_it().unwrap();
        for shellish in ["; rm -rf ~", "$(whoami)", "`id`", "a & b", "> /etc/hosts"] {
            let raw = BTreeMap::from([("repo".to_owned(), RawValue::Text(shellish.to_owned()))]);
            assert!(crate::value::validate(&SPEC, &raw).is_err(), "{shellish:?} was accepted");
        }
        assert_eq!(plan_it().unwrap().args, baseline.args);
        assert_eq!(baseline.args, vec!["--version"], "the only argument is the app's own flag");
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        assert_eq!(plan_it().unwrap(), plan_it().unwrap());
    }

    #[test]
    fn planning_writes_nothing_to_disk() {
        // لا `repo` ولا `folder` ولا `destination` في هذه العملية: لا موضع
        // واحد كان يُخطَّط له كتابة. الاختبار يثبّت ذلك على أرضٍ حقيقية — مجلدٌ
        // مؤقّت لا علاقة له بالعملية يبقى فارغًا بعد تخطيطاتٍ متكرّرة.
        let s = Scratch::new("git-version-clean").unwrap();
        let untouched = s.dir("لا علاقة له بالعملية");
        let before = s.names(&untouched);

        for _ in 0..10 {
            plan_it().unwrap();
        }
        assert_eq!(s.names(&untouched), before, "planning must not touch the filesystem at all");
    }
}
