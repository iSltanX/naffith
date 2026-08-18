//! البحث عن نصٍّ داخل الملفات **المتتبَّعة** في مستودع Git، باستخدام `git grep`.
//!
//! ## بديلٌ متعمَّد لا نسخةٌ مكرَّرة من «بحث في نص»
//!
//! هذه العملية أُضيفت بعد نقاشٍ صريح مع صاحب المنتج، لا كنسخةٍ ثانية باسمٍ آخر
//! من `text.search`. الفرق حقيقي في القدرة لا في الاسم: `text.search` تبحث
//! بـ`grep` العادية في **كل** ملفّات المجلد الذي يختاره المستخدم — بما فيها
//! `node_modules`، ومجلد `.git` نفسه، ونواتج البناء في `target`، وكل ما
//! يتجاهله `.gitignore` عمدًا لأنه لا يريده أحد في نتائج بحثه. و`git grep` هنا
//! تبحث **فقط في الملفّات التي يتتبّعها Git**: لا حاجة لتجاهلٍ يدوي، ولا بطء
//! بحثٍ في مجلداتٍ ضخمة لم يقصدها المستخدم أصلًا. من يعمل داخل مستودع ويبحث عن
//! نصٍّ في شيفرته يريد هذه القائمة المصفّاة سلفًا، لا التي تشمل كل شيء على
//! القرص.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> grep -n -F [-i] -e <النمط>
//! ```
//!
//! ## `-F` و`-e` وحدود ما يقدر عليه البانِي
//!
//! السبب الأمني والمنطقي لاختيار `-F` هنا مطابقٌ تمامًا لما شُرح في رأس
//! `text_search.rs` (catastrophic backtracking، وتفسير النمط تعبيرًا نمطيًّا لا
//! نصًّا حرفيًّا)، ولا يُعاد شرحه هنا. وكذلك حدود `-e` أمام نمطٍ يبدأ بشرطة:
//! `Argv::value`/`explained_value` ترفض كل قيمةٍ تبدأ بشرطة بلا معرفة أنها تلي
//! `-e`، فالرفض هنا يقع في `validate_pattern` قبل أن يصل الوسيط إلى البانِي —
//! نفس القرار، ونفس السبب.
//!
//! ## النمط: لا فراغ، ولا محارف تحكّم، ولا تشذيب
//!
//! نفس منطق `text_search.rs` بحرفه: نمطٌ فارغ يُطابق كل سطرٍ في كل ملفٍّ متتبَّع
//! فيتحوّل البحث إلى سردٍ للشجرة كلها، فيُرفض. ومحارف التحكّم تُرفض لأن سطرًا
//! جديدًا داخل النمط يعني نمطين لا نمطًا واحدًا عند `grep` نفسها، ويعني في
//! «سَطْر» أمرًا يُعرض على سطرين — فيُقرأ غير ما يُنفَّذ. ولا يُشذَّب النمط: الفراغ
//! في طرفيه جزءٌ مما يبحث عنه المستخدم.
//!
//! ## قراءةٌ محضة
//!
//! لا رايةَ كتابةٍ في هذا الأمر. `git grep` تقرأ من الشجرة المتتبَّعة ولا تكتب
//! شيئًا، فلا ناتج ولا مؤقّت — و«إظهار في Finder» يفتح المستودع نفسه.
//!
//! ## ما لا تفحصه هذه العملية
//!
//! لا تسأل هل في المستودع commit واحد على الأقل. مستودعٌ حديث الإنشاء بلا
//! تسجيلٍ بعد لا يملك شجرةً يقرأها `git grep`، فيخرج الأمر برمزٍ غير صفري —
//! والتخطيط لا يشغّل شيئًا كي يعرف ذلك سلفًا.

