//! من كتب كل سطرٍ في ملف، ومتى، وفي أي commit، باستخدام `git blame`.
//!
//! غياب `git` عن الجهاز وسببُ `-C <المستودع>` في كل أوامر هذا القسم مشروحان في
//! رأس `git_init.rs`، ولا يُعادان هنا.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/git -C <المستودع> blame --line-porcelain -- <الملف>
//! ```
//!
//! ## لماذا `--line-porcelain` لا الصيغة الافتراضية
//!
//! الصيغة الافتراضية لـ`git blame` تطبع سطرًا يشبه هذا:
//!
//! ```text
//! ^a1b2c3d (اسم المؤلف الكامل 2024-01-15 10:00:00 +0300  42) نص السطر
//! ```
//!
//! والمؤلف والتاريخ ورقم السطر كلها مضغوطةٌ داخل قوسٍ واحد، بفراغاتٍ فاصلة لا
//! حرفَ تحديدٍ واحدًا يُعتمد عليه: اسم المؤلف نفسه قد يحوي فراغًا — وهو الشائع —
//! بل وقوس إغلاقٍ في اسمٍ مستعار، فلا حدّ أمينٌ بين «هنا ينتهي الاسم» و«هنا
//! يبدأ التاريخ». من يريد قراءة هذا الخرج برمجيًا — وهذا بالضبط ما ستفعله شاشة
//! النتيجة لاحقًا — يحتاج تخمين حدود الحقول بتعبيرٍ نمطي هشّ، لا تحليلها بثقة.
//!
//! `--line-porcelain` تكسر كل سطرٍ منطقيٍّ إلى عدّة أسطر خرج، كلٌّ منها حقلٌ
//! واحد بعنوانٍ صريح في بدايته (`author `، `author-time `، `summary `،
//! `committer `، وغيرها)، قبل أن يظهر سطر المحتوى نفسه — ودومًا **مسبوقًا بحرف
//! tab**. هذا تحليلٌ حتميٌّ لا تخمينٌ بالفراغات: من سيقرأ هذا الخرج لاحقًا (في
//! `result.rs`، خارج هذا الملفّ تمامًا) يعرف بيقين أين ينتهي كل حقل، بحرف tab
//! نفسه لا بعدّ الأقواس.
//!
//! والقرار هنا **لصالح من سيحلّل الخرج لاحقًا لا لهذا الملفّ نفسه**: هذا
//! الملفّ لا يقرأ حرفًا واحدًا مما تطبعه `git`، هو يبني الأمر الصحيح فقط. لكن
//! الصيغة يجب أن تُختار هنا، عند من يعرف أي أداةٍ ستُشغَّل وبأي رايات، لا
//! تُترك ليُعاد اختيارها عند من سيحلّل خرجها لاحقًا بلا سلطةٍ على الأمر نفسه.
//!
//! ## `--` قبل اسم الملف
//!
//! تفصل رايات `git blame` عن اسم الملف، احتياطًا لملفٍّ اسمه يبدأ بشرطة —
//! حالةٌ نادرة لكن ممكنة — بنفس المنطق المستعمل في هذا المشروع لفصل الرايات
//! عن القيم في مواضع أخرى (انظر `text_search.rs` و`disk_hash.rs`).
//!
//! ## ما لا تفحصه هذه العملية
//!
//! لا تتحقّق صراحةً أن `path` يقع داخل `repo` تحديدًا. المدخلان يُتحقَّق من
//! كلٍّ منهما على حدة — `repo` جذرُ مستودعٍ قائم، و`path` ملفٌّ قائم فعلًا على
//! القرص الآن — ولا فحصٌ يربط بينهما. إن أعطى المستخدم مسارًا خارج المستودع
//! الذي اختاره، فـ`git blame` نفسها ترفض الأمر بخطأ واضح، ويظهر ذلك في نافذة
//! الخرج بعد التشغيل لا قبله.
//!
//! وهذا مقبولٌ ومتعمَّد، نظير القرار في رأس `git_commit.rs` (فقرة «ما لا
//! تفحصه هذه العملية»): بعض الأسئلة أجدر أن تُترك لتُجيب عنها `git` نفسها لا
//! أن يُعاد بناء منطقها في Rust. والسؤال هنا — «هل هذا الملف من هذا المستودع؟»
//! — سؤالٌ عن شجرة الملفات تعرفه `git` تمام المعرفة، وإعادة صياغته هنا تعني
//! مقارنة مسارين بعد حلّ الروابط والصعود في الآباء: منطقٌ تملكه `git` أصلًا،
//! ولا حاجة لتكراره كي يُختبر مرّتين وقد يختلف الجوابان.
//!
//! خلافًا لعمليةٍ تقرأ ملفًا من تاريخ Git بمرجعٍ (كـ«عرض ملفٍ من commit»)،
//! `path` هنا يجب أن يكون **قائمًا على القرص الآن**: `InputKind::ExistingFile`
//! تفحص ذلك عند التحقّق من المدخلات، قبل أن يصل شيءٌ إلى `plan`.

