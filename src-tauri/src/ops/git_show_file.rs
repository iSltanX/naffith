//! عرض محتوى ملفٍّ عند مرجعٍ بعينه من تاريخ مستودع Git، باستخدام
//! `git show <مرجع>:<مسار>`.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> show <المرجع>:<المسار>
//! ```
//!
//! ## لماذا رمزٌ واحدٌ ملتصق لا رمزان
//!
//! `<مرجع>:<مسار>` صيغةٌ واحدة عند `git` نفسها (‏`gitrevisions(7)`، صيغة
//! «‏rev:path»)، لا رايةً تتبعها قيمة. ومع ذلك المبدأ الذي بُني عليه لصقهما في
//! `Argv` هو نفسه الذي بُنيت عليه `--format=zip` في `git_archive.rs`: وسيطٌ
//! واحدٌ في `argv` لا اثنان يمنع أن يوجد موضعٌ بينهما ينزلق فيه رمزٌ ثالث — لا
//! من الشيفرة (لا نداء بين `explained_value` الواحدة وأختها)، ولا من العرض
//! (المستخدم يرى «‏HEAD:README.md» كتلةً واحدة، لا رايةً وقيمة).
//!
//! والفرق الحقيقي عن `--format=zip`: هناك القيمتان ثابتتان في الشيفرة، وهنا
//! القيمتان من المستخدم عبر حقلين منفصلين في الواجهة — لأن كلًّا منهما
//! يُتحقَّق بقواعد مختلفة عن الآخر (مرجعٌ يُتحقَّق كمرجع، ومسارٌ يُتحقَّق
//! كمسار) — ولا يُدمجان في وسيطٍ واحد إلا بعد أن يجتاز كلٌّ منهما فحصه الخاص
//! منفردًا.
//!
//! ## المسار داخل تاريخ Git لا داخل شجرة العمل
//!
//! هذا هو ما يميّز هذه العملية عن بقيّة قسم Git: `path` هنا اسمٌ داخل شجرةٍ
//! **مسجَّلة** عند `ref`، لا بالضرورة موجودًا على القرص الآن. ملفٌّ حُذف في
//! commit لاحق، أو أُعيدت تسميته، يبقى قابلًا للعرض عند مرجعٍ يسبق ذلك —
//! وهذا بالضبط الاستعمال الذي صُمّمت له هذه العملية: النظر في نسخةٍ قديمة من
//! ملفٍّ لم يعد له وجودٌ بذلك الاسم اليوم.
//!
//! ولهذا **لا `InputKind::ExistingFile`** لحقل `path`: ذلك يفحص وجود الملف
//! على القرص الآن، وهو سؤالٌ لا صلة له بما تفعله هذه العملية — بل قد يرفض
//! بالضبط الحالة التي وُجدت العملية من أجلها. حقل `path` نصٌّ يُتحقَّق ببنيته
//! وحدها (‏`checked_history_path`)، والجواب عن «هل هذا الملف موجودٌ فعلًا عند
//! هذا المرجع؟» يأتي من `git` نفسها وقت التنفيذ: رمز خروجٍ غير صفري ورسالةٌ
//! في نافذة الخرج، لا رفضٌ عند التخطيط.
//!
//! ## وما لا تتحقّق منه هذه العملية
//!
//! لا تسأل هل `ref` مرجعٌ صحيحٌ موجود فعلًا، ولا هل `path` موجودٌ عند ذلك
//! المرجع. الجوابان يحتاجان تشغيل `git` — والتخطيط لا يشغّل شيئًا. وفي
//! الحالين يخرج الأمر برمزٍ غير صفري وتقول الشاشة «لم تكتمل العملية».
//!
//! ## لا تكتب شيئًا
//!
//! قراءةٌ محضة: لا ناتج ولا مؤقّت. «إظهار في Finder» يفتح مجلد المستودع نفسه.

