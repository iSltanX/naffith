//! ما الذي يُبقي ملفًّا مفتوحًا داخل هذا المجلد؟ باستخدام `lsof`.
//!
//! ## السؤال الذي تجيب عنه
//!
//! ليس «ما المفتوح على هذا الجهاز؟» — ذاك جوابٌ بآلاف الأسطر لا يقرؤه أحد.
//! بل السؤال العمليّ الذي يُطرح في لحظةٍ بعينها: «لماذا لا أستطيع إخراج هذا
//! القرص؟»، «لماذا يرفض هذا المجلد أن يُحذف؟». الجهاز يقول «قيد الاستخدام»
//! ولا يقول **بيد مَن**، فيبقى المستخدم أمام رسالةٍ صحيحةٍ لا يستطيع أن يفعل
//! بها شيئًا. وهذه العملية تسمّي البرنامج الماسك ورقمه، فيصير للرسالة تتمّة.
//!
//! ## الصيغة
//!
//! ```text
//! /usr/sbin/lsof -nP +D <المجلد>
//! ```
//!
//! ## `+D` لا `+d`
//!
//! الرايتان متجاورتان في صفحة الدليل ويفرّق بينهما حرفٌ واحد، والفرق بينهما
//! هو الفرق بين جوابٍ وسرابٍ يشبه الجواب: `+d` تنظر في مستوًى واحد — ما كان
//! مفتوحًا في المجلد نفسه دون أبنائه — و`+D` تنزل الشجرة كاملةً إلى قرارها.
//!
//! ومَن يسأل «ما الذي يمسك شيئًا هنا؟» لا يُجاب بمستوًى واحد أبدًا: الملفّ
//! الممسوك يكون في العادة في عمقٍ ما — قاعدة بياناتٍ داخل مجلد تطبيق، أو
//! سجلٌّ مفتوح تحت `Logs`، أو ملفٌّ يكتب فيه نسخٌ جارٍ. `+d` على مثل هذه
//! الشجرة تطبع لا شيء، ولا شيءٌ هنا يُقرأ جوابًا: «لا أحد يمسك المجلد». وهذه
//! أسوأ من غياب الجواب، لأنها تُقنع السائل بأن سؤاله انتهى وتصرفه عن الموضع
//! الوحيد الذي فيه ما يبحث عنه.
//!
//! ## وثمنُ `+D` معلَنٌ قبل الضغط
//!
//! `+D` تمشي الشجرة كلّها قبل أن تطبع سطرًا واحدًا: تُعدّد كل ما تحت المجلد
//! ثم تقابل تلك القائمة بما هو مفتوح. فعلى شجرةٍ ضخمة — `node_modules`، أو
//! مكتبة صورٍ فيها عشرات الآلاف — يقع الانتظار **قبل** أول سطر لا بعده، أي
//! أن الشاشة تبقى صامتة تمامًا ولا شيء فيها يدلّ على أن شيئًا يجري.
//!
//! ولهذا `warn.lsof.deep_scan` **ثابتٌ في كل خطة لا مشروط**، على نمط
//! `warn.profiler.slow` في `system_report.rs` تمامًا. وجعلُه مشروطًا كان
//! يحتاج أن نمسح الشجرة كي نعرف أكبيرةٌ هي — أي أن ندفع الكلفة التي نحذّر
//! منها كي نقرّر أنحذّر. والبطء هنا صفةٌ في **شكل الأمر** لا حالةٌ تطرأ على
//! بعض المجلدات، فيُقال في كل مرّة. والإلغاء متاحٌ في أي لحظة بلا أثر: لا
//! شيء يُكتب أصلًا.
//!
//! ## القائمة جزئية، ويُقال ذلك دائمًا
//!
//! بلا صلاحيات مدير لا ترى `lsof` إلا ما تملكه عمليات المستخدم الحالي. فمجلدٌ
//! يمسكه `mds` أو خدمةٌ تعمل بحساب النظام يظهر هنا وكأن لا أحد يمسكه. وهذا
//! المنتج لا يطلب صلاحيات المدير ولا يعرض على المستخدم أن يمنحها — رفعُ
//! الصلاحيات لعرض قائمةٍ مقايضةٌ خاسرة، وبديلُها الصادق أن تُعلَن حدود
//! القائمة قبل قراءتها. فـ`warn.lsof.user_scope` ثابتٌ في كل خطة لنفس السبب
//! المشروح في `net_ports.rs`: هو ليس حالةً عارضة بل صفةٌ في كل نتيجة.
//!
//! ## `-nP` هنا: احتياطٌ معلَن، لا مكسبٌ مقيس
//!
//! الرايتان لا تعملان إلا على الملفات الشبكية: `-n` تمنع تحويل العناوين إلى
//! أسماء مضيفات، و`-P` تمنع تحويل أرقام المنافذ إلى أسماء خدمات. و`+D` وحدها
//! تحصر ما تختاره `lsof` في ما تحت المجلد، فلا مقبس شبكيّ في هذه القائمة
//! أصلًا — أي أن أثر الرايتين على مجلدٍ محليّ صفر.
//!
//! فلماذا تبقيان؟ لأن الأمر في هذا التطبيق يُعرض كي يُنسخ ويُشغَّل في صدفة
//! المستخدم، والرايتان تجعلانه لا يسأل الشبكة شيئًا في أي حالٍ لم نتوقّعها.
//! وهذا كل ما يُقال لهما: هذا الملفّ لا يخترع لهما فائدةً لم تُقَس، ولا يعيد
//! هنا كلام `net_ports.rs` عن DNS المقطوع على مجلدٍ في القرص المحلي.
//!
//! ## **بلا** `--`
//!
//! خلاصة `files_tree_size.rs` نفسها — «‏`--` يُضاف حيث يعمل لا حيث يبدو
//! مرتّبًا» — ببرهانٍ مختلف. هناك كان السبب أن `du` لا توثّق الفاصل أصلًا،
//! وهنا `lsof` **توثّقه**: صفحة دليلها تصفه بأنه علامة نهاية الرايات المفتاحية
//! وبداية **أسماء الملفات**.
//!
//! لكن المسار في هذا الأمر ليس اسم ملفٍّ يُبحث عنه، بل **قيمة الراية `+D`**.
//! والموضع الوحيد الذي كان `--` يقع فيه قبل المسار هو بين الراية وقيمتها،
//! وهناك يكسر الأمر لا يؤمّنه: مقيسٌ تجريبيًّا أن `lsof -nP +D -- <مجلد>`
//! تخرج بخطأ «‏`+d` not followed by a directory path». وما بعد المسار لا
//! أسماء فيه يفصلها الفاصل عن شيء، فيصير رمزًا زائدًا في أمرٍ يُعرض للقراءة.
//!
//! والحاجة إليه معدومة على كل حال: المسار مطلقٌ ومُتحقَّق منه في `paths.rs`
//! فيبدأ بـ`/` حتمًا، ولا يمكن أن يُقرأ راية.
//!
//! ## رمز الخروج لا يحمل جوابًا — والخرج هو الجواب
//!
//! مقيسٌ تجريبيًّا على `lsof 4.91` (المرافقة لـmacOS): `+D` تخرج بـ`1` حين لا
//! يكون شيءٌ مفتوحًا في المجلد، **وتخرج بـ`1` أيضًا حين تجد شيئًا وتطبعه**.
//! ولا تخرج بـ`0` إلا إذا كانت كلُّ مدخلةٍ في الشجرة مفتوحة — المجلد نفسه وكل
//! ملفٍّ فيه — وهي حالةٌ لا تقع خارج الاختبار. والسبب أن `+D` توسّع الشجرة إلى
//! قائمة بحث، وكلُّ مدخلةٍ فيها لم تُوجد مفتوحةً تُحسب «فشلًا في العثور».
//!
//! فالنتيجة تُقرأ من الخرج لا من الرمز: قائمةٌ فارغةٌ بلا رسالة خطأٍ على
//! `stderr` جوابٌ صامتٌ صحيح — «لا شيء يمسك شيئًا هنا» — لا عطبٌ في التشغيل.
//! هذا الملفّ يبني الأمر ولا يحكم على نتيجته؛ ترجمة الرمز إلى معنًى تقع في
//! `result.rs` وحده، وهذا الموضع يوثّق الحقيقة التي تُبنى عليها تلك الترجمة.
//!
//! ## ما لا تفعله
//!
//! تقرأ وتطبع، ولا تُنهي شيئًا. أن تعرف مَن يمسك الملفّ شيء، وأن تُنهيه شيءٌ
//! آخر له شاشته وخطره ومدخلاته — و`lsof` لا تملك رايةً تُنهي عمليةً أصلًا.

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "disk.directory.open_handles";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.disk.directory.open_handles.title",
    description_key: "op.disk.directory.open_handles.description",
    category: Category::Disk,
    // قراءةٌ محضة: تسأل النواة عن جدول الملفات المفتوحة وتطبع ما تراه. لا
    // تفتح شيئًا ولا تغلقه ولا تكتب بايتًا.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::LSOF,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("folder", InputKind::ExistingDir)],
    sort_order: 60,
    search_terms: &["lsof", "open", "handles", "مقابض", "مفتوحة", "مجلد", "يمسك", "إخراج", "قرص"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let folder = inputs.dir("folder")?;

    Argv::tool(tools::LSOF, "explain.lsof.tool")
        .flag("-nP", "explain.lsof.no_lookups")
        .flag("+D", "explain.lsof.directory_tree")
        .path(folder)
        // «أين أنظر؟» له جوابٌ هنا: المجلد الذي سُئل عنه. ولا ناتج يُنتَج،
        // فلا جواب ثانٍ يزاحمه.
        .reveal(folder)
        // ثابتان لا مشروطان: البطء صفةُ `+D` نفسها، وجزئيّةُ القائمة صفةُ
        // `lsof` بلا صلاحيات مدير. كلاهما في كل نتيجة لا في بعضها.
        .warn("warn.lsof.deep_scan")
        .warn("warn.lsof.user_scope")
        // وهذا وحده مشروط: يقع فعلًا حين يختار المستخدم رابطًا رمزيًّا.
        .warn_opt(warn_if_resolved(inputs, "folder", folder, "warn.source.resolved"))
        .read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    /// الرايات التي تعلنها هذه العملية في شيفرتها. الاختبارات تمرّ عليها لا
    /// على ذاكرة من كتبها. و`+D` بينها رغم أنها لا تبدأ بشرطة: «راية» صفةٌ في
    /// الشرح لا شكلٌ في النصّ.
    const DECLARED_FLAGS: &[&str] = &["-nP", "+D"];

    fn raw(folder: &Path) -> BTreeMap<String, RawValue> {
        BTreeMap::from([("folder".to_owned(), RawValue::Path(folder.display().to_string()))])
    }

    fn plan_with(folder: &Path) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(folder))?)
    }

    fn args_of(cmd: &PlannedCommand) -> Vec<String> {
        cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect()
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
        let found =
            listed.iter().find(|o| o.id == ID).expect("disk.directory.open_handles must be listed");
        assert_eq!(found.category, Category::Disk);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
        assert_eq!(found.inputs.len(), 1);
        assert!(found.inputs[0].required, "a scan of nowhere is not a scan");
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_more() {
        let s = Scratch::new("lsof-argv").unwrap();
        let folder = s.dir("المجلد");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        let cmd = plan_with(&folder).unwrap();

        assert_eq!(cmd.program, Path::new("/usr/sbin/lsof"));
        assert_eq!(
            args_of(&cmd),
            vec!["-nP".to_owned(), "+D".to_owned(), folder.display().to_string()]
        );
        assert!(cmd.artifact.is_none(), "asking who holds a file produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "the folder is an argument, not a working directory");
        assert!(cmd.estimate.is_none(), "measuring the tree would cost what we warn about");
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("lsof-explain").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn every_flag_is_explained_and_the_path_stays_bare_data() {
        // `-nP` و`+D` لا يفهمهما أحدٌ بالنظر، فرايةٌ بلا مفتاح شرحٍ هنا تعني
        // سطرًا في «سَطْر» بلا معنى. والمسار بيانات: يُعرض ولا يُشرح.
        let s = Scratch::new("lsof-tokens").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder).unwrap();
        assert_eq!(cmd.explain[0].role, TokenRole::Tool);
        assert!(cmd.explain[0].key.is_some());
        for token in cmd.explain.iter().skip(1).take(2) {
            assert_eq!(token.role, TokenRole::Flag, "{:?} is not declared a flag", token.token);
            assert!(token.key.is_some(), "{:?} carries no explanation", token.token);
        }
        let path_token = cmd.explain.last().unwrap();
        assert_eq!(path_token.role, TokenRole::Path);
        assert!(path_token.key.is_none(), "a path is data, not a keyword to explain");
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("lsof-dashes").unwrap();
        let folder = s.dir("-rf");

        let cmd = plan_with(&folder).unwrap();
        for a in &cmd.args {
            if DECLARED_FLAGS.contains(&a.to_string_lossy().as_ref()) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn the_end_of_flags_separator_is_deliberately_absent() {
        // ‏`lsof` توثّقه، لكنه يفصل الرايات عن **أسماء الملفات** — والمسار هنا
        // قيمةُ الراية `+D`. وضعُه قبله يكسر الأمر: مقيسٌ أن `+D -- <مجلد>`
        // تخرج بخطأ «‏+d not followed by a directory path».
        let s = Scratch::new("lsof-noflagsep").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder).unwrap();
        assert!(!cmd.args.iter().any(|a| a == "--"), "`--` here separates nothing from nothing");
    }

    #[test]
    fn the_slow_scan_and_the_partial_view_are_announced_on_every_single_run() {
        let s = Scratch::new("lsof-warn").unwrap();
        let folder = s.dir("م");

        for _ in 0..3 {
            let cmd = plan_with(&folder).unwrap();
            assert_eq!(cmd.warnings, vec!["warn.lsof.deep_scan", "warn.lsof.user_scope"]);
        }
    }

    #[test]
    fn an_empty_folder_still_produces_a_valid_argv() {
        // «لا شيء مفتوح هنا» جوابٌ مشروع، والحكم عليه يقع بعد التنفيذ لا هنا.
        let s = Scratch::new("lsof-empty").unwrap();
        let folder = s.dir("فارغ");

        let cmd = plan_with(&folder).unwrap();
        assert_eq!(
            args_of(&cmd),
            vec!["-nP".to_owned(), "+D".to_owned(), folder.display().to_string()]
        );
    }

    #[test]
    fn a_symlinked_folder_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("lsof-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(&cmd.args[2]), real.as_path());
        assert_eq!(
            cmd.warnings,
            vec!["warn.lsof.deep_scan", "warn.lsof.user_scope", "warn.source.resolved"]
        );
    }

    #[test]
    fn a_file_where_a_folder_is_expected_is_refused_by_its_own_field_name() {
        let s = Scratch::new("lsof-notdir").unwrap();
        let file = s.file("ملف", b"data");
        assert_eq!(refusal(plan_with(&file)), ("err.path.not_dir", Some("folder")));
    }

    #[test]
    fn a_folder_that_does_not_exist_is_refused_by_its_own_field_name() {
        let s = Scratch::new("lsof-missing").unwrap();
        let ghost = s.path().join("لا-وجود-له");
        assert_eq!(refusal(plan_with(&ghost)), ("err.path.missing", Some("folder")));
    }

    #[test]
    fn a_relative_folder_is_refused_before_it_becomes_an_argument() {
        assert_eq!(
            refusal(plan_with(Path::new("مجلد/نسبي"))),
            ("err.path.relative", Some("folder"))
        );
    }

    #[test]
    fn a_folder_sent_as_free_text_is_refused_not_coerced() {
        // المواصفة تعلن `ExistingDir`، فقيمةٌ من نوع نصّ لا تُحوَّل ضمنًا إلى
        // مسار: التحويل الضمني هو الباب الذي يدخل منه نصٌّ لم يمرّ بسياسة
        // المسارات.
        let raw = BTreeMap::from([("folder".to_owned(), RawValue::Text("/tmp".to_owned()))]);
        let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
        assert_eq!(refusal(r), ("err.input.type", Some("folder")));
    }

    #[test]
    fn the_folder_is_required_and_its_absence_names_its_own_field() {
        let r = crate::value::validate(&SPEC, &BTreeMap::new()).map(|i| plan(&i).unwrap());
        assert_eq!(refusal(r), ("err.input.missing", Some("folder")));
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        let s = Scratch::new("lsof-smuggle").unwrap();
        let folder = s.dir("م");

        for smuggled in ["depth", "user", "+d", "-t", "flags"] {
            let mut with_extra = raw(&folder);
            with_extra.insert(smuggled.to_owned(), RawValue::Text("1".to_owned()));
            let r = crate::value::validate(&SPEC, &with_extra).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_in_the_folder_name_stays_literal_and_adds_no_argument() {
        let s = Scratch::new("lsof-shellish").unwrap();

        for name in ["مجلد 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b", "a | b"]
        {
            let folder = s.dir(name);
            let cmd = plan_with(&folder).unwrap();
            assert_eq!(Path::new(&cmd.args[2]), folder.as_path());
            assert_eq!(cmd.args.len(), 3, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn planning_leaves_the_folder_untouched() {
        let s = Scratch::new("lsof-clean").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        for _ in 0..10 {
            plan_with(&folder).unwrap();
        }
        assert_eq!(s.names(&folder), vec!["ملف".to_string()]);
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        let s = Scratch::new("lsof-stable").unwrap();
        let folder = s.dir("م");
        assert_eq!(plan_with(&folder).unwrap(), plan_with(&folder).unwrap());
    }
}
