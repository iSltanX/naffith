//! تكامل حقيقي: مجلد فعلي ← `ditto` فعلية ← أرشيف يُفكّ ويُقارن.
//!
//! لا محاكاة ولا أداة بديلة. المسار هنا هو مسار الإنتاج نفسه:
//!
//! ```text
//! planner::plan  →  PlanStore  →  verify_still_valid  →  executor::run
//! ```
//!
//! ثم يُفكّ الأرشيف وتُقارن البنية والمحتوى بايتًا بايت، ويُتحقَّق أن المصدر لم
//! يتغيّر وأن الوجهة لا تحمل أثرًا واحدًا زائدًا. وكل ما يُنشأ يُحذف بعده.

use naffith_core::executor::{self, Outcome, OutputLine};
use naffith_core::journal::{Journal, State};
use naffith_core::planner::{self, PlanResponse};
use naffith_core::plans::{PlanStore, PlanToken, SessionId};
use naffith_core::policy::Policy;
use naffith_core::spec::Conflict;
use naffith_core::value::RawValue;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::sync::{mpsc, oneshot};

/// مساحة اختبار داخل المنزل، تُحذف كاملة عند الخروج من أي مسار.
///
/// المنزل لا `/var`: الجذور المسموحة لا تشمل مواضع `tempfile`، واختبارٌ يمرّ
/// بالمسار الإنتاجي لا بدّ أن يمرّ بسياسة المسارات كاملة.
struct Scratch(PathBuf);

impl Scratch {
    fn new(tag: &str) -> Option<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let base = home.join(format!(
            ".naffith-it-{tag}-{}-{}",
            std::process::id(),
            naffith_core::plans::random_suffix()
        ));
        std::fs::create_dir_all(&base).ok()?;
        Some(Scratch(base.canonicalize().ok()?))
    }
    fn path(&self) -> &Path {
        &self.0
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

/// قرص صغير مركّب تحت `/Volumes` — وهو جذر مسموح — لإعادة إنتاج امتلاء الوجهة.
///
/// امتلاء القرص ليس حالة نظرية: `ditto` عندها تخرج بـ 1 **وتترك أرشيفًا
/// جزئيًا** خلفها. هذا بالضبط ما يوجد نمطُ «المؤقّت ثم الترقية» لاحتوائه،
/// ولا سبيل لإثباته دون قرص حقيقي يمتلئ.
struct TinyVolume {
    image: PathBuf,
    mount: PathBuf,
}

/// البادئة التي تملكها هذه الاختبارات وحدها تحت `/private/tmp`.
const TINY_IMAGE_PREFIX: &str = "/private/tmp/naffith-tiny-";

/// يفصل أي قرص تركه تشغيلٌ سابق قُتل قبل أن يعمل `Drop`.
///
/// ليس ترفًا: قرصٌ متسرّب تحت `/Volumes` عطّل فعلًا خطوة تحزيم DMG في بناء
/// المنتج. و`Drop` لا يعمل تحت SIGKILL، فالكنس عند البدء هو التنظيف الوحيد
/// الممكن. المطابقة على **مسار الصورة** بالبادئة أعلاه لا على اسم الجهاز:
/// قرصٌ للمستخدم يصادف تشابه اسمه لا يجوز أن يُفصل.
fn sweep_leaked_volumes() {
    let Ok(out) = std::process::Command::new("/usr/bin/hdiutil").arg("info").output() else {
        return;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut ours = false;
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("image-path") {
            ours = path.trim_start_matches([' ', '\t', ':']).starts_with(TINY_IMAGE_PREFIX);
        } else if ours {
            if let Some(i) = line.find("/Volumes/") {
                let _ = std::process::Command::new("/usr/bin/hdiutil")
                    .args(["detach", "-quiet", "-force", line[i..].trim()])
                    .status();
            }
        }
    }
    if let Ok(read) = std::fs::read_dir("/private/tmp") {
        for e in read.flatten() {
            let p = e.path();
            if p.to_string_lossy().starts_with(TINY_IMAGE_PREFIX) {
                let _ = std::fs::remove_file(p);
            }
        }
    }
}

impl TinyVolume {
    /// يعيد `None` إن تعذّر إنشاء القرص أو تركيبه — لا نُفشل الحزمة كلها
    /// لأن البيئة لا تسمح بـ `hdiutil`.
    fn new(megabytes: u32) -> Option<Self> {
        sweep_leaked_volumes();
        let image = PathBuf::from(format!(
            "{TINY_IMAGE_PREFIX}{}-{}.dmg",
            std::process::id(),
            naffith_core::plans::random_suffix()
        ));
        let volname = format!("naffith-{}", naffith_core::plans::random_suffix());
        let ok = std::process::Command::new("/usr/bin/hdiutil")
            .args([
                "create",
                "-size",
                &format!("{megabytes}m"),
                "-fs",
                "HFS+",
                "-volname",
                &volname,
                "-quiet",
            ])
            .arg(&image)
            .status()
            .ok()?
            .success();
        if !ok {
            return None;
        }
        let out = std::process::Command::new("/usr/bin/hdiutil")
            .args(["attach", "-nobrowse"])
            .arg(&image)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let mount =
            text.lines().find_map(|l| l.find("/Volumes/").map(|i| l[i..].trim().to_owned()))?;
        Some(TinyVolume { image, mount: PathBuf::from(mount) })
    }
    fn path(&self) -> &Path {
        &self.mount
    }
}

impl Drop for TinyVolume {
    fn drop(&mut self) {
        let _ = std::process::Command::new("/usr/bin/hdiutil")
            .args(["detach", "-quiet", "-force"])
            .arg(&self.mount)
            .status();
        let _ = std::fs::remove_file(&self.image);
    }
}

/// بصمة شجرة: المسار النسبي ← محتواه. تُستخدم لإثبات التطابق ولإثبات أن
/// المصدر لم يتغيّر.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            let rel = path.strip_prefix(root).unwrap().to_string_lossy().into_owned();
            if kind.is_dir() {
                out.insert(format!("{rel}/"), Vec::new());
                stack.push(path);
            } else if kind.is_file() {
                out.insert(rel, std::fs::read(&path).unwrap());
            }
        }
    }
    out
}

