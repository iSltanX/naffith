//! تصدير لقطةٍ من مستودع Git إلى أرشيف ZIP، باستخدام `git archive`.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> archive --format=zip -o <المؤقّت> <المرجع>
//! ```
//!
//! الوسيط بعد `-o` **مؤقّت** لا نهائي: الأرشيف يُبنى باسمٍ عابر داخل مجلد
//! الوجهة نفسه، ولا يُرقّى إلى اسمه إلا بعد خروجٍ ناجح. فانقطاعٌ في المنتصف —
//! أو مرجعٌ لا تعرفه `git` فتخرج بخطأ — لا يترك في وجهة المستخدم ملفًا نصفيًّا
//! يحمل الاسم الصحيح ويبدو أرشيفًا مكتملًا.
//!
//! ## لماذا `git archive` لا ضغط المجلد
//!
//! عملية «ضغط مجلد» تضغط ما على القرص: `node_modules` ومجلد `.git` بتاريخه
//! كاملًا وملفات المحرّر المؤقّتة وكل ما يتجاهله `.gitignore`. و`git archive`
//! تصدّر **الشجرة المسجَّلة عند مرجعٍ بعينه** من مخزن الكائنات: ما تتبّعه Git
//! وحده، بلا `.git`، وباحترام `export-ignore` في `.gitattributes`. فالناتج هو
//! «المشروع» لا «المجلد».
//!
//! وثمن ذلك معلَن في الوصف: **ما لم يُسجَّل لا يدخل الأرشيف**. من عدّل ملفًا ولم
//! يسجّله يجد في الأرشيف نسخته القديمة، لا التي يراها في محرّره الآن.
//!
//! ## ولماذا لا يُرفض أن تكون الوجهة داخل المستودع
//!
//! في عمليات النسخ والضغط تُرفض وجهةٌ داخل المصدر: الأداة تقرأ الشجرة التي
//! تكتب فيها فيدخل الأرشيف في نفسه. وهنا المصدر ليس شجرة العمل أصلًا بل مخزن
//! الكائنات عند مرجعٍ مسجَّل، فالملف الجديد في مجلد العمل لا يدخل فيما يُقرأ.
//! أقصى أثره أن يظهر الأرشيف ملفًا غير متتبَّع في «حالة المستودع» — وهذا خبرٌ
//! لا عطب، ورفضٌ بلا سببٍ حقيقي كان سيمنع الحالة الشائعة: أرشيفٌ يُحفظ بجانب
//! المشروع.
//!
//! ## ولا تقدير للحجم
//!
//! العمليات التي تمسح شجرةً تعرض تقديرًا لحجمها. وهنا كان التقدير سيقيس مجلد
//! العمل — بـ`.git` وبكل ما يتجاهله Git — أي رقمًا لا علاقة له بالأرشيف الناتج.
//! ورقمٌ خاطئ يُعرض بثقة أسوأ من غياب الرقم، فلا تقدير ولا تحذير مساحة.

use crate::atomic;
use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::paths;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::ffi::OsStr;
use std::path::Path;

pub const ID: &str = "git.archive";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.archive.title",
    description_key: "op.git.archive.description",
    category: Category::Git,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::Refuse,
    inputs: &[
        InputSpec::new("repo", InputKind::ExistingDir),
        // ١٠٠ بايتٍ حدٌّ سخيّ: أطول اسم مرجعٍ معقول (`refs/tags/…`) أقصر منه
        // بكثير، وبصمة الـcommit الكاملة أربعون محرفًا.
        InputSpec::new("revision", InputKind::Text { max_len: 100 }),
        InputSpec::new("destination", InputKind::TargetDir),
        InputSpec::new("archive_name", InputKind::NewName { ext: Some("zip") }),
    ],
    sort_order: 60,
    search_terms: &[
        "git",
        "archive",
        "أرشيف",
        "تصدير",
        "export",
        "zip",
        "لقطة",
        "snapshot",
        "إصدار",
        "release",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let repo = inputs.dir("repo")?;
    require_repo(repo, "repo")?;
    let revision = checked_revision(inputs.text("revision")?)?;
    let destination = inputs.target_dir("destination")?;
    let name = inputs.name("archive_name")?;

    let final_path = paths::new_file_in(destination, OsStr::new(name)).map_err(|e| {
        let field =
            if matches!(e, CoreError::InvalidName { .. }) { "archive_name" } else { "destination" };
        e.on_input(field)
    })?;

    // `symlink_metadata` لا يتبع الروابط: رابطٌ معلَّق بالاسم النهائي اسمٌ مشغول.
    if std::fs::symlink_metadata(&final_path).is_ok() {
        return Err(CoreError::DestinationExists.on_input("archive_name"));
    }

    let temp = atomic::temp_path_for(&final_path)?;

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(repo)
        .flag("archive", "explain.git.archive")
        // رمزٌ واحد لا رمزان: `--format zip` صيغةٌ صحيحة أيضًا، لكن الملتصقة
        // تجعل الراية وقيمتها كتلةً واحدة في العرض وفي `argv` معًا، فلا يوجد
        // موضعٌ يمكن أن ينزلق إليه وسيطٌ بينهما.
        .flag("--format=zip", "explain.git.archive.format")
        .flag("-o", "explain.git.archive.output")
        .explained_path(&temp, "explain.role.temp")
        // مُتحقَّق أعلاه أنه لا يبدأ بشرطة؛ و`Argv::value` تفحص ذلك ثانيةً ولا
        // تعتمد على المتصل بها.
        .explained_value(revision, "explain.git.revision")
        // في كل خطة: الفرق بين «ما في محرّري» و«ما سجّلتُه» هو أكثر ما يفاجئ
        // في هذه العملية.
        .warn("warn.git.archive.committed_only");

    if let Some(key) = warn_if_resolved(inputs, "repo", repo, "warn.git.repo.resolved") {
        argv = argv.warn(key);
    }
    if let Some(key) =
        warn_if_resolved(inputs, "destination", destination, "warn.destination.resolved")
    {
        argv = argv.warn(key);
    }

    argv.producing(Artifact::file(temp, final_path))
}

