//! أدوات مشتركة بين ملفات العمليات.
//!
//! ## لماذا يوجد هذا الملف
//!
//! `compress_ditto` كان العملية الوحيدة، فكتب بانيَ وسائطه في ملفه. ومع
//! عشرات العمليات صار ذلك البانيَ نفسه مكتوبًا عشرات المرّات — وكل نسخةٍ منه
//! فرصةٌ لأن تُبنى `argv` في موضعٍ ويُبنى شرحُها في موضعٍ آخر، وهو بالضبط ما
//! تقوم المعمارية على منعه.
//!
//! هنا `Argv` تبني الاثنين من مصدرٍ واحد: كل نداءٍ يضيف رمزًا إلى الأمر
//! وشرحَه معًا، فلا يمكن لأحدهما أن يوجد بلا الآخر. واختبارٌ في كل عملية
//! يقارن `argv` بالشرح رمزًا برمز، فلو انفصلا يومًا سقط.
//!
//! ## الأخطاء تُجمَّع ولا تُرمى
//!
//! كل خطوةٍ تعيد `Self` لا `Result<Self>`، وتحتفظ بأول خطأ في `failure`.
//! السبب صياغيّ بحت: سلسلةٌ من عشر خطوات كلٌّ منها `?` تجعل بناء الأمر
//! جملةً مقطّعة، وتُغري بكتابة الشرح بعيدًا عن الوسيط الذي يشرحه. و`build`
//! هي الموضع الوحيد الذي يُرمى فيه الخطأ، فلا يضيع.
//!
//! ## وما لا يوجد هنا
//!
//! لا دالّة تقبل نصًّا وتحوّله إلى وسائط. الوسيط يُضاف واحدًا واحدًا بنوعه
//! المعلَن (راية، مسار، قيمة)، ولا توجد في هذا الملف صيغةٌ تعبّر عن «قسّم هذا
//! النصّ على المسافات» — وهي الصيغة التي كانت ستعيد الصدفة من بابٍ خلفي.

use crate::error::{CoreError, Result};
use crate::estimate::SizeEstimate;
use crate::spec::{Artifact, ExplainToken, PlannedCommand, TokenRole};
use crate::tools::Tool;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct Argv {
    program: PathBuf,
    args: Vec<OsString>,
    explain: Vec<ExplainToken>,
    cwd: Option<PathBuf>,
    warnings: Vec<&'static str>,
    estimate: Option<SizeEstimate>,
    reveal_target: Option<PathBuf>,
    extra_path: Vec<PathBuf>,
    failure: Option<CoreError>,
}

impl Argv {
    /// يبدأ أمرًا بأداةٍ تُحلّ لحظتها، ويجعل اسمها أول رمزٍ في الشرح.
    ///
    /// الحلّ هنا لا عند الإقلاع: غياب الأداة بين اللحظتين حالةٌ واقعية
    /// (تحديث نظام، أدوات Xcode تُزال).
    pub fn tool(tool: Tool, explain_key: &'static str) -> Self {
        match tool.resolve() {
            Ok(program) => {
                let explain = vec![ExplainToken::new(program.display().to_string(), explain_key)
                    .with_role(TokenRole::Tool)];
                Self {
                    program,
                    args: Vec::new(),
                    explain,
                    cwd: None,
                    warnings: Vec::new(),
                    estimate: None,
                    reveal_target: None,
                    extra_path: Vec::new(),
                    failure: None,
                }
            }
            Err(e) => Self {
                program: PathBuf::from(tool.absolute),
                args: Vec::new(),
                explain: Vec::new(),
                cwd: None,
                warnings: Vec::new(),
                estimate: None,
                reveal_target: None,
                extra_path: Vec::new(),
                failure: Some(e),
            },
        }
    }

