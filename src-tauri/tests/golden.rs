//! اختبارات ذهبية: مدخلات محدّدة ← `argv` متوقّع حرفيًا.
//!
//! غرضها أن يفشل البناء إن تغيّر أمرٌ دون قصد. تغيير راية أو ترتيب وسيط يجب
//! أن يكون قرارًا واعيًا يُحدَّث معه هذا الملف، لا أثرًا جانبيًا لإعادة هيكلة.

use naffith_core::planner;
use naffith_core::plans::{PlanStore, PlanToken};
use naffith_core::policy::Policy;
use naffith_core::value::RawValue;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn inputs(pairs: &[(&str, RawValue)]) -> BTreeMap<String, RawValue> {
    pairs.iter().map(|(k, v)| ((*k).to_owned(), v.clone())).collect()
}

/// مساحة اختبار داخل المنزل: الجذور المسموحة لا تشمل `/var` حيث يضع
/// `tempfile` مجلداته.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Option<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let base = home.join(format!(".naffith-golden-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).ok()?;
        Some(Scratch(base.canonicalize().ok()?))
    }
    fn dir(&self, name: &str) -> PathBuf {
        let p = self.0.join(name);
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn compress_inputs(source: &Path, destination: &Path, name: &str) -> BTreeMap<String, RawValue> {
    inputs(&[
        ("source", RawValue::Path(source.display().to_string())),
        ("destination", RawValue::Path(destination.display().to_string())),
        ("archive_name", RawValue::Text(name.to_owned())),
    ])
}

/// يخطّط عملية ويعيد `argv` كما سيُنفَّذ.
fn argv_for(op_id: &str, pairs: &[(&str, RawValue)]) -> Vec<String> {
    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(&mut store, &session, Policy::dev(), op_id, &inputs(pairs))
        .unwrap_or_else(|e| panic!("planning {op_id} failed: {e}"));
    plan.argv_display
}

#[test]
fn echo_builds_the_expected_argv() {
    assert_eq!(
        argv_for("internal.echo", &[("message", RawValue::Text("hello".into()))]),
        vec!["/bin/echo", "hello"]
    );
}

#[test]
fn the_program_is_an_absolute_path_never_a_bare_name() {
    let argv = argv_for("internal.echo", &[("message", RawValue::Text("x".into()))]);
    assert!(
        argv[0].starts_with('/'),
        "the program must be absolute so PATH cannot be hijacked: {argv:?}"
    );
}

#[test]
fn arabic_text_passes_through_unchanged_as_one_argument() {
    let message = "نقل مجلد المشروع إلى النسخ الاحتياطي";
    let argv = argv_for("internal.echo", &[("message", RawValue::Text(message.into()))]);
    assert_eq!(argv, vec!["/bin/echo", message]);
    assert_eq!(argv.len(), 2, "Arabic with spaces must stay a single argument");
}

#[test]
fn spaces_do_not_split_an_argument() {
    let message = "a b   c";
    let argv = argv_for("internal.echo", &[("message", RawValue::Text(message.into()))]);
    assert_eq!(argv.len(), 2);
    assert_eq!(argv[1], message);
}

#[test]
fn shell_metacharacters_stay_inside_one_literal_argument() {
    // هذا هو الاختبار الذي يثبت أن لا صدفة في المسار. لو مرّ نصّ كهذا عبر
    // `sh -c` لصار أمرين. هنا هو وسيط واحد بحروفه.
    let payloads = [
        "; rm -rf ~",
        "$(whoami)",
        "`id`",
        "a && b",
        "a | b",
        "a > /tmp/pwned",
        "--flag-that-looks-real",
        "'quoted' \"double\"",
        "newline\\ninside",
    ];
    for p in payloads {
        let argv = argv_for("internal.echo", &[("message", RawValue::Text(p.into()))]);
        assert_eq!(argv.len(), 2, "payload {p:?} must not split into extra arguments");
        assert_eq!(argv[1], p, "payload {p:?} must be passed verbatim");
    }
}

#[test]
fn the_explanation_covers_the_command_and_is_generated_with_it() {
    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::dev(),
        "internal.echo",
        &inputs(&[("message", RawValue::Text("مرحبا".into()))]),
    )
    .unwrap();

    // سَطْر ونَفِّذ ناتجا نفس الاستدعاء: كل رمز في الشرح يقابل شيئًا في الأمر.
    let joined: Vec<&str> = plan.explain.iter().map(|t| t.token.as_str()).collect();
    assert!(joined.contains(&"echo"));
    assert!(joined.contains(&"مرحبا"));
    assert_eq!(
        plan.explain.iter().filter(|t| t.key.is_some()).count(),
        1,
        "only the tool itself carries an explanation key; the payload is data"
    );
}

