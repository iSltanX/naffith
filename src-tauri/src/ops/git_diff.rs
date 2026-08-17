//! ملخّص التعديلات في مستودع Git، باستخدام `git diff --stat`.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> diff --stat --exit-code [--staged]
//! ```
//!
//! ## لماذا `--stat` لا الفرق الكامل
//!
//! الفرق الكامل مخرجٌ بطول التعديل نفسه: مئات الأسطر بعلامات `+` و`-` وسياقٍ
//! حولها. وهو مخرجٌ يُقرأ في مُصفِّحٍ يقفز بين الملفات ويلوّن، لا في نافذة خرجٍ
//! نصّية لا تفاعل فيها.
//!
//! `--stat` تعطي الجواب الذي يُسأل قبل التسجيل: أي الملفات تغيّرت، وكم سطرًا
//! في كلٍّ منها. ومن أراد أن يقرأ التعديل سطرًا سطرًا فمكانه محرّره أو الطرفية —
//! وهذا حدٌّ معلَن في الوصف لا مفاجأةٌ بعد التشغيل.
//!
//! ## `--staged` أم `--cached`
//!
//! الرايتان مترادفتان حرفيًا في `git`. اختيرت `--staged` لأنها اسمُ الحالة كما
//! يسمّيها Git نفسها اليوم في مخرجاته («‏Changes to be committed»، `git
//! restore --staged`)، بينما `--cached` بقيّةٌ من مفرداتٍ أقدم لا تعني للقارئ
//! شيئًا. الأثر واحد، والفرق في أن الشرح يبقى قابلًا للتصديق حين يقرأه المستخدم
//! في وثائق Git.
//!
//! ## ما الذي يقيسه كلٌّ من الوضعين
//!
//! * بلا `--staged`: ما بين **شجرة العمل** ومنطقة الإدراج — أي ما عدّلتَه ولم
//!   تُدرِجه.
//! * مع `--staged`: ما بين **منطقة الإدراج** وآخر commit — أي ما أدرجتَه ولم
//!   تسجّله.
//!
//! وهما مجموعتان منفصلتان لا واحدة أوسع من الأخرى: ملفٌ عُدّل ثم أُدرج ثم عُدّل
//! ثانيةً يظهر في الاثنتين، وبأرقامٍ مختلفة. لهذا الوضعان اختيارٌ صريح لا
//! تخمينًا من حال المستودع.
//!
//! ## وما لا يُظهره الوضعان معًا
//!
//! الملفات الجديدة غير المتتبَّعة. `git diff` تقارن ما يعرفه Git، وملفٌ لم
//! يُضَف قط ليس طرفًا في أي مقارنة — لا يظهر هنا ولا في `--staged`. تلك حالةٌ
//! تكشفها عملية «حالة المستودع».

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "git.diff";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.diff.title",
    description_key: "op.git.diff.description",
    category: Category::Git,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("repo", InputKind::ExistingDir),
        // اختياريّة: الوضع الغالب هو النظر فيما لم يُدرَج بعد، والحقل المطلوب
        // الوحيد هو المستودع — فالنموذج لا يكتمل بلا اختيارٍ حقيقي من المستخدم.
        InputSpec::new("staged", InputKind::Flag).optional(),
    ],
    sort_order: 40,
    search_terms: &[
        "git",
        "diff",
        "فرق",
        "تعديلات",
        "changes",
        "مقارنة",
        "staged",
        "مدرج",
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
        .flag("diff", "explain.git.diff")
        .flag("--stat", "explain.git.diff.stat")
        // لا نقرأ نص الخرج لنستنتج الدلالة. Git تعرّف 0 = لا فروق و1 = توجد
        // فروق حين تُطلب هذه الراية، والنواة تحوّل الرمزين إلى جوابين ناجحين.
        .flag("--exit-code", "explain.git.diff.exit_code");

    // الراية تُضاف أو لا تُضاف؛ ولا صيغة هنا تجعل قيمةً من المستخدم تصير رايةً.
    // مدخل `Flag` منطقيّ لا نصّي (انظر `RawValue` في `value.rs`)، فأقصى ما
    // تستطيع الواجهة قوله «نعم» أو «لا» على رايةٍ مكتوبةٍ هنا حرفيًا.
    if inputs.flag("staged") {
        argv = argv.flag("--staged", "explain.git.diff.staged");
    }

    argv = argv.reveal(repo);

    if let Some(key) = warn_if_resolved(inputs, "repo", repo, "warn.git.repo.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

/// يتحقّق أن المجلد المختار **جذرُ** مستودع، وينسب الرفض إلى حقله.
///
/// `symlink_metadata` لا `is_dir`: في شجرة عملٍ إضافية وفي الوحدات الفرعية
/// يكون `.git` ملفًا نصّيًا يشير إلى موضع المستودع، فشرطُ «مجلد» كان سيرفض
/// مستودعًا سليمًا. والجذرُ وحده مقبول: `git -C <فرعي> diff` تصعد إلى الجذر
/// وتلخّص تعديلات المستودع كله بينما تقول الشاشة اسم المجلد الفرعي.
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

    fn plan_with(repo: &Path, staged: Option<bool>) -> Result<PlannedCommand> {
        let mut raw =
            BTreeMap::from([("repo".to_owned(), RawValue::Path(repo.display().to_string()))]);
        if let Some(on) = staged {
            raw.insert("staged".to_owned(), RawValue::Flag(on));
        }
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
        let found = listed.iter().find(|o| o.id == ID).expect("git.diff must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form_when_the_switch_is_left_alone() {
        let s = Scratch::new("git-diff-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let cmd = plan_with(&repo, None).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 5);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "diff");
        assert_eq!(args[3], "--stat");
        assert_eq!(args[4], "--exit-code");

        assert!(cmd.artifact.is_none());
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
        assert_eq!(cmd.reveal_target.as_deref(), Some(repo.as_path()));
    }

    #[test]
    fn the_switch_adds_exactly_one_flag_and_only_when_it_is_on() {
        let s = Scratch::new("git-diff-staged").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let on = args_of(&plan_with(&repo, Some(true)).unwrap());
        assert_eq!(
            on,
            vec!["-C", repo.to_str().unwrap(), "diff", "--stat", "--exit-code", "--staged"]
        );

        // «لا» صريحةٌ من الواجهة تساوي غيابَ الحقل تمامًا: لا راية تُضاف، ولا
        // رايةُ نفيٍ تُخترع.
        let off = args_of(&plan_with(&repo, Some(false)).unwrap());
        assert_eq!(off, args_of(&plan_with(&repo, None).unwrap()));
        assert_eq!(off.len(), 5);
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-diff-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        for staged in [None, Some(true)] {
            let cmd = plan_with(&repo, staged).unwrap();
            let mut expected = vec![cmd.program.display().to_string()];
            expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
            let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
            assert_eq!(shown, expected, "with staged = {staged:?}");
        }
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-diff-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, Some(true)).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role != TokenRole::Path) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-diff-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };

        let cmd = plan_with(&repo, Some(true)).unwrap();
        let chosen = &cmd.args[1];
        assert!(cannot_be_read_as_a_flag(chosen), "{chosen:?} would be read as a flag");
    }

    #[test]
    fn shell_syntax_in_the_repository_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-diff-shellish").unwrap();

        let shellish = ["مستودع 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"];
        for name in shellish {
            let Some(repo) = repo_in(&s, name) else { return };
            let cmd = plan_with(&repo, None).unwrap();
            assert_eq!(cmd.args.len(), 5, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), repo.as_path());
        }
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-diff-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        assert_eq!(refusal(plan_with(&plain, None)), ("err.path.not_dir", Some("repo")));
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-diff-missing").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"), None)),
            ("err.path.missing", Some("repo"))
        );
    }

    #[test]
    fn a_repository_outside_the_allowed_roots_is_refused() {
        assert_eq!(refusal(plan_with(Path::new("/etc"), None)), ("err.path.outside", Some("repo")));
    }

    #[test]
    fn a_relative_repository_path_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("نسبي/هنا"), None)),
            ("err.path.relative", Some("repo"))
        );
    }

    #[test]
    fn a_switch_sent_as_text_instead_of_a_boolean_is_refused_not_coerced() {
        // الواجهة لا تستطيع تهريب نصٍّ في موضع راية: النوع يُفحص قبل التخطيط.
        let s = Scratch::new("git-diff-type").unwrap();
        let dir = s.dir("م");
        let raw = BTreeMap::from([
            ("repo".to_owned(), RawValue::Path(dir.display().to_string())),
            ("staged".to_owned(), RawValue::Text("--exec=rm".to_owned())),
        ]);
        let r = crate::value::validate(&SPEC, &raw).map(|_| ());
        assert!(matches!(r, Err(CoreError::WrongInputType { id: "staged" })), "got {r:?}");
    }

    #[test]
    fn a_symlinked_repository_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("git-diff-symlink").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, None).unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_repository_raises_no_warning_at_all() {
        let s = Scratch::new("git-diff-quiet").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        assert_eq!(plan_with(&repo, Some(true)).unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn planning_writes_nothing_into_the_repository() {
        let s = Scratch::new("git-diff-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let before = s.names(&repo);

        for _ in 0..10 {
            plan_with(&repo, Some(true)).unwrap();
        }
        assert_eq!(s.names(&repo), before, "planning must not touch the repository");
    }
}