    /// يبدأ أمرًا بأداةٍ مسارها **لم يُعرف وقت البناء** — اختاره المستخدم، أو
    /// اشتُقّ من اختياره — بعد أن مرّ بسياسة المسارات في `plan()` بالفعل.
    ///
    /// نظير `tool()` تمامًا في كل شيء إلا مصدر المسار: `Tool` ثابتٌ في
    /// الشيفرة (‏`/usr/bin/ditto`)، وهذا مسارٌ في `Inputs` بعد أن صار مطلقًا
    /// ومحلولًا. الفحص نفسه (‏`resolve_executable`) يقع على الاثنين، فلا
    /// يمكن لأداةٍ من أيّ مصدرٍ أن تُنفَّذ بلا تحقّق.
    pub fn resolved_tool(id: &'static str, program: &Path, explain_key: &'static str) -> Self {
        match crate::tools::resolve_executable(id, program) {
            Ok(program) => {
                let explain = vec![ExplainToken::new(program.display().to_string(), explain_key)
                    .with_role(TokenRole::Tool)];
                Self {
                    program,
                    args: Vec::new(),
                    explain,
                    cwd: None,
                    warnings: Vec::new(),
                    estimate: None,
                    reveal_target: None,
                    extra_path: Vec::new(),
                    failure: None,
                }
            }
            Err(e) => Self {
                program: program.to_path_buf(),
                args: Vec::new(),
                explain: Vec::new(),
                cwd: None,
                warnings: Vec::new(),
                estimate: None,
                reveal_target: None,
                extra_path: Vec::new(),
                failure: Some(e),
            },
        }
    }

    fn fail(mut self, e: CoreError) -> Self {
        if self.failure.is_none() {
            self.failure = Some(e);
        }
        self
    }

    /// راية مشروحة (`-c`، `--keepParent`).
    pub fn flag(mut self, text: &'static str, key: &'static str) -> Self {
        self.args.push(OsString::from(text));
        self.explain.push(ExplainToken::new(text, key).with_role(TokenRole::Flag));
        self
    }

    /// راية بلا شرحٍ خاص. لِما يشرحه ما قبله (‏`256` بعد `-a` مثلًا لها شرحها،
    /// أمّا `-a` نفسها فقد تُشرح مع قيمتها في مفتاحٍ واحد).
    pub fn bare_flag(mut self, text: &'static str) -> Self {
        self.args.push(OsString::from(text));
        self.explain.push(ExplainToken::bare(text).with_role(TokenRole::Flag));
        self
    }

    /// قيمةٌ تتبع رايةً (‏`256` بعد `-a`، `jpeg` بعد `-s format`).
    ///
    /// **تُرفض قيمةٌ تبدأ بشرطة.** القيم هنا تأتي من `Choice` أو `Number` أو من
    /// نصٍّ نقّته النواة، فبدؤها بشرطة يعني إمّا عطبَ برمجةٍ وإمّا مدخلًا أفلت من
    /// التنقية — وفي الحالين تصير قيمةٌ رايةً، وهي الثغرة الكلاسيكية.
    pub fn value(mut self, text: impl AsRef<str>) -> Self {
        let text = text.as_ref();
        if text.starts_with('-') {
            return self.fail(CoreError::ArgumentLooksLikeFlag);
        }
        self.args.push(OsString::from(text));
        self.explain.push(ExplainToken::bare(text));
        self
    }

    /// قيمةٌ مشروحة: نفس `value` مع مفتاح شرح.
    pub fn explained_value(mut self, text: impl AsRef<str>, key: &'static str) -> Self {
        let text = text.as_ref();
        if text.starts_with('-') {
            return self.fail(CoreError::ArgumentLooksLikeFlag);
        }
        self.args.push(OsString::from(text));
        self.explain.push(ExplainToken::new(text, key));
        self
    }

