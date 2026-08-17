//! الفرق بين نقطتين من تاريخ مستودع Git، باستخدام `git diff --stat`.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> diff --stat --exit-code <من> <إلى>
//! ```
//!
//! ## بم تختلف هذه عن `git.diff`
//!
//! `git.diff` تقارن شجرة العمل — ما لم يُدرَج بعد — بمنطقة الإدراج أو بآخر
//! commit، حسب راية `--staged`. طرفا المقارنة هناك **ثابتان ضمنيًا**: «الآن»
//! في مقابل «آخر ما سُجّل». والمستخدم لا يختار شيئًا سوى أيّهما.
//!
//! هذه العملية على النقيض: لا طرف ثابت. المستخدمان `from` و`to` **مرجعان
//! صريحان** يكتبهما المستخدم — فرعان، وسمان، بصمتا commit، أو أيّ مزيج منها —
//! وتقارن `git` الشجرتين المسجَّلتين عندهما في التاريخ، بلا صلةٍ بحالة شجرة
//! العمل أصلًا. فمن أراد «ماذا تغيّر بين هذا الإصدار وذاك» يستعمل هذه، ومن
//! أراد «ماذا عدّلتُ ولم أُسجّله بعد» يستعمل `git.diff`.
//!
//! ولنفس سبب `git.diff`: `--stat` لا الفرق الكامل، و`--exit-code` كي يُقرأ
//! رمز الخروج لا نصّ الخرج — 0 «لا فروق» و1 «توجد فروق»، وكلاهما جوابٌ ناجح
//! هنا لا فشلًا. انظر رأس `git_diff.rs` للتفصيل الكامل في السببين، فهما نفس
//! السببين هنا حرفيًا.
//!
//! ## ما لا تفحصه هذه العملية
//!
//! لا تتحقّق أن `from` أو `to` مرجعٌ موجود فعلًا في المستودع. ذلك يحتاج تشغيل
//! `git` — والتخطيط لا يشغّل شيئًا. مرجعٌ غير موجود يخرج منه الأمر برمزٍ غير
//! صفري ورسالةٍ من `git` نفسها في نافذة الخرج، وهذا كافٍ: الفحص هنا يمنع فقط
//! ما يُقرأ رايةً أو يغيّر شكل الأمر المعروض، لا كل خطأٍ ممكن.

