//! تسجيل التعديلات في مستودع Git، باستخدام `git commit -a`.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> commit -a -m <الرسالة>
//! ```
//!
//! ## الحقل الذي حُذف بدل أن يُزيَّف
//!
//! كان في هذه العملية مدخلٌ ثالث: راية «ضمّ الملفات الجديدة». وهي مستحيلة
//! بأمرٍ واحد: `-a` تُدرِج **المعدَّل والمحذوف** من الملفات المتتبَّعة، ولا تمسّ
//! ملفًا جديدًا لم يُدرَج قط. ضمّ الجديد يحتاج `git add` قبلها — أي أمرين، وهذا
//! التطبيق يخطّط أمرًا واحدًا ويعرضه كما هو.
//!
//! فكان أمام الحقل ثلاثة مصائر:
//!
//! 1. **أن يبقى ولا يفعل شيئًا.** أسوأها: المستخدم يؤشّر عليه، فيُسجَّل commit
//!    لا يحمل ملفاته الجديدة، وتقول الشاشة «تمّ». يكتشف الغياب بعد أيامٍ — أو
//!    لا يكتشفه ويظن أن نسخته محفوظة.
//! 2. **أن يبقى ويصير الأمر أمرين.** لكن `executor` يشغّل أمرًا واحدًا ويقيسه
//!    بمخرجٍ واحد ورمز خروجٍ واحد. سلسلةٌ من أمرين تحتاج نموذجًا آخر للفشل
//!    الجزئي — أن ينجح `add` ويفشل `commit` حالةٌ لها اسم ولها تنظيف — وهذا
//!    ليس شيئًا يُضاف في هامش عملية.
//! 3. **أن يُحذف ويُقال السبب.** وهو ما جرى. والقول ليس في التوثيق وحده: تحذير
//!    `warn.git.untracked` يُرفع في **كل** خطة، لا حين يؤشّر المستخدم على شيء —
//!    لأن الحدّ في الأمر نفسه لا في اختياره.
//!
//! الملفات الجديدة تُدرَج اليوم من الطرفية بـ`git add`، ثم تُسجَّل من هنا. وعملية
//! «إدراج ملفات» مسجَّلة في خارطة الطريق باسمها ونموذج فشلها.
//!
//! ## الرسالة: تُقصّ ولا تُعاد كتابتها
//!
//! تُقصّ من الفراغ الطرفي فقط — و`git` تفعل ذلك بنفسها في تنقية الرسالة، فالقصّ
//! هنا لا يغيّر ما سيُسجَّل، بل يجعل ما يُعرض مطابقًا لما يدخل التاريخ.
//! وما عدا ذلك يُمرَّر حرفًا بحرف: الفاصلة المنقوطة وعلامة الدولار والاقتباس
//! محارف عادية، لأن الأمر مصفوفة وسائط لا نصٌّ يمرّ بمفسّر.
//!
//! ## ما لا تفحصه هذه العملية
//!
//! لا تسأل هل هناك ما يُسجَّل أصلًا، ولا هل ضُبط اسم المستخدم وبريده في إعداد
//! Git. الجوابان يحتاجان تشغيل `git` — والتخطيط لا يشغّل شيئًا. وفي الحالين
//! يخرج الأمر برمزٍ غير صفري وتقول الشاشة «لم تكتمل العملية»، والوصف يذكر
//! السببين كي يُقرأ الفشل على وجهه.