    /// اسمُ ملفٍ كما هو على القرص — بايتاتٍ لا نصًّا.
    ///
    /// لِـ`tar -C <أب> <الاسم>`: الاسم النسبي يمنع الأرشيف من حمل مسارٍ مطلق،
    /// وأسماء الملفات على macOS بايتات قد لا تكون UTF-8 صالحًا. المرور بـ
    /// `String` كان سيفسد الاسم أو يرفضه، وكلاهما خطأ على ملفٍ مشروع.
    ///
    /// والفحص على البايتات كذلك: اسمٌ يبدأ بشرطة يُقرأ رايةً مهما كان ترميزه.
    pub fn os_value(mut self, name: &OsStr) -> Self {
        let bytes = name.as_bytes();
        if bytes.is_empty() || bytes.starts_with(b"-") {
            return self.fail(CoreError::ArgumentLooksLikeFlag);
        }
        if bytes.contains(&b'/') {
            return self.fail(CoreError::PathTraversal);
        }
        self.args.push(name.to_os_string());
        self.explain.push(ExplainToken::bare(name.to_string_lossy().into_owned()));
        self
    }

    /// مسارٌ مطلق. الشرح يتركه بلا مفتاح: المسار بيانات لا راية.
    ///
    /// **يُرفض المسار غير المطلق.** كل مسارٍ يصل إلى هنا مرّ بـ`paths.rs` فصار
    /// مطلقًا ومحلولًا، والفحص يثبّت ذلك بدل الاعتماد عليه — ومسارٌ مطلق يبدأ
    /// بـ`/` حتمًا، فلا يمكن أن يُقرأ رايةً.
    pub fn path(mut self, p: &Path) -> Self {
        if !p.is_absolute() {
            return self.fail(CoreError::PathNotAbsolute);
        }
        self.args.push(p.as_os_str().to_os_string());
        self.explain
            .push(ExplainToken::bare(p.to_string_lossy().into_owned()).with_role(TokenRole::Path));
        self
    }

    /// مسارٌ مشروح — للمصدر والمؤقّت، حيث يستحقّ الموضع كلمة.
    pub fn explained_path(mut self, p: &Path, key: &'static str) -> Self {
        if !p.is_absolute() {
            return self.fail(CoreError::PathNotAbsolute);
        }
        self.args.push(p.as_os_str().to_os_string());
        self.explain.push(ExplainToken {
            token: p.to_string_lossy().into_owned(),
            key: Some(key),
            role: TokenRole::Path,
        });
        self
    }

    /// فاصل نهاية الرايات. بعده تُقرأ كل الوسائط بياناتٍ مهما بدت.
    ///
    /// يُضاف لكل أداةٍ تفهمه وتقبل مدخلاتٍ من المستخدم: حزامُ أمانٍ فوق كون
    /// المسارات مطلقة أصلًا. وليس كل أداةٍ تفهمه — `sips` مثلًا لا تفهمه —
    /// فيُضاف حيث يعمل لا حيث يبدو مرتّبًا.
    pub fn end_of_flags(mut self) -> Self {
        self.args.push(OsString::from("--"));
        self.explain
            .push(ExplainToken::new("--", "explain.end_of_flags").with_role(TokenRole::Flag));
        self
    }

    /// مجلد عملٍ معلَن. للأدوات التي تعمل على «هنا» (‏`git init`).
    pub fn in_directory(mut self, cwd: &Path) -> Self {
        if !cwd.is_absolute() {
            return self.fail(CoreError::PathNotAbsolute);
        }
        self.cwd = Some(cwd.to_path_buf());
        self
    }

    /// ‏`PATH` الذي يُمنح للطفل وحده، فوق البيئة الممسوحة. انظر توثيق
    /// `PlannedCommand::extra_path` — القرار الأمني هناك، وهذا موضع تفعيله.
    ///
    /// كل دليلٍ هنا يُرفض إن لم يكن مطلقًا: دليلٌ نسبي في `PATH` يعني «الدليل
    /// الحالي وقت التشغيل»، وهذا بالضبط ما تمنعه بقيّة هذا الملف من كل مسارٍ
    /// آخر.
    pub fn with_extra_path(mut self, dirs: Vec<PathBuf>) -> Self {
        for d in &dirs {
            if !d.is_absolute() {
                return self.fail(CoreError::PathNotAbsolute);
            }
        }
        self.extra_path = dirs;
        self
    }