use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "git.diff.commits";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.diff.commits.title",
    description_key: "op.git.diff.commits.description",
    category: Category::Git,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("repo", InputKind::ExistingDir),
        // ١٠٠ بايتٍ حدٌّ سخيّ: أطول اسم مرجعٍ معقول (`refs/tags/…`) أقصر منه
        // بكثير، وبصمة الـcommit الكاملة أربعون محرفًا. نفس الحدّ في `git.archive`.
        InputSpec::new("from", InputKind::Text { max_len: 100 }),
        InputSpec::new("to", InputKind::Text { max_len: 100 }),
    ],
    sort_order: 80,
    search_terms: &[
        "git",
        "diff",
        "compare",
        "commits",
        "فرق",
        "مقارنة",
        "بين",
        "تسجيلين",
        "مراجع",
        "مستودع",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let repo = inputs.dir("repo")?;
    require_repo(repo, "repo")?;
    let from = checked_revision(inputs.text("from")?, "from")?;
    let to = checked_revision(inputs.text("to")?, "to")?;

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(repo)
        .flag("diff", "explain.git.diff")
        .flag("--stat", "explain.git.diff.stat")
        // لا نقرأ نص الخرج لنستنتج الدلالة. Git تعرّف 0 = لا فروق و1 = توجد
        // فروق حين تُطلب هذه الراية، والنواة تحوّل الرمزين إلى جوابين ناجحين.
        .flag("--exit-code", "explain.git.diff.exit_code")
        // مُتحقَّق أعلاه أنهما لا يبدآن بشرطة؛ و`Argv::explained_value` تفحص
        // ذلك ثانيةً ولا تعتمد على المتصل بها.
        .explained_value(from, "explain.git.diff.from")
        .explained_value(to, "explain.git.diff.to");

    argv = argv.reveal(repo);

    if let Some(key) = warn_if_resolved(inputs, "repo", repo, "warn.git.repo.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

/// يفحص مرجعًا واحدًا (`from` أو `to`) ويعيده مقصوصَ الطرفين. لا ينقّيه ولا
/// يعيد كتابته.
///
/// نفس منطق `checked_revision` في `git_archive.rs` حرفيًا؛ الفرق الوحيد أن
/// هذه العملية تملك **حقلين** يحملان مرجعًا لا حقلًا واحدًا، فالرفض يُنسب إلى
/// `field` الذي يُستدعى به — لا إلى اسمٍ ثابت — كي يعرف المستخدم أيّهما يصلح.
///
/// المرجع نصٌّ تفهمه `git` وحدها: `HEAD`، أو وسمٌ (`v1.2.0`)، أو فرع
/// (`main`، `feature/x`)، أو بصمة commit. ولذلك **لا يُرفض `/`** هنا وإن كان
/// يُرفض في أسماء الملفات: هو فاصلٌ مشروع في أسماء المراجع.
///
/// وما يُرفض:
///
/// * **الفارغ** — `git diff` بمرجعٍ فارغ يخرج بخطأ لا يسمّي الحقل، والرفض هنا
///   يسبقه كي يحمل اسم الحقل الذي يصلحه المستخدم.
/// * **محرف الصفر** — لا يعبر حدّ استدعاء النظام أصلًا.
/// * **محارف التحكّم والفراغ** — أسماء المراجع في Git لا تحتملها أصلًا
///   (`git check-ref-format`)، وسطرٌ جديد أو مسافةٌ داخل وسيطٍ واحد يجعل
///   «سَطْر» يعرض رمزين حيث يوجد رمز، فيُقرأ غير ما يُنفَّذ.
/// * **ما يبدأ بشرطة** — قد يُقرأ رايةً من رايات `git diff`.
/// * **ما فيه `..`** — طرفا هذه المقارنة يُكتبان في حقلين منفصلين أصلًا، فمرجعٌ
///   يحمل `..` بداخله إمّا مدًى مكتوبًا في غير موضعه وإمّا محاولة تمرير وسيطٍ
///   إضافي داخل قيمةٍ واحدة. ورفضُه هنا برسالةٍ في الحقل أوضح من رسالةٍ من
///   الأداة في نافذة الخرج.
///
/// ## نسبة الرفض إلى سببٍ من مجموعة مغلقة
///
/// نفس التبرير في `git_archive.rs`: `NameRejection` كُتبت لأسماء الملفات، فلا
/// عضو فيها يقول «فيه فراغ» ولا «يبدأ بشرطة». الفراغ منسوبٌ إلى
/// `ContainsControl`، والشرطة البادئة إلى `LeadingDot` — أقرب الموجود معنًى.
fn checked_revision<'a>(raw: &'a str, field: &'static str) -> Result<&'a str> {
    let rejected = |reason: NameRejection| CoreError::InvalidName { reason }.on_input(field);
    // القصّ قبل الفحص: من ينسخ بصمة commit من مخرجٍ ينسخ معه فراغًا طرفيًا،
    // ورفضُه على ذلك تشدّدٌ لا يحمي شيئًا. وما بقي بعد القصّ هو ما يدخل الأمر
    // حرفيًا.
    let revision = raw.trim();

    if revision.is_empty() {
        return Err(rejected(NameRejection::Empty));
    }
    // قبل فحص محارف التحكّم كي يُسمّى الصفر باسمه: كلاهما محرف تحكّم، ولأحدهما
    // صنفٌ خاصّ وسببٌ خاص.
    if revision.contains('\0') {
        return Err(rejected(NameRejection::ContainsNul));
    }
    if revision.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(rejected(NameRejection::ContainsControl));
    }
    if revision.starts_with('-') {
        return Err(rejected(NameRejection::LeadingDot));
    }
    if revision.contains("..") {
        return Err(rejected(NameRejection::DotOrDotDot));
    }
    Ok(revision)
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

    fn plan_with(repo: &Path, from: &str, to: &str) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("repo".to_owned(), RawValue::Path(repo.display().to_string())),
            ("from".to_owned(), RawValue::Text(from.to_owned())),
            ("to".to_owned(), RawValue::Text(to.to_owned())),
        ]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    fn reason(r: Result<PlannedCommand>) -> NameRejection {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(CoreError::OnInput { source, .. }) => match *source {
                CoreError::InvalidName { reason } => reason,
                other => panic!("expected an invalid name, got {other:?}"),
            },
            Err(other) => panic!("expected an attributed refusal, got {other:?}"),
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
        let found = listed.iter().find(|o| o.id == ID).expect("git.diff.commits must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let s = Scratch::new("git-diff-commits-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let cmd = plan_with(&repo, "main", "feature").unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 7);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "diff");
        assert_eq!(args[3], "--stat");
        assert_eq!(args[4], "--exit-code");
        assert_eq!(args[5], "main");
        assert_eq!(args[6], "feature");

        assert!(cmd.artifact.is_none());
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
        assert_eq!(cmd.reveal_target.as_deref(), Some(repo.as_path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-diff-commits-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "v1.0", "v2.0").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-diff-commits-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "HEAD~1", "HEAD").unwrap();
        for token in cmd.explain.iter().filter(|t| t.role != TokenRole::Path) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-diff-commits-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };

        let cmd = plan_with(&repo, "main", "feature").unwrap();
        let chosen = &cmd.args[1];
        assert!(cannot_be_read_as_a_flag(chosen), "{chosen:?} would be read as a flag");
    }

    #[test]
    fn shell_syntax_in_the_repository_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-diff-commits-shellish").unwrap();

        let shellish = ["مستودع 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"];
        for name in shellish {
            let Some(repo) = repo_in(&s, name) else { return };
            let cmd = plan_with(&repo, "main", "feature").unwrap();
            assert_eq!(cmd.args.len(), 7, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), repo.as_path());
        }
    }

    #[test]
    fn a_branch_name_with_a_slash_is_accepted_because_it_is_a_reference_not_a_file_name() {
        let s = Scratch::new("git-diff-commits-slash").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "refs/tags/v1.0", "refs/heads/main").unwrap();
        assert_eq!(cmd.args[5].to_string_lossy(), "refs/tags/v1.0");
        assert_eq!(cmd.args[6].to_string_lossy(), "refs/heads/main");
    }

    #[test]
    fn an_empty_from_is_refused_and_blamed_on_its_own_field() {
        let s = Scratch::new("git-diff-commits-empty-from").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["", "   ", "\t"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, "HEAD")),
                ("err.name.invalid", Some("from")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&repo, bad, "HEAD")), NameRejection::Empty, "{bad:?}");
        }
    }

    #[test]
    fn an_empty_to_is_refused_and_blamed_on_its_own_field() {
        let s = Scratch::new("git-diff-commits-empty-to").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["", "   ", "\t"] {
            assert_eq!(
                refusal(plan_with(&repo, "HEAD", bad)),
                ("err.name.invalid", Some("to")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&repo, "HEAD", bad)), NameRejection::Empty, "{bad:?}");
        }
    }

    #[test]
    fn a_from_that_starts_with_a_dash_is_refused_before_it_can_be_read_as_a_flag() {
        let s = Scratch::new("git-diff-commits-dash-from").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["--exec=rm", "-HEAD", "-o/etc/passwd"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, "HEAD")),
                ("err.name.invalid", Some("from")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&repo, bad, "HEAD")), NameRejection::LeadingDot, "{bad:?}");
        }
        // ولا يكفي أن يصلح `to` كي يمرّ `from` الفاسد.
        assert_eq!(refusal(plan_with(&repo, "-x", "main")), ("err.name.invalid", Some("from")));
    }

    #[test]
    fn a_to_that_starts_with_a_dash_is_refused_before_it_can_be_read_as_a_flag() {
        let s = Scratch::new("git-diff-commits-dash-to").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["--exec=rm", "-HEAD", "-o/etc/passwd"] {
            assert_eq!(
                refusal(plan_with(&repo, "HEAD", bad)),
                ("err.name.invalid", Some("to")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&repo, "HEAD", bad)), NameRejection::LeadingDot, "{bad:?}");
        }
        // ولا يكفي أن يصلح `from` كي يمرّ `to` الفاسد.
        assert_eq!(refusal(plan_with(&repo, "main", "-x")), ("err.name.invalid", Some("to")));
    }

    #[test]
    fn a_from_that_carries_two_dots_is_refused_and_blamed_on_its_own_field() {
        let s = Scratch::new("git-diff-commits-range-from").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["main..feature", "v1.0...v2.0", "../خارج"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, "HEAD")),
                ("err.name.invalid", Some("from")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, bad, "HEAD")),
                NameRejection::DotOrDotDot,
                "{bad:?}"
            );
        }
        // ولا يكفي أن يصلح `to` كي يمرّ `from` الفاسد.
        assert_eq!(refusal(plan_with(&repo, "a..b", "main")), ("err.name.invalid", Some("from")));
    }

    #[test]
    fn a_to_that_carries_two_dots_is_refused_and_blamed_on_its_own_field() {
        let s = Scratch::new("git-diff-commits-range-to").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["main..feature", "v1.0...v2.0", "../خارج"] {
            assert_eq!(
                refusal(plan_with(&repo, "HEAD", bad)),
                ("err.name.invalid", Some("to")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, "HEAD", bad)),
                NameRejection::DotOrDotDot,
                "{bad:?}"
            );
        }
        // ولا يكفي أن يصلح `from` كي يمرّ `to` الفاسد.
        assert_eq!(refusal(plan_with(&repo, "main", "a..b")), ("err.name.invalid", Some("to")));
    }

    #[test]
    fn a_revision_carrying_a_nul_is_named_by_its_own_reason() {
        let s = Scratch::new("git-diff-commits-nul-rev").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(reason(plan_with(&repo, "HEAD\0", "HEAD")), NameRejection::ContainsNul);
        assert_eq!(reason(plan_with(&repo, "HEAD", "HEAD\0")), NameRejection::ContainsNul);
    }

    #[test]
    fn a_revision_with_whitespace_or_control_characters_is_refused() {
        let s = Scratch::new("git-diff-commits-space-rev").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["HEAD 1", "v1.0\nv2.0", "a\tb", "بين كلمتين"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, "HEAD")),
                ("err.name.invalid", Some("from")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, "HEAD", bad)),
                NameRejection::ContainsControl,
                "{bad:?}"
            );
        }
    }

    #[test]
    fn the_revisions_are_trimmed_at_their_edges_and_never_rewritten_inside() {
        let s = Scratch::new("git-diff-commits-trim").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "  v1.0  ", "  v2.0  ").unwrap();
        assert_eq!(cmd.args[5].to_string_lossy(), "v1.0");
        assert_eq!(cmd.args[6].to_string_lossy(), "v2.0");
    }

    #[test]
    fn an_over_long_from_is_refused_by_the_declared_limit() {
        let s = Scratch::new("git-diff-commits-long-from").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let long = "a".repeat(101);
        assert_eq!(refusal(plan_with(&repo, &long, "HEAD")), ("err.input.type", Some("from")));
    }

    #[test]
    fn an_over_long_to_is_refused_by_the_declared_limit() {
        let s = Scratch::new("git-diff-commits-long-to").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let long = "a".repeat(101);
        assert_eq!(refusal(plan_with(&repo, "HEAD", &long)), ("err.input.type", Some("to")));
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-diff-commits-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        assert_eq!(
            refusal(plan_with(&plain, "main", "feature")),
            ("err.path.not_dir", Some("repo"))
        );
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-diff-commits-missing").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"), "main", "feature")),
            ("err.path.missing", Some("repo"))
        );
    }

    #[test]
    fn a_repository_outside_the_allowed_roots_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("/etc"), "main", "feature")),
            ("err.path.outside", Some("repo"))
        );
    }

    #[test]
    fn a_relative_repository_path_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("نسبي/هنا"), "main", "feature")),
            ("err.path.relative", Some("repo"))
        );
    }

    #[test]
    fn a_symlinked_repository_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("git-diff-commits-symlink").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "main", "feature").unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_repository_raises_no_warning_at_all() {
        let s = Scratch::new("git-diff-commits-quiet").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        assert_eq!(plan_with(&repo, "main", "feature").unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn planning_writes_nothing_into_the_repository() {
        let s = Scratch::new("git-diff-commits-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let before = s.names(&repo);

        for _ in 0..10 {
            plan_with(&repo, "main", "feature").unwrap();
        }
        assert_eq!(s.names(&repo), before, "planning must not touch the repository");
    }
}
