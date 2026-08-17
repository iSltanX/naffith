//! سرد محتويات مجلدٍ واحد، سطرًا لكل عنصر، باستخدام `ls`.
//!
//! ## الصيغة
//!
//! ```text
//! /bin/ls -1 -A -p -- <المجلد>
//! ```
//!
//! ## `-1`: سطرٌ واحد لكل عنصر
//!
//! المبدئي على طرفيّة تفاعلية أعمدةٌ مرصوصة (‏`ls` تكتشف أنها لا تتحدّث إلى
//! طرفية هنا فتتحوّل لسطرٍ واحدٍ تلقائيًا، لكن `-1` تجعل ذلك معلَنًا لا متكِّلًا
//! على اكتشافٍ ضمنيّ قد يختلف بين بيئة وأخرى). سطرٌ واحدٌ لكل عنصر هو الشكل
//! الوحيد الذي يُقرأ آليًا اسمًا اسمًا دون تخمين أين ينتهي عمودٌ ويبدأ آخر.
//!
//! ## `-A` لا `-a`
//!
//! كلاهما يُظهر الملفّات المخفيّة (‏التي تبدأ بنقطة) — وهذا مقصود: مجلدٌ لا
//! يعرض إعداداته المخفية (‏`.git`، `.env`) ليس جوابًا صادقًا عن «ماذا في هذا
//! المجلد؟». لكن `-a` تُدرج أيضًا `.` و`..` أنفسهما، وهما ليسا عنصرين في
//! المجلد بل إشارتين إليه وإلى أبيه؛ إدراجهما في قائمةٍ يُفترض أنها محتويات
//! المجلد كذبٌ صغير. `-A` تُظهر كل شيءٍ آخر وتستثني هذين الاثنين وحدهما.
//!
//! ## `-p`: الفرق الوحيد بين ملفٍّ ومجلد في هذه القائمة
//!
//! `-1` تُسقط الأعمدة، و`ls` بلا رايةٍ أخرى كانت ستطبع الاسمين `صور` (مجلد)
//! و`صور.pdf` (ملف) بلا ما يميّز أحدهما عن الآخر. `-p` تُلحق `/` بآخر كل اسم
//! مجلد، فيصير هذا الفرق ظاهرًا في السطر نفسه بلا استدعاء `stat` على كل
//! عنصر. لا تُستعمل `-F` (‏تُلحق أيضًا `*` للتنفيذي و`@` للرابط الرمزي وغيرها)
//! لأن السؤال هنا واحد فقط: «أهذا مجلد؟» — لا «ما نوع كل عنصر؟».
//!
//! ## `--`
//!
//! مسار المجلد مطلقٌ ومُتحقَّق منه فيبدأ بـ`/` حتمًا، فلا يمكن أن يُقرأ رايةً
//! بأي حال. لكن القاعدة في هذا المنتج أن يُضاف `--` حيث تفهمه الأداة، حزامَ
//! أمانٍ إضافيًا بلا كلفة — و`ls` في BSD تفهمه، خلافًا لِـ`du` (انظر
//! `files_tree_size.rs`).
//!
//! ## مجلدٌ فارغ ليس خطأً
//!
//! هذا الملفّ يبني الأمر فحسب؛ هو لا يقرأ خرج `ls` ولا يحكم عليه. مجلدٌ فارغٌ
//! ينتج أمرًا صحيحًا تمامًا كمجلدٍ مزدحم — القائمة الفارغة إجابةٌ مشروعة عن
//! «ماذا في هذا المجلد؟»، والقرار في عرضها فارغةً أو برسالةٍ خاصة يقع في طبقة
//! تحليل النتائج بعد التنفيذ، لا هنا.
//!
//! ## ما لا تفعله
//!
//! لا تنزل إلى المجلدات الفرعية (‏لا `-R`)، ولا تقيس أحجامًا (‏لا `-l` ولا
//! `-s`)؛ هذه أسئلةٌ أخرى تجيب عنها عملياتٌ أخرى (‏`files.tree.size`،
//! `files.find.large`). هذه العملية تجيب عن سؤالٍ واحد: ماذا يوجد مباشرةً
//! داخل هذا المجلد؟

use crate::error::Result;
use crate::ops::common::{warn_if_resolved, Argv};
use crate::spec::*;
use crate::tools;
use crate::value::Inputs;

