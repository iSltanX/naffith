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
use naffith_core::spec::{Artifact, Conflict, PlannedCommand};
use naffith_core::value::{Inputs, RawValue};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::sync::{mpsc, oneshot};

/// مساحة اختبار داخل المنزل، تُحذف كاملة عند الخروج من أي مسار.
///
/// المنزل لا `/var`: الجذور المسموحة لا تشمل مواضع `tempfile`، واختبارٌ يمرّ
/// بالمسار الإنتاجي لا بدّ أن يمرّ بسياسة المسارات كاملة.
struct Scratch(PathBuf);

/// البادئة التي تملكها هذه الاختبارات وحدها داخل المنزل.
const SCRATCH_PREFIX: &str = ".naffith-it-";

/// يحذف مساحات تشغيلٍ سابق قُتل قبل أن يعمل `Drop`.
///
/// `Drop` لا يعمل تحت SIGKILL ولا تحت `cargo test` مقطوعًا بـ Ctrl-C، فتتراكم
/// مجلدات في منزل المستخدم. الشرط قبل الحذف أن تكون العملية صاحبة المجلد قد
/// ماتت فعلًا: `cargo test` آخر يعمل الآن بالتوازي مجلداتُه ليست قمامة.
fn sweep_stale_scratch() {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return };
    let Ok(read) = std::fs::read_dir(&home) else { return };
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(rest) = name.strip_prefix(SCRATCH_PREFIX) else { continue };
        // الشكل: `<وسم>-<معرّف العملية>-<عشوائي>`، والمعرّف قبل الأخير.
        let mut parts = rest.rsplit('-');
        let _random = parts.next();
        let Some(Ok(pid)) = parts.next().map(str::parse::<u32>) else { continue };
        if pid == std::process::id() || process_alive(pid) {
            continue;
        }
        let _ = std::fs::remove_dir_all(entry.path());
    }
}

