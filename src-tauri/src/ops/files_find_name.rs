//! البحث عن ملف بالاسم أو بنمطٍ منه، باستخدام `find`.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/bin/find <المجلد> -iname <النمط>
//! ```
//!
//! ## `-iname` لا `-name`
//!
//! نظام الملفات الافتراضي على macOS (‏APFS في وضعه المعتاد) لا يفرّق بين
//! `Report.pdf` و`report.pdf` عند الفتح، فمن يبحث عن الثاني يتوقّع أن يجد
//! الأول. و`-name` كانت ستفرّق، فتُخرج قائمةً فارغة عن ملفٍ يراه المستخدم
//! بعينه في Finder — وهو أسوأ ما يمكن أن تفعله أداة بحث. الثمن نتائجُ أوسع
//! على أقراصٍ حسّاسة لحالة الأحرف، وهو ثمنٌ في الاتجاه الآمن.
//!
//! ## النمط يُمرَّر كما هو، ولا تفسّره صدفة
//!
//! `*` و`?` و`[…]` تفسّرها **`find` نفسها**، لا صدفة. لا صدفة في المسار أصلًا:
//! النمط يصل الأداةَ وسيطًا واحدًا كما كتبه المستخدم، بلا اقتباسٍ يُضاف وبلا
//! توسيعٍ يقع في الطريق. ولهذا لا يحتاج النمط اقتباسًا هنا بينما يحتاجه في
//! Terminal — وهو فرقٌ يستحق أن يُقال في الشرح.
//!
//! ## ولماذا لا يُلمس النمط
//!
//! لا `trim` ولا إضافة `*` حول ما كتبه المستخدم. النمط **نمطٌ لا اسم**: إضافة
//! نجمتين تجعل بحثًا عن `.env` يُخرج `.environment` و`my.env.bak`، وقصُّ
//! المسافات يجعل بحثًا عن `تقرير ` يطابق `تقرير` — وفي الحالين تُظهر الأداة
//! غير ما طُلب منها وتقول إنها وجدته. ما يُرفض هنا يُرفض صراحةً، وما يُقبل
//! يُمرَّر حرفًا بحرف.
//!
//! ## ما يُرفض ولماذا
//!
//! | الحالة | السبب |
//! |---|---|
//! | فارغ أو كلّه فراغ | ‏`-iname ''` لا تطابق شيئًا أبدًا: سؤالٌ لا جواب له |
//! | فيه `/` | ‏`-iname` تطابق **اسم المدخلة** لا المسار، فالشرطة لا تطابق شيئًا؛ والباحث بها يظنّ أنه يقصر البحث على مجلدٍ فرعي (‏ذاك `-path`) |
//! | فيه `\0` | لا يمكن أن يعبر حدّ استدعاء النظام أصلًا |
//! | فيه محرف تحكّم | سطرٌ جديد داخل النمط يجعل الشرح والسجلّ يعرضان سطرين حيث يوجد وسيطٌ واحد، فيُقرأ غير ما يُنفَّذ |
//! | يبدأ بشرطة | `find` كانت ستقرأه شرطًا من شروطها لا نمطًا |
//!
//! والصنف المُعلَن للشرطة البادئة هو `LeadingDot`. ليس اسمه دقيقًا — لا يوجد
//! في `NameRejection` صنفٌ للشرطة — لكنه أقرب الموجود معنًى: «حرفٌ في الصدر
//! يغيّر معنى ما بعده». وبقيّة الأصناف كانت ستقول للمستخدم شيئًا **غير صحيح**
//! («لا يجوز أن يحتوي الاسم على `/`» عن نمطٍ لا شرطة مائلة فيه)، وهو أسوأ من
//! اسمٍ داخليّ غير دقيق. إن أُضيف صنفٌ للشرطة يومًا فهذا موضعه.