fn compress_inputs(source: &Path, destination: &Path, name: &str) -> BTreeMap<String, RawValue> {
    BTreeMap::from([
        ("source".to_owned(), RawValue::Path(source.display().to_string())),
        ("destination".to_owned(), RawValue::Path(destination.display().to_string())),
        ("archive_name".to_owned(), RawValue::Text(name.to_owned())),
    ])
}

/// يخطّط عملية الضغط بسياسة الإنتاج.
fn plan_compress(
    store: &mut PlanStore,
    session: &SessionId,
    source: &Path,
    destination: &Path,
    name: &str,
) -> PlanResponse {
    planner::plan(
        store,
        session,
        Policy::production(),
        "compress.folder.zip",
        &compress_inputs(source, destination, name),
    )
    .expect("planning must succeed")
}

/// ينفّذ خطة محفوظة بنفس تسلسل `lib.rs`: انتزاع الرمز، إعادة التحقّق، تشغيل.
async fn execute_plan(
    store: &mut PlanStore,
    session: &SessionId,
    token: &str,
) -> (Outcome, Vec<OutputLine>) {
    let stored = store
        .take(&PlanToken::from(token.to_owned()), session)
        .expect("the token must be redeemable exactly once");
    stored.verify_still_valid().expect("preconditions must still hold");

    let (tx, mut rx) = mpsc::channel(256);
    let (_cancel_tx, cancel_rx) = oneshot::channel();
    let task = tokio::spawn(executor::run(stored.command.clone(), tx, cancel_rx));

    let mut lines = Vec::new();
    while let Some(l) = rx.recv().await {
        lines.push(l);
    }
    (task.await.unwrap(), lines)
}

/// يفكّ أرشيفًا باستخدام نفس الأداة، إلى مجلد فارغ.
fn extract(archive: &Path, into: &Path) {
    let status = std::process::Command::new("/usr/bin/ditto")
        .arg("-x")
        .arg("-k")
        .arg(archive)
        .arg(into)
        .status()
        .expect("ditto must be runnable to extract");
    assert!(status.success(), "extracting the archive failed: {status:?}");
}