use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "git.commit";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.commit.title",
    description_key: "op.git.commit.description",
    category: Category::Git,
    danger: Danger::Modifies,
    visibility: Visibility::Production,
    tool: tools::GIT,
    // التسجيل يكتب في تاريخ المستودع لا في ملفٍ باسمٍ يختاره المستخدم، فلا اسم
    // نهائي يتضارب. والتراجع ممكن (‏`git reset`)، ولذلك `Modifies` لا
    // `Destructive`.
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("repo", InputKind::ExistingDir),
        // ٥٠٠ بايتًا: أطول من سطر عنوانٍ معقول بكثير، وأقصر من أن تصير الرسالة
        // مستندًا. والحدّ بالبايتات لا بالحروف — والحرف العربي بايتان — وهو ما
        // يقوله نصّ الحقل بدل أن يكتشفه المستخدم عند الرفض.
        InputSpec::new("message", InputKind::Text { max_len: 500 }),
    ],
    sort_order: 30,
    search_terms: &[
        "git",
        "commit",
        "تعديلات",
        "حفظ",
        "تسجيل",
        "message",
        "رسالة",
        "مستودع",
        "repo",
        "save",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let repo = inputs.dir("repo")?;
    require_repo(repo, "repo")?;
    let message = checked_message(inputs.text("message")?)?;

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(repo)
        .flag("commit", "explain.git.commit")
        .flag("-a", "explain.git.commit.all")
        .flag("-m", "explain.git.commit.message")
        // مُتحقَّق أعلاه أنها لا تبدأ بشرطة؛ و`Argv::value` تفحص ذلك ثانيةً ولا
        // تعتمد على المتصل بها.
        .value(message)
        // في كل خطة، لا عند اختيارٍ من المستخدم: الحدّ في `-a` نفسها.
        .warn("warn.git.untracked")
        .reveal(repo);

    if let Some(key) = warn_if_resolved(inputs, "repo", repo, "warn.git.repo.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

/// يفحص رسالة الـcommit ويعيدها مقصوصةَ الطرفين. لا يعيد كتابتها.
///
/// أربعة رفضٍ، ولكلٍّ سببه:
///
/// * **الفارغة أو كلّها فراغ** — `git commit -m ''` تفشل عند `git` نفسها،
///   والرفض هنا يسبقها كي يحمل اسم الحقل الذي يصلحه المستخدم بدل رسالةٍ من
///   الأداة في نافذة الخرج.
/// * **ما فيه محرف الصفر** — لا يعبر حدّ استدعاء النظام أصلًا.
/// * **ما فيه محرف تحكّم** — ومنه سطرٌ جديد. وهذا رفضٌ **بثمن معلن**: الرسالة
///   متعدّدة الفقرات ممارسةٌ مشروعة في Git، وهي ممنوعة من هنا. السبب أن
///   أطروحة المنتج كلها أن المعروض هو المنفَّذ — وسطرٌ جديد داخل وسيطٍ واحد
///   يجعل «سَطْر» يعرض سطرين حيث يوجد وسيطٌ واحد، فيُقرأ غير ما يُنفَّذ. من أراد
///   متنًا فليضفه بـ`git commit --amend` خارج التطبيق.
/// * **ما يبدأ بشرطة** — قد تُقرأ رايةً. والصنف المُعلَن `LeadingDot`؛ ليس اسمه
///   دقيقًا — لا صنف للشرطة في `NameRejection` — لكنه أقرب الموجود معنًى:
///   «حرفٌ في الصدر يغيّر معنى ما بعده». وبقيّة الأصناف كانت ستقول شيئًا غير
///   صحيح، وذلك أسوأ من اسمٍ داخليّ غير دقيق.
fn checked_message(raw: &str) -> Result<&str> {
    let rejected = |reason: NameRejection| CoreError::InvalidName { reason }.on_input("message");
    let message = raw.trim();

    if message.is_empty() {
        return Err(rejected(NameRejection::Empty));
    }
    // قبل فحص محارف التحكّم كي يُسمّى الصفر باسمه: كلاهما محرف تحكّم، ولأحدهما
    // صنفٌ خاصّ وسببٌ خاص.
    if message.contains('\0') {
        return Err(rejected(NameRejection::ContainsNul));
    }
    if message.chars().any(char::is_control) {
        return Err(rejected(NameRejection::ContainsControl));
    }
    if message.starts_with('-') {
        return Err(rejected(NameRejection::LeadingDot));
    }
    Ok(message)
}

/// يتحقّق أن المجلد المختار **جذرُ** مستودع، وينسب الرفض إلى حقله.
///
/// `symlink_metadata` لا `is_dir`: في شجرة عملٍ إضافية وفي الوحدات الفرعية
/// يكون `.git` ملفًا نصّيًا يشير إلى موضع المستودع، فشرطُ «مجلد» كان سيرفض
/// مستودعًا سليمًا. والجذرُ وحده مقبول: `git` تصعد من المجلد الفرعي إلى الجذر
/// وتسجّل تعديلات المستودع كله، بينما تقول الشاشة اسم المجلد الفرعي — وفي
/// عمليةٍ تكتب في التاريخ لا يصحّ أن يكون المعروض غير المنفَّذ.
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

    fn plan_with(repo: &Path, message: &str) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("repo".to_owned(), RawValue::Path(repo.display().to_string())),
            ("message".to_owned(), RawValue::Text(message.to_owned())),
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
        let found = listed.iter().find(|o| o.id == ID).expect("git.commit must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Modifies);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    /// الحقل المحذوف، مُثبَّتًا في اختبار.
    ///
    /// عودته يومًا بلا أمرين حقيقيين تُسقط هذا الاختبار — وهذا هو الغرض: القرار
    /// مكتوبٌ في الوثيقة أعلاه، وهذا ما يمنع نقضه بالسهو.
    #[test]
    fn the_operation_offers_no_switch_it_cannot_honour() {
        assert!(
            !SPEC.inputs.iter().any(|i| i.id == "include_untracked"),
            "`git commit -a` cannot stage new files; a switch that pretends otherwise is a lie"
        );
        assert_eq!(SPEC.inputs.len(), 2);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_message_is_the_last_argument() {
        let s = Scratch::new("git-commit-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let cmd = plan_with(&repo, "إصلاح خطأ في القراءة").unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 6);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "commit");
        assert_eq!(args[3], "-a");
        assert_eq!(args[4], "-m");
        assert_eq!(args[5], "إصلاح خطأ في القراءة");

        assert!(cmd.artifact.is_none());
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
        assert_eq!(cmd.reveal_target.as_deref(), Some(repo.as_path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-commit-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "رسالة").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-commit-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "رسالة").unwrap();
        // المسار والرسالة بيانات المستخدم، وما عداهما رموزٌ اختارها التطبيق
        // فلكلٍّ منها مفتاح شرح.
        for token in cmd.explain.iter().filter(|t| t.role == TokenRole::Flag) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn the_limit_of_dash_a_is_announced_in_every_single_plan() {
        let s = Scratch::new("git-commit-warn").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "رسالة").unwrap();
        assert!(cmd.warnings.contains(&"warn.git.untracked"), "{:?}", cmd.warnings);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-commit-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };

        let cmd = plan_with(&repo, "رسالة").unwrap();
        for (i, a) in cmd.args.iter().enumerate() {
            // المواضع ٠ و٢ و٣ و٤ رموزٌ اختارها التطبيق؛ ما عداها بيانات.
            if [0, 2, 3, 4].contains(&i) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "argument {i} ({a:?}) would be read as a flag");
        }
    }

    #[test]
    fn shell_syntax_in_the_message_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-commit-shellish").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let shellish = ["إصلاح 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"];
        for message in shellish {
            let cmd = plan_with(&repo, message).unwrap();
            assert_eq!(cmd.args.len(), 6, "{message:?} must not add arguments");
            assert_eq!(cmd.args[5].to_string_lossy(), message);
        }
    }

    #[test]
    fn an_empty_message_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-commit-empty").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["", "   ", "\t \t"] {
            let refused = refusal(plan_with(&repo, bad));
            assert_eq!(refused, ("err.name.invalid", Some("message")), "{bad:?}");
            assert_eq!(reason(plan_with(&repo, bad)), NameRejection::Empty, "{bad:?}");
        }
    }

    #[test]
    fn a_multi_line_message_is_refused_so_the_shown_command_stays_one_token() {
        let s = Scratch::new("git-commit-lines").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["سطر\nسطر", "قبل\rبعد", "فيه\tجدولة"] {
            let refused = refusal(plan_with(&repo, bad));
            assert_eq!(refused, ("err.name.invalid", Some("message")), "{bad:?}");
            assert_eq!(reason(plan_with(&repo, bad)), NameRejection::ContainsControl, "{bad:?}");
        }
    }

    #[test]
    fn a_message_carrying_a_nul_is_named_by_its_own_reason() {
        let s = Scratch::new("git-commit-nul").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(reason(plan_with(&repo, "رسالة\0")), NameRejection::ContainsNul);
    }

    #[test]
    fn a_message_that_starts_with_a_dash_is_refused_before_it_can_be_read_as_a_flag() {
        let s = Scratch::new("git-commit-dash-msg").unwrap();
        let repo = marked_repo(&s, "مستودع");

        // الرفض منسوبٌ إلى الحقل: `Argv::value` كانت سترفضه أيضًا، لكن بخطأٍ لا
        // يحمل اسم الحقل الذي يصلحه المستخدم.
        assert_eq!(
            refusal(plan_with(&repo, "-m رسالة مزروعة")),
            ("err.name.invalid", Some("message"))
        );
        assert_eq!(reason(plan_with(&repo, "--amend")), NameRejection::LeadingDot);
    }

    #[test]
    fn the_message_is_trimmed_at_its_edges_and_never_rewritten_inside() {
        let s = Scratch::new("git-commit-trim").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "  إصلاح   الفجوة  ").unwrap();
        assert_eq!(cmd.args[5].to_string_lossy(), "إصلاح   الفجوة", "inner spacing is the user's");
    }

    #[test]
    fn an_over_long_message_is_refused_by_the_declared_limit() {
        let s = Scratch::new("git-commit-long").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let long = "a".repeat(501);
        assert_eq!(refusal(plan_with(&repo, &long)), ("err.input.type", Some("message")));
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-commit-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        assert_eq!(refusal(plan_with(&plain, "رسالة")), ("err.path.not_dir", Some("repo")));
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-commit-missing").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"), "رسالة")),
            ("err.path.missing", Some("repo"))
        );
    }

    #[test]
    fn a_repository_outside_the_allowed_roots_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("/etc"), "رسالة")),
            ("err.path.outside", Some("repo"))
        );
    }

    #[test]
    fn a_symlinked_repository_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("git-commit-symlink").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "رسالة").unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_records_nothing_by_itself() {
        let s = Scratch::new("git-commit-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let before = s.names(&repo);

        for _ in 0..10 {
            plan_with(&repo, "رسالة").unwrap();
        }
        assert_eq!(s.names(&repo), before, "planning must not touch the repository");
    }
}
