//! سجلّ آخر التسجيلات في مستودع Git، باستخدام `git log`.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> log --format=%H%x09%ad%x09%an%x09%s --date=short -n <العدد>
//! ```
//!
//! ## لماذا صيغةٌ مبنيّة لا المخرج المعتاد
//!
//! `git log` بلا رايات تكتب فقرةً لكل commit: البصمة كاملةً، والمؤلف
//! والبريد، والتاريخ بصيغة RFC 2822 الطويلة، ثم الرسالة بفراغٍ بادئ — أربعة
//! أسطر أو أكثر لكل سطرٍ واحدٍ من المعنى. هذا مخرجٌ يُقرأ في طرفيةٍ يُمرَّر
//! فيها إلى `less`، لا في نافذة خرجٍ نصّية لا تفاعل فيها. `--format` تختصر كل
//! commit إلى سطرٍ واحد بالحقول التي تُقرأ فعلًا: البصمة، والتاريخ، والمؤلف،
//! وعنوان الرسالة — بترتيبٍ ثابت يفصل بينه tab.
//!
//! ## رمزٌ واحد لا أربعة
//!
//! `--format=%H%x09%ad%x09%an%x09%s` رمزٌ ملتصقٌ واحد، على نظير `--format=zip`
//! في `git_archive.rs`: الراية وقيمتها كتلةٌ واحدة في `argv` وفي العرض معًا،
//! فلا يوجد موضعٌ بينهما يمكن أن ينزلق إليه وسيطٌ. و`%x09` هي صيغة `git`
//! الرسمية لمحرف tab داخل `--format` (انظر `git help log`، قسم PRETTY
//! FORMATS) — أوضح في الشيفرة المصدرية من تضمين محرف tab حرفي لا يُرى بالعين
//! في قراءة الملفّ.
//!
//! والفاصل tab لا فراغ: اسم المؤلف وعنوان الرسالة يحملان فراغاتٍ داخلية
//! بلا استثناء تقريبًا، وفاصلٌ يتكرّر داخل الحقل يفسد أي تقسيمٍ لاحقٍ للأعمدة.
//! ‏tab لا يظهر في هذه الحقول عمليًا، فيبقى العمود عمودًا.
//!
//! ## `--date=short` رايةٌ منفصلة
//!
//! `%ad` وحدها تطبع تاريخ المؤلف بصيغة `git` الافتراضية (RFC 2822 الطويلة).
//! `--date=short` راية عامّة في `git log` تُعدّل **معنى** `%ad` نفسها إلى
//! `YYYY-MM-DD` — لذلك هي راية قائمة بذاتها لا جزءٌ من نصّ `--format`، رغم أن
//! أثرها لا يظهر إلا فيه.
//!
//! ## لماذا عنوان الرسالة (`%s`) لا متنها الكامل (`%B`)
//!
//! `%s` سطرٌ واحد بحكم اتفاق `git` نفسه: أول سطرٍ من الرسالة، وهو ما يُعرض في
//! كل أداةٍ تلخّص تاريخًا (`git log --oneline`، وواجهات GitHub وGitLab). أمّا
//! المتن الكامل فقد يحمل فقراتٍ وأسطرًا فارغة، وإدراجه هنا يكسر الافتراض الذي
//! تقوم عليه هذه الصيغة: **سطرٌ واحدٌ لكل commit**. من أراد المتن الكامل
//! فمكانه محرّره أو الطرفية — تمامًا كما لا تعرض عملية «فرق» فرقًا كاملًا
//! (انظر `git_diff.rs`).
//!
//! ## `-n <العدد>`: رايةٌ ثم قيمتها، بلا مفتاح شرحٍ ثانٍ
//!
//! نظير `-c <العدد>` في `net_ping.rs` بالضبط: رايةٌ مشروحة تليها القيمة
//! الرقمية بلا شرحٍ منفصل — فرايتُها تقول ما هي، ومفتاحٌ ثانٍ للعدد نفسه كان
//! سيقول «هذا هو العدد الذي اخترته»، وهي جملةٌ تشغل سطرًا في الشرح ولا تضيف
//! معرفة. والمدى (١ إلى ٥٠٠) والقيمة الافتراضية (٥٠) معلَنان في `InputSpec`
//! ذاته، فتتحقّق منهما النواة قبل أن يصل التخطيط إلى هذا الملفّ.
//!
//! ## ولا مدخل غيرهما
//!
//! لا حقل فرعٍ ولا مؤلّف ولا مدى تواريخ. `git log` تقبل هذا كلّه، لكن كل
//! خيارٍ منها يعني وسيطًا إضافيًا في `argv` — وهذه العملية تُجيب سؤالًا واحدًا
//! بعينه: «ما آخر ما سُجِّل هنا؟» على الفرع الحالي (`HEAD`)، دون فرزٍ أو
//! تصفيةٍ إضافية. من أراد فرزًا يقرأ العدد الكافي من هذا السجلّ بعينه.
//!
//! ## ما لا تفعله هذه العملية
//!
//! لا تتصل بالشبكة ولا تجلب من بعيد: السجلّ هنا هو ما سُجِّل محليًا في مخزن
//! الكائنات، لا ما على الخادم البعيد. ولا تسأل هل يوجد التزامٌ واحد على
//! الأقل — مستودعٌ بلا تسجيلات يُخرج `git log` منه برمزٍ غير صفري، والوصف
//! يذكر ذلك بدل أن يُفحص هنا سلفًا (انظر فلسفة «ما لا تفحصه» في `git_commit.rs`).

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "git.log";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.log.title",
    description_key: "op.git.log.description",
    category: Category::Git,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("repo", InputKind::ExistingDir),
        InputSpec::new("limit", InputKind::Number { min: 1, max: 500, default: 50 }),
    ],
    sort_order: 70,
    search_terms: &[
        "git",
        "log",
        "history",
        "commits",
        "سجلّ",
        "تاريخ",
        "تسجيلات",
        "مستودع",
        "repo",
        "commit",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let repo = inputs.dir("repo")?;
    require_repo(repo, "repo")?;
    let limit = inputs.number("limit")?;

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(repo)
        .flag("log", "explain.git.log")
        .flag("--format=%H%x09%ad%x09%an%x09%s", "explain.git.log.format")
        .flag("--date=short", "explain.git.log.date")
        .flag("-n", "explain.git.log.limit")
        .value(limit.to_string())
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
/// مستودعًا سليمًا. والجذرُ وحده مقبول: `git -C <فرعي> log` تصعد إلى الجذر
/// وتسرد تسجيلات المستودع كله بينما تقول الشاشة اسم المجلد الفرعي.
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

    /// `limit` هو `Number` **مطلوب**، لا اختياري: `default` في `InputSpec`
    /// إرشادٌ للواجهة تملأ به الحقل أول مرّة، لا تعبئةً تلقائية في النواة. من
    /// هنا `None` تعني «استعمل القيمة الافتراضية ٥٠ صراحةً»، لا «اترك الحقل
    /// غائبًا» — إغفاله كليًا كان يرفض بـ`MissingInput`، لا يفترض ٥٠.
    fn plan_with(repo: &Path, limit: Option<i64>) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("repo".to_owned(), RawValue::Path(repo.display().to_string())),
            ("limit".to_owned(), RawValue::Text(limit.unwrap_or(50).to_string())),
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
        let found = listed.iter().find(|o| o.id == ID).expect("git.log must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form_with_the_default_limit() {
        let s = Scratch::new("git-log-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let cmd = plan_with(&repo, None).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 7);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "log");
        assert_eq!(args[3], "--format=%H%x09%ad%x09%an%x09%s");
        assert_eq!(args[4], "--date=short");
        assert_eq!(args[5], "-n");
        assert_eq!(args[6], "50", "the declared default in InputSpec");

        assert!(cmd.artifact.is_none());
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
        assert_eq!(cmd.reveal_target.as_deref(), Some(repo.as_path()));
    }

    #[test]
    fn the_chosen_limit_is_the_only_thing_the_form_changes() {
        let s = Scratch::new("git-log-limit").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        for limit in [1, 50, 500] {
            let args = args_of(&plan_with(&repo, Some(limit)).unwrap());
            assert_eq!(args.len(), 7, "no limit may add or drop an argument");
            assert_eq!(args[5], "-n");
            assert_eq!(args[6], limit.to_string());
        }
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-log-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, Some(10)).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-log-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, Some(10)).unwrap();
        // المسار والعدد بيانات المستخدم، وما عداهما رموزٌ اختارها التطبيق فلكلٍّ
        // منها مفتاح شرح.
        for token in cmd.explain.iter().filter(|t| t.role == TokenRole::Flag) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-log-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };

        let cmd = plan_with(&repo, Some(10)).unwrap();
        for (i, a) in cmd.args.iter().enumerate() {
            // المواضع ٠ و٢ و٣ و٤ و٥ رموزٌ اختارها التطبيق؛ ما عداها بيانات.
            if [0, 2, 3, 4, 5].contains(&i) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "argument {i} ({a:?}) would be read as a flag");
        }
    }

    #[test]
    fn shell_syntax_in_the_repository_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-log-shellish").unwrap();

        let shellish = ["مستودع 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"];
        for name in shellish {
            let Some(repo) = repo_in(&s, name) else { return };
            let cmd = plan_with(&repo, None).unwrap();
            assert_eq!(cmd.args.len(), 7, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), repo.as_path());
        }
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-log-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        assert_eq!(refusal(plan_with(&plain, None)), ("err.path.not_dir", Some("repo")));
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-log-missing").unwrap();
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
    fn a_limit_outside_the_declared_range_is_refused_by_the_core() {
        let s = Scratch::new("git-log-range").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for out in [0, 501, -1] {
            assert_eq!(refusal(plan_with(&repo, Some(out))), ("err.input.range", Some("limit")));
        }
    }

    #[test]
    fn a_symlinked_repository_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("git-log-symlink").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, None).unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_repository_raises_no_warning_at_all() {
        let s = Scratch::new("git-log-quiet").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        assert_eq!(plan_with(&repo, Some(10)).unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn planning_writes_nothing_into_the_repository() {
        let s = Scratch::new("git-log-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let before = s.names(&repo);

        for _ in 0..10 {
            plan_with(&repo, Some(10)).unwrap();
        }
        assert_eq!(s.names(&repo), before, "planning must not touch the repository");
    }
}