/// يبني شجرة مصدر فيها عربية ومسافات ومجلد فرعي وملف فارغ.
fn build_source(root: &Path) {
    std::fs::write(root.join("ملف عربي.txt"), "محتوى عربي فيه سطر\nوسطر ثانٍ\n").unwrap();
    std::fs::write(root.join("with spaces.txt"), b"latin content").unwrap();
    std::fs::write(root.join("فارغ.bin"), b"").unwrap();

    let nested = root.join("مجلد فرعي");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(nested.join("nested file.txt"), b"nested payload").unwrap();
    std::fs::write(nested.join("ملف داخلي.txt"), "بيانات داخل مجلد فرعي").unwrap();

    let deeper = nested.join("أعمق");
    std::fs::create_dir(&deeper).unwrap();
    std::fs::write(deeper.join("deep.bin"), vec![7u8; 4096]).unwrap();
}

// ════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn a_real_folder_is_compressed_extracted_and_compared_byte_for_byte() {
    let scratch =
        Scratch::new("roundtrip").expect("HOME must be set for the path policy to be exercised");
    let source = scratch.dir("مشروع التصميم");
    let destination = scratch.dir("النسخ الاحتياطية");
    build_source(&source);

    let before = snapshot(&source);

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "نسخة ٢٠٢٦");
    let final_path = PathBuf::from(plan.produces.clone().unwrap());
    let temp_path = PathBuf::from(plan.writes_to.clone().unwrap());

    let (outcome, _lines) = execute_plan(&mut store, &session, plan.token.as_str()).await;

    // ── ١. النتيجة ────────────────────────────────────────────────────
    assert_eq!(
        outcome,
        Outcome::Success { produced: Some(final_path.display().to_string()) },
        "the run must report success carrying the FINAL path"
    );

    // ── ٢. الناتج ─────────────────────────────────────────────────────
    assert!(final_path.exists(), "the archive must exist at its final name");
    assert!(!temp_path.exists(), "the temporary file must not survive a success");
    let magic = std::fs::read(&final_path).unwrap();
    assert_eq!(&magic[..2], b"PK", "the output must actually be a PKZip archive");

    // ── ٣. الفكّ والمقارنة ────────────────────────────────────────────
    let extracted = scratch.dir("مفكوك");
    extract(&final_path, &extracted);

    // `--keepParent` تعني أن اسم المجلد المصدر يصير جذرًا داخل الأرشيف.
    let root_inside = extracted.join("مشروع التصميم");
    assert!(
        root_inside.is_dir(),
        "--keepParent must place the source folder at the archive root, saw: {:?}",
        std::fs::read_dir(&extracted).unwrap().flatten().map(|e| e.file_name()).collect::<Vec<_>>()
    );

    let mut after_extract = snapshot(&root_inside);
    // `--sequesterRsrc` تضع البيانات الوصفية في `__MACOSX`، وهي ليست جزءًا من
    // شجرة المستخدم فتُستثنى من المقارنة.
    after_extract.retain(|k, _| !k.starts_with("__MACOSX"));

    assert_eq!(after_extract, before, "the extracted tree must match the source exactly");

    // مقارنة صريحة لملف عربي، كي لا يكون التطابق تطابق خرائط فارغة.
    assert_eq!(
        std::fs::read_to_string(root_inside.join("ملف عربي.txt")).unwrap(),
        "محتوى عربي فيه سطر\nوسطر ثانٍ\n"
    );
    assert_eq!(
        std::fs::read(root_inside.join("مجلد فرعي/أعمق/deep.bin")).unwrap(),
        vec![7u8; 4096]
    );
    assert!(!after_extract.is_empty(), "the comparison must have compared something");

    // ── ٤. المصدر لم يتغيّر ───────────────────────────────────────────
    assert_eq!(snapshot(&source), before, "compressing must not touch the source");

    // ── ٥. لا أثر زائد في الوجهة ──────────────────────────────────────
    let leftovers: Vec<String> = std::fs::read_dir(&destination)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != "نسخة ٢٠٢٦.zip")
        .collect();
    assert_eq!(leftovers, Vec::<String>::new(), "only the archive may remain");
}