use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "git.grep";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.grep.title",
    description_key: "op.git.grep.description",
    category: Category::Git,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("repo", InputKind::ExistingDir),
        // ٢٠٠ محرفًا: نفس حدّ `text.search` لنفس السبب — أطول من أي نصٍّ يُبحث
        // عنه فعلًا، وأقصر من أن يصير حمولة في سطرٍ يُعرض ويُقيَّد.
        InputSpec::new("pattern", InputKind::Text { max_len: 200 }),
        InputSpec::new("ignore_case", InputKind::Flag).optional(),
    ],
    sort_order: 110,
    search_terms: &[
        "git",
        "grep",
        "search",
        "code",
        "بحث",
        "كود",
        "شفرة",
        "داخل",
        "متتبَّع",
        "مستودع",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let repo = inputs.dir("repo")?;
    require_repo(repo, "repo")?;
    let pattern = inputs.text("pattern")?;
    let ignore_case = inputs.flag("ignore_case");

    validate_pattern(pattern)?;

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(repo)
        .flag("grep", "explain.git.grep")
        .flag("-n", "explain.git.grep.line_numbers")
        .flag("-F", "explain.git.grep.fixed_string");

    // الراية الاختيارية بين `-F` و`-e`، بالضبط كما تقول صيغة الأمر أعلاه: ترتيبٌ
    // ثابت يجعل الأمر المعروض هو نفسه في كل تشغيل.
    if ignore_case {
        argv = argv.flag("-i", "explain.git.grep.ignore_case");
    }

    argv = argv
        .flag("-e", "explain.git.grep.pattern_flag")
        .explained_value(pattern, "explain.git.grep.pattern")
        .reveal(repo);

    if let Some(key) = warn_if_resolved(inputs, "repo", repo, "warn.git.repo.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

/// يفحص النمط قبل أن يصير وسيطًا. نفس فحص `text_search.rs::validate_pattern`
/// بحرفه — ثلاثة شروط، ولكلٍّ منها ما يمنعه بعينه.
fn validate_pattern(pattern: &str) -> Result<()> {
    if pattern.is_empty() {
        return Err(CoreError::InvalidName { reason: NameRejection::Empty }.on_input("pattern"));
    }
    if pattern.chars().any(|c| c.is_control()) {
        return Err(
            CoreError::InvalidName { reason: NameRejection::ContainsControl }.on_input("pattern")
        );
    }
    // يبنيه `-e` نظريًا، ويرفضه `Argv::explained_value` عمليًا. الرفض هنا كي
    // يُنسب إلى حقله ويُقرأ في مكانه، بدل أن يخرج من البانِي بلا حقلٍ يدلّ عليه.
    if pattern.starts_with('-') {
        return Err(CoreError::ArgumentLooksLikeFlag.on_input("pattern"));
    }
    Ok(())
}

/// يتحقّق أن المجلد المختار **جذرُ** مستودع، وينسب الرفض إلى حقله.
///
/// `symlink_metadata` لا `is_dir`: في شجرة عملٍ إضافية وفي الوحدات الفرعية
/// يكون `.git` ملفًا نصّيًا يشير إلى موضع المستودع، فشرطُ «مجلد» كان سيرفض
/// مستودعًا سليمًا. والجذرُ وحده مقبول: `git -C <فرعي> grep` تصعد إلى الجذر
/// وتبحث في المستودع كله بينما تقول الشاشة اسم المجلد الفرعي.
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

    fn plan_with(repo: &Path, pattern: &str, ignore_case: Option<bool>) -> Result<PlannedCommand> {
        let mut raw = BTreeMap::from([
            ("repo".to_owned(), RawValue::Path(repo.display().to_string())),
            ("pattern".to_owned(), RawValue::Text(pattern.to_owned())),
        ]);
        if let Some(on) = ignore_case {
            raw.insert("ignore_case".to_owned(), RawValue::Flag(on));
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
        let found = listed.iter().find(|o| o.id == ID).expect("git.grep must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form_when_the_switch_is_left_alone() {
        let s = Scratch::new("git-grep-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let cmd = plan_with(&repo, "كلمة", None).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 7);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "grep");
        assert_eq!(args[3], "-n");
        assert_eq!(args[4], "-F");
        assert_eq!(args[5], "-e");
        assert_eq!(args[6], "كلمة");

        assert!(cmd.artifact.is_none());
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
        assert_eq!(cmd.reveal_target.as_deref(), Some(repo.as_path()));
    }

    #[test]
    fn the_pattern_is_the_last_argument_and_carried_literally() {
        let s = Scratch::new("git-grep-pattern").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "كلمة", None).unwrap();
        let args = args_of(&cmd);
        assert_eq!(args.len(), 7);
        assert_eq!(args[6], "كلمة");
    }

    #[test]
    fn the_switch_adds_exactly_one_flag_between_fixed_string_and_the_pattern_flag() {
        let s = Scratch::new("git-grep-case").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let on = args_of(&plan_with(&repo, "Word", Some(true)).unwrap());
        assert_eq!(on, vec!["-C", repo.to_str().unwrap(), "grep", "-n", "-F", "-i", "-e", "Word"]);

        // «لا» صريحةٌ من الواجهة، وغياب الحقل: كلاهما لا يضيف رايةً ولا يخترع
        // رايةَ نفي.
        let off_explicit = args_of(&plan_with(&repo, "Word", Some(false)).unwrap());
        let off_absent = args_of(&plan_with(&repo, "Word", None).unwrap());
        assert_eq!(off_explicit, off_absent);
        assert!(!off_absent.iter().any(|a| a == "-i"), "{off_absent:?}");
        assert_eq!(off_absent.len(), 7);
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-grep-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        for ignore_case in [None, Some(true)] {
            let cmd = plan_with(&repo, "نص", ignore_case).unwrap();
            let mut expected = vec![cmd.program.display().to_string()];
            expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
            let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
            assert_eq!(shown, expected, "with ignore_case = {ignore_case:?}");
        }
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-grep-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "نص", Some(true)).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role != TokenRole::Path) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-grep-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };

        let cmd = plan_with(&repo, "نص", Some(true)).unwrap();
        let chosen = &cmd.args[1];
        assert!(cannot_be_read_as_a_flag(chosen), "{chosen:?} would be read as a flag");
    }

    #[test]
    fn shell_syntax_in_the_repository_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-grep-shellish").unwrap();

        let shellish = ["مستودع 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"];
        for name in shellish {
            let Some(repo) = repo_in(&s, name) else { return };
            let cmd = plan_with(&repo, "نص", None).unwrap();
            assert_eq!(cmd.args.len(), 7, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), repo.as_path());
        }
    }

    #[test]
    fn shell_syntax_in_a_pattern_stays_one_literal_argument() {
        let s = Scratch::new("git-grep-shellish-pattern").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        for pattern in ["a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "\"مقتبس\"", "'مفرد'"]
        {
            let cmd = plan_with(&repo, pattern, None).unwrap();
            assert_eq!(cmd.args[6], std::ffi::OsString::from(pattern));
            assert_eq!(cmd.args.len(), 7, "{pattern:?} must not add arguments");
        }
    }

    #[test]
    fn a_regex_pattern_is_carried_literally_because_of_fixed_string() {
        let s = Scratch::new("git-grep-regex").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        for pattern in ["a.b", "(", "(a+)+b", "^بداية$", "*", "[", "\\d+", "a|b"] {
            let cmd = plan_with(&repo, pattern, None).unwrap();
            assert_eq!(cmd.args[6], std::ffi::OsString::from(pattern), "{pattern:?} was rewritten");
            assert_eq!(cmd.args.len(), 7, "{pattern:?} must not add arguments");
        }
    }

    #[test]
    fn a_pattern_is_never_trimmed_because_the_spaces_are_part_of_the_search() {
        let s = Scratch::new("git-grep-spaces").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "  كلمة  ", None).unwrap();
        assert_eq!(cmd.args[6], std::ffi::OsString::from("  كلمة  "));
    }

    #[test]
    fn an_empty_pattern_is_refused_because_it_would_list_the_whole_tree() {
        let s = Scratch::new("git-grep-empty").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(refusal(plan_with(&repo, "", None)), ("err.name.invalid", Some("pattern")));
    }

    #[test]
    fn a_pattern_carrying_a_control_character_is_refused() {
        let s = Scratch::new("git-grep-control").unwrap();
        let repo = marked_repo(&s, "مستودع");
        for pattern in ["أ\nب", "أ\tب", "أ\rب", "أ\u{0}ب"] {
            assert_eq!(
                refusal(plan_with(&repo, pattern, None)),
                ("err.name.invalid", Some("pattern")),
                "{pattern:?} must be refused"
            );
        }
    }

    #[test]
    fn a_pattern_that_begins_with_a_dash_is_refused_with_its_own_field() {
        let s = Scratch::new("git-grep-dash-pattern").unwrap();
        let repo = marked_repo(&s, "مستودع");
        for pattern in ["-i", "--include=*", "-rf ~"] {
            assert_eq!(
                refusal(plan_with(&repo, pattern, None)),
                ("err.arg.flag", Some("pattern")),
                "{pattern:?} must be refused"
            );
        }
    }

    #[test]
    fn an_over_long_pattern_is_refused_before_it_becomes_an_argument() {
        let s = Scratch::new("git-grep-long").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let long = "ا".repeat(201);
        assert_eq!(refusal(plan_with(&repo, &long, None)), ("err.input.type", Some("pattern")));
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-grep-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        assert_eq!(refusal(plan_with(&plain, "نص", None)), ("err.path.not_dir", Some("repo")));
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-grep-missing").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"), "نص", None)),
            ("err.path.missing", Some("repo"))
        );
    }

    #[test]
    fn a_repository_outside_the_allowed_roots_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("/etc"), "نص", None)),
            ("err.path.outside", Some("repo"))
        );
    }

    #[test]
    fn a_relative_repository_path_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("نسبي/هنا"), "نص", None)),
            ("err.path.relative", Some("repo"))
        );
    }

    #[test]
    fn a_switch_sent_as_text_instead_of_a_boolean_is_refused_not_coerced() {
        // الواجهة لا تستطيع تهريب نصٍّ في موضع راية: النوع يُفحص قبل التخطيط.
        let s = Scratch::new("git-grep-type").unwrap();
        let dir = s.dir("م");
        let raw = BTreeMap::from([
            ("repo".to_owned(), RawValue::Path(dir.display().to_string())),
            ("pattern".to_owned(), RawValue::Text("نص".to_owned())),
            ("ignore_case".to_owned(), RawValue::Text("--exec=rm".to_owned())),
        ]);
        let r = crate::value::validate(&SPEC, &raw).map(|_| ());
        assert!(matches!(r, Err(CoreError::WrongInputType { id: "ignore_case" })), "got {r:?}");
    }

    #[test]
    fn a_symlinked_repository_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("git-grep-symlink").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "نص", None).unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_repository_raises_no_warning_at_all() {
        let s = Scratch::new("git-grep-quiet").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        assert_eq!(plan_with(&repo, "نص", Some(true)).unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn planning_writes_nothing_into_the_repository() {
        let s = Scratch::new("git-grep-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let before = s.names(&repo);

        for _ in 0..10 {
            plan_with(&repo, "نص", Some(true)).unwrap();
        }
        assert_eq!(s.names(&repo), before, "planning must not touch the repository");
    }
}