use crate::error::{CoreError, NameRejection, Result};
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "files.find.name";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.find.name.title",
    description_key: "op.files.find.name.description",
    category: Category::Files,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::FIND,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("folder", InputKind::ExistingDir),
        InputSpec::new("pattern", InputKind::Text { max_len: 200 }),
    ],
    sort_order: 60,
    search_terms: &["find", "بحث", "اسم", "name", "search", "نمط", "pattern", "locate", "ابحث"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let folder = inputs.dir("folder")?;
    let pattern = validated_pattern(inputs.text("pattern")?)?;

    let mut argv = Argv::tool(tools::FIND, "explain.find.tool")
        .path(folder)
        .flag("-iname", "explain.find.iname")
        // مُتحقَّق أنه لا يبدأ بشرطة أعلاه؛ و`Argv::value` تفحص ذلك ثانيةً
        // ولا تعتمد على المتصل بها.
        .value(pattern)
        .reveal(folder);

    if let Some(key) = warn_if_resolved(inputs, "folder", folder, "warn.source.resolved") {
        argv = argv.warn(key);
    }

    argv.read_only()
}

/// يفحص النمط ويعيده **كما هو** إن قُبل. لا ينقّي ولا يعيد الكتابة.
///
/// الفحص هنا لا في `value.rs`: `Text` صيغةٌ عامّة يستعملها كل حقلٍ نصّي في
/// المنتج، وقواعد النمط قواعدُ هذه العملية وحدها. تعميمها هناك كان سيمنع
/// عمليةً أخرى من قبول نصٍّ فيه `/` بحقّ.
fn validated_pattern(raw: &str) -> Result<&str> {
    let rejected = |reason: NameRejection| CoreError::InvalidName { reason }.on_input("pattern");

    if raw.trim().is_empty() {
        return Err(rejected(NameRejection::Empty));
    }
    if raw.contains('/') {
        return Err(rejected(NameRejection::ContainsSeparator));
    }
    // قبل فحص محارف التحكّم كي يُسمّى الصفر باسمه: كلاهما محرف تحكّم، وأحدهما
    // له صنفٌ خاصّ وسببٌ خاصّ.
    if raw.contains('\0') {
        return Err(rejected(NameRejection::ContainsNul));
    }
    if raw.chars().any(char::is_control) {
        return Err(rejected(NameRejection::ContainsControl));
    }
    if raw.starts_with('-') {
        return Err(rejected(NameRejection::LeadingDot));
    }
    Ok(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    fn raw(folder: &Path, pattern: &str) -> BTreeMap<String, RawValue> {
        BTreeMap::from([
            ("folder".to_owned(), RawValue::Path(folder.display().to_string())),
            ("pattern".to_owned(), RawValue::Text(pattern.to_owned())),
        ])
    }

    fn plan_with(folder: &Path, pattern: &str) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(folder, pattern))?)
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
                other => panic!("expected an invalid-name refusal, got {other:?}"),
            },
            Err(other) => panic!("expected an attributed refusal, got {other:?}"),
        }
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("files.find.name must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_writes_nothing() {
        let s = Scratch::new("iname-argv").unwrap();
        let folder = s.dir("المجلد");

        let cmd = plan_with(&folder, "*.pdf").unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/usr/bin/find"));
        assert_eq!(Path::new(&args[0]), folder.as_path());
        assert_eq!(args[1], "-iname");
        assert_eq!(args[2], "*.pdf");
        assert_eq!(args.len(), 3);

        assert!(cmd.artifact.is_none(), "find reports; it never produces a file");
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
        assert!(!args.iter().any(|a| a == "-delete" || a == "-exec"), "find must only report");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("iname-explain").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder, "تقرير*").unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("iname-dashes").unwrap();
        let folder = s.dir("-rf");

        let cmd = plan_with(&folder, "*.pdf").unwrap();
        for i in [0usize, 2] {
            assert!(
                cannot_be_read_as_a_flag(&cmd.args[i]),
                "{:?} would be read as a flag",
                cmd.args[i]
            );
        }
    }

    #[test]
    fn an_empty_or_blank_pattern_is_refused() {
        let s = Scratch::new("iname-empty").unwrap();
        let folder = s.dir("م");

        for bad in ["", " ", "   ", "\t"] {
            assert_eq!(
                refusal(plan_with(&folder, bad)),
                ("err.name.invalid", Some("pattern")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&folder, bad)), NameRejection::Empty, "{bad:?}");
        }
    }

    #[test]
    fn a_pattern_carrying_a_path_separator_is_refused() {
        let s = Scratch::new("iname-slash").unwrap();
        let folder = s.dir("م");

        for bad in ["مجلد/ملف", "/absolute", "a/*.pdf", "../خارج"] {
            assert_eq!(
                refusal(plan_with(&folder, bad)),
                ("err.name.invalid", Some("pattern")),
                "{bad:?}"
            );
            assert_eq!(
                reason(plan_with(&folder, bad)),
                NameRejection::ContainsSeparator,
                "{bad:?}"
            );
        }
    }

    #[test]
    fn a_pattern_carrying_a_nul_byte_is_refused_and_named_as_such() {
        let s = Scratch::new("iname-nul").unwrap();
        let folder = s.dir("م");
        assert_eq!(reason(plan_with(&folder, "ملف\0.pdf")), NameRejection::ContainsNul);
    }

    #[test]
    fn a_pattern_carrying_a_control_character_is_refused() {
        let s = Scratch::new("iname-control").unwrap();
        let folder = s.dir("م");

        for bad in ["سطر\nثانٍ", "tab\there", "bell\u{7}"] {
            assert_eq!(
                refusal(plan_with(&folder, bad)),
                ("err.name.invalid", Some("pattern")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&folder, bad)), NameRejection::ContainsControl, "{bad:?}");
        }
    }

    #[test]
    fn a_pattern_that_would_be_read_as_a_find_predicate_is_refused() {
        let s = Scratch::new("iname-dash").unwrap();
        let folder = s.dir("م");

        for bad in ["-delete", "-name", "-x"] {
            assert_eq!(
                refusal(plan_with(&folder, bad)),
                ("err.name.invalid", Some("pattern")),
                "{bad:?}"
            );
            assert_eq!(reason(plan_with(&folder, bad)), NameRejection::LeadingDot, "{bad:?}");
        }
    }

    #[test]
    fn a_pattern_longer_than_the_declared_limit_is_refused_by_the_spec_itself() {
        let s = Scratch::new("iname-long").unwrap();
        let folder = s.dir("م");
        assert_eq!(
            refusal(plan_with(&folder, &"a".repeat(201))),
            ("err.input.type", Some("pattern"))
        );
    }

    #[test]
    fn an_accepted_pattern_reaches_the_argument_untouched() {
        let s = Scratch::new("iname-verbatim").unwrap();
        let folder = s.dir("م");

        // المسافات والنجوم والنقاط البادئة والعربية كلها تمرّ حرفًا بحرف.
        let accepted = [" مسافة في الصدر", "تقرير ٢٠٢٦ *.pdf", ".env", "*", "a?b[cd]"];
        for pattern in accepted {
            let cmd = plan_with(&folder, pattern).unwrap();
            assert_eq!(cmd.args[2].to_string_lossy(), pattern, "the pattern must not be rewritten");
            assert_eq!(cmd.args.len(), 3);
        }
    }

    #[test]
    fn shell_syntax_in_the_pattern_stays_literal_and_adds_no_argument() {
        let s = Scratch::new("iname-shellish").unwrap();
        let folder = s.dir("م");

        for pattern in ["a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "'مقتبس'", "a|b"] {
            let cmd = plan_with(&folder, pattern).unwrap();
            assert_eq!(cmd.args[2].to_string_lossy(), pattern);
            assert_eq!(cmd.args.len(), 3, "{pattern:?} must not add arguments");
        }
    }

    #[test]
    fn a_symlinked_folder_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("iname-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link, "*.pdf").unwrap();
        assert_eq!(Path::new(&cmd.args[0]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_the_folder_untouched() {
        let s = Scratch::new("iname-clean").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        for _ in 0..10 {
            plan_with(&folder, "*.pdf").unwrap();
        }
        assert_eq!(s.names(&folder), vec!["ملف".to_string()]);
    }
}