#[tokio::test]
async fn the_archive_of_an_arabic_named_folder_extracts_under_the_same_name() {
    let scratch =
        Scratch::new("arabicname").expect("HOME must be set for the path policy to be exercised");
    let source = scratch.dir("مجلدٌ عربيٌّ خالص");
    let destination = scratch.dir("و");
    std::fs::write(source.join("ملف.txt"), "نص").unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "أرشيف عربي");
    let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;
    assert!(outcome.is_success(), "got {outcome:?}");

    let extracted = scratch.dir("فكّ");
    extract(Path::new(&plan.produces.unwrap()), &extracted);
    assert!(
        extracted.join("مجلدٌ عربيٌّ خالص/ملف.txt").is_file(),
        "the Arabic name must survive the round trip unchanged"
    );
}

#[tokio::test]
async fn a_name_carrying_shell_syntax_produces_exactly_that_file() {
    let scratch =
        Scratch::new("shellname").expect("HOME must be set for the path policy to be exercised");
    let source = scratch.dir("it's a folder; rm -rf");
    let destination = scratch.dir("و");
    std::fs::write(source.join("$(whoami).txt"), b"literal").unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let name = "don't `run` me; echo $HOME";
    let plan = plan_compress(&mut store, &session, &source, &destination, name);
    let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;
    assert!(outcome.is_success(), "got {outcome:?}");

    // اسم الملف على القرص هو النص بحروفه — لا شيء فُسّر ولا شيء نُفّذ.
    let produced = PathBuf::from(plan.produces.unwrap());
    assert_eq!(produced.file_name().unwrap().to_str().unwrap(), format!("{name}.zip"));

    let extracted = scratch.dir("فكّ");
    extract(&produced, &extracted);
    assert!(extracted.join("it's a folder; rm -rf/$(whoami).txt").is_file());
    // ولا شيء نُفّذ: `rm -rf` لم يمسّ شيئًا.
    assert!(source.is_dir() && destination.is_dir());
}

#[tokio::test]
async fn ditto_failing_leaves_no_archive_and_no_temporary_file() {
    // نُفشل `ditto` بحذف المصدر بعد أن حُفظت الخطة، ثم نتجاوز إعادة التحقّق
    // عمدًا كي نصل إلى فشل الأداة نفسها لا إلى رفض الشرط المسبق.
    let scratch =
        Scratch::new("toolfail").expect("HOME must be set for the path policy to be exercised");
    let source = scratch.dir("مصدر");
    let destination = scratch.dir("وجهة");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "ناتج");
    let final_path = PathBuf::from(plan.produces.clone().unwrap());
    let temp_path = PathBuf::from(plan.writes_to.clone().unwrap());

    let stored = store.take(&PlanToken::from(plan.token.as_str().to_owned()), &session).unwrap();
    std::fs::remove_dir_all(&source).unwrap();

    let (tx, mut rx) = mpsc::channel(256);
    let (_cancel_tx, cancel_rx) = oneshot::channel();
    let task = tokio::spawn(executor::run(stored.command.clone(), tx, cancel_rx));
    let mut lines = Vec::new();
    while let Some(l) = rx.recv().await {
        lines.push(l);
    }
    let outcome = task.await.unwrap();

    assert!(
        matches!(outcome, Outcome::Failed { .. }),
        "a missing source must fail the run: {outcome:?}"
    );
    assert!(!final_path.exists(), "a failed run must not produce an archive");
    assert!(!temp_path.exists(), "a failed run must not leave a partial file");
    assert_eq!(std::fs::read_dir(&destination).unwrap().count(), 0, "the destination stays clean");
    assert!(!lines.is_empty(), "ditto must have said why on stderr");
}

