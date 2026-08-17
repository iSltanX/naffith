//! معمارية المعالج وحدها: `arm64` أو `x86_64`، باستخدام `uname -m`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/uname -m
//! ```
//!
//! راية واحدة تطبع سطرًا واحدًا لا غير: اسم المعمارية العارية، بلا اسم النظام
//! ولا نسخة النواة ولا اسم الجهاز ولا التاريخ — وهو ما تطبعه `uname -a` معًا.
//!
//! ## لماذا عمليةٌ قائمة بذاتها لا امتدادًا لـ`system.info`
//!
//! `system.info` تستدعي `sw_vers` (اسم macOS ورقمه وبناؤه)، وهذه تستدعي
//! `uname` (معمارية المعالج) — أداتان مختلفتان تمامًا، لكل واحدةٍ منهما
//! مسارها الخاص. و`PlannedCommand` في هذا المنتج ناتج تخطيطٍ واحد: برنامجٌ
//! واحدٌ ووسائط ثابتة، بلا صدفة تصل بينهما ولا `&&` يسلسل أمرين. فلا سبيل
//! لأن تجيب عمليةٌ واحدة عن السؤالين معًا مهما تجاورا على الشاشة —
//! التجاور في الواجهة لا يبرّر الدمج في الشيفرة.
//!
//! ## ولماذا `-m` لا `uname -a`
//!
//! `uname -a` تطبع سطرًا واحدًا يحشد فيه اسم النظام (`Darwin`) واسم الجهاز
//! ونسخة النواة والتاريخ والمعمارية معًا — جوابٌ عن خمسة أسئلة لسؤالٍ واحد.
//! من ينسخ ذلك السطر إلى منتدًى أو تذكرة دعمٍ ينسخ معه اسم جهازه. `-m` تجيب
//! عن السؤال وحده: أي معالج، `arm64` (Apple Silicon) أو `x86_64` (Intel)،
//! ولا شيء غيره.
//!
//! ## بلا مسار يختاره المستخدم
//!
//! السؤال هنا عن الجهاز نفسه لا عن ملفٍّ أو مجلد — نظير `system.info` تمامًا.
//! لا `inputs` هنا، فلا نموذج يُعرض ولا قيمةٌ تدخل الأمر من خارج الشيفرة.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تكتب شيئًا ولا تتصل بشبكة. تسأل النواة عن معمارية المعالج الذي تعمل
//! عليه، وتتوقّف عند ذلك.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "system.architecture";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.system.architecture.title",
    description_key: "op.system.architecture.description",
    category: Category::System,
    // سطرٌ واحد من قراءة. لا تُنشئ ولا تعدّل ولا تتلف.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::UNAME,
    conflict: Conflict::NoArtifact,
    inputs: &[],
    sort_order: 60,
    search_terms: &["uname", "architecture", "arm64", "x86_64", "معمارية", "معالج", "بنية", "جهاز"],
    plan,
};

fn plan(_inputs: &Inputs) -> Result<PlannedCommand> {
    // رمزان فقط: اسم الأداة، ثم راية `-m` وحدها. كلٌّ منهما بمفتاح شرحه
    // الخاص — لا شرحٌ واحد يغطّي الاثنين، فلا يمكن أن يظهر أحدهما بلا تفسير.
    Argv::tool(tools::UNAME, "explain.uname.tool").flag("-m", "explain.uname.machine").read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("system.architecture must be listed");
        assert_eq!(found.category, Category::System);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert!(found.inputs.is_empty(), "this screen has no form");
    }

    #[test]
    fn the_argv_is_the_tool_and_the_machine_flag_alone() {
        let cmd = plan_it().unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/uname"));
        assert_eq!(cmd.args, vec!["-m"]);
        assert!(cmd.artifact.is_none(), "reading an architecture produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "the architecture of the machine does not depend on `here`");
        assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
        assert!(cmd.warnings.is_empty(), "one bare word needs no caveat");
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
    fn every_token_is_explained_by_its_own_key() {
        // الأمر رمزان: اسم الأداة، ثم `-m`. لو خلا أحدهما من مفتاح شرحٍ لكانت
        // شاشة «سَطْر» ناقصةً لا مختصرةً في هذه العملية.
        let cmd = plan_it().unwrap();
        assert_eq!(cmd.explain.len(), 2);
        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert_eq!(cmd.explain[0].key, Some("explain.uname.tool"));
        assert_eq!(cmd.explain[1].role, TokenRole::Flag);
        assert_eq!(cmd.explain[1].key, Some("explain.uname.machine"));
        assert_eq!(cmd.explain[1].token, "-m");
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        for smuggled in ["field", "target", "-a", "verbose"] {
            let raw = BTreeMap::from([(smuggled.to_owned(), RawValue::Text("1".to_owned()))]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_cannot_reach_the_argv_because_there_is_nowhere_to_put_it() {
        let baseline = plan_it().unwrap();
        for shellish in ["; rm -rf ~", "$(whoami)", "`id`", "a & b", "> /etc/hosts"] {
            let raw = BTreeMap::from([("what".to_owned(), RawValue::Text(shellish.to_owned()))]);
            assert!(crate::value::validate(&SPEC, &raw).is_err(), "{shellish:?} was accepted");
        }
        assert_eq!(plan_it().unwrap().args, baseline.args);
        assert_eq!(baseline.args, vec!["-m"], "uname takes exactly the one declared flag");
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        assert_eq!(plan_it().unwrap(), plan_it().unwrap());
    }
}
