//! المساحة الحرة على كل نظام ملفات مركَّب، باستخدام `df`.
//!
//! ## الصيغة
//!
//! ```text
//! /bin/df -h
//! ```
//!
//! لا مدخلات ولا مسارات. وسيطٌ واحد معلَن في الشيفرة، وليس في العملية موضعٌ
//! يستطيع نصٌّ قادم من الواجهة أن يصير فيه وسيطًا — لا لأن الفحص يمنعه، بل
//! لأن الحقل الذي كان سيحمله غير موجود أصلًا.
//!
//! ## لماذا `-h` لا الخرج الافتراضي
//!
//! `df` بلا رايات على macOS تعدّ بكتل ٥١٢ بايت. عمودٌ يقول `1953595632` جوابٌ
//! لا يُقرأ، ومن يقرؤه سيقسمه على ٢ ثم على ١٠٢٤ مرّتين — أي أن الأداة تركت
//! الحساب للقارئ.
//!
//! والأسوأ أن ذلك الافتراضي **يتبع البيئة**: متغيّر `BLOCKSIZE` يغيّر وحدة
//! العدّ. النواة تمسح البيئة قبل الإطلاق (`env_clear` في `executor.rs`) فخرج
//! التطبيق ثابت مهما كانت صدفة المستخدم، لكن التطبيق يعرض الأمر ويدعو إلى
//! نسخه — وفي صدفة المستخدم البيئة ليست ممسوحة. أمرٌ يُعرض هنا ويُنسخ هناك
//! فيعطي جدولًا آخر ينقض ما يقوم عليه «سَطْر» كلّه. `-h` تتجاوز `BLOCKSIZE`،
//! فالأمر يعني الشيء نفسه في الموضعين.
//!
//! ## ولماذا `-h` لا `-H`
//!
//! الاثنتان تكتبان الأرقام بوحداتٍ يقرؤها البشر، ويختلفان في الأساس: `-h`
//! تعدّ بـ ١٠٢٤ وتلحق `Gi`، و`-H` تعدّ بـ ١٠٠٠ وتلحق `G`.
//!
//! `-H` كانت ستطابق الرقم الذي يعرضه Finder و«حول هذا الماك»، لأن أبل تعدّ
//! بالألف. وثمنها أن اللاحقة `G` لا تقول بأي أساس قيست، فيصير الرقم صحيحًا
//! بلا ما يثبت صحّته. `-h` تدفع الثمن المعاكس: الرقم يقلّ عمّا يعرضه Finder
//! بنحو ٧٪ لكل غيغابايت، واللاحقة `Gi` **تقول لماذا**. اخترنا الثانية لأن
//! عملَ هذا المنتج أن يُفهِم لا أن يجاري رقمًا آخر على شاشةٍ أخرى — والفرق
//! معلَن في وصف العملية لا مسكوتٌ عنه.
//!
//! ## ما يظهر في الجدول، ولا يُصفّى
//!
//! **كل** نظام ملفات مركَّب، ومنه ما لا يستطيع هذا التطبيق الكتابة فيه بحال:
//! وحدة النظام `/` مركَّبةٌ للقراءة فقط منذ macOS 10.15، و`/System/Volumes/*`
//! و`/dev` و`map auto_home` مواضع يديرها النظام، وقد تظهر أقراص شبكية أو
//! صور قرصٍ مركَّبة لغيرنا.
//!
//! لا تُحذف هذه السطور ولا تُميَّز. حذفها كان يعني أن التطبيق **يفسّر** خرج
//! الأداة ويعيد كتابته، وهو ما لا يفعله في أي موضع: الشاشة تعرض ما طبعته
//! الأداة كما طبعته، فما يراه المستخدم هنا هو ما سيراه لو نسخ الأمر وشغّله.
//!
//! ## حدٌّ لا نصلحه بالنيابة عن الأداة
//!
//! في APFS تتقاسم الوحداتُ داخل الحاوية الواحدة مساحةً واحدة، فتعلن كلّها
//! نفس الرقم في عمود المتاح. جمعُ العمود لا يعطي مجموع الجهاز، ونقصانُ المتاح
//! في وحدةٍ قد يكون سببه امتلاء وحدةٍ أخرى. `df` تقول ما تراه من كل نقطة
//! تركيب، ولا تعرف الحاوية — والوصف يقول ذلك بدل أن يعِد بما لا يُوفى.

