//! البحث عن نصٍّ داخل ملفات مجلد، باستخدام `grep`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/grep -r -n [-i] -F -e <النمط> -- <المجلد>
//! ```
//!
//! ## `-F` ليست تحسينًا: هي ما يجعل الحقل حقل بحثٍ لا حقل برمجة
//!
//! بدونها يقرأ `grep` ما يكتبه المستخدم **تعبيرًا نمطيًّا**. فمن يبحث عن
//! `a.b` يجد `axb`، ومن يبحث عن `(` يحصل على خطأ صياغة، ومن يبحث عن `$1.00`
//! لا يجد شيئًا. وهذه ليست حالاتٍ نادرة: النقطة والقوس وعلامة الدولار محارف
//! عادية في نصٍّ عادي.
//!
//! والأثر الثاني أمنيّ لا تجميلي: التعبير النمطي مدخلٌ قابلٌ للتنفيذ. نمطٌ
//! مثل `(a+)+b` على ملفٍ كبير يجعل المطابقة أسّية الزمن (‏catastrophic
//! backtracking) فتلتهم العملية المعالج حتى يلغيها المستخدم. `-F` تجعل النمط
//! نصًّا ثابتًا يُطابَق حرفًا بحرف، فيسقط الباب من أصله.
//!
//! والبديل — أن نعلن حقلًا ثانيًا «بحث بتعبير نمطي» — مؤجَّل عمدًا: التعبير
//! النمطي لغةٌ ثانية داخل الحقل، وشرحُها في «سَطْر» شرحُ لغةٍ لا شرحُ أمر.
//!
//! ## `-e` وحدود ما يقدر عليه البانِي
//!
//! `-e` تعني «ما بعدي نمطٌ لا راية»، وهي في `man grep` الحلّ المعلَن لنمطٍ
//! يبدأ بشرطة. لكن `Argv::value` في `common.rs` ترفض **كل** قيمةٍ تبدأ
//! بشرطة، ولا تعرف أن هذه تلي `-e`. والقاعدة عامّة عمدًا: تليينُها لعمليةٍ
//! واحدة يلينها لكل العمليات، وتلك القاعدة هي التي تمنع أن تصير قيمةٌ رايةً
//! في أي أمرٍ يبنيه المنتج.
//!
//! فالنتيجة أن نمطًا يبدأ بشرطة يُرفض هنا برسالةٍ منسوبة إلى حقل النمط، ولا
//! يُمرَّر. وهذا حدٌّ حقيقي يُقال ولا يُخفى: علاجُه أن يتعلّم `Argv` صيغةً
//! تقول «قيمةٌ محميّة بـ`-e` قبلها»، وهو تغييرٌ في ملفٍّ مشترك بين كل
//! العمليات — مسجَّلٌ في خارطة الطريق. و`-e` تبقى في الأمر رغم ذلك: هي التي
//! تجعل الرفض قابلًا للرفع لاحقًا بلا تغيير شكل الأمر.
//!
//! ## النمط: لا فراغ، ولا محارف تحكّم، ولا تشذيب
//!
//! نمطٌ فارغ يطابق كل سطر في كل ملف، فيتحوّل البحث إلى سرد الشجرة كلها ثم
//! قصّها عند سقف الأسطر في `executor`. يُرفض.
//!
//! ومحارف التحكّم تُرفض لسببين: السطر الجديد داخل النمط يعني عند `grep`
//! **نمطين** لا نمطًا واحدًا (يقول `man grep`: «قد يتألف النمط من سطرٍ أو
//! أكثر»)، ويعني في «سَطْر» وفي السجلّ أمرًا يُعرض على سطرين — فيُقرأ غير ما
//! يُنفَّذ. وهو نفس المنطق الذي يرفض به `value.rs` محارفَ التحكّم في العنوان.
//!
//! ولا يُشذَّب النمط: المسافة في أوّله أو آخره جزءٌ مما يبحث عنه المستخدم،
//! وتشذيبها يجعل الأمر يبحث عن غير ما كُتب في الحقل.
//!
//! ## حدٌّ معروف: `grep` تخرج بـ‎1‎ حين لا تجد شيئًا
//!
//! يقول `man grep`: `0` وُجد سطرٌ أو أكثر، `1` لم يوجد، وما فوق خطأ. يترجم
//! عقد النتائج الصفر إلى `matches` والواحد إلى `no_matches`؛ كلاهما جوابٌ
//! مكتمل، ولا تحتاج الواجهة إلى تخمينه من الخرج.
//!
//! ## لا تكتب شيئًا
//!
//! قراءةٌ محضة: لا ناتج ولا مؤقّت. و«إظهار في Finder» يفتح المجلد المبحوث
//! فيه. والاستبدال داخل الملفات ليس هنا ولن يكون في هذه العملية: تعديلٌ في
//! مكانه لا يمرّ بالترقية الذرّية التي يقوم عليها هذا المنتج.

