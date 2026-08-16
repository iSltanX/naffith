//! إنشاء مستودع Git داخل مجلدٍ قائم، باستخدام `git init`.
//!
//! ## `git` قد لا تكون على هذا الجهاز
//!
//! هذه أول عمليات قسم Git، وفيها يُقال ما يسري على الستّ جميعًا: `git` على
//! macOS ليست جزءًا من النظام الأساسي، بل تأتي مع أدوات سطر الأوامر في Xcode.
//! والموجود في `/usr/bin/git` على جهازٍ لم تُثبَّت فيه هذه الأدوات وسيطٌ يسأل
//! المستخدم أن يثبّتها. ولهذا وُجد `Availability`: الفهرس يحلّ الأداة لحظة
//! قراءته، فتظهر العملية **معطّلةً باسم أداتها الغائبة** لا مخفيّةً. إخفاؤها كان
//! سيجعل المستخدم يظنّ أن المنتج لا يعرف Git أصلًا، بدل أن يعرف أن أمرًا واحدًا
//! ينقصه (`xcode-select --install`).
//!
//! ## ولماذا `-C <المجلد>` في كل أمرٍ من أوامر هذا القسم
//!
//! `git` أداةٌ تعمل على «المستودع الذي أنا فيه»: تبحث عن `.git` في مجلد العمل
//! ثم فيما فوقه. والتطبيق **لا يغيّر مجلد عمله أبدًا** — عمليةٌ تفعل ذلك تجعل
//! كل ما يليها في العملية نفسها رهنَ حالةٍ عامّة غير مرئية في الأمر المعروض.
//!
//! `-C <المسار>` تقول لـ`git` نفسها: انتقلي إلى هذا المجلد قبل أن تفعلي شيئًا.
//! فالمجلد الذي سيُعمل عليه **مكتوبٌ في الأمر أمام المستخدم**، لا مستنتَجٌ من
//! سياقٍ لا يراه. والبديل — `Argv::in_directory` — كان ينقل المعلومة إلى حقلٍ
//! خارج `argv`، فيصير «سَطْر» يعرض `git init` بلا أن يقول أين.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المجلد> init
//! ```
//!
//! و`init` تدخل الأمر عبر `flag` لا `value`، وهي ليست رايةً بحرفها. السبب أنها
//! ليست بيانات المستخدم كذلك: هي رمزٌ من رموز الأمر نفسه، اختاره التطبيق لا
//! المستخدم. ولو دخلت `value` لظهرت في «سَطْر» بلون القيم — أي بلون ما كتبه
//! المستخدم — فيبدو اسم الأمر الفرعي كأنه مدخلٌ يمكن تغييره.
//!
//! ## ما ترفضه هذه العملية
//!
//! مجلدٌ فيه `.git` أصلًا. `git init` على مستودعٍ قائم لا يتلف شيئًا — تعيد
//! كتابة بعض ملفات الإعداد وتخرج بنجاح — لكنها **ليست ما قصده** من اختار
//! «أنشئ مستودعًا هنا». الرفض هنا يقول له إن المجلد مستودعٌ بالفعل، بدل أن
//! تقول الشاشة «تمّ» عن أمرٍ لم يفعل ما فهمه منه.
//!
//! ## وما لا تفحصه
//!
//! لا تسأل هل المجلد **داخل** مستودعٍ أكبر. `git init` في مجلدٍ فرعي من مستودع
//! تُنشئ مستودعًا متداخلًا — وهو شيءٌ نادرًا ما يُقصد. لكن الجواب يحتاج صعودًا
//! في الآباء إلى ما فوق الجذور المسموحة، وحكمًا على مجلداتٍ لم يخترها المستخدم.
//! فالحدّ معلَنٌ في الوصف بدل أن يُخمَّن في الشيفرة.

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "git.init";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.init.title",
    description_key: "op.git.init.description",
    category: Category::Git,
    danger: Danger::Creates,
    visibility: Visibility::Production,
    tool: tools::GIT,
    // لا ناتج مؤقّت يُرقّى: ما تُنشئه `git init` مجلد `.git` داخل مجلدٍ اختاره
    // المستخدم، لا ملفًا باسمٍ قد يتضارب. والأثر معلَنٌ في `Danger::Creates`.
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("folder", InputKind::TargetDir)],
    sort_order: 10,
    search_terms: &[
        "git",
        "init",
        "مستودع",
        "repository",
        "repo",
        "تهيئة",
        "إنشاء مستودع",
        "version control",
        "إصدارات",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    // `TargetDir` لا `ExistingDir`: `git init` تكتب في المجلد، وفحص الكتابة في
    // `paths::target_dir` محاولةٌ فعلية لا قراءةُ بتات صلاحيات — فقرصٌ للقراءة
    // فقط أو قاعدة ACL تُكتشف هنا لا بعد أن يُعرض الأمر ويُشغَّل.
    let folder = inputs.target_dir("folder")?;

    // `symlink_metadata` لا `metadata`: `.git` قد يكون مجلدًا (المستودع
    // المعتاد)، أو ملفًا نصّيًا (شجرة عملٍ إضافية أو وحدة فرعية)، أو رابطًا
    // معلَّقًا. الثلاثة تعني «هنا مستودعٌ أو أثرُ مستودع»، والفحص الذي لا يتبع
    // الروابط هو وحده الذي يراها كلها.
    if std::fs::symlink_metadata(folder.join(".git")).is_ok() {
        return Err(CoreError::DestinationExists.on_input("folder"));
    }

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(folder)
        .flag("init", "explain.git.init")
        // لا `Artifact`، فسؤال «أين أنظر؟» يحتاج جوابًا معلَنًا.
        .reveal(folder);

    if let Some(key) = warn_if_resolved(inputs, "folder", folder, "warn.git.folder.resolved") {
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

    fn plan_with(folder: &Path) -> Result<PlannedCommand> {
        let raw =
            BTreeMap::from([("folder".to_owned(), RawValue::Path(folder.display().to_string()))]);
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

    /// هل تُحلّ `git` على هذه الآلة؟
    ///
    /// التخطيط **يحلّ** الأداة ولا يشغّلها، فوجود الملف التنفيذي هو الشرط الذي
    /// يفرّق بين خطةٍ تُبنى وخطةٍ تُرفض بـ`ToolMissing`. وعلى macOS يوجد
    /// `/usr/bin/git` عادةً حتى بلا أدوات Xcode — وسيطًا يطلب تثبيتها — فتُحلّ
    /// الأداة وتُبنى الخطة، وهو الصواب: العملية تُعرض متاحة، والفشل إن وقع يقع
    /// عند التنفيذ لا عند التخطيط. والفحص هنا للحالة النادرة التي لا يوجد فيها
    /// الملف أصلًا، فتُترك الآلة وشأنها بدل أن يسقط اختبارٌ على ما ليس عيبًا في
    /// الشيفرة.
    fn git_resolves() -> bool {
        Path::new("/usr/bin/git").exists()
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("git.init must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Creates);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_names_the_folder_inside_the_command() {
        if !git_resolves() {
            return;
        }
        let s = Scratch::new("git-init-argv").unwrap();
        let folder = s.dir("مشروع");

        let cmd = plan_with(&folder).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 3);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), folder.as_path());
        assert_eq!(args[2], "init");

        assert!(cmd.artifact.is_none(), "git init writes inside a folder the user already owns");
        assert!(cmd.stdout_to.is_none());
        assert!(
            cmd.cwd.is_none(),
            "the working directory is said in the argv with -C, never set behind it"
        );
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
        assert!(!folder.join(".git").exists(), "planning must create nothing");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        if !git_resolves() {
            return;
        }
        let s = Scratch::new("git-init-explain").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        if !git_resolves() {
            return;
        }
        let s = Scratch::new("git-init-keys").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder).unwrap();
        for token in cmd.explain.iter().filter(|t| t.role != TokenRole::Path) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        if !git_resolves() {
            return;
        }
        let s = Scratch::new("git-init-dashes").unwrap();
        let folder = s.dir("-rf");

        let cmd = plan_with(&folder).unwrap();
        // الموضع ٠ رايةٌ معلَنة، والموضع ٢ اسم الأمر الفرعي. ما عداهما بيانات،
        // ومسارٌ اسمه `-rf` يبقى مطلقًا فيبدأ بشرطة مائلة لا بشرطة.
        let chosen = &cmd.args[1];
        assert!(cannot_be_read_as_a_flag(chosen), "{chosen:?} would be read as a flag");
    }

    #[test]
    fn shell_syntax_in_the_folder_name_is_carried_literally_into_one_argument() {
        if !git_resolves() {
            return;
        }
        let s = Scratch::new("git-init-shellish").unwrap();

        for name in ["مشروع 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let folder = s.dir(name);
            let cmd = plan_with(&folder).unwrap();
            assert_eq!(cmd.args.len(), 3, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), folder.as_path());
        }
    }

    /// المستودع القائم يُرفض، ولا يُعاد إنشاؤه فوق نفسه.
    ///
    /// لا يحتاج هذا الاختبار أن تكون `git` مثبَّتة: الرفض يقع قبل حلّ الأداة.
    #[test]
    fn a_folder_that_is_already_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-init-exists").unwrap();
        let folder = s.dir("مستودع");
        s.dir("مستودع/.git");

        assert_eq!(refusal(plan_with(&folder)), ("err.dest.exists", Some("folder")));
    }

    #[test]
    fn a_git_file_left_by_a_worktree_counts_as_a_repository_too() {
        // في شجرة عملٍ إضافية وفي الوحدات الفرعية يكون `.git` ملفًا لا مجلدًا.
        // شرط «مجلد» كان سيمرّ فوقه ويُنشئ مستودعًا فوق مستودع.
        let s = Scratch::new("git-init-worktree").unwrap();
        let folder = s.dir("شجرة");
        std::fs::write(folder.join(".git"), b"gitdir: /somewhere/else").unwrap();

        assert_eq!(refusal(plan_with(&folder)), ("err.dest.exists", Some("folder")));
    }

    #[test]
    fn a_dangling_git_symlink_counts_as_a_repository_too() {
        let s = Scratch::new("git-init-dangling").unwrap();
        let folder = s.dir("مجلد");
        std::os::unix::fs::symlink(s.path().join("لا-وجود-له"), folder.join(".git")).unwrap();

        assert_eq!(refusal(plan_with(&folder)), ("err.dest.exists", Some("folder")));
    }

    #[test]
    fn a_folder_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-init-missing").unwrap();
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"))),
            ("err.path.missing", Some("folder"))
        );
    }

    #[test]
    fn a_folder_outside_the_allowed_roots_is_refused() {
        assert_eq!(refusal(plan_with(Path::new("/etc"))), ("err.path.outside", Some("folder")));
    }

    #[test]
    fn a_relative_folder_is_refused() {
        assert_eq!(
            refusal(plan_with(Path::new("نسبي/هنا"))),
            ("err.path.relative", Some("folder"))
        );
    }

    #[test]
    fn a_symlinked_folder_is_resolved_and_the_substitution_is_announced() {
        if !git_resolves() {
            return;
        }
        let s = Scratch::new("git-init-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.folder.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn an_ordinary_folder_raises_no_warning_at_all() {
        if !git_resolves() {
            return;
        }
        let s = Scratch::new("git-init-quiet").unwrap();
        let folder = s.dir("مشروع");
        assert_eq!(plan_with(&folder).unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn planning_creates_no_repository_of_its_own() {
        if !git_resolves() {
            return;
        }
        let s = Scratch::new("git-init-clean").unwrap();
        let folder = s.dir("مشروع");

        for _ in 0..10 {
            plan_with(&folder).unwrap();
        }
        assert_eq!(s.names(&folder), Vec::<String>::new(), "planning must write nothing");
    }
}
