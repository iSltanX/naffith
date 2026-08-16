//! سرد الفروع المحلية المدموجة، باستخدام `git branch --merged`.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> branch --merged
//! ```
//!
//! ## الحذف غير مُنفَّذ عمدًا
//!
//! المطلوب في خارطة الطريق كان «تنظيف الفروع المدموجة». وهذه العملية **تسرد
//! المرشّحين ولا تحذف**، لثلاثة أسباب لا واحد:
//!
//! 1. **الحذف ليس أمرًا واحدًا.** `git branch -d` تقبل الفروع وسائطَ، فحذفُ ما
//!    تسرده هذه العملية يعني: اقرأ المخرج، حلّله، ابنِ أمرًا ثانيًا من سطوره.
//!    أي أن نصًّا خرج من أداةٍ يصير وسائط أمرٍ تالٍ — وهو بالضبط الطريق الذي
//!    أُغلق في هذه المعمارية (`RawValue` لا تعبّر عن وسيط، و`argv` تُبنى في
//!    شيفرة مترجَمة).
//! 2. **«مدموج» ليست حقيقةً مطلقة.** `--merged` تعني «مدموجٌ في HEAD الحالي»،
//!    لا في `main`. فالقائمة نفسها تتغيّر بتغيّر الفرع الذي أنت عليه — ومن
//!    يحذف بناءً عليها وهو على فرعٍ قديم يحذف عملًا لم يصل إلى شيء.
//! 3. **التراجع ليس مضمونًا.** فرعٌ محذوف تبقى لقطاته في مخزن الكائنات مدّةً،
//!    ويُستعاد بـ`git reflog` — لمن يعرف. ومن لا يعرف يكون قد فقد المؤشّر
//!    الوحيد إلى عمله. عمليةٌ بهذا الأثر تحتاج تأكيدًا يعرض الأسماء واحدًا واحدًا
//!    وخطةً متعدّدة الخطوات، وهو ما لا تصوغه هذه النواة اليوم.
//!
//! فعملية «حذف الفروع المدموجة» مسجَّلة في خارطة الطريق باسمها، ومشروطةٌ بنموذج
//! خطةٍ متعدّدة الخطوات. وهذه العملية تُجهّز قرارها: تريك القائمة كي تحذف بيدك
//! ما تريد، عارفًا في أي فرعٍ أنت.
//!
//! ## ولماذا القائمة المحلية وحدها
//!
//! لا `-r` ولا `-a`: الفروع البعيدة لا تُحذف من هنا أصلًا، وسردُها يضاعف طول
//! المخرج بأسماء `origin/…` التي لا يستطيع القارئ أن يفعل بها شيئًا في هذه
//! الشاشة. والمخرج يعلّم الفرع الحالي بنجمة، وهو ما يجعل التحذير أعلاه مقروءًا
//! في المخرج نفسه لا في كلامٍ عنه.

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "git.branches.merged";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.branches.merged.title",
    description_key: "op.git.branches.merged.description",
    category: Category::Git,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("repo", InputKind::ExistingDir)],
    sort_order: 50,
    search_terms: &[
        "git",
        "branch",
        "فروع",
        "فرع",
        "merged",
        "مدموج",
        "تنظيف",
        "cleanup",
        "مستودع",
        "repo",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let repo = inputs.dir("repo")?;
    require_repo(repo, "repo")?;

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(repo)
        .flag("branch", "explain.git.branch")
        .flag("--merged", "explain.git.branch.merged")
        // القائمة تُقرأ نسبةً إلى الفرع الحالي، ومن لا يعرف ذلك يقرأها خطأً.
        .warn("warn.git.merged.head")
        .reveal(repo);

    if let Some(key) = warn_if_resolved(inputs, "repo", repo, "warn.git.repo.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

/// يتحقّق أن المجلد المختار **جذرُ** مستودع، وينسب الرفض إلى حقله.
///
/// `symlink_metadata` لا `is_dir`: في شجرة عملٍ إضافية وفي الوحدات الفرعية
/// يكون `.git` ملفًا نصّيًا يشير إلى موضع المستودع، فشرطُ «مجلد» كان سيرفض
/// مستودعًا سليمًا. والجذرُ وحده مقبول: `git` تصعد من المجلد الفرعي إلى الجذر
/// وتجيب عن المستودع كله بينما تقول الشاشة اسم المجلد الفرعي.
fn require_repo(repo: &Path, field: &'static str) -> Result<()> {
    if std::fs::symlink_metadata(repo.join(".git")).is_err() {
        return Err(CoreError::NotADirectory.on_input(field));
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
    use std::path::PathBuf;

    fn plan_with(repo: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([("repo".to_owned(), RawValue::Path(repo.display().to_string()))]);
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

    /// مستودعٌ حقيقي، أو `None` إن كانت `git` غائبة. انظر `git_status.rs`
    /// لتفصيل لماذا لا يكفي وجود الملف.
    fn repo_in(s: &Scratch, name: &str) -> Option<PathBuf> {
        let dir = s.dir(name);
        let ok = std::process::Command::new("/usr/bin/git")
            .arg("-C")
            .arg(&dir)
            .arg("init")
            .arg("--quiet")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
        ok.then_some(dir)
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("git.branches.merged must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let s = Scratch::new("git-branch-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let cmd = plan_with(&repo).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 4);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "branch");
        assert_eq!(args[3], "--merged");

        assert!(cmd.artifact.is_none());
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
        assert_eq!(cmd.reveal_target.as_deref(), Some(repo.as_path()));
    }

    /// الادّعاء المكتوب في الوثيقة أعلاه، مُختبَرًا لا موعودًا.
    ///
    /// رايات الحذف هي `-d` و`-D`. لو دخل واحدٌ منها الأمر يومًا لسقط هذا
    /// الاختبار — وسقوطه هنا أرخص بكثير من اكتشافه على فروع مستخدمٍ حقيقي.
    #[test]
    fn no_deleting_flag_can_ever_enter_this_command() {
        let s = Scratch::new("git-branch-readonly").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let args = args_of(&plan_with(&repo).unwrap());
        for deleting in ["-d", "-D", "--delete", "--force"] {
            assert!(!args.iter().any(|a| a == deleting), "{deleting} must not be in the argv");
        }
        assert_eq!(args.len(), 4, "the listing form is the only form this operation can take");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-branch-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-branch-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role != TokenRole::Path) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn the_meaning_of_merged_is_announced_in_every_plan() {
        let s = Scratch::new("git-branch-warn").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo).unwrap();
        assert!(cmd.warnings.contains(&"warn.git.merged.head"), "{:?}", cmd.warnings);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-branch-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };

        let cmd = plan_with(&repo).unwrap();
        let chosen = &cmd.args[1];
        assert!(cannot_be_read_as_a_flag(chosen), "{chosen:?} would be read as a flag");
    }

    #[test]
    fn shell_syntax_in_the_repository_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-branch-shellish").unwrap();

        let shellish = ["مستودع 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"];
        for name in shellish {
            let Some(repo) = repo_in(&s, name) else { return };
            let cmd = plan_with(&repo).unwrap();
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), repo.as_path());
        }
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-branch-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        assert_eq!(refusal(plan_with(&plain)), ("err.path.not_dir", Some("repo")));
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-branch-missing").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"))),
            ("err.path.missing", Some("repo"))
        );
    }

    #[test]
    fn a_repository_outside_the_allowed_roots_is_refused() {
        assert_eq!(refusal(plan_with(Path::new("/etc"))), ("err.path.outside", Some("repo")));
    }

    #[test]
    fn a_relative_repository_path_is_refused() {
        assert_eq!(refusal(plan_with(Path::new("نسبي/هنا"))), ("err.path.relative", Some("repo")));
    }

    #[test]
    fn a_symlinked_repository_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("git-branch-symlink").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_writes_nothing_into_the_repository() {
        let s = Scratch::new("git-branch-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let before = s.names(&repo);

        for _ in 0..10 {
            plan_with(&repo).unwrap();
        }
        assert_eq!(s.names(&repo), before, "planning must not touch the repository");
    }
}