#[tokio::test]
async fn a_run_that_cannot_be_promoted_is_not_recorded_as_a_success() {
    // الاختبار الحاسم لادّعاء «لا نجاح قبل الترقية»: الأداة تخرج بصفر، ثم
    // تفشل الترقية لأن الاسم النهائي ظهر بينهما. النتيجة يجب أن تكون خطأً،
    // والملف الموجود يجب أن يبقى كما هو.
    let scratch =
        Scratch::new("promote").expect("HOME must be set for the path policy to be exercised");
    let source = scratch.dir("مصدر");
    let destination = scratch.dir("وجهة");
    std::fs::write(source.join("ملف"), b"data").unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "ناتج");
    let final_path = PathBuf::from(plan.produces.clone().unwrap());
    let temp_path = PathBuf::from(plan.writes_to.clone().unwrap());

    let stored = store.take(&PlanToken::from(plan.token.as_str().to_owned()), &session).unwrap();
    // شخص آخر سبقنا إلى الاسم بعد التخطيط.
    std::fs::write(&final_path, b"PRECIOUS EXISTING DATA").unwrap();

    let (tx, mut rx) = mpsc::channel(256);
    let (_cancel_tx, cancel_rx) = oneshot::channel();
    let task = tokio::spawn(executor::run(stored.command.clone(), tx, cancel_rx));
    while rx.recv().await.is_some() {}
    let outcome = task.await.unwrap();

    assert_eq!(
        outcome,
        Outcome::Error { key: "err.dest.exists" },
        "a failed promotion is not a success"
    );
    assert_eq!(
        std::fs::read(&final_path).unwrap(),
        b"PRECIOUS EXISTING DATA",
        "the pre-existing file must be untouched"
    );
    assert!(!temp_path.exists(), "and the temporary file must still be cleaned up");

    // والسجل يقيّدها فشلًا، لا نجاحًا.
    let journal = Journal::new(None);
    journal.record(naffith_core::journal::Entry::new(
        "run-1",
        "compress.folder.zip",
        State::from_outcome(&outcome),
    ));
    assert_eq!(journal.recent()[0].state.name(), "failed");
    assert_eq!(journal.produced_for("run-1"), None, "nothing may be revealable");
}

#[tokio::test]
async fn re_verification_before_execute_catches_a_source_deleted_after_planning() {
    let scratch =
        Scratch::new("stale").expect("HOME must be set for the path policy to be exercised");
    let source = scratch.dir("مصدر");
    let destination = scratch.dir("وجهة");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "ناتج");

    std::fs::remove_dir_all(&source).unwrap();

    let stored = store.take(&PlanToken::from(plan.token.as_str().to_owned()), &session).unwrap();
    let r = stored.verify_still_valid();
    assert!(
        matches!(r, Err(naffith_core::error::CoreError::PlanStale { .. })),
        "the gap between plan and execute is real and must be re-checked: {r:?}"
    );
    assert_eq!(std::fs::read_dir(&destination).unwrap().count(), 0);
}

#[tokio::test]
async fn re_verification_catches_the_final_name_appearing_after_planning() {
    let scratch =
        Scratch::new("appeared").expect("HOME must be set for the path policy to be exercised");
    let source = scratch.dir("مصدر");
    let destination = scratch.dir("وجهة");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "ناتج");

    std::fs::write(plan.produces.clone().unwrap(), b"someone got here first").unwrap();

    let stored = store.take(&PlanToken::from(plan.token.as_str().to_owned()), &session).unwrap();
    assert!(matches!(
        stored.verify_still_valid(),
        Err(naffith_core::error::CoreError::PlanStale {
            detail: naffith_core::error::StaleReason::FinalPathAppeared
        })
    ));
    assert_eq!(
        std::fs::read(plan.produces.unwrap()).unwrap(),
        b"someone got here first",
        "the file that appeared must be untouched"
    );
}