    pub fn warn(mut self, key: &'static str) -> Self {
        if !self.warnings.contains(&key) {
            self.warnings.push(key);
        }
        self
    }

    /// تحذيرٌ قد لا يكون. يخدم `warn_if_resolved` وأخواتها.
    ///
    /// `for key in Option` كان يعمل ويقرأه المدقّق حلقةً على شيءٍ ليس مجموعة.
    /// هذه تقول ما يُقصد.
    pub fn warn_opt(self, key: Option<&'static str>) -> Self {
        match key {
            Some(k) => self.warn(k),
            None => self,
        }
    }

    pub fn warn_all(mut self, keys: Vec<&'static str>) -> Self {
        for k in keys {
            self = self.warn(k);
        }
        self
    }

    pub fn with_estimate(mut self, estimate: SizeEstimate) -> Self {
        self.estimate = Some(estimate);
        self
    }

    /// ما يفتحه «إظهار في Finder» بعد نجاحٍ لا يُنتج ناتجًا مؤقّتًا.
    ///
    /// عمليةٌ تُنتج `Artifact` لا تحتاجه — مسارها النهائي هو الجواب — وإعلانه
    /// معها يُتجاهَل في `build`، كي لا يوجد جوابان لسؤالٍ واحد.
    pub fn reveal(mut self, target: &Path) -> Self {
        if !target.is_absolute() {
            return self.fail(CoreError::PathNotAbsolute);
        }
        self.reveal_target = Some(target.to_path_buf());
        self
    }

    /// أمرٌ لا ينتج ملفًا.
    pub fn read_only(self) -> Result<PlannedCommand> {
        self.build(None, None)
    }

    /// أمرٌ ينتج ناتجًا مؤقّتًا يُرقّى بعد النجاح وحده.
    pub fn producing(self, artifact: Artifact) -> Result<PlannedCommand> {
        self.build(Some(artifact), None)
    }

    /// أمرٌ يكتب ناتجه على الخرج القياسي، فيُوجَّه إلى المؤقّت.
    ///
    /// الوجهة هي `artifact.temp` نفسه دائمًا، ولا سبيل لقول غير ذلك من هنا:
    /// `planner` يرفض أي توجيهٍ إلى موضعٍ آخر، وهذا التوقيع يجعل الرفض غير
    /// قابلٍ للتحقّق أصلًا من هذا الطريق.
    pub fn producing_via_stdout(self, artifact: Artifact) -> Result<PlannedCommand> {
        let temp = artifact.temp.clone();
        self.build(Some(artifact), Some(temp))
    }

    fn build(
        self,
        artifact: Option<Artifact>,
        stdout_to: Option<PathBuf>,
    ) -> Result<PlannedCommand> {
        if let Some(e) = self.failure {
            return Err(e);
        }
        let artifact_declared = artifact.is_some();
        Ok(PlannedCommand {
            program: self.program,
            args: self.args,
            cwd: self.cwd,
            explain: self.explain,
            warnings: self.warnings,
            artifact,
            estimate: self.estimate,
            stdout_to,
            // ناتجٌ مؤقّت يجيب عن «أين أنظر؟» بنفسه، فلا يُقبل جوابان.
            reveal_target: if artifact_declared { None } else { self.reveal_target },
            extra_path: self.extra_path,
        })
    }
}

/// هل يمكن أن يُقرأ هذا الوسيط رايةً؟ تستعمله اختبارات كل عملية.
pub fn cannot_be_read_as_a_flag(arg: &OsStr) -> bool {
    !arg.as_bytes().starts_with(b"-")
}