use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;
use std::path::Path;

pub const ID: &str = "text.search";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.text.search.title",
    description_key: "op.text.search.description",
    category: Category::Text,
    // قراءةٌ محضة: لا تكتب بايتًا واحدًا على القرص.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::GREP,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("folder", InputKind::ExistingDir),
        // ٢٠٠ محرفًا: أطول من أي نصٍّ يُبحث عنه فعلًا، وأقصر من أن يصير حمولة
        // في سطرٍ يُعرض ويُقيَّد.
        InputSpec::new("pattern", InputKind::Text { max_len: 200 }),
        InputSpec::new("ignore_case", InputKind::Flag).optional(),
    ],
    sort_order: 50,
    search_terms: &["grep", "بحث", "search", "find", "ابحث", "نص", "text", "داخل", "محتوى"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let folder = inputs.dir("folder")?;
    let pattern = inputs.text("pattern")?;
    let ignore_case = inputs.flag("ignore_case");

    validate_pattern(pattern)?;

    let mut argv = Argv::tool(tools::GREP, "explain.grep.tool")
        .flag("-r", "explain.grep.recursive")
        .flag("-n", "explain.grep.line_numbers");

    // الراية الاختيارية في موضعها قبل `-F`: ترتيب الرايات في `grep` لا يغيّر
    // المعنى، لكن ترتيبًا ثابتًا يجعل الأمر المعروض هو نفسه في كل تشغيل،
    // فيتعلّم المستخدم شكله بدل أن يعيد قراءته.
    if ignore_case {
        argv = argv.flag("-i", "explain.grep.ignore_case");
    }

    argv.flag("-F", "explain.grep.fixed_string")
        .flag("-e", "explain.grep.pattern_follows")
        .value(pattern)
        .end_of_flags()
        .explained_path(folder, "explain.role.search_folder")
        .warn_all(warnings_for(inputs, folder))
        .reveal(folder)
        .read_only()
}

fn warnings_for(inputs: &Inputs, folder: &Path) -> Vec<&'static str> {
    let mut warnings = Vec::new();
    warnings.extend(warn_if_resolved(inputs, "folder", folder, "warn.source.resolved"));
    warnings
}