use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "git.show.file";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.show.file.title",
    description_key: "op.git.show.file.description",
    category: Category::Git,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("repo", InputKind::ExistingDir),
        // ١٠٠ بايتٍ: نفس حدّ `revision` في `git_archive.rs`، لنفس السبب —
        // سخيّ فوق أطول اسم مرجعٍ معقول وفوق بصمة commit الكاملة معًا.
        InputSpec::new("ref", InputKind::Text { max_len: 100 }),
        // ٤٠٩٦ بايتًا: حدّ المسار الذي يفرضه النظام نفسه على macOS
        // (‏`PATH_MAX`)، لا رقمٌ اختير جزافًا.
        InputSpec::new("path", InputKind::Text { max_len: 4096 }),
    ],
    sort_order: 90,
    search_terms: &[
        "git",
        "show",
        "history",
        "file",
        "commit",
        "عرض",
        "ملفّ",
        "تاريخ",
        "نسخة",
        "قديمة",
        "مستودع",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let repo = inputs.dir("repo")?;
    require_repo(repo, "repo")?;
    let revision = checked_revision(inputs.text("ref")?)?;
    let path = checked_history_path(inputs.text("path")?)?;
    // مُتحقَّق أعلاه أن كلًّا منهما مقصوصٌ ولا يحمل ما يفسد بنية الوسيط. الدمج
    // هنا وحده — لا في `Argv` — لأن اللصق نفسه ليس فحصًا، بل تركيب قيمتين
    // فُحصتا كلٌّ على حدة.
    let combined = format!("{revision}:{path}");

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(repo)
        .flag("show", "explain.git.show")
        .explained_value(combined, "explain.git.show.target")
        .reveal(repo);

    if let Some(key) = warn_if_resolved(inputs, "repo", repo, "warn.git.repo.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

/// يفحص المرجع ويعيده مقصوصَ الطرفين. لا ينقّيه ولا يعيد كتابته.
///
/// نسخةٌ من `checked_revision` في `git_archive.rs` بحرفها، بفارقٍ واحد: هذه
/// العملية تملك حقل مرجعٍ واحدًا فقط، فاسمه (`"ref"`) مكتوبٌ هنا مباشرةً بدل
/// أن يُمرَّر معاملًا لا يستعمله أحدٌ غيره.
///
/// المرجع نصٌّ تفهمه `git` وحدها: `HEAD`، أو وسمٌ (`v1.2.0`)، أو فرع
/// (`main`، `feature/x`)، أو بصمة commit. ولذلك **لا يُرفض `/`** هنا وإن كان
/// يُرفض في `checked_history_path`: هو فاصلٌ مشروع في أسماء المراجع، ورفضُه
/// كان سيمنع أكثر أسماء الفروع شيوعًا.
///
/// وما يُرفض:
///
/// * **الفارغ** — `git show :` بمرجعٍ فارغ صيغةٌ لا تعني ما يقصده هذا الحقل،
///   والرفض هنا يسبق `git` كي يحمل اسم الحقل الذي يصلحه المستخدم.
/// * **محرف الصفر** — لا يعبر حدّ استدعاء النظام أصلًا.
/// * **محارف التحكّم والفراغ** — أسماء المراجع في Git لا تحتملها أصلًا
///   (`git check-ref-format`)، وسطرٌ جديد أو مسافةٌ داخل وسيطٍ واحد يجعل
///   «سَطْر» يعرض رمزين حيث يوجد رمز، فيُقرأ غير ما يُنفَّذ.
/// * **ما يبدأ بشرطة** — قد يُقرأ رايةً من رايات `git show`.
/// * **ما فيه `..`** — `a..b` مدًى لا مرجعًا مفردًا، و`git show` تنتظر مرجعًا
///   واحدًا هنا فترفضه. ورفضُه في الحقل أوضح من رسالةٍ من الأداة في نافذة
///   الخرج، ويمنع كذلك أن يظهر في الأمر المعروض رمزٌ يشبه صعودًا في المسارات.
///
/// ## نسبة الرفض إلى سببٍ من مجموعة مغلقة
///
/// كما في `git_archive.rs`: `NameRejection` كُتبت لأسماء الملفات، فالفراغ
/// منسوبٌ إلى `ContainsControl`، والشرطة البادئة إلى `LeadingDot` — أقرب
/// الموجود معنًى: «حرفٌ في الصدر يغيّر معنى ما بعده».
fn checked_revision(raw: &str) -> Result<&str> {
    let rejected = |reason: NameRejection| CoreError::InvalidName { reason }.on_input("ref");
    // القصّ قبل الفحص: من ينسخ بصمة commit من مخرجٍ ينسخ معه فراغًا طرفيًا،
    // ورفضُه على ذلك تشدّدٌ لا يحمي شيئًا.
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

/// يفحص المسار النسبي داخل تاريخ Git ويعيده مقصوصَ الطرفين. لا يعيد كتابته.
///
/// هذا ليس مسار ملفٍّ على القرص الآن — انظر الشرح أعلى الملفّ — بل اسمٌ داخل
/// شجرةٍ مسجَّلة عند مرجعٍ بعينه، قد لا يقابله شيءٌ في مساحة العمل الحالية.
/// ولذلك قواعده قواعد **مسار** لا قواعد **مرجع** كما في `checked_revision`،
/// وأبرز فرق: الفراغ العادي مقبولٌ هنا.
///
/// وما يُرفض:
///
/// * **الفارغ** (بعد القصّ) — `git show <مرجع>:` بمسارٍ فارغ صيغةٌ مختلفة
///   تمامًا (تعرض شجرة الجذر عند ذلك المرجع)، وليست ما يقصده حقلٌ اسمه
///   «المسار».
/// * **محرف الصفر** — لا يعبر حدّ استدعاء النظام أصلًا.
/// * **محارف التحكّم — لا الفراغ العادي.** خلافًا لـ`checked_revision`: فراغٌ
///   واحد داخل اسم ملفّ («‏My File.txt») مشروعٌ تمامًا وليس فاصلًا يُلبس الأمر
///   معنًى آخر كما في اسم مرجع. أمّا سطرٌ جديد أو جدولة فتجعل «سَطْر» يعرض
///   الوسيط الواحد على أكثر من سطر، فيُقرأ غير ما يُنفَّذ.
/// * **ما يبدأ بـ`/`** — المسار هنا نسبيٌّ داخل الشجرة لا مطلقًا. المسار الذي
///   يقبله هذا الحقل هو ما يظهر في `git ls-tree` نسبيًا لا أكثر، وبادئةٌ
///   بشرطةٍ مائلة صيغةٌ مختلفة لا داعي لفتحها هنا.
/// * **`..` كعنصر مسارٍ كامل** — بين فاصلي `/`، أو في البداية، أو في النهاية.
///   الفحص يقسّم المسار على `/` ويفحص كل جزءٍ بمفرده لا الاحتواء الخام —
///   استلهامًا من `reject_dotdot` في `paths.rs`، لكن بفحص كل مكوّنٍ لا مكوّن
///   `ParentDir` من `std::path::Component` وحده، لأن المسار هنا نصٌّ داخل
///   شجرة Git لا مسارًا حقيقيًا يُبنى بـ`Path::components()` على هذا النظام.
///   اسمُ ملفٍّ مشروع مثل `a..b.txt` يحوي النصّ `..` دون أن يكون صعودًا في
///   المسارات، ورفضه كان سيرفض ملفاتٍ حقيقية بلا سبب. أمّا `..` عنصرًا كاملًا
///   فيعني صعودًا خارج الشجرة التي يعرضها `git show`، ولا معنى صحيحًا له هنا.
///
/// ## نسبة الرفض إلى سببٍ من مجموعة مغلقة
///
/// نفس ملاحظة `checked_revision`: `NameRejection` كُتبت لأسماء الملفات، وليس
/// فيها عضوٌ باسم «مسارٌ مطلق حيث يُنتظر نسبي». فالبادئة `/` منسوبةٌ إلى
/// `LeadingDot` بالمنطق نفسه الذي نُسبت به الشرطة البادئة في `checked_revision`:
/// «حرفٌ في الصدر يغيّر معنى ما بعده» — وهنا الحرف `/` يحوّل المسار من نسبيّ
/// إلى ما يشبه المطلق. و`..` عنصرًا كاملًا منسوبٌ إلى `DotOrDotDot`، وهذا
/// تطابقٌ حقيقي لا تقريب: هو الاسم نفسه المرفوض بالسبب نفسه في
/// `paths::sanitize_name`.
fn checked_history_path(raw: &str) -> Result<&str> {
    let rejected = |reason: NameRejection| CoreError::InvalidName { reason }.on_input("path");
    let path = raw.trim();

    if path.is_empty() {
        return Err(rejected(NameRejection::Empty));
    }
    // قبل فحص محارف التحكّم كي يُسمّى الصفر باسمه: كلاهما محرف تحكّم، ولأحدهما
    // صنفٌ خاصّ وسببٌ خاص.
    if path.contains('\0') {
        return Err(rejected(NameRejection::ContainsNul));
    }
    // لا `is_whitespace()` هنا خلافًا لـ`checked_revision`: الفراغ العادي جزءٌ
    // مشروع من اسم ملفّ، وليس فاصلًا بين رموزٍ منفصلة.
    if path.chars().any(char::is_control) {
        return Err(rejected(NameRejection::ContainsControl));
    }
    if path.starts_with('/') {
        return Err(rejected(NameRejection::LeadingDot));
    }
    if path.split('/').any(|component| component == "..") {
        return Err(rejected(NameRejection::DotOrDotDot));
    }
    Ok(path)
}

/// يتحقّق أن المجلد المختار **جذرُ** مستودع، وينسب الرفض إلى حقله.
///
/// `symlink_metadata` لا `is_dir`: في شجرة عملٍ إضافية وفي الوحدات الفرعية
/// يكون `.git` ملفًا نصّيًا يشير إلى موضع المستودع، فشرطُ «مجلد» كان سيرفض
/// مستودعًا سليمًا. والجذرُ وحده مقبول: `git -C <فرعي> show` تصعد إلى الجذر
/// وتقرأ تاريخ المستودع كله بينما تقول الشاشة اسم المجلد الفرعي.
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

    fn plan_with(repo: &Path, reference: &str, path: &str) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("repo".to_owned(), RawValue::Path(repo.display().to_string())),
            ("ref".to_owned(), RawValue::Text(reference.to_owned())),
            ("path".to_owned(), RawValue::Text(path.to_owned())),
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
        let found = listed.iter().find(|o| o.id == ID).expect("git.show.file must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_the_reference_and_path_are_one_argument() {
        let s = Scratch::new("git-show-file-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };

        let cmd = plan_with(&repo, "HEAD", "src/main.rs").unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 4);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "show");
        assert_eq!(args[3], "HEAD:src/main.rs");

        assert!(cmd.artifact.is_none());
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
        assert_eq!(cmd.reveal_target.as_deref(), Some(repo.as_path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-show-file-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "HEAD", "README.md").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-show-file-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "HEAD", "README.md").unwrap();
        for token in cmd.explain.iter().filter(|t| t.role != TokenRole::Path) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-show-file-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };

        let cmd = plan_with(&repo, "HEAD", "README.md").unwrap();
        let chosen = &cmd.args[1];
        assert!(cannot_be_read_as_a_flag(chosen), "{chosen:?} would be read as a flag");
    }

    /// المسار قد يبدأ بشرطة بذاته، لكن المرجع دائمًا أوّلًا في الوسيط الملتصق
    /// والمرجع مُتحقَّق أنه لا يبدأ بشرطة — فالوسيط الناتج لا يمكن أن يُقرأ
    /// رايةً مهما كان المسار.
    #[test]
    fn a_path_that_looks_like_a_flag_is_still_safe_because_the_reference_leads() {
        let s = Scratch::new("git-show-file-path-dash").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "HEAD", "-rf").unwrap();
        assert_eq!(cmd.args[3], "HEAD:-rf");
        assert!(cannot_be_read_as_a_flag(&cmd.args[3]));
    }

    #[test]
    fn shell_syntax_in_the_repository_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-show-file-shellish-repo").unwrap();

        let shellish = ["مستودع 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"];
        for name in shellish {
            let Some(repo) = repo_in(&s, name) else { return };
            let cmd = plan_with(&repo, "HEAD", "README.md").unwrap();
            assert_eq!(cmd.args.len(), 4, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), repo.as_path());
        }
    }

    #[test]
    fn shell_syntax_and_spaces_in_the_path_stay_inside_one_combined_argument() {
        let s = Scratch::new("git-show-file-shellish-path").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        // الفراغ مقبولٌ في المسار خلافًا للمرجع.
        for path in ["ملف 'اليوم'.txt", "a; rm -rf ~", "$(whoami)", "back`tick`", "with space.txt"]
        {
            let cmd = plan_with(&repo, "HEAD", path).unwrap();
            assert_eq!(cmd.args.len(), 4, "{path:?} must not add arguments");
            assert_eq!(cmd.args[3].to_string_lossy(), format!("HEAD:{path}"));
        }
    }

    #[test]
    fn a_name_with_two_dots_that_is_not_a_full_path_component_is_accepted() {
        let s = Scratch::new("git-show-file-dotted-name").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        for path in ["a..b.txt", "src/v1..2/file.txt", "..hidden"] {
            let cmd = plan_with(&repo, "HEAD", path).unwrap();
            assert_eq!(
                cmd.args[3].to_string_lossy(),
                format!("HEAD:{path}"),
                "{path:?} must not be rewritten"
            );
        }
    }

    #[test]
    fn a_branch_name_with_a_slash_is_accepted_because_it_is_a_reference_not_a_file_name() {
        let s = Scratch::new("git-show-file-slash-ref").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "refs/tags/v1.0", "README.md").unwrap();
        assert_eq!(cmd.args[3], "refs/tags/v1.0:README.md");
    }

    #[test]
    fn the_reference_and_the_path_are_each_trimmed_at_their_edges_and_never_rewritten_inside() {
        let s = Scratch::new("git-show-file-trim").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        let cmd = plan_with(&repo, "  HEAD  ", "  src/main.rs  ").unwrap();
        assert_eq!(cmd.args[3], "HEAD:src/main.rs");
    }

    #[test]
    fn an_empty_reference_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-show-file-empty-ref").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["", "   ", "\t"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, "README.md")),
                ("err.name.invalid", Some("ref")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&repo, bad, "README.md")), NameRejection::Empty, "{bad:?}");
        }
    }

    #[test]
    fn a_reference_with_whitespace_or_control_characters_is_refused() {
        let s = Scratch::new("git-show-file-space-ref").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["HEAD 1", "v1.0\nv2.0", "a\tb", "بين كلمتين"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, "README.md")),
                ("err.name.invalid", Some("ref")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, bad, "README.md")),
                NameRejection::ContainsControl,
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_reference_carrying_a_nul_is_named_by_its_own_reason() {
        let s = Scratch::new("git-show-file-nul-ref").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(reason(plan_with(&repo, "HEAD\0", "README.md")), NameRejection::ContainsNul);
    }

    #[test]
    fn a_reference_that_starts_with_a_dash_is_refused_before_it_can_be_read_as_a_flag() {
        let s = Scratch::new("git-show-file-dash-ref").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["-o/etc/passwd", "--output=/etc/passwd", "-HEAD"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, "README.md")),
                ("err.name.invalid", Some("ref")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, bad, "README.md")),
                NameRejection::LeadingDot,
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_reference_that_carries_two_dots_is_refused_as_a_range_not_a_single_revision() {
        let s = Scratch::new("git-show-file-range-ref").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["main..feature", "v1.0...v2.0"] {
            assert_eq!(
                refusal(plan_with(&repo, bad, "README.md")),
                ("err.name.invalid", Some("ref")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, bad, "README.md")),
                NameRejection::DotOrDotDot,
                "{bad:?}"
            );
        }
    }

    #[test]
    fn an_over_long_reference_is_refused_by_the_declared_limit() {
        let s = Scratch::new("git-show-file-long-ref").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let long = "a".repeat(101);
        assert_eq!(refusal(plan_with(&repo, &long, "README.md")), ("err.input.type", Some("ref")));
    }

    #[test]
    fn an_empty_path_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-show-file-empty-path").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["", "   ", "\t"] {
            assert_eq!(
                refusal(plan_with(&repo, "HEAD", bad)),
                ("err.name.invalid", Some("path")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&repo, "HEAD", bad)), NameRejection::Empty, "{bad:?}");
        }
    }

    #[test]
    fn a_path_carrying_a_nul_is_named_by_its_own_reason() {
        let s = Scratch::new("git-show-file-nul-path").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(reason(plan_with(&repo, "HEAD", "a\0b")), NameRejection::ContainsNul);
    }

    #[test]
    fn a_path_with_control_characters_is_refused_but_a_plain_space_is_not() {
        let s = Scratch::new("git-show-file-control-path").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["a\nb", "a\tb", "a\rb"] {
            assert_eq!(
                refusal(plan_with(&repo, "HEAD", bad)),
                ("err.name.invalid", Some("path")),
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
    fn an_absolute_path_is_refused_because_the_target_is_relative_inside_the_repository() {
        let s = Scratch::new("git-show-file-absolute-path").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["/etc/passwd", "/README.md"] {
            assert_eq!(
                refusal(plan_with(&repo, "HEAD", bad)),
                ("err.name.invalid", Some("path")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&repo, "HEAD", bad)), NameRejection::LeadingDot, "{bad:?}");
        }
    }

    #[test]
    fn a_path_that_escapes_with_dotdot_as_a_full_component_is_refused() {
        let s = Scratch::new("git-show-file-dotdot-path").unwrap();
        let repo = marked_repo(&s, "مستودع");

        for bad in ["..", "../etc/passwd", "a/../b", "a/b/.."] {
            assert_eq!(
                refusal(plan_with(&repo, "HEAD", bad)),
                ("err.name.invalid", Some("path")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&repo, "HEAD", bad)),
                NameRejection::DotOrDotDot,
                "{bad:?}"
            );
        }
    }

    #[test]
    fn an_over_long_path_is_refused_by_the_declared_limit() {
        let s = Scratch::new("git-show-file-long-path").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let long = "a".repeat(4097);
        assert_eq!(refusal(plan_with(&repo, "HEAD", &long)), ("err.input.type", Some("path")));
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-show-file-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        assert_eq!(
            refusal(plan_with(&plain, "HEAD", "README.md")),
            ("err.path.not_dir", Some("repo"))
        );
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-show-file-missing").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"), "HEAD", "README.md")),
            ("err.path.missing", Some("repo"))
        );
    }

    #[test]
    fn a_repository_outside_the_allowed_roots_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("/etc"), "HEAD", "README.md")),
            ("err.path.outside", Some("repo"))
        );
    }

    #[test]
    fn a_relative_repository_path_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("نسبي/هنا"), "HEAD", "README.md")),
            ("err.path.relative", Some("repo"))
        );
    }

    #[test]
    fn a_symlinked_repository_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("git-show-file-symlink").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "HEAD", "README.md").unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_repository_raises_no_warning_at_all() {
        let s = Scratch::new("git-show-file-quiet").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        assert_eq!(plan_with(&repo, "HEAD", "README.md").unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn planning_writes_nothing_into_the_repository() {
        let s = Scratch::new("git-show-file-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let before = s.names(&repo);

        for _ in 0..10 {
            plan_with(&repo, "HEAD", "README.md").unwrap();
        }
        assert_eq!(s.names(&repo), before, "planning must not touch the repository");
    }
}