#[test]
fn planning_creates_no_artifact() {
    // خطّة ليست تنفيذًا: لا ناتج ولا ملف مؤقّت قبل الضغط على «نفّذ».
    //
    // ملحوظة صادقة: التخطيط ليس بلا أثر تمامًا. عمليةٌ لها مدخل `TargetDir`
    // تُنشئ ملف فحصٍ بحجم صفر داخل الوجهة وتحذفه فورًا، لأن قراءة بتات
    // الصلاحيات تكذب تحت ACL أو قرصٍ للقراءة فقط. `echo` لا وجهة لها فلا
    // يجري ذلك هنا — وسيحتاج M1 اختبارًا صريحًا لهذا السلوك.
    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::dev(),
        "internal.echo",
        &inputs(&[("message", RawValue::Text("x".into()))]),
    )
    .unwrap();
    assert!(plan.produces.is_none(), "echo produces nothing");
}

// ── ضغط مجلد في ZIP ────────────────────────────────────────────────────

#[test]
fn compress_builds_the_expected_ditto_argv() {
    let s = Scratch::new("argv").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("مجلد المشروع");
    let dst = s.dir("النسخ");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "نسخة اليوم"),
    )
    .unwrap();

    // الرايات مثبّتة حرفيًا: تغييرها قرار منتج يُحدَّث معه هذا الاختبار.
    assert_eq!(
        &plan.argv_display[..5],
        &[
            "/usr/bin/ditto".to_string(),
            "-c".to_string(),
            "-k".to_string(),
            "--sequesterRsrc".to_string(),
            "--keepParent".to_string(),
        ]
    );
    assert_eq!(Path::new(&plan.argv_display[5]), src.as_path());
    assert_eq!(plan.argv_display.len(), 7, "tool + 4 flags + source + output");
    assert_eq!(plan.produces.as_deref(), Some(dst.join("نسخة اليوم.zip").to_str().unwrap()));
    assert_eq!(plan.argv_display[6], plan.writes_to.clone().unwrap());
}

#[test]
fn the_last_argument_is_the_temporary_name_not_the_final_one() {
    // «سَطْر» يعرض الأمر الحقيقي. الأمر الحقيقي يكتب إلى اسم مؤقّت، والترقية
    // خطوة تالية يقوم بها التطبيق. عرض الاسم النهائي هنا كان كذبًا مريحًا.
    let s = Scratch::new("temp").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("م");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "ناتج"),
    )
    .unwrap();

    let last = plan.argv_display.last().unwrap();
    assert_ne!(Some(last.as_str()), plan.produces.as_deref());
    assert!(Path::new(last).file_name().unwrap().to_str().unwrap().ends_with(".part"));
    assert_eq!(Path::new(last).parent(), Some(dst.as_path()), "must share a filesystem");
}