/// يفحص النمط قبل أن يصير وسيطًا. ثلاثة شروط، ولكلٍّ منها ما يمنعه بعينه.
fn validate_pattern(pattern: &str) -> Result<()> {
    if pattern.is_empty() {
        return Err(CoreError::InvalidName { reason: NameRejection::Empty }.on_input("pattern"));
    }
    if pattern.chars().any(|c| c.is_control()) {
        return Err(
            CoreError::InvalidName { reason: NameRejection::ContainsControl }.on_input("pattern")
        );
    }
    // يبنيه `-e` نظريًا، ويرفضه `Argv::value` عمليًا. الرفض هنا كي يُنسب إلى
    // حقله ويُقرأ في مكانه، بدل أن يخرج من البانِي بلا حقلٍ يدلّ عليه.
    if pattern.starts_with('-') {
        return Err(CoreError::ArgumentLooksLikeFlag.on_input("pattern"));
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

    /// الرايات المعلَنة في هذا الأمر. ما عداها في `argv` بيانات.
    const DECLARED_FLAGS: &[&str] = &["-r", "-n", "-i", "-F", "-e", "--"];

    fn raw(folder: &Path, pattern: &str, ignore_case: bool) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("folder".to_owned(), RawValue::Path(folder.display().to_string())),
            ("pattern".to_owned(), RawValue::Text(pattern.to_owned())),
            ("ignore_case".to_owned(), RawValue::Flag(ignore_case)),
        ])
    }

    fn plan_with(folder: &Path, pattern: &str, ignore_case: bool) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(folder, pattern, ignore_case))?)
    }

    fn refusal(r: Result<PlannedCommand>) -> (&'static str, Option<&'static str>) {
        match r {
            Ok(_) => panic!("expected a refusal, got a plan"),
            Err(e) => (e.key(), e.input()),
        }
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("text.search must be listed");
        assert_eq!(found.category, Category::Text);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact, "a reader has no final name to protect");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_is_written() {
        let s = Scratch::new("grep-argv").unwrap();
        let folder = s.dir("المشروع");

        let cmd = plan_with(&folder, "كلمة", false).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/grep"));
        assert_eq!(args[0], "-r");
        assert_eq!(args[1], "-n");
        assert_eq!(args[2], "-F");
        assert_eq!(args[3], "-e");
        assert_eq!(args[4], "كلمة");
        assert_eq!(args[5], "--");
        assert_eq!(Path::new(&args[6]), folder.as_path());
        assert_eq!(args.len(), 7);

        assert!(cmd.artifact.is_none(), "grep produces no file");
        assert!(cmd.stdout_to.is_none(), "matches are streamed, not captured into a file");
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
    }

    #[test]
    fn the_case_flag_appears_only_when_asked_and_always_in_the_same_place() {
        let s = Scratch::new("grep-case").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder, "Word", true).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args[0], "-r");
        assert_eq!(args[1], "-n");
        assert_eq!(args[2], "-i");
        assert_eq!(args[3], "-F");
        assert_eq!(args[4], "-e");
        assert_eq!(args[5], "Word");
        assert_eq!(args.len(), 8);

        let without = plan_with(&folder, "Word", false).unwrap();
        assert!(!without.args.iter().any(|a| a == "-i"), "{:?}", without.args);
    }

    #[test]
    fn the_fixed_string_flag_is_present_in_every_plan() {
        // الضمانة التي تجعل النقطة نقطةً والقوس قوسًا. لو سقطت يومًا سقط هذا.
        let s = Scratch::new("grep-fixed").unwrap();
        let folder = s.dir("م");
        for (pattern, case) in [("a.b", false), ("(", true), ("$1.00", false)] {
            let cmd = plan_with(&folder, pattern, case).unwrap();
            assert!(cmd.args.iter().any(|a| a == "-F"), "{pattern:?}: {:?}", cmd.args);
        }
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("grep-explain").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder, "نص", true).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("grep-dashes").unwrap();
        let folder = s.dir("-rf");

        let cmd = plan_with(&folder, "نص", true).unwrap();
        for arg in &cmd.args {
            if DECLARED_FLAGS.iter().any(|flag| arg.as_os_str() == std::ffi::OsStr::new(flag)) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(arg), "{arg:?} would be read as a flag");
        }
    }

    #[test]
    fn a_regex_pattern_is_carried_literally_and_adds_no_argument() {
        let s = Scratch::new("grep-regex").unwrap();
        let folder = s.dir("م");

        for pattern in ["a.b", "(", "(a+)+b", "^بداية$", "*", "[", "\\d+", "a|b"] {
            let cmd = plan_with(&folder, pattern, false).unwrap();
            assert_eq!(cmd.args[4], std::ffi::OsString::from(pattern), "{pattern:?} was rewritten");
            assert_eq!(cmd.args.len(), 7, "{pattern:?} must not add arguments");
        }
    }

    #[test]
    fn shell_syntax_in_a_pattern_stays_one_literal_argument() {
        let s = Scratch::new("grep-shellish").unwrap();
        let folder = s.dir("م");

        for pattern in ["a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "\"مقتبس\"", "'مفرد'"]
        {
            let cmd = plan_with(&folder, pattern, false).unwrap();
            assert_eq!(cmd.args[4], std::ffi::OsString::from(pattern));
            assert_eq!(cmd.args.len(), 7, "{pattern:?} must not add arguments");
        }
    }

    #[test]
    fn an_empty_pattern_is_refused_because_it_would_list_the_whole_tree() {
        let s = Scratch::new("grep-empty").unwrap();
        let folder = s.dir("م");
        assert_eq!(refusal(plan_with(&folder, "", false)), ("err.name.invalid", Some("pattern")));
    }

    #[test]
    fn a_pattern_carrying_a_control_character_is_refused() {
        let s = Scratch::new("grep-control").unwrap();
        let folder = s.dir("م");
        // سطرٌ جديد = نمطان عند `grep`، وسطران في الأمر المعروض.
        for pattern in ["أ\nب", "أ\tب", "أ\rب", "أ\u{0}ب"] {
            assert_eq!(
                refusal(plan_with(&folder, pattern, false)),
                ("err.name.invalid", Some("pattern")),
                "{pattern:?} must be refused"
            );
        }
    }

    #[test]
    fn a_pattern_that_begins_with_a_dash_is_refused_with_its_own_field() {
        // `-e` تكفي عند `grep`، ولا تكفي عند البانِي — والحدّ يُقال في مكانه.
        let s = Scratch::new("grep-dash-pattern").unwrap();
        let folder = s.dir("م");
        for pattern in ["-i", "--include=*", "-rf ~"] {
            assert_eq!(
                refusal(plan_with(&folder, pattern, false)),
                ("err.arg.flag", Some("pattern")),
                "{pattern:?} must be refused"
            );
        }
    }

    #[test]
    fn an_over_long_pattern_is_refused_before_it_becomes_an_argument() {
        let s = Scratch::new("grep-long").unwrap();
        let folder = s.dir("م");
        let long = "ا".repeat(201);
        assert_eq!(refusal(plan_with(&folder, &long, false)), ("err.input.type", Some("pattern")));
    }

    #[test]
    fn a_pattern_is_never_trimmed_because_the_spaces_are_part_of_the_search() {
        let s = Scratch::new("grep-spaces").unwrap();
        let folder = s.dir("م");
        let cmd = plan_with(&folder, "  كلمة  ", false).unwrap();
        assert_eq!(cmd.args[4], std::ffi::OsString::from("  كلمة  "));
    }

    #[test]
    fn the_result_contract_replaces_the_obsolete_exit_code_warning() {
        let s = Scratch::new("grep-warn").unwrap();
        let folder = s.dir("م");
        let cmd = plan_with(&folder, "نص", false).unwrap();
        assert!(!cmd.warnings.contains(&"warn.grep.exit_code"), "{:?}", cmd.warnings);
    }

    #[test]
    fn a_symlinked_folder_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("grep-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "نص", false).unwrap();
        assert_eq!(Path::new(&cmd.args[6]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_writes_nothing_to_disk() {
        let s = Scratch::new("grep-clean").unwrap();
        let folder = s.dir("م");
        for _ in 0..10 {
            plan_with(&folder, "نص", false).unwrap();
        }
        assert_eq!(s.names(&folder), Vec::<String>::new(), "the folder must stay untouched");
    }
}