fn process_alive(pid: u32) -> bool {
    std::process::Command::new("/bin/ps")
        .args(["-p", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(true)
}

impl Scratch {
    fn new(tag: &str) -> Option<Self> {
        sweep_stale_scratch();
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let base = home.join(format!(
            "{SCRATCH_PREFIX}{tag}-{}-{}",
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
        Self::formatted(megabytes, "HFS+")
    }

    /// نفسه بصيغة نظام ملفات مُعطاة. `ExFAT` هنا ليست ترفًا: هي صيغة كل ذاكرة
    /// USB وبطاقة كاميرا، و`/Volumes` جذرٌ مسموح من أجلها تحديدًا.
    fn formatted(megabytes: u32, fs: &str) -> Option<Self> {
        sweep_leaked_volumes();
        let image = PathBuf::from(format!(
            "{TINY_IMAGE_PREFIX}{}-{}.dmg",
            std::process::id(),
            naffith_core::plans::random_suffix()
        ));
        // ١١ محرفًا بالضبط: هذا سقف لصيقة FAT/exFAT، و`hdiutil create` يخرج
        // بـ ١ إن تجاوزته. `naffith-<١٦ خانة>` كان يتجاوزه، فكان القرص لا
        // يُنشأ والاختبار «ينجح» بتخطّيه صامتًا — وهو أسوأ من فشله.
        let volname = format!("naf{}", &naffith_core::plans::random_suffix()[..8]);
        let created = std::process::Command::new("/usr/bin/hdiutil")
            .args(["create", "-size", &format!("{megabytes}m"), "-fs", fs, "-volname", &volname])
            .arg(&image)
            .output()
            .ok()?;
        // بيئةٌ بلا `hdiutil` تُتخطّى بصمت، أما `hdiutil` موجودة ترفض أمرنا
        // فخطأٌ فينا نحن ويجب أن يُسمَع: التخطّي الصامت هو ما جعل اختبار exFAT
        // «يمرّ» وهو لم يعمل أصلًا.
        assert!(
            created.status.success(),
            "hdiutil refused to create a {fs} volume — fix the arguments, do not skip:\n{}",
            String::from_utf8_lossy(&created.stderr)
        );
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
///
/// الروابط الرمزية تُبصم بهدفها لا بمحتوى هدفها، وبسابقة تميّزها عن ملف عادي:
/// كانت تُهمَل صمتًا على طرفَي المقارنة معًا، فرابطٌ يضيع في الأرشيف أو ينقلب
/// نسخةً من هدفه كان يمرّ دون أن يسقط شيء.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            let rel = path.strip_prefix(root).unwrap().to_string_lossy().into_owned();
            if kind.is_symlink() {
                let mut mark = b"symlink -> ".to_vec();
                mark.extend_from_slice(std::fs::read_link(&path).unwrap().as_os_str().as_bytes());
                out.insert(rel, mark);
            } else if kind.is_dir() {
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

    // رابط رمزي نسبي: يجب أن يعود من الأرشيف رابطًا بنفس الهدف، لا نسخةً من
    // الملف ولا لا شيء.
    std::os::unix::fs::symlink("ملف عربي.txt", root.join("اختصار")).unwrap();
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
    // حارس على البصمة نفسها: لو عادت تتجاهل الروابط لصارت المقارنة أدناه
    // تقارن غيابًا بغياب وتمرّ على رابط ضائع.
    assert_eq!(
        before.get("اختصار").map(|v| String::from_utf8_lossy(v).into_owned()),
        Some("symlink -> ملف عربي.txt".to_owned()),
        "the snapshot must actually record symlinks for the comparison to mean anything"
    );

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
async fn a_tool_that_exits_zero_without_writing_never_becomes_a_zero_byte_archive() {
    // الدرز بين ملفّين، مقطوعًا بالمسار الكامل لا بالنيّة.
    //
    // `plans::Preconditions::claim_temp` صار يُنشئ المؤقّت حصريًا قبل الإطلاق
    // (‏سدًّا لسباق زرع رابط رمزي مكانه)، فصار المؤقّت **موجودًا دائمًا** لحظة
    // `atomic::ArtifactGuard::commit`. وشرط «لم تُنتج الأداة شيئًا» هناك كان
    // `!temp.exists()` — أي شرطٌ لم يعد يتحقّق أبدًا. النتيجة: أداةٌ تخرج بصفر
    // دون أن تكتب بايتًا واحدًا تُرقّي أرشيفًا بحجم صفر إلى اسمه النهائي،
    // ويُبلَّغ المستخدم **نجاحًا**، ويُقيَّد في السجل ناتجًا قابلًا للإظهار.
    //
    // الاختبار الوحيد الذي كان يغطّي الحالة يبني الحارس مباشرة بلا `verify`،
    // فلا يمرّ بالحجز ولا يمكن أن يلتقط الانحراف. هذا يمرّ بالسلسلة كلها:
    // ‏`insert` → `take` → `verify_still_valid` (‏فيقع الحجز) → `executor::run`
    // (‏فتقع الترقية). فإن عاد أحد الطرفين إلى شرط الوجود سقط هنا.
    //
    // `ditto` اليوم لا تخرج بصفر دون أن تكتب — والمقصود أن الثابتة لا تتّكل
    // على ذلك، فالحارس يَعِد بها في تعليقه هو.
    let Some(s) = Scratch::new("emptyoutput") else { return };
    let destination = s.dir("وجهة");
    let final_path = destination.join("ناتج.zip");
    let temp = naffith_core::atomic::temp_path_for(&final_path).unwrap();

    let command = PlannedCommand {
        // تخرج بصفر ولا تكتب شيئًا: محاكاة الأداة التي «نجحت» بلا ناتج.
        program: PathBuf::from("/usr/bin/true"),
        args: vec![],
        cwd: None,
        explain: vec![],
        warnings: vec![],
        artifact: Some(Artifact { temp: temp.clone(), final_path: final_path.clone() }),
        estimate: None,
    };

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let (token, _) =
        store.insert(&session, "compress.folder.zip", Inputs::default(), command).unwrap();

    let stored = store.take(&token, &session).unwrap();
    stored.verify_still_valid().expect("preconditions must hold on a clean destination");

    // الطرف الأول من الدرز: الحجز وقع فعلًا، والمؤقّت الآن موجود وفارغ.
    assert!(temp.exists(), "claim_temp must create the temporary file, not merely inspect it");
    assert_eq!(
        std::fs::metadata(&temp).unwrap().len(),
        0,
        "and it must be empty — which is exactly what makes exists() a dead check"
    );

    let (tx, mut rx) = mpsc::channel(256);
    let (_cancel_tx, cancel_rx) = oneshot::channel();
    let task = tokio::spawn(executor::run(stored.command.clone(), tx, cancel_rx));
    while rx.recv().await.is_some() {}
    let outcome = task.await.unwrap();

    // والطرف الثاني: الترقية رفضت.
    assert_eq!(
        outcome,
        Outcome::Error { key: "err.output.empty" },
        "a tool that produced nothing must not be reported as a success"
    );
    assert!(!final_path.exists(), "no zero-byte archive may carry the name the user chose");
    assert!(!temp.exists(), "and the claimed placeholder must not be left in the destination");
    assert_eq!(
        std::fs::read_dir(&destination).unwrap().count(),
        0,
        "the destination must be exactly as it was"
    );

    // ولا شيء يمكن إظهاره: النتيجة خطأ لا نجاح.
    let journal = Journal::new(None);
    journal.record(naffith_core::journal::Entry::new(
        "run-empty",
        "compress.folder.zip",
        State::from_outcome(&outcome),
    ));
    assert_eq!(journal.produced_for("run-empty"), None);
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

#[tokio::test]
async fn compressing_onto_an_exfat_volume_produces_the_archive_instead_of_destroying_it() {
    // الترقية كانت `hard_link` وحدها، و exFAT لا تعرف الروابط الصلبة فترجع
    // ENOTSUP. النتيجة: `ditto` تكتب الأرشيف كاملًا، ثم يُحذف ويرى المستخدم
    // `err.commit`. و exFAT ليست حالة طرفية — هي صيغة ذاكرة USB المعتادة،
    // و`/Volumes` فُتح جذرًا مسموحًا من أجلها.
    let Some(vol) = TinyVolume::formatted(16, "ExFAT") else {
        eprintln!("skipped: hdiutil could not create an ExFAT volume in this environment");
        return;
    };
    let Some(s) = Scratch::new("exfat") else { return };
    let source = s.dir("مشروع");
    std::fs::write(source.join("ملف عربي.txt"), "محتوى").unwrap();
    std::fs::write(source.join("data.bin"), vec![3u8; 2048]).unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, vol.path(), "نسخة");
    let final_path = PathBuf::from(plan.produces.clone().unwrap());
    let temp_path = PathBuf::from(plan.writes_to.clone().unwrap());

    let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;

    assert_eq!(
        outcome,
        Outcome::Success { produced: Some(final_path.display().to_string()) },
        "a filesystem without hard links must not cost the user the finished archive"
    );
    assert!(final_path.exists(), "the archive must exist at its final name on ExFAT");
    assert!(!temp_path.exists(), "and the temporary file must not survive");
    assert_eq!(&std::fs::read(&final_path).unwrap()[..2], b"PK");

    // ويُفكّ فعلًا: الترقية نقلت الملف كاملًا لا صدفةً منه.
    let extracted = s.dir("فكّ");
    extract(&final_path, &extracted);
    assert_eq!(std::fs::read_to_string(extracted.join("مشروع/ملف عربي.txt")).unwrap(), "محتوى");
}

#[tokio::test]
async fn an_existing_name_on_an_exfat_volume_is_still_never_replaced() {
    // القيمة كلها في هذا: البديل الذي يصلح exFAT لا يجوز أن يفتح بابًا
    // للاستبدال الصامت الذي وُجد `hard_link` أصلًا كي يغلقه.
    let Some(vol) = TinyVolume::formatted(16, "ExFAT") else {
        eprintln!("skipped: hdiutil could not create an ExFAT volume in this environment");
        return;
    };
    let Some(s) = Scratch::new("exfatclash") else { return };
    let source = s.dir("مصدر");
    std::fs::write(source.join("ملف"), b"data").unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, vol.path(), "ناتج");
    let final_path = PathBuf::from(plan.produces.clone().unwrap());
    let temp_path = PathBuf::from(plan.writes_to.clone().unwrap());

    let stored = store.take(&PlanToken::from(plan.token.as_str().to_owned()), &session).unwrap();
    // شخص آخر سبقنا إلى الاسم بعد التخطيط، فلا تلتقطه إعادة التحقّق.
    std::fs::write(&final_path, b"PRECIOUS EXISTING DATA").unwrap();

    let (tx, mut rx) = mpsc::channel(256);
    let (_cancel_tx, cancel_rx) = oneshot::channel();
    let task = tokio::spawn(executor::run(stored.command.clone(), tx, cancel_rx));
    while rx.recv().await.is_some() {}
    let outcome = task.await.unwrap();

    assert_eq!(outcome, Outcome::Error { key: "err.dest.exists" });
    assert_eq!(std::fs::read(&final_path).unwrap(), b"PRECIOUS EXISTING DATA");
    assert!(!temp_path.exists(), "and the temporary file must still be cleaned up");
}

#[tokio::test]
async fn extended_attributes_and_resource_forks_survive_the_round_trip() {
    // هذا هو المبرّر المعلَن كله لاختيار `ditto` على `zip`: `zip` تُسقط هذه
    // البيانات صامتة. مبرّرٌ بلا اختبار ادّعاء.
    let scratch = Scratch::new("metadata").expect("HOME must be set");
    let source = scratch.dir("مشروع");
    let destination = scratch.dir("وجهة");
    let file = source.join("ملف موسوم.txt");
    std::fs::write(&file, b"body").unwrap();

    // `/usr/bin/xattr` لا `libc`: مكتبة `libc` تبعيةٌ للنواة لا للاختبارات،
    // والأداة موجودة في كل macOS.
    let set = std::process::Command::new("/usr/bin/xattr")
        .args(["-w", "com.naffith.test", "قيمة عربية"])
        .arg(&file)
        .status()
        .expect("xattr must be runnable");
    assert!(set.success());
    // الشوكة المصدرية تُكتب بمسارها الخاص، وهي أقدم من xattr وتُحفظ كذلك.
    std::fs::write(file.join("..namedfork/rsrc"), b"RESOURCE FORK DATA").unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "موسوم");
    let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;
    assert!(outcome.is_success(), "got {outcome:?}");

    let extracted = scratch.dir("فكّ");
    extract(Path::new(&plan.produces.unwrap()), &extracted);
    let out = extracted.join("مشروع/ملف موسوم.txt");

    let read = std::process::Command::new("/usr/bin/xattr")
        .args(["-p", "com.naffith.test"])
        .arg(&out)
        .output()
        .expect("xattr must be runnable");
    assert_eq!(
        String::from_utf8_lossy(&read.stdout).trim(),
        "قيمة عربية",
        "the extended attribute must survive — losing it is what zip does and why ditto was chosen"
    );
    assert_eq!(
        std::fs::read(out.join("..namedfork/rsrc")).unwrap(),
        b"RESOURCE FORK DATA",
        "the resource fork must survive too"
    );
}

/// يقرأ الفهرس المركزي لأرشيف ZIP: (‏رايات عامة، اسم خام).
///
/// قراءة بايتات لا استدعاء `unzip`: البتّ الذي نفحصه (‏١١) لا تعرضه أي أداة،
/// وأثره يظهر مشوّهًا فقط.
fn central_directory(archive: &Path) -> Vec<(u16, Vec<u8>)> {
    let bytes = std::fs::read(archive).unwrap();
    let u16_at = |i: usize| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
    let u32_at = |i: usize| {
        u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]) as usize
    };

    let eocd = (0..bytes.len().saturating_sub(3))
        .rev()
        .find(|&i| &bytes[i..i + 4] == b"PK\x05\x06")
        .expect("every zip ends with an end-of-central-directory record");
    let count = u16_at(eocd + 10) as usize;
    let mut at = u32_at(eocd + 16);

    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        assert_eq!(&bytes[at..at + 4], b"PK\x01\x02", "malformed central directory");
        let flags = u16_at(at + 8);
        let (name_len, extra_len, comment_len) =
            (u16_at(at + 28) as usize, u16_at(at + 30) as usize, u16_at(at + 32) as usize);
        entries.push((flags, bytes[at + 46..at + 46 + name_len].to_vec()));
        at += 46 + name_len + extra_len + comment_len;
    }
    entries
}

#[tokio::test]
async fn the_archive_carries_utf8_names_without_the_utf8_flag_and_the_plan_says_so() {
    // `ditto` تكتب الأسماء UTF-8 خامًا وتترك البت ١١ صفرًا. أداةٌ تتبع
    // APPNOTE.TXT تقرؤها CP437 فتخرج مشوّهة: `unzip -l` يعرض `??????`.
    // لا راية في `ditto` تضبط البت (‏راجع `man ditto`)، والترقيع بعد الإنتاج
    // يخصّ ملفًا آخر — فالواجب هنا شيئان: أن يُقال قبل التنفيذ، وأن يُثبَّت
    // السلوك كي يكون تغيّره غدًا قرارًا واعيًا لا مفاجأة.
    let scratch = Scratch::new("efs").expect("HOME must be set");
    let source = scratch.dir("مجلد عربي");
    let destination = scratch.dir("و");
    std::fs::write(source.join("ملف تجريبي.txt"), "نص").unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "أرشيف");
    assert!(
        plan.warnings.contains(&"warn.zip.name_encoding"),
        "an Arabic tree must be told its names may not read correctly elsewhere: {:?}",
        plan.warnings
    );

    let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;
    assert!(outcome.is_success(), "got {outcome:?}");

    let entries = central_directory(Path::new(&plan.produces.unwrap()));
    let arabic: Vec<_> = entries.iter().filter(|(_, name)| !name.is_ascii()).collect();
    assert!(!arabic.is_empty(), "the archive must actually contain non-ASCII names");
    for (flags, name) in &arabic {
        assert!(
            std::str::from_utf8(name).is_ok(),
            "ditto writes raw UTF-8 name bytes: {:?}",
            String::from_utf8_lossy(name)
        );
        assert_eq!(
            flags & 0x0800,
            0,
            "pinned: ditto leaves the UTF-8 (EFS) bit clear. If this ever fails, ditto changed \
             and warn.zip.name_encoding should go — check before deleting this assertion."
        );
    }
}

#[tokio::test]
async fn a_file_named_like_an_appledouble_sidecar_is_dropped_and_the_plan_warned_first() {
    // خسارة بيانات صامتة: `--sequesterRsrc` تعامل `._*` سِجلًّا مصاحبًا لا
    // ملفًا، فلا يظهر في الأرشيف ولا في `__MACOSX`. لا يمكن منعها دون
    // `--norsrc` التي تُسقط البيانات الوصفية كلها — فتُعلَن.
    let scratch = Scratch::new("appledouble").expect("HOME must be set");
    let source = scratch.dir("مصدر");
    let destination = scratch.dir("وجهة");
    std::fs::write(source.join("ordinary.txt"), b"kept").unwrap();
    std::fs::write(source.join("._notes"), b"user data, not a sidecar").unwrap();

    let mut store = PlanStore::new();
    let session = store.register_session().unwrap();
    let plan = plan_compress(&mut store, &session, &source, &destination, "ناتج");
    assert!(
        plan.warnings.contains(&"warn.source.appledouble"),
        "the loss must be announced before the user commits to it: {:?}",
        plan.warnings
    );

    let (outcome, _) = execute_plan(&mut store, &session, plan.token.as_str()).await;
    assert!(outcome.is_success(), "got {outcome:?}");

    // وهذا ما يثبّت أن التحذير ليس احتياطًا نظريًا: الملف يضيع فعلًا.
    let extracted = scratch.dir("فكّ");
    extract(Path::new(&plan.produces.unwrap()), &extracted);
    assert!(extracted.join("مصدر/ordinary.txt").is_file());
    assert!(
        !extracted.join("مصدر/._notes").exists(),
        "pinned: ditto drops it. If this ever fails, --sequesterRsrc changed and the warning \
         should go — check before deleting this assertion."
    );
}

#[test]
fn a_scratch_directory_left_by_a_killed_run_is_swept_and_a_live_one_is_not() {
    // `Drop` لا يعمل تحت SIGKILL، فمجلدات الاختبار كانت تتراكم في منزل
    // المستخدم بلا حدّ. والشرط الآخر أهمّ: `cargo test` يعمل الآن بالتوازي
    // مجلداتُه ليست قمامة، وكنسُها كان سيُفشل تشغيلًا سليمًا.
    let home = PathBuf::from(std::env::var_os("HOME").expect("HOME must be set"));
    // معرّف لا يمكن أن يكون حيًّا: أعلى من سقف معرّفات العمليات على macOS.
    let dead = home.join(format!("{SCRATCH_PREFIX}killed-4294967290-{}", "0".repeat(16)));
    let live = home.join(format!(
        "{SCRATCH_PREFIX}running-{}-{}",
        std::process::id(),
        naffith_core::plans::random_suffix()
    ));
    std::fs::create_dir_all(dead.join("محتوى")).unwrap();
    std::fs::create_dir_all(&live).unwrap();

    sweep_stale_scratch();

    assert!(!dead.exists(), "a scratch directory whose owner is gone must be removed");
    assert!(live.exists(), "a scratch directory of a running process must be left alone");
    let _ = std::fs::remove_dir_all(&live);
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