use crate::error::{CoreError, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "git.blame";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.git.blame.title",
    description_key: "op.git.blame.description",
    category: Category::Git,
    // قراءةٌ محضة: تسند كل سطرٍ إلى commit وتطبع، ولا تكتب بايتًا واحدًا.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GIT,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("repo", InputKind::ExistingDir),
        InputSpec::new("path", InputKind::ExistingFile),
    ],
    sort_order: 100,
    search_terms: &[
        "git",
        "blame",
        "author",
        "history",
        "line",
        "مؤلف",
        "تاريخ",
        "سطر",
        "من",
        "كتب",
        "مستودع",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let repo = inputs.dir("repo")?;
    require_repo(repo, "repo")?;
    let path = inputs.file("path")?;

    let mut argv = Argv::tool(tools::GIT, "explain.git.tool")
        .flag("-C", "explain.git.dash_c")
        .path(repo)
        .flag("blame", "explain.git.blame")
        .flag("--line-porcelain", "explain.git.blame.porcelain")
        .end_of_flags()
        .path(path)
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
/// وتُسنِد كل سطرٍ إلى commit من تاريخ المستودع كله، بينما تقول الشاشة اسم
/// المجلد الفرعي.
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

    fn plan_with(repo: &Path, path: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("repo".to_owned(), RawValue::Path(repo.display().to_string())),
            ("path".to_owned(), RawValue::Path(path.display().to_string())),
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
    /// حلّ الأداة (أو لا تصل إلى `plan` أصلًا)، فتمرّ على كل جهاز.
    fn marked_repo(s: &Scratch, name: &str) -> PathBuf {
        let dir = s.dir(name);
        s.dir(&format!("{name}/.git"));
        dir
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("git.blame must be listed");
        assert_eq!(found.category, Category::Git);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.tool, "git");
    }

    #[test]
    fn the_argv_is_the_documented_form() {
        let s = Scratch::new("git-blame-argv").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };
        let file = s.file("مستودع/ملف.rs", b"fn main() {}\n");

        let cmd = plan_with(&repo, &file).unwrap();
        let args = args_of(&cmd);

        assert_eq!(cmd.program, Path::new("/usr/bin/git"));
        assert_eq!(args.len(), 6);
        assert_eq!(args[0], "-C");
        assert_eq!(Path::new(&args[1]), repo.as_path());
        assert_eq!(args[2], "blame");
        assert_eq!(args[3], "--line-porcelain");
        assert_eq!(args[4], "--");
        assert_eq!(Path::new(&args[5]), file.as_path());

        assert!(cmd.artifact.is_none());
        assert!(cmd.cwd.is_none(), "the repository is named in the argv, not behind it");
        assert!(cmd.stdout_to.is_none());
        assert_eq!(cmd.reveal_target.as_deref(), Some(repo.as_path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("git-blame-explain").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let file = s.file("م/ملف.rs", b"x");

        let cmd = plan_with(&repo, &file).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_token_the_operation_chose_carries_its_own_explanation() {
        let s = Scratch::new("git-blame-keys").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let file = s.file("م/ملف.rs", b"x");

        let cmd = plan_with(&repo, &file).unwrap();
        // المساران بيانات لا رموزًا اختارها التطبيق، وما عداهما يحمل مفتاح شرح.
        for token in cmd.explain.iter().filter(|t| t.role != TokenRole::Path) {
            assert!(token.key.is_some(), "{} is shown with no explanation", token.token);
        }
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("git-blame-dashes").unwrap();
        let Some(repo) = repo_in(&s, "-rf") else { return };
        let file = s.file("-rf/-ملف.rs", b"x");

        let cmd = plan_with(&repo, &file).unwrap();
        for (i, a) in cmd.args.iter().enumerate() {
            // المواضع ٠ و٢ و٣ و٤ رموزٌ اختارها التطبيق؛ ما عداها بيانات.
            if [0, 2, 3, 4].contains(&i) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "argument {i} ({a:?}) would be read as a flag");
        }
    }

    #[test]
    fn shell_syntax_in_the_repository_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-blame-shellish-repo").unwrap();

        for name in ["مستودع 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"]
        {
            let Some(repo) = repo_in(&s, name) else { return };
            let file = s.file(&format!("{name}/ملف.rs"), b"x");
            let cmd = plan_with(&repo, &file).unwrap();
            assert_eq!(cmd.args.len(), 6, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[1]), repo.as_path());
        }
    }

    #[test]
    fn shell_syntax_in_the_file_name_is_carried_literally_into_one_argument() {
        let s = Scratch::new("git-blame-shellish-file").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };

        for name in ["ملف 'اليوم'.rs", "a;rm.rs", "$(whoami).rs", "back`tick`.rs", "a&b.rs"]
        {
            let file = s.file(&format!("م/{name}"), b"x");
            let cmd = plan_with(&repo, &file).unwrap();
            assert_eq!(cmd.args.len(), 6, "{name:?} must not add arguments");
            assert_eq!(Path::new(&cmd.args[5]), file.as_path());
        }
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-blame-plain").unwrap();
        let plain = s.dir("مجلد عادي");
        let file = s.file("مجلد عادي/ملف.rs", b"x");
        assert_eq!(refusal(plan_with(&plain, &file)), ("err.path.not_dir", Some("repo")));
    }

    #[test]
    fn a_repository_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-blame-missing-repo").unwrap();
        let file = s.file("ملف.rs", b"x");
        assert_eq!(
            refusal(plan_with(&s.path().join("لا-وجود-له"), &file)),
            ("err.path.missing", Some("repo"))
        );
    }

    #[test]
    fn a_repository_outside_the_allowed_roots_is_refused() {
        let s = Scratch::new("git-blame-outside-repo").unwrap();
        let file = s.file("ملف.rs", b"x");
        assert_eq!(
            refusal(plan_with(Path::new("/etc"), &file)),
            ("err.path.outside", Some("repo"))
        );
    }

    #[test]
    fn a_relative_repository_path_is_refused() {
        let s = Scratch::new("git-blame-relative-repo").unwrap();
        let file = s.file("ملف.rs", b"x");
        assert_eq!(
            refusal(plan_with(Path::new("نسبي/هنا"), &file)),
            ("err.path.relative", Some("repo"))
        );
    }

    #[test]
    fn a_file_that_does_not_exist_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-blame-missing-file").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(
            refusal(plan_with(&repo, &s.path().join("لا-وجود-له.rs"))),
            ("err.path.missing", Some("path"))
        );
    }

    #[test]
    fn a_directory_given_as_the_file_is_refused_and_blamed_on_its_field() {
        let s = Scratch::new("git-blame-dir-as-file").unwrap();
        let repo = marked_repo(&s, "مستودع");
        let dir_as_file = s.dir("مستودع/مجلد فرعي");
        assert_eq!(refusal(plan_with(&repo, &dir_as_file)), ("err.path.missing", Some("path")));
    }

    #[test]
    fn a_file_outside_the_allowed_roots_is_refused() {
        let s = Scratch::new("git-blame-outside-file").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(
            refusal(plan_with(&repo, Path::new("/etc/hosts"))),
            ("err.path.outside", Some("path"))
        );
    }

    #[test]
    fn a_relative_file_path_is_refused() {
        let s = Scratch::new("git-blame-relative-file").unwrap();
        let repo = marked_repo(&s, "مستودع");
        assert_eq!(
            refusal(plan_with(&repo, Path::new("نسبي/ملف.rs"))),
            ("err.path.relative", Some("path"))
        );
    }

    #[test]
    fn a_symlinked_repository_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("git-blame-symlink-repo").unwrap();
        let Some(real) = repo_in(&s, "الحقيقي") else { return };
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let file = s.file("الحقيقي/ملف.rs", b"x");

        let cmd = plan_with(&link, &file).unwrap();
        assert_eq!(Path::new(&cmd.args[1]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.git.repo.resolved"), "{:?}", cmd.warnings);
    }

    /// المسار المرمَّز فيه الملف يُحلّ في الأمر المعروض دومًا — كل مسارٍ يمرّ
    /// بـ`paths.rs` يُحلّ بصرف النظر عن أي تحذير — لكن `path` لا يحمل تحذيره
    /// الخاص، بخلاف `repo`: القرار معلَنٌ في رأس الملفّ لا صامتًا هنا.
    #[test]
    fn a_symlinked_file_is_resolved_in_the_command_but_raises_no_warning_of_its_own() {
        let s = Scratch::new("git-blame-symlink-file").unwrap();
        let Some(repo) = repo_in(&s, "مستودع") else { return };
        let real = s.file("مستودع/حقيقي.rs", b"x");
        let link = s.path().join("مستودع/رابط.rs");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&repo, &link).unwrap();
        assert_eq!(Path::new(&cmd.args[5]), real.as_path(), "the command must name what it reads");
        assert_eq!(cmd.warnings, Vec::<&str>::new(), "no warning is declared for `path`");
    }

    #[test]
    fn an_ordinary_case_raises_no_warning_at_all() {
        let s = Scratch::new("git-blame-quiet").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let file = s.file("م/ملف.rs", b"x");
        assert_eq!(plan_with(&repo, &file).unwrap().warnings, Vec::<&str>::new());
    }

    #[test]
    fn planning_writes_nothing_into_the_repository() {
        let s = Scratch::new("git-blame-clean").unwrap();
        let Some(repo) = repo_in(&s, "م") else { return };
        let file = s.file("م/ملف.rs", "محتوى ثابت".as_bytes());
        let before = s.names(&repo);

        for _ in 0..10 {
            plan_with(&repo, &file).unwrap();
        }
        assert_eq!(s.names(&repo), before, "planning must not touch the repository");
        assert_eq!(std::fs::read(&file).unwrap(), "محتوى ثابت".as_bytes());
    }
}
