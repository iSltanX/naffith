//! فتح مجلد في Finder، باستخدام `open`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/open <المجلد>
//! ```
//!
//! ## العملية الوحيدة التي كلّ أثرها تسليمُ مسارٍ إلى Finder
//!
//! لا تكتب شيئًا، ولا تقرأ محتوى، ولا تُنتج نصًّا على الخرج. تخرج بصفر بعد أن
//! تطلب من `LaunchServices` إظهار نافذة. وهذا يجعلها الحدّ الأدنى المفيد الذي
//! تُقاس عليه بقيّة العمليات: لو لم يظهر أمرُها في «سَطْر» كما يظهر أمرُ الضغط
//! لكان معنى ذلك أن الشاشة تعرض بعض ما يُنفَّذ لا كلَّه.
//!
//! ## ومع ذلك تمرّ بسياسة المسارات كاملةً
//!
//! هذا ليس تشدّدًا بلا سبب. المسار يُحلّ من روابطه، ويُفحص أنه تحت جذرٍ مسموح
//! (‏المنزل أو قرصٌ مركَّب)، ويُرفض إن كان في موضعٍ محميّ — فـ`~/.ssh` لا
//! تُفتح من هنا. السبب أن فتح نافذةٍ ليس فعلًا محايدًا: النافذة تعرض أسماء ما
//! في المجلد، وقد تولّد Finder معايناتٍ لمحتوياته. وسياسةٌ تُطبَّق على
//! العمليات الثقيلة وتُرفع عن «الخفيفة» ليست سياسة.
//!
//! ## `open` هنا وفي `reveal.rs` — أداةٌ واحدة بدورين
//!
//! `reveal.rs` يستعمل الأداة نفسها لإظهار ناتج تشغيلٍ سابق، والفرق في مصدر
//! المسار لا في الأمر: هناك المسار **تُخرجه النواة** من سجلّها، وهنا
//! **يختاره المستخدم** فيدخل من باب `InputKind::ExistingDir` ويمرّ بكل ما
//! يمرّ به أي مسارٍ آخر في هذا المنتج.
//!
//! ## ولماذا بلا `--` وبلا رايات
//!
//! `open -a` تشغّل تطبيقًا بعينه، و`-e` تفتح في محرّر، و`-R` تُظهر العنصر داخل
//! مجلده. ولا واحدة منها مطلوبة لـ«افتح هذا المجلد»، وكلٌّ منها كانت ستضيف
//! سؤالًا إلى نموذجٍ سؤاله واحد. والمسار مطلق فيبدأ بـ`/`، فلا يمكن أن يُقرأ
//! راية، و`--` لا يضيف هنا شيئًا.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "files.open";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.open.title",
    description_key: "op.files.open.description",
    category: Category::Files,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::OPEN,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("folder", InputKind::ExistingDir)],
    sort_order: 80,
    search_terms: &["open", "فتح", "finder", "إظهار", "reveal", "افتح المجلد", "مجلد"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let folder = inputs.dir("folder")?;

    let mut argv = Argv::tool(tools::OPEN, "explain.open.tool").path(folder).reveal(folder);

    if let Some(key) = warn_if_resolved(inputs, "folder", folder, "warn.source.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn raw(folder: &Path) -> BTreeMap<String, RawValue> {
        BTreeMap::from([("folder".to_owned(), RawValue::Path(folder.display().to_string()))])
    }

    fn plan_with(folder: &Path) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(folder))?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("files.open must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_path_and_nothing_else() {
        let s = Scratch::new("open-argv").unwrap();
        let folder = s.dir("المجلد");

        let cmd = plan_with(&folder).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/open"));
        assert_eq!(Path::new(&args[0]), folder.as_path());
        assert_eq!(args.len(), 1, "no flags, no separator, no second path");

        assert!(cmd.artifact.is_none(), "open writes nothing");
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.estimate.is_none(), "nothing is measured; nothing is copied");
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
    }

    #[test]
    fn no_launcher_flag_is_smuggled_in() {
        // ‏`-a` تشغّل تطبيقًا بعينه. غيابها ليس سهوًا، فيُثبَّت باختبار.
        let s = Scratch::new("open-noflags").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder).unwrap();
        for forbidden in ["-a", "-e", "-t", "-R", "--args", "--"] {
            assert!(!cmd.args.iter().any(|x| x == forbidden), "{forbidden} must not appear");
        }
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("open-explain").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("open-dashes").unwrap();
        let folder = s.dir("-rf");

        let cmd = plan_with(&folder).unwrap();
        for a in cmd.args.iter() {
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn a_protected_location_is_refused_even_though_nothing_would_be_written() {
        // السياسة لا تُرفع عن العمليات «الخفيفة»: النافذة تعرض ما في المجلد.
        //
        // الاختبار يتحقّق من مقدّماته قبل أن يدّعي شيئًا: جهازٌ بلا `~/.ssh`
        // لا يقول عن هذه القاعدة شيئًا، وجهازٌ يضعها خارج المنزل برابطٍ رمزي
        // يُرفض بقاعدةٍ أخرى (‏«خارج الجذور المسموحة»‏) فيقيس غير المقصود.
        let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else { return };
        let Ok(home) = home.canonicalize() else { return };
        let ssh = home.join(".ssh");
        let Ok(resolved) = ssh.canonicalize() else { return };
        if !resolved.is_dir() || !resolved.starts_with(&home) {
            return;
        }
        assert_eq!(refusal(plan_with(&ssh)), ("err.path.protected", Some("folder")));
    }

    #[test]
    fn a_location_outside_the_allowed_roots_is_refused() {
        // ‏`/etc` موجود ومقروء، ورفضه سياسة لا حادث.
        assert_eq!(refusal(plan_with(Path::new("/etc"))), ("err.path.outside", Some("folder")));
    }

    #[test]
    fn a_file_is_not_a_folder_and_is_refused_as_such() {
        let s = Scratch::new("open-file").unwrap();
        let file = s.file("مستند.txt", b"data");
        assert_eq!(refusal(plan_with(&file)), ("err.path.not_dir", Some("folder")));
    }

    #[test]
    fn shell_syntax_in_the_folder_name_stays_literal_and_adds_no_argument() {
        let s = Scratch::new("open-shellish").unwrap();

        for name in ["مجلد 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let folder = s.dir(name);
            let cmd = plan_with(&folder).unwrap();
            assert_eq!(Path::new(&cmd.args[0]), folder.as_path());
            assert_eq!(cmd.args.len(), 1, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_symlinked_folder_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("open-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(&cmd.args[0]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_the_folder_untouched() {
        let s = Scratch::new("open-clean").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        for _ in 0..10 {
            plan_with(&folder).unwrap();
        }
        assert_eq!(s.names(&folder), vec!["ملف".to_string()]);
    }
}