/// يحذّر إن كان ما اختاره المستخدم غير ما سيُنفَّذ عليه الأمر.
///
/// رابطٌ رمزي يُحلّ صامتًا مفاجأةٌ لا خدمة. مشتركةٌ لأن كل عملية تقبل مسارًا
/// تحتاجها، ونسيانُها في واحدةٍ منها يعني أن تلك العملية وحدها تفاجئ.
pub fn warn_if_resolved(
    inputs: &crate::value::Inputs,
    id: &'static str,
    resolved: &Path,
    key: &'static str,
) -> Option<&'static str> {
    let given = inputs.as_given(id)?;
    (Path::new(given) != resolved).then_some(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools;

    #[test]
    fn the_explanation_is_the_argv_itself_token_for_token() {
        let cmd = Argv::tool(tools::ECHO, "explain.echo.cmd")
            .flag("-n", "explain.echo.cmd")
            .value("hello")
            .path(Path::new("/tmp"))
            .read_only()
            .unwrap();

        let mut expected = vec![cmd.program.display().to_string()];
        expected.extend(cmd.args.iter().map(|a| a.to_string_lossy().into_owned()));
        let shown: Vec<String> = cmd.explain.iter().map(|t| t.token.clone()).collect();
        assert_eq!(shown, expected);
    }

    #[test]
    fn a_value_that_looks_like_a_flag_is_refused() {
        let r = Argv::tool(tools::ECHO, "explain.echo.cmd").value("-rf").read_only();
        assert!(matches!(r, Err(CoreError::ArgumentLooksLikeFlag)), "got {r:?}");
    }

    #[test]
    fn a_relative_path_is_refused() {
        let r = Argv::tool(tools::ECHO, "explain.echo.cmd")
            .path(Path::new("relative/here"))
            .read_only();
        assert!(matches!(r, Err(CoreError::PathNotAbsolute)), "got {r:?}");
    }

    #[test]
    fn the_first_failure_is_the_one_reported() {
        // خطأٌ لاحق لا يطمس أوّلًا: أوّل ما انكسر هو ما يفيد المستخدم.
        let r = Argv::tool(tools::ECHO, "explain.echo.cmd")
            .value("-first")
            .path(Path::new("also/bad"))
            .read_only();
        assert!(matches!(r, Err(CoreError::ArgumentLooksLikeFlag)));
    }

    #[test]
    fn a_missing_tool_fails_the_whole_build_not_just_its_own_step() {
        let ghost = Tool::new("ghost", "/usr/bin/definitely-not-installed-xyz");
        let r = Argv::tool(ghost, "explain.echo.cmd").value("x").read_only();
        assert!(matches!(r, Err(CoreError::ToolMissing { id: "ghost" })), "got {r:?}");
    }

    #[test]
    fn warnings_are_deduplicated_so_the_screen_never_repeats_itself() {
        let cmd = Argv::tool(tools::ECHO, "explain.echo.cmd")
            .warn("warn.source.empty")
            .warn("warn.source.empty")
            .read_only()
            .unwrap();
        assert_eq!(cmd.warnings, vec!["warn.source.empty"]);
    }

    #[test]
    fn a_read_only_command_declares_no_artifact_and_no_redirect() {
        let cmd = Argv::tool(tools::ECHO, "explain.echo.cmd").value("x").read_only().unwrap();
        assert!(cmd.artifact.is_none());
        assert!(cmd.stdout_to.is_none());
    }

    #[test]
    fn a_stdout_producing_command_redirects_into_its_own_temp_and_nowhere_else() {
        let artifact = Artifact::file(PathBuf::from("/tmp/.t.part"), PathBuf::from("/tmp/out.txt"));
        let cmd = Argv::tool(tools::CAT, "explain.cat.tool")
            .path(Path::new("/tmp"))
            .producing_via_stdout(artifact.clone())
            .unwrap();
        assert_eq!(cmd.stdout_to.as_deref(), Some(artifact.temp.as_path()));
    }
}