/// المطلب المركزي: ما يُعرض هو ما يُنفَّذ.
///
/// لا يقارن هذا بين نصّين بناهما نفس السطر — بل يخرج `argv` المخزَّنة من
/// المخزن (وهي التي ستذهب إلى `execve` حرفيًا) ويقارنها بما رأته الواجهة.
#[test]
fn the_argv_shown_to_the_user_is_the_argv_the_core_stored_for_execution() {
    let s = Scratch::new("identity").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("مصدر 'غريب' & مساحات");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "تقرير; نهائي"),
    )
    .unwrap();
    let shown = plan.argv_display.clone();

    let stored = store.take(&PlanToken::from(plan.token.as_str().to_owned()), &session).unwrap();

    let mut executed = vec![stored.command.program.display().to_string()];
    executed.extend(stored.command.args.iter().map(|a| a.to_string_lossy().into_owned()));

    assert_eq!(shown, executed, "the displayed command must be the executed command");
}

/// و«سَطْر» يرسم رموز الشرح نفسها، بنفس الترتيب.
#[test]
fn the_explanation_tokens_are_the_argv_token_for_token() {
    let s = Scratch::new("explain").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("م");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "ناتج"),
    )
    .unwrap();

    let explained: Vec<String> = plan.explain.iter().map(|t| t.token.clone()).collect();
    assert_eq!(explained, plan.argv_display, "سَطْر must not invent or drop a token");

    // كل راية تحمل مفتاح شرح؛ المسارات بيانات.
    let flags: Vec<&str> = plan
        .explain
        .iter()
        .filter(|t| t.key.is_some() && t.token.starts_with('-'))
        .map(|t| t.token.as_str())
        .collect();
    assert_eq!(flags, vec!["-c", "-k", "--sequesterRsrc", "--keepParent"]);
}

#[test]
fn arabic_and_spaced_paths_stay_single_arguments() {
    let s = Scratch::new("arabic").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("مجلد فيه مسافات و «علامات»");
    let dst = s.dir("وجهة الملفات");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "نسخة ٢٠٢٦"),
    )
    .unwrap();

    assert_eq!(plan.argv_display.len(), 7, "spaces must not split an argument");
    assert_eq!(Path::new(&plan.argv_display[5]), src.as_path());
    assert!(plan.produces.unwrap().ends_with("نسخة ٢٠٢٦.zip"));
}

#[test]
fn a_single_quote_in_a_name_never_splits_or_escapes_an_argument() {
    let s = Scratch::new("quote").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("it's a folder");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "don't $(touch) `me`; rm -rf"),
    )
    .unwrap();

    assert_eq!(plan.argv_display.len(), 7);
    assert_eq!(Path::new(&plan.argv_display[5]), src.as_path());
    // الاسم يصل نظام الملفات بحروفه، بلا هروب ولا حذف.
    assert!(plan.produces.unwrap().ends_with("don't $(touch) `me`; rm -rf.zip"));
}

#[test]
fn the_zip_extension_is_added_exactly_once() {
    let s = Scratch::new("ext").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("م");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    for (given, expected) in
        [("تقرير", "تقرير.zip"), ("تقرير.zip", "تقرير.zip"), ("تقرير.ZIP", "تقرير.ZIP")]
    {
        let plan = planner::plan(
            &mut store,
            &session,
            Policy::production(),
            "compress.folder.zip",
            &compress_inputs(&src, &dst, given),
        )
        .unwrap();
        assert_eq!(
            Path::new(&plan.produces.unwrap()).file_name().unwrap().to_str().unwrap(),
            expected,
            "for {given:?}"
        );
    }
}

#[test]
fn the_plan_declares_creates_and_does_not_create_anything() {
    let s = Scratch::new("creates").expect("HOME must be set for the path policy to be exercised");
    let src = s.dir("م");
    let dst = s.dir("و");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = planner::plan(
        &mut store,
        &session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(&src, &dst, "ناتج"),
    )
    .unwrap();

    assert_eq!(plan.danger, naffith_core::spec::Danger::Creates);
    assert!(!Path::new(&plan.produces.unwrap()).exists(), "planning must not produce the archive");
    assert!(!Path::new(&plan.writes_to.unwrap()).exists(), "nor the temporary file");
    assert_eq!(std::fs::read_dir(&dst).unwrap().count(), 0, "and no probe may linger");
}
