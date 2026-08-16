//! حالة مستودع Git، باستخدام `git status`.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> status --short --branch
//! ```
//!
//! ## لماذا `--short` لا المخرج المعتاد
//!
//! `git status` بلا رايات تكتب فقراتٍ إرشادية: «استعمل `git restore` للتراجع»،
//! «استعمل `git add` للإدراج»، ثم قائمة الملفات مبعثرةً بينها. المخرج المختصر
//! سطرٌ لكل ملف: حرفان يقولان حاله في منطقة الإدراج وفي شجرة العمل، ثم اسمه. وهذا
//! ما يُقرأ في نافذةِ خرجٍ لا تفاعل فيها — النصائح تفترض طرفيةً يكتب فيها
//! المستخدم أمره التالي، وهذا التطبيق لا يعطيه واحدة.
//!
//! ## ولماذا `--branch` معها
//!
//! `--short` وحدها **تحذف اسم الفرع**. ومستخدمٌ يقرأ قائمة تعديلاتٍ لا يعرف
//! على أي فرعٍ هي يقرأ نصف الجواب: نفس القائمة تعني شيئًا على `main` وشيئًا
//! آخر على فرعٍ يظنّ أنه غادره. الراية تعيد السطر الأول: الفرع، وتقدّمه أو
//! تأخّره عن نظيره البعيد إن وُجد.
//!
//! ## قراءةٌ محضة
//!
//! لا رايةَ كتابةٍ في هذا الأمر، ولا مدخل في هذه العملية يمكن أن يصير راية:
//! المدخل الوحيد مسار، و`RawValue` لا تملك صيغةً تعبّر عن راية أصلًا.
//! و`git status` نفسها قد تكتب ملفّ الفهرس (`.git/index`) لتحديث ذاكرة
//! `stat` — وهو تحديثٌ لا يمسّ محتوى المستودع ولا شجرة العمل، ولذلك تبقى
//! العملية `Safe`.
//!
//! ## ما لا تفعله
//!
//! لا تتصل بالشبكة. «متأخّر بثلاثة» تُحسب من آخر جلبٍ فعله المستخدم بنفسه، لا
//! من سؤال الخادم الآن — فمستودعٌ لم يُجلب منذ أسبوع يقول «محدَّث» وهو ليس كذلك.

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "git.status";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.status.title",
    description_key: "op.git.status.description",
    category: Category::Git,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("repo", InputKind::ExistingDir)],
    sort_order: 20,
    search_terms: &[
        "git",
        "status",
        "حالة",
        "مستودع",
        "repo",
        "تعديلات",
        "changes",
        "فرع",
        "branch",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let repo = inputs.dir("repo")?;
    require_repo(repo, "repo")?;

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(repo)
        .flag("status", "explain.git.status")
        .flag("--short", "explain.git.status.short")
        .flag("--branch", "explain.git.status.branch")
        .reveal(repo);

    if let Some(key) = warn_if_resolved(inputs, "repo", repo, "warn.git.repo.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

/// يتحقّق أن المجلد المختار **جذرُ** مستودع، وينسب الرفض إلى حقله.
///
/// مكتوبةٌ في كل ملفٍ يحتاجها لا في وحدةٍ مشتركة: ثلاثة أسطر مكرّرة أرخص من
/// وحدةٍ تربط ست عمليات ببعضها، وتُقرأ في موضعها بلا قفزٍ إلى ملفٍ آخر.
///
/// ثلاث قرارات فيها:
///
/// 1. **`symlink_metadata` لا `is_dir`.** في شجرة عملٍ إضافية (`git worktree`)
///    وفي الوحدات الفرعية يكون `.git` **ملفًا** نصّيًا يشير إلى موضع المستودع.
///    شرطُ «مجلد» كان سيرفض مستودعًا سليمًا يعمل عليه صاحبه كل يوم.
/// 2. **الجذر وحده.** `git` نفسها تقبل مجلدًا فرعيًا وتصعد إلى الجذر، لكن
///    مخرجها عندها عن **المستودع كله** بينما تقول الشاشة اسم المجلد الفرعي.
///    الرفض هنا يجعل ما يُعرض هو ما يُقرأ. والثمن معلَن: من يختار مجلدًا فرعيًا
///    يُطلب منه أن يصعد بنفسه.
/// 3. **`NotADirectory` لا `PathMissing`.** المجلد موجودٌ فعلًا — مرّ بسياسة
///    المسارات كاملة — وما ينقصه أنه ليس من النوع المطلوب. و«لا يوجد شيء في
///    هذا المسار» كانت ستُرسل المستخدم يبحث عن مجلدٍ يراه أمامه.
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

    /// مستودعٌ حقيقي داخل مساحة الاختبار، أو `None` إن كانت `git` غائبة.
    ///
    /// الفخّ هنا أن `/usr/bin/git` **موجود** على جهازٍ بلا أدوات Xcode: هو وسيطٌ
    /// يطلب تثبيتها ثم يفشل. فالسؤال ليس «أيوجد الملف؟» بل «أينجح `git init`؟».
    /// والتشغيل هنا تجهيزُ اختبارٍ لا سلوكُ منتج: شيفرة الإنتاج لا تشغّل شيئًا
    /// بنفسها، بل تبني أمرًا يشغّله `executor`.
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

    /// مجلدٌ يحمل علامة مستودعٍ بلا تشغيل `git`. لاختبارات الرفض التي تقع قبل
    /// حلّ الأداة، فتمرّ على كل جهاز.
    fn marked_repo(s: &Scratch, name: &str) -> PathBuf {
        let dir = s.dir(name);
        s.dir(&format!("{name}/.git"));
        dir
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("git.status must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let s = Scratch::new("git-status-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let cmd = plan_with(&repo).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 5);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "status");
        assert_eq!(args[3], "--short");
        assert_eq!(args[4], "--branch");

        assert!(cmd.artifact.is_none());
        assert!(cmd.stdout_to.is_none());
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
        assert_eq!(cmd.reveal_target.as_deref(), Some(repo.as_path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-status-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-status-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role != TokenRole::Path) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn the_short_form_never_loses_the_branch_line() {
        let s = Scratch::new("git-status-branch").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let args = args_of(&plan_with(&repo).unwrap());
        assert!(args.contains(&"--short".to_owned()));
        assert!(
            args.contains(&"--branch".to_owned()),
            "a file list with no branch name is half an answer"
        );
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-status-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };

        let cmd = plan_with(&repo).unwrap();
        let chosen = &cmd.args[1];
        assert!(cannot_be_read_as_a_flag(chosen), "{chosen:?} would be read as a flag");
    }

    #[test]
    fn shell_syntax_in_the_repository_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-status-shellish").unwrap();

        let shellish = ["مستودع 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"];
        for name in shellish {
            let Some(repo) = repo_in(&s, name) else { return };
            let cmd = plan_with(&repo).unwrap();
            assert_eq!(cmd.args.len(), 5, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), repo.as_path());
        }
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-status-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        assert_eq!(refusal(plan_with(&plain)), ("err.path.not_dir", Some("repo")));
    }

    #[test]
    fn a_subfolder_of_a_repository_is_refused_rather_than_answered_about_its_parent() {
        // `git -C <فرعي> status` تصعد إلى الجذر وتجيب عن المستودع كله، بينما
        // تقول الشاشة اسم المجلد الفرعي. الرفض يبقي المعروض هو المقروء.
        let s = Scratch::new("git-status-subdir").unwrap();
        marked_repo(&s, "مستودع");
        let inner = s.dir("مستودع/src");

        assert_eq!(refusal(plan_with(&inner)), ("err.path.not_dir", Some("repo")));
    }

    #[test]
    fn a_worktree_whose_git_is_a_file_is_accepted_as_a_repository() {
        let s = Scratch::new("git-status-worktree").unwrap();
        let dir = s.dir("شجرة");
        std::fs::write(dir.join(".git"), b"gitdir: /somewhere/else").unwrap();

        // الأداة قد تغيب، فيكون الرفض الوحيد الممكن رفضَ الأداة لا رفضَ المجلد.
        match plan_with(&dir) {
            Ok(cmd) => assert_eq!(Path::new(&cmd.args[1]), dir.as_path()),
            Err(e) => assert_eq!(e.key(), "err.tool.missing", "the folder itself must be accepted"),
        }
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-status-missing").unwrap();
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
        let s = Scratch::new("git-status-symlink").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_repository_raises_no_warning_at_all() {
        let s = Scratch::new("git-status-quiet").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        assert_eq!(plan_with(&repo).unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn planning_writes_nothing_into_the_repository() {
        let s = Scratch::new("git-status-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let before = s.names(&repo);

        for _ in 0..10 {
            plan_with(&repo).unwrap();
        }
        assert_eq!(s.names(&repo), before, "planning must not touch the repository");
    }
}