pub const ID: &str = "files.list";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.files.list.title",
    description_key: "op.files.list.description",
    category: Category::Files,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: tools::LS,
    conflict: Conflict::NoArtifact,
    inputs: &[InputSpec::new("folder", InputKind::ExistingDir)],
    sort_order: 90,
    search_terms: &[
        "ls",
        "list",
        "folder",
        "directory",
        "قائمة",
        "محتويات",
        "مجلد",
        "ملفات",
        "عرض",
    ],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let folder = inputs.dir("folder")?;

    let mut argv = Argv::tool(tools::LS, "explain.ls.list_tool")
        .flag("-1", "explain.ls.one_per_line")
        .flag("-A", "explain.ls.almost_all")
        .flag("-p", "explain.ls.mark_directories")
        .end_of_flags()
        .path(folder)
        .reveal(folder);

    if let Some(key) = warn_if_resolved(inputs, "folder", folder, "warn.source.resolved") {
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

    fn raw(folder: &Path) -> BTreeMap<String, RawValue> {
        BTreeMap::from([("folder".to_owned(), RawValue::Path(folder.display().to_string()))])
    }

    fn plan_with(folder: &Path) -> Result<PlannedCommand> {
        plan(&crate::value::validate(&SPEC, &raw(folder))?)
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("files.list must be listed");
        assert_eq!(found.category, Category::Files);
        assert_eq!(found.danger, Danger::Safe);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_is_the_documented_form_and_writes_nothing() {
        let s = Scratch::new("ls-argv").unwrap();
        let folder = s.dir("المجلد");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        let cmd = plan_with(&folder).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();

        assert_eq!(cmd.program, Path::new("/bin/ls"));
        assert_eq!(args[0], "-1");
        assert_eq!(args[1], "-A");
        assert_eq!(args[2], "-p");
        assert_eq!(args[3], "--");
        assert_eq!(Path::new(&args[4]), folder.as_path());
        assert_eq!(args.len(), 5);

        assert!(cmd.artifact.is_none(), "ls lists; it never produces a file");
        assert_eq!(cmd.reveal_target.as_deref(), Some(folder.as_path()));
    }

    #[test]
    fn the_explanation_is_the_argv_itself_not_a_second_rendering() {
        let s = Scratch::new("ls-explain").unwrap();
        let folder = s.dir("م");

        let cmd = plan_with(&folder).unwrap();
        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn no_argument_can_be_mistaken_for_a_flag() {
        let s = Scratch::new("ls-dashes").unwrap();
        let folder = s.dir("-rf");

        let cmd = plan_with(&folder).unwrap();
        // ‏-1، -A، -p رايات معلَنة؛ -- علامة فاصلة؛ ما بعدها مسارٌ لا يُقرأ رايةً.
        assert!(
            cannot_be_read_as_a_flag(&cmd.args[4]),
            "{:?} would be read as a flag",
            cmd.args[4]
        );
    }

    #[test]
    fn an_empty_folder_still_produces_a_valid_argv() {
        // القائمة الفارغة إجابةٌ مشروعة؛ الحكم عليها يقع بعد التنفيذ لا هنا.
        let s = Scratch::new("ls-empty").unwrap();
        let folder = s.dir("فارغ");

        let cmd = plan_with(&folder).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        let expected: Vec<String> = vec![
            "-1".to_owned(),
            "-A".to_owned(),
            "-p".to_owned(),
            "--".to_owned(),
            folder.display().to_string(),
        ];
        assert_eq!(args, expected);
    }

    #[test]
    fn an_ordinary_folder_raises_no_noisy_warning() {
        let s = Scratch::new("ls-quiet").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        let cmd = plan_with(&folder).unwrap();
        assert!(cmd.warnings.is_empty(), "unexpected warnings: {:?}", cmd.warnings);
    }

    #[test]
    fn shell_syntax_in_the_folder_name_stays_literal_and_adds_no_argument() {
        let s = Scratch::new("ls-shellish").unwrap();

        for name in ["مجلد 'اليوم'", "a; rm -rf ~", "$(whoami)", "back`tick`", "a & b"] {
            let folder = s.dir(name);
            let cmd = plan_with(&folder).unwrap();
            assert_eq!(Path::new(&cmd.args[4]), folder.as_path());
            assert_eq!(cmd.args.len(), 5, "{name:?} must not add arguments");
        }
    }

    #[test]
    fn a_symlinked_folder_is_resolved_and_the_substitution_is_announced() {
        let s = Scratch::new("ls-symlink").unwrap();
        let real = s.dir("الحقيقي");
        let link = s.path().join("رابط");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let cmd = plan_with(&link).unwrap();
        assert_eq!(Path::new(&cmd.args[4]), real.as_path());
        assert!(cmd.warnings.contains(&"warn.source.resolved"), "{:?}", cmd.warnings);
    }

    #[test]
    fn planning_leaves_the_folder_untouched() {
        let s = Scratch::new("ls-clean").unwrap();
        let folder = s.dir("م");
        std::fs::write(folder.join("ملف"), b"data").unwrap();

        for _ in 0..10 {
            plan_with(&folder).unwrap();
        }
        assert_eq!(s.names(&folder), vec!["ملف".to_string()]);
    }
}