/// يفحص المرجع ويعيده مقصوصَ الطرفين. لا ينقّيه ولا يعيد كتابته.
///
/// المرجع نصٌّ تفهمه `git` وحدها: `HEAD`، أو وسمٌ (`v1.2.0`)، أو فرع
/// (`main`، `feature/x`)، أو بصمة commit. ولذلك **لا يُرفض `/`** هنا وإن كان
/// يُرفض في أسماء الملفات: هو فاصلٌ مشروع في أسماء المراجع، ورفضُه كان سيمنع
/// أكثر أسماء الفروع شيوعًا.
///
/// وما يُرفض:
///
/// * **الفارغ** — `git archive` بلا مرجعٍ تطبع استعمالها وتخرج بخطأ، والرفض
///   هنا يسبقها كي يحمل اسم الحقل الذي يصلحه المستخدم.
/// * **محرف الصفر** — لا يعبر حدّ استدعاء النظام أصلًا.
/// * **محارف التحكّم والفراغ** — أسماء المراجع في Git لا تحتملها أصلًا
///   (`git check-ref-format`)، وسطرٌ جديد أو مسافةٌ داخل وسيطٍ واحد يجعل
///   «سَطْر» يعرض رمزين حيث يوجد رمز، فيُقرأ غير ما يُنفَّذ.
/// * **ما يبدأ بشرطة** — قد يُقرأ رايةً من رايات `git archive`.
/// * **ما فيه `..`** — `a..b` مدًى لا شجرة، و`git archive` تصدّر شجرةً واحدة
///   فترفضه. ورفضُه هنا برسالةٍ في الحقل أوضح من رسالةٍ من الأداة في نافذة
///   الخرج. ويمنع في الوقت نفسه أن يظهر في الأمر المعروض رمزٌ يشبه صعودًا في
///   المسارات، فيقرأه المستخدم على غير معناه.
///
/// ## نسبة الرفض إلى سببٍ من مجموعة مغلقة
///
/// `NameRejection` في `error.rs` كُتبت لأسماء الملفات، وليس فيها عضوٌ يقول «فيه
/// فراغ» ولا «يبدأ بشرطة». وإضافة عضوٍ تعني تعديل ملفٍّ مشترك لا تملكه عملية.
/// فالفراغ منسوبٌ إلى `ContainsControl` — النصّ المعروض «الاسم يحتوي محارف
/// تحكّم» لا يسمّي المسافة، لكنه لا يقود المستخدم إلى إفساد مدخلٍ صحيح — والشرطة
/// البادئة إلى `LeadingDot`، وهو أقرب الموجود معنًى: «حرفٌ في الصدر يغيّر معنى
/// ما بعده». وما عداهما منسوبٌ إلى سببه الحقيقي. ونصّ الحقل يقول القاعدة كاملةً
/// قبل أن يقع الرفض.
fn checked_revision(raw: &str) -> Result<&str> {
    let rejected = |reason: NameRejection| CoreError::InvalidName { reason }.on_input("revision");
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
/// مستودعًا سليمًا. والجذرُ وحده مقبول: `git` تصعد من المجلد الفرعي إلى الجذر
/// وتصدّر المستودع كله بينما تقول الشاشة اسم المجلد الفرعي.
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

    fn plan_with(
        repo: &Path,
        revision: &str,
        destination: &Path,
        name: &str,
    ) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("repo".to_owned(), RawValue::Path(repo.display().to_string())),
            ("revision".to_owned(), RawValue::Text(revision.to_owned())),
            ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
            ("archive_name".to_owned(), RawValue::Text(name.to_owned())),
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
        let found = listed.iter().find(|o| o.id == ID).expect("git.archive must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::Refuse);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_writes_to_a_temporary_name() {
        let s = Scratch::new("git-archive-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };
        let dst = s.dir("الوجهة");

        let cmd = plan_with(&repo, "HEAD", &dst, "لقطة").unwrap();
        let args = args_of(&cmd);
        let artifact = cmd.artifact.as_ref().unwrap();

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 7);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "archive");
        assert_eq!(args[3], "--format=zip");
        assert_eq!(args[4], "-o");
        assert_eq!(Path::new(&args[5]), artifact.temp.as_path());
        assert_eq!(args[6], "HEAD");

        assert_eq!(artifact.final_path, dst.join("لقطة.zip"));
        assert_eq!(artifact.kind, ArtifactKind::File);
        assert!(!artifact.temp.exists(), "planning must create nothing");
        assert!(cmd.stdout_to.is_none(), "git writes the file itself with -o");
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-archive-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let dst = s.dir("و");

        let cmd = plan_with(&repo, "HEAD", &dst, "ل").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-archive-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let dst = s.dir("و");

        let cmd = plan_with(&repo, "v1.0", &dst, "ل").unwrap();
        for token in cmd.explain.iter().filter(|t| t.role != TokenRole::Path) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn the_extension_is_added_once_and_never_doubled() {
        let s = Scratch::new("git-archive-ext").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let dst = s.dir("و");

        for (given, expected) in
            [("لقطة", "لقطة.zip"), ("لقطة.zip", "لقطة.zip"), ("لقطة.ZIP", "لقطة.ZIP")]
        {
            let cmd = plan_with(&repo, "HEAD", &dst, given).unwrap();
            assert_eq!(
                cmd.artifact.unwrap().final_path.file_name().unwrap(),
                OsStr::new(expected),
                "for input {given:?}"
            );
        }
    }

    #[test]
    fn the_gap_between_the_editor_and_the_last_commit_is_announced_in_every_plan() {
        let s = Scratch::new("git-archive-warn").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let dst = s.dir("و");

        let cmd = plan_with(&repo, "HEAD", &dst, "ل").unwrap();
        assert!(cmd.warnings.contains(&"warn.git.archive.committed_only"), "{:?}", cmd.warnings);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-archive-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };
        let dst = s.dir("و");

        let cmd = plan_with(&repo, "HEAD", &dst, "-x").unwrap();
        for (i, a) in cmd.args.iter().enumerate() {
            // المواضع ٠ و٢ و٣ و٤ رموزٌ اختارها التطبيق؛ ما عداها بيانات.
            if [0, 2, 3, 4].contains(&i) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "argument {i} ({a:?}) would be read as a flag");
        }
    }

    #[test]
    fn shell_syntax_stays_literal_and_adds_no_arguments() {
        let s = Scratch::new("git-archive-shellish").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let dst = s.dir("و");

        // في الاسم — الفراغ مقبولٌ في أسماء الملفات.
        for name in ["لقطة 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let cmd = plan_with(&repo, "HEAD", &dst, name).unwrap();
            assert_eq!(cmd.args.len(), 7, "{name:?} must not add arguments");
            assert_eq!(cmd.artifact.unwrap().final_path, dst.join(format!("{name}.zip")));
        }

        // وفي المرجع — بلا فراغ، لأن الفراغ مرفوض في المراجع.
        for revision in ["a;rm", "$(whoami)", "back`tick`", "a&b"] {
            let cmd = plan_with(&repo, revision, &dst, "ل").unwrap();
            assert_eq!(cmd.args.len(), 7, "{revision:?} must not add arguments");
            assert_eq!(cmd.args[6].to_string_lossy(), revision);
        }
    }

    #[test]
    fn a_branch_name_with_a_slash_is_accepted_because_it_is_a_reference_not_a_file_name() {
        let s = Scratch::new("git-archive-slash").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let dst = s.dir("و");

        let cmd = plan_with(&repo, "refs/tags/v1.0", &dst, "ل").unwrap();
        assert_eq!(cmd.args[6].to_string_lossy(), "refs/tags/v1.0");
    }

    #[test]
    fn an_empty_revision_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-archive-empty-rev").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let dst = s.dir("و");

        for bad in ["", "   ", "\t"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, &dst, "ل")),
                ("err.name.invalid", Some("revision")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&repo, bad, &dst, "ل")), NameRejection::Empty, "{bad:?}");
        }
    }

    #[test]
    fn a_revision_with_whitespace_or_control_characters_is_refused() {
        let s = Scratch::new("git-archive-space-rev").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let dst = s.dir("و");

        for bad in ["HEAD 1", "v1.0\nv2.0", "a\tb", "بين كلمتين"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, &dst, "ل")),
                ("err.name.invalid", Some("revision")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, bad, &dst, "ل")),
                NameRejection::ContainsControl,
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_revision_carrying_a_nul_is_named_by_its_own_reason() {
        let s = Scratch::new("git-archive-nul-rev").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let dst = s.dir("و");
        assert_eq!(reason(plan_with(&repo, "HEAD\0", &dst, "ل")), NameRejection::ContainsNul);
    }

    #[test]
    fn a_revision_that_starts_with_a_dash_is_refused_before_it_can_be_read_as_a_flag() {
        let s = Scratch::new("git-archive-dash-rev").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let dst = s.dir("و");

        for bad in ["-o/etc/passwd", "--output=/etc/passwd", "-HEAD"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, &dst, "ل")),
                ("err.name.invalid", Some("revision")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, bad, &dst, "ل")),
                NameRejection::LeadingDot,
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_revision_that_carries_two_dots_is_refused_as_a_range_not_a_tree() {
        let s = Scratch::new("git-archive-range-rev").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let dst = s.dir("و");

        for bad in ["main..feature", "v1.0...v2.0", "../خارج"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, &dst, "ل")),
                ("err.name.invalid", Some("revision")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, bad, &dst, "ل")),
                NameRejection::DotOrDotDot,
                "{bad:?}"
            );
        }
    }

    #[test]
    fn the_revision_is_trimmed_at_its_edges_and_never_rewritten_inside() {
        let s = Scratch::new("git-archive-trim").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let dst = s.dir("و");

        let cmd = plan_with(&repo, "  v1.0  ", &dst, "ل").unwrap();
        assert_eq!(cmd.args[6].to_string_lossy(), "v1.0");
    }

    #[test]
    fn an_over_long_revision_is_refused_by_the_declared_limit() {
        let s = Scratch::new("git-archive-long-rev").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let dst = s.dir("و");
        let long = "a".repeat(101);
        assert_eq!(
            refusal(plan_with(&repo, &long, &dst, "ل")),
            ("err.input.type", Some("revision"))
        );
    }

    #[test]
    fn an_existing_archive_name_stops_the_plan_and_leaves_the_file_untouched() {
        let s = Scratch::new("git-archive-exists").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let dst = s.dir("و");
        std::fs::write(dst.join("موجود.zip"), b"PRECIOUS").unwrap();

        assert_eq!(
            refusal(plan_with(&repo, "HEAD", &dst, "موجود")),
            ("err.dest.exists", Some("archive_name"))
        );
        assert_eq!(std::fs::read(dst.join("موجود.zip")).unwrap(), b"PRECIOUS");
    }

    #[test]
    fn a_name_that_would_break_the_path_is_blamed_on_the_name_not_the_destination() {
        let s = Scratch::new("git-archive-badname").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let dst = s.dir("و");

        for bad in ["", "   ", "أ/ب", ".مخفي", "..", "اسم\n"] {
            assert_eq!(
                refusal(plan_with(&repo, "HEAD", &dst, bad)),
                ("err.name.invalid", Some("archive_name")),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-archive-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        let dst = s.dir("و");
        assert_eq!(
            refusal(plan_with(&plain, "HEAD", &dst, "ل")),
            ("err.path.not_dir", Some("repo"))
        );
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-archive-missing").unwrap();
        let dst = s.dir("و");
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"), "HEAD", &dst, "ل")),
            ("err.path.missing", Some("repo"))
        );
    }

    #[test]
    fn a_destination_that_does_not_exist_is_refused_and_blamed_on_its_own_field() {
        let s = Scratch::new("git-archive-nodest").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(
            refusal(plan_with(&repo, "HEAD", &s.path().join("لا-وجود-لها"), "ل")),
            ("err.path.missing", Some("destination"))
        );
    }

    #[test]
    fn a_destination_outside_the_allowed_roots_is_refused() {
        let s = Scratch::new("git-archive-outside").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(
            refusal(plan_with(&repo, "HEAD", Path::new("/etc"), "ل")),
            ("err.path.outside", Some("destination"))
        );
    }

    #[test]
    fn a_symlinked_repository_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("git-archive-symlink").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let dst = s.dir("و");

        let cmd = plan_with(&link, "HEAD", &dst, "ل").unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_the_destination_clean() {
        let s = Scratch::new("git-archive-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let dst = s.dir("و");

        for _ in 0..10 {
            plan_with(&repo, "HEAD", &dst, "ل").unwrap();
        }
        assert_eq!(s.names(&dst), Vec::<String>::new(), "the destination must stay clean");
    }
}