#[tokio::test]
async fn two_archives_of_the_same_folder_can_be_made_side_by_side() {
    // العملية لا تستبدل شيئًا، فاسمان مختلفان ينتجان أرشيفين مستقلين.
    let scratch =
        Scratch::new("twice").expect("HOME must be set for the path policy to be exercised");
    let source = scratch.dir("مصدر");
    let destination = scratch.dir("وجهة");
    std::fs::write(source.join("ملف"), b"payload").unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();

    for name in ["نسخة أولى", "نسخة ثانية"] {
        let plan = plan_compress(&mut store, &session, &source, &destination, name);
        let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;
        assert!(outcome.is_success(), "{name}: {outcome:?}");
    }

    let mut names: Vec<String> = std::fs::read_dir(&destination)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["نسخة أولى.zip".to_string(), "نسخة ثانية.zip".to_string()]);
}

#[tokio::test]
async fn nothing_is_left_behind_anywhere_after_a_full_successful_run() {
    let scratch =
        Scratch::new("clean").expect("HOME must be set for the path policy to be exercised");
    let source = scratch.dir("مصدر");
    let destination = scratch.dir("وجهة");
    build_source(&source);

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "ناتج");
    let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;
    assert!(outcome.is_success(), "got {outcome:?}");

    // لا ملف فحص كتابة، ولا ملف مؤقّت، لا في الوجهة ولا في المصدر ولا فوقهما.
    for dir in [scratch.path(), &source, &destination] {
        let stray: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(".naffith-"))
            .collect();
        assert_eq!(stray, Vec::<String>::new(), "stray internal files in {}", dir.display());
    }
}

#[tokio::test]
async fn a_destination_that_runs_out_of_space_leaves_nothing_behind() {
    let Some(vol) = TinyVolume::new(2) else {
        eprintln!("skipped: hdiutil could not create a scratch volume in this environment");
        return;
    };
    let Some(s) = Scratch::new("nospace") else { return };

    // مصدر أكبر من القرص، **وغير قابل للضغط**. أول محاولة استعملت نمطًا
    // حسابيًا فانضغط 5MB إلى ما دون 2MB ونجح الضغط — البيانات هنا يجب أن
    // تأتي من مصدر عشوائي حقيقي وإلا لم يمتلئ القرص أصلًا.
    let src = s.dir("كبير");
    // `read_exact` وليس `read`: `/dev/urandom` تيّار لا ينتهي، وقراءته كاملة
    // لا تعود أبدًا.
    let mut blob = vec![0u8; 8 * 1024 * 1024];
    std::fs::File::open("/dev/urandom").unwrap().read_exact(&mut blob).unwrap();
    std::fs::write(src.join("big.bin"), &blob).unwrap();

    let before: Vec<_> = std::fs::read_dir(vol.path())
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.file_name()))
        .collect();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &src, vol.path(), "لن-يتسع");

    // التخطيط يحذّر مسبقًا لأن الشجرة أكبر من المساحة الحرة.
    assert!(
        plan.warnings.contains(&"warn.space.low"),
        "the plan should warn before running: {:?}",
        plan.warnings
    );

    let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;
    assert!(
        matches!(outcome, Outcome::Failed { .. }),
        "running out of space must be reported as a failure, got {outcome:?}"
    );

    // ditto وحدها كانت ستترك أرشيفًا جزئيًا بحجم ميغابايتين. الحارس يحذفه.
    let after: Vec<_> = std::fs::read_dir(vol.path())
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.file_name()))
        .collect();
    assert_eq!(
        after.len(),
        before.len(),
        "the full destination must be left exactly as it was, found: {after:?}"
    );
    assert!(
        !vol.path().join("لن-يتسع.zip").exists(),
        "no final archive may exist after a failed run"
    );
    assert!(
        !std::fs::read_dir(vol.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| e.file_name().to_string_lossy().starts_with(".naffith-")),
        "no temporary artefact may survive a failed run"
    );
}

#[test]
fn the_plan_names_the_tool_that_will_actually_run() {
    let Some(s) = Scratch::new("tool") else { return };
    let src = s.dir("مصدر");
    let dest = s.dir("وجهة");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &src, &dest, "أرشيف");

    assert_eq!(plan.tool.id, "ditto");
    assert_eq!(plan.tool.path, "/usr/bin/ditto");
    // الحقل المعلَن هو نفسه البرنامج في الأمر المعروض — لا وصف منفصل عنه
    // يمكن أن يفترق عمّا سيُنفَّذ.
    assert_eq!(plan.argv_display[0], plan.tool.path);
}