use crate::error::Result;
use crate::ops::common::Argv;
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "disk.free";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.disk.free.title",
    description_key: "op.disk.free.description",
    category: Category::Disk,
    // تقرأ جداول التركيب ولا تكتب بايتًا واحدًا.
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::DF,
    conflict: Conflict::NoArtifact,
    // عمليةٌ بلا مدخلات مشروعة: السؤال عن الجهاز لا عن مجلدٍ بعينه.
    //
    // `df <مسار>` كانت ستقصر الجواب على نظام ملفات واحد، وتحتاج حقلًا. وهذا
    // سؤالٌ آخر — «كم بقي في القرص الذي يحوي هذا؟» — لا هذا. وتركُها بلا
    // مدخلات يجعل شكل `argv` ثابتًا لا يتغيّر بين تشغيلٍ وآخر.
    inputs: &[],
    sort_order: 10,
    search_terms: &[
        "df",
        "مساحة",
        "حرة",
        "خالية",
        "قرص",
        "تخزين",
        "free",
        "space",
        "disk",
        "storage",
    ],
    plan,
};

fn plan(_inputs: &Inputs) -> Result<PlannedCommand> {
    // لا `.reveal(…)`: لا ملف ولا مجلد أنتجته هذه العملية، وزرُّ الإظهار الذي
    // يفتح موضعًا لم يُنتَج جوابٌ عن سؤالٍ لم يُطرح.
    Argv::tool(tools::DF, "explain.df.tool").flag("-h", "explain.df.human").read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::common::cannot_be_read_as_a_flag;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::Path;

    /// الرايات التي تعلنها هذه العملية في شيفرتها. الاختبارات تمرّ عليها لا
    /// على ذاكرة من كتبها.
    const DECLARED_FLAGS: &[&str] = &["-h"];

    fn plan_it() -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &BTreeMap::new())?)
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
        let found = listed.iter().find(|o| o.id == ID).expect("disk.free must be listed");
        assert_eq!(found.category, Category::Disk);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_nothing_more() {
        let cmd = plan_it().unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/bin/df"));
        assert_eq!(args, vec!["-h".to_string()]);
        assert!(cmd.artifact.is_none(), "a reading operation produces no file");
        assert!(cmd.stdout_to.is_none(), "nothing is redirected to disk");
        assert!(cmd.cwd.is_none(), "df answers about the machine, not about `here`");
        assert!(cmd.reveal_target.is_none(), "there is no produced path to reveal");
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let cmd = plan_it().unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        // لا وسيط بيانات هنا أصلًا. الاختبار يثبّت الأمرين معًا: أن كل ما يبدأ
        // بشرطة رايةٌ **معلَنة في الشيفرة**، وأن ما عداها لا يمكن أن يُقرأ راية.
        let cmd = plan_it().unwrap();
        for a in &cmd.args {
            if DECLARED_FLAGS.contains(&a.to_string_lossy().as_ref()) {
                continue;
            }
            assert!(cannot_be_read_as_a_flag(a), "{a:?} would be read as a flag");
        }
    }

    #[test]
    fn the_operation_declares_no_inputs_so_no_field_can_reach_the_argv() {
        assert!(SPEC.inputs.is_empty(), "disk.free must stay input-free");
    }

    #[test]
    fn an_input_this_operation_never_declared_is_refused_not_ignored() {
        // لا حقل في الشاشة يرسل هذا. لكن الحدّ لا يثق بالشاشة: مفتاحٌ لا تعرفه
        // المواصفة يعني إمّا واجهةً قديمة وإمّا محاولة تهريب، وكلاهما رفضٌ.
        for smuggled in ["path", "-h", "flags", "/etc/passwd"] {
            let raw = BTreeMap::from([(
                smuggled.to_owned(),
                RawValue::Text("/System/Volumes/Data".to_owned()),
            )]);
            let r = crate::value::validate(&SPEC, &raw).map(|i| plan(&i).unwrap());
            assert_eq!(refusal(r), ("err.input.unexpected", None), "{smuggled:?} slipped through");
        }
    }

    #[test]
    fn shell_syntax_cannot_reach_the_argv_because_there_is_nowhere_to_put_it() {
        // نظير اختبار «المحارف الحرفية» في العمليات ذات المدخلات: هناك يمرّ
        // النصّ وسيطًا واحدًا، وهنا لا يمرّ أصلًا — والأمر يبقى بايتًا ببايت
        // هو نفسه مهما حاولت الواجهة.
        let baseline = plan_it().unwrap();
        for shellish in ["; rm -rf ~", "$(whoami)", "`id`", "a & b", "'/'", "|| true"] {
            let raw = BTreeMap::from([("message".to_owned(), RawValue::Text(shellish.to_owned()))]);
            assert!(crate::value::validate(&SPEC, &raw).is_err(), "{shellish:?} was accepted");
        }
        assert_eq!(plan_it().unwrap().args, baseline.args);
        assert_eq!(baseline.args.len(), 1, "df takes exactly one declared flag");
    }

    #[test]
    fn planning_twice_gives_the_same_command() {
        // لا حالة ولا عشوائية ولا اسم مؤقّت: عمليةُ قراءةٍ بلا مدخلات يجب أن
        // تكون دالّةً ثابتة، وإلا كان في المسار شيءٌ لم نقصده.
        assert_eq!(plan_it().unwrap(), plan_it().unwrap());
    }
}