#[test]
fn the_plan_states_the_conflict_policy_before_anything_runs() {
    let Some(s) = Scratch::new("policy") else { return };
    let src = s.dir("مصدر");
    let dest = s.dir("وجهة");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &src, &dest, "أرشيف");
    assert_eq!(plan.conflict, Conflict::Refuse);
}

/// السياسة المعلَنة ليست وصفًا: كل عملية تعلن `Refuse` مطالَبة بأن ترفض فعلًا.
/// هذا الاختبار يمسح الفهرس كله، فعمليةٌ تُضاف غدًا وتعلن `Refuse` دون أن
/// تنفّذها تسقط هنا لا على ملفات مستخدم.
#[test]
fn every_operation_declaring_refusal_actually_refuses_a_taken_name() {
    let Some(s) = Scratch::new("declared") else { return };
    let src = s.dir("مصدر");
    std::fs::write(src.join("f"), b"x").unwrap();
    let dest = s.dir("وجهة");

    for op in naffith_core::registry::list(Policy::production()) {
        let spec = naffith_core::registry::find(op.id, Policy::production()).unwrap();
        if spec.conflict != Conflict::Refuse {
            continue;
        }
        let mut store = PlanStore::new();
        let session = store.register_session().unwrap();

        let first = planner::plan(
            &mut store,
            &session,
            Policy::production(),
            op.id,
            &compress_inputs(&src, &dest, "محجوز"),
        )
        .unwrap_or_else(|e| panic!("{} should plan on a clean destination: {e:?}", op.id));

        // احتلال الاسم النهائي بالضبط الذي أعلنته الخطة.
        std::fs::write(first.produces.as_ref().unwrap(), "سبقك غيرك").unwrap();

        let again = planner::plan(
            &mut store,
            &session,
            Policy::production(),
            op.id,
            &compress_inputs(&src, &dest, "محجوز"),
        );
        // المفتاح لا النوع: الخطأ قد يُلَفّ بنسبته إلى حقل، والمفتاح يبقى.
        let key = again.as_ref().err().map(|e| e.key());
        assert_eq!(
            key,
            Some("err.dest.exists"),
            "{} declares Conflict::Refuse but planning over a taken name gave {:?}",
            op.id,
            again.map(|p| p.produces)
        );
        // والأهم: الملف الموجود لم يُمَسّ.
        assert_eq!(
            std::fs::read(first.produces.as_ref().unwrap()).unwrap(),
            "سبقك غيرك".as_bytes()
        );
    }
}

#[tokio::test]
async fn the_size_is_reported_as_an_estimate_of_the_source_not_of_the_archive() {
    let Some(s) = Scratch::new("estimate") else { return };
    let src = s.dir("مصدر");
    // بيانات مكرّرة تنضغط بشدّة — فيصير الفرق بين التقدير والناتج ظاهرًا.
    std::fs::write(src.join("a.txt"), vec![b'a'; 200_000]).unwrap();
    std::fs::write(src.join("ب.txt"), vec![b'b'; 100_000]).unwrap();
    let dest = s.dir("وجهة");

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &src, &dest, "مضغوط");

    let est = plan.estimate.as_ref().expect("a compress plan must carry an estimate");
    assert_eq!(est.approx_source_bytes, 300_000, "the estimate sums the tree before compression");
    assert_eq!(est.scanned_entries, 2);
    assert!(est.complete, "a two-file tree is scanned fully");

    let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;
    assert!(matches!(outcome, Outcome::Success { .. }), "got {outcome:?}");

    // إثبات أن الحقل تقدير لا وعد: الأرشيف أصغر بكثير ممّا أُعلن.
    let produced = std::fs::metadata(plan.produces.as_ref().unwrap()).unwrap().len();
    assert!(
        produced < est.approx_source_bytes,
        "the archive ({produced}) should be smaller than the estimate ({}) — \
         which is exactly why the number must be labelled an estimate",
        est.approx_source_bytes
    );
}
