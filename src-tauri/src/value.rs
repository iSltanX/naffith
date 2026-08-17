//! قيم المدخلات: ما تُرسله الواجهة، وما تصير إليه بعد التحقّق.
//!
//! خاصية أمنية مقصودة: `RawValue` لا تملك أي صيغة تعبّر عن *أمر* أو *وسيط*
//! أو *مسار أداة*. أقصى ما تستطيع الواجهة قوله هو «هذا مسار» أو «هذا نص» أو
//! «هذه راية منطقية». بناء الأمر يقع كاملًا في `planner.rs`، ولا يوجد مسار في
//! الشيفرة يسمح لسلسلة نصية قادمة من الواجهة بأن تصبح وسيطًا غير مُتحقَّق منه.

use crate::error::{CoreError, Result};
use crate::paths;
use crate::spec::{InputKind, OperationSpec};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// ما يعبر حدّ IPC. ثلاث صيغ لا رابعة.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum RawValue {
    Path(String),
    Text(String),
    Flag(bool),
}

/// قيمة بعد التحقّق. المسارات هنا مُحلّة الروابط ومُتحقَّق من سياستها.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Dir(PathBuf),
    File(PathBuf),
    /// ملفٌ أو مجلد. العملية التي تعلن `ExistingPath` لا تفرّق بينهما.
    AnyPath(PathBuf),
    TargetDir(PathBuf),
    /// اسم ملف مُنقّى، بلا فاصل مسار.
    Name(String),
    Text(String),
    /// قيمةٌ من قائمةٍ مغلقة، مُتحقَّق أنها إحدى خياراتها المعلَنة.
    Choice(&'static str),
    Number(i64),
    /// عنوان `http`/`https` مُتحقَّق من صيغته.
    Url(String),
    Flag(bool),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inputs {
    values: BTreeMap<&'static str, Value>,
    /// المسارات **كما كتبها المستخدم**، قبل حلّ الروابط الرمزية.
    ///
    /// لا تُستخدم في بناء أي وسيط — الوسائط تُبنى من `values` المُتحقَّق منها
    /// وحدها. غرضها واحد: أن يلاحظ المخطِّط أن ما اختاره المستخدم ليس ما
    /// سيُنفَّذ عليه الأمر، فيحذّر. رابطٌ رمزي يُحلّ صامتًا مفاجأةٌ لا خدمة.
    as_given: BTreeMap<&'static str, String>,
}

impl Inputs {
    pub fn get(&self, id: &str) -> Option<&Value> {
        self.values.get(id)
    }

    /// النصّ الخام لمسارٍ كما وصل من الواجهة. للمقارنة والتحذير فقط.
    pub fn as_given(&self, id: &str) -> Option<&str> {
        self.as_given.get(id).map(String::as_str)
    }

    pub fn dir(&self, id: &'static str) -> Result<&Path> {
        match self.values.get(id) {
            Some(Value::Dir(p)) => Ok(p),
            Some(_) => Err(CoreError::WrongInputType { id }),
            None => Err(CoreError::MissingInput(id)),
        }
    }

    pub fn target_dir(&self, id: &'static str) -> Result<&Path> {
        match self.values.get(id) {
            Some(Value::TargetDir(p)) => Ok(p),
            Some(_) => Err(CoreError::WrongInputType { id }),
            None => Err(CoreError::MissingInput(id)),
        }
    }

    pub fn name(&self, id: &'static str) -> Result<&str> {
        match self.values.get(id) {
            Some(Value::Name(s)) => Ok(s),
            Some(_) => Err(CoreError::WrongInputType { id }),
            None => Err(CoreError::MissingInput(id)),
        }
    }

    pub fn text(&self, id: &'static str) -> Result<&str> {
        match self.values.get(id) {
            Some(Value::Text(s)) => Ok(s),
            Some(_) => Err(CoreError::WrongInputType { id }),
            None => Err(CoreError::MissingInput(id)),
        }
    }

    /// مسارٌ قائم أيًّا كان نوعه: مجلد، أو ملف، أو `ExistingPath`.
    ///
    /// يقبل الثلاثة عمدًا: عمليةٌ تعلن `ExistingFile` وأخرى تعلن `ExistingPath`
    /// كلتاهما تريد «المسار الذي اختاره المستخدم»، وقصرُه على صيغةٍ واحدة كان
    /// سيجبر كل عملية على معرفة أي دالّةٍ تناديها.
    pub fn any_path(&self, id: &'static str) -> Result<&Path> {
        match self.values.get(id) {
            Some(Value::AnyPath(p) | Value::Dir(p) | Value::File(p) | Value::TargetDir(p)) => Ok(p),
            Some(_) => Err(CoreError::WrongInputType { id }),
            None => Err(CoreError::MissingInput(id)),
        }
    }

    pub fn file(&self, id: &'static str) -> Result<&Path> {
        match self.values.get(id) {
            Some(Value::File(p)) => Ok(p),
            Some(_) => Err(CoreError::WrongInputType { id }),
            None => Err(CoreError::MissingInput(id)),
        }
    }

    pub fn choice(&self, id: &'static str) -> Result<&'static str> {
        match self.values.get(id) {
            Some(Value::Choice(s)) => Ok(s),
            Some(_) => Err(CoreError::WrongInputType { id }),
            None => Err(CoreError::MissingInput(id)),
        }
    }

    pub fn number(&self, id: &'static str) -> Result<i64> {
        match self.values.get(id) {
            Some(Value::Number(n)) => Ok(*n),
            Some(_) => Err(CoreError::WrongInputType { id }),
            None => Err(CoreError::MissingInput(id)),
        }
    }

    pub fn url(&self, id: &'static str) -> Result<&str> {
        match self.values.get(id) {
            Some(Value::Url(s)) => Ok(s),
            Some(_) => Err(CoreError::WrongInputType { id }),
            None => Err(CoreError::MissingInput(id)),
        }
    }

    pub fn flag(&self, id: &'static str) -> bool {
        matches!(self.values.get(id), Some(Value::Flag(true)))
    }

    /// المجلدات والملفات القائمة التي شارك بها المستخدم. تُستخدم لبصمة
    /// الشروط المسبقة قبل التنفيذ.
    pub fn existing_paths(&self) -> Vec<&Path> {
        self.values
            .values()
            .filter_map(|v| match v {
                Value::Dir(p) | Value::File(p) | Value::AnyPath(p) | Value::TargetDir(p) => {
                    Some(p.as_path())
                }
                _ => None,
            })
            .collect()
    }

    /// المدخلات كما تُقيَّد في السجل: معرّفٌ وقيمةٌ نصّية، والسرّيّ منقَّح.
    ///
    /// تُبنى من القيم **المُتحقَّق منها** لا من الخام: المسار الذي يُعاد التشغيل
    /// به يجب أن يكون المسار الذي نُفِّذ عليه فعلًا — أي المحلول — لا الرابط
    /// الرمزي الذي كتبه المستخدم وقد يشير اليوم إلى موضعٍ آخر.
    pub fn journal_form(
        &self,
        op: &crate::spec::OperationSpec,
    ) -> Vec<crate::journal::InputRecord> {
        self.values
            .iter()
            .map(|(id, value)| {
                let secret = op.input(id).is_some_and(|i| i.secret);
                crate::journal::InputRecord {
                    id: (*id).to_owned(),
                    value: if secret { None } else { Some(value.as_journal_text()) },
                }
            })
            .collect()
    }

    /// Render command arguments for the persistent run journal without
    /// retaining any value declared secret by the operation specification.
    ///
    /// Redacting `journal_form` alone is not sufficient: the same value may
    /// also occur in the argv that the Run Log persists and renders. Signed
    /// download URLs are the concrete case today. The executed command is not
    /// changed; this method is only for the audit record.
    pub fn journal_args(
        &self,
        op: &crate::spec::OperationSpec,
        args: &[std::ffi::OsString],
    ) -> Vec<String> {
        const REDACTED: &str = "[redacted]";

        let secrets: Vec<String> = self
            .values
            .iter()
            .filter(|(id, _)| op.input(id).is_some_and(|input| input.secret))
            .map(|(_, value)| value.as_journal_text())
            // An empty secret is not useful as a substring and would redact
            // every argument. Required secret inputs are validated elsewhere,
            // but this keeps the persistence helper fail-safe on its own.
            .filter(|value| !value.is_empty())
            .collect();

        args.iter()
            .map(|arg| {
                let rendered = arg.to_string_lossy();
                if secrets.iter().any(|secret| rendered.contains(secret)) {
                    REDACTED.to_owned()
                } else {
                    rendered.into_owned()
                }
            })
            .collect()
    }
}

impl Value {
    /// تمثيل القيمة نصًّا لإعادة ملء النموذج. يوافق ما تنتظره الواجهة في
    /// `FormValues`: الراية `'1'` أو `''`، وما عداها نصّه.
    pub fn as_journal_text(&self) -> String {
        match self {
            Value::Dir(p) | Value::File(p) | Value::AnyPath(p) | Value::TargetDir(p) => {
                p.display().to_string()
            }
            Value::Name(s) | Value::Text(s) | Value::Url(s) => s.clone(),
            Value::Choice(s) => (*s).to_owned(),
            Value::Number(n) => n.to_string(),
            Value::Flag(b) => (if *b { "1" } else { "" }).to_owned(),
        }
    }
}

/// يتحقّق من المدخلات الخام مقابل مواصفة العملية.
///
/// يرفض المفاتيح غير المعلَنة بدل تجاهلها: مفتاح لا تعرفه المواصفة يعني إما
/// واجهة قديمة أو محاولة تهريب، وكلاهما يستحق الرفض لا الصمت.
pub fn validate(op: &OperationSpec, raw: &BTreeMap<String, RawValue>) -> Result<Inputs> {
    for key in raw.keys() {
        if op.input(key).is_none() {
            return Err(CoreError::UnexpectedInput(key.clone()));
        }
    }

    let mut out = BTreeMap::new();
    let mut given = BTreeMap::new();

    for spec in op.inputs {
        let Some(raw_value) = raw.get(spec.id) else {
            if spec.required {
                return Err(CoreError::MissingInput(spec.id));
            }
            continue;
        };

        // كل خطأ هنا يُنسب إلى حقله: «المسار غير موجود» بلا حقلٍ رسالةٌ لا
        // تدلّ المستخدم على ما يصلحه.
        let value = match (spec.kind, raw_value) {
            (InputKind::ExistingDir, RawValue::Path(s)) => {
                Value::Dir(paths::existing_dir(Path::new(s)).map_err(|e| e.on_input(spec.id))?)
            }
            (InputKind::ExistingFile, RawValue::Path(s)) => {
                Value::File(paths::existing_file(Path::new(s)).map_err(|e| e.on_input(spec.id))?)
            }
            (InputKind::ExistingPath, RawValue::Path(s)) => {
                Value::AnyPath(paths::existing_path(Path::new(s)).map_err(|e| e.on_input(spec.id))?)
            }
            (InputKind::TargetDir, RawValue::Path(s)) => {
                Value::TargetDir(paths::target_dir(Path::new(s)).map_err(|e| e.on_input(spec.id))?)
            }
            (InputKind::NewName { ext }, RawValue::Text(s)) => {
                let clean = paths::sanitize_name(s).map_err(|e| e.on_input(spec.id))?;
                Value::Name(match ext {
                    Some(e) => paths::ensure_extension(&clean, e),
                    None => clean,
                })
            }
            (InputKind::NewDirName, RawValue::Text(s)) => {
                Value::Name(paths::sanitize_name(s).map_err(|e| e.on_input(spec.id))?)
            }
            (InputKind::Text { max_len }, RawValue::Text(s)) => {
                if s.len() > max_len {
                    return Err(CoreError::WrongInputType { id: spec.id });
                }
                Value::Text(s.clone())
            }
            // القيمة العائدة `&'static str` من قائمة المواصفة لا نسخةٌ من نصّ
            // الواجهة: ما يدخل الأمر يخرج من الشيفرة المُترجَمة، وما وصل من
            // الواجهة لا يفعل إلا أن يختار بينها.
            (InputKind::Choice { options }, RawValue::Text(s)) => {
                let chosen = options
                    .iter()
                    .find(|o| o.value == s.as_str())
                    .ok_or(CoreError::WrongInputType { id: spec.id })?;
                Value::Choice(chosen.value)
            }
            (InputKind::Number { min, max, .. }, RawValue::Text(s)) => {
                let n: i64 =
                    s.trim().parse().map_err(|_| CoreError::WrongInputType { id: spec.id })?;
                if n < min || n > max {
                    return Err(CoreError::NumberOutOfRange { id: spec.id, min, max });
                }
                Value::Number(n)
            }
            (InputKind::Url, RawValue::Text(s)) => Value::Url(sanitize_url(s, spec.id)?),
            (InputKind::Flag, RawValue::Flag(b)) => Value::Flag(*b),

            // نوع لا يطابق ما تعلنه المواصفة — رفض، لا تحويل ضمني.
            _ => return Err(CoreError::WrongInputType { id: spec.id }),
        };

        if let RawValue::Path(s) = raw_value {
            given.insert(spec.id, s.clone());
        }
        out.insert(spec.id, value);
    }

    Ok(Inputs { values: out, as_given: given })
}

/// أقصى طول عنوان يُقبل. أطول من أي عنوان معقول، وأقصر من أن يصير حمولة.
const MAX_URL_LEN: usize = 2048;

/// ينقّي عنوانًا قبل أن يصير وسيطًا لـ `curl`.
///
/// ثلاثة شروط، وكلٌّ منها يمنع شيئًا بعينه:
///
/// 1. **`http` أو `https` وحدهما.** `curl` تفهم `file:` و`scp:` و`smb:` وغيرها،
///    فعنوانٌ بلا فحص المخطّط يصير قراءةً لملفٍ محليّ يتجاوز `paths.rs` كاملًا.
/// 2. **لا محارف تحكّم ولا فراغ.** سطرٌ جديد داخل العنوان يفصله إلى شيئين في
///    كل سياق يُعرض فيه — الشرح والسجلّ — فيُقرأ غير ما يُنفَّذ.
/// 3. **لا يبدأ بشرطة.** المسارات المطلقة محميّةٌ ببدئها بـ `/`، والعنوان ليس
///    كذلك: `-o/etc/passwd` عنوانٌ صالح الشكل ورايةٌ صالحة معًا.
fn sanitize_url(raw: &str, id: &'static str) -> Result<String> {
    let url = raw.trim();
    if url.is_empty() || url.len() > MAX_URL_LEN {
        return Err(CoreError::InvalidUrl { id });
    }
    let lower = url.to_ascii_lowercase();
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        return Err(CoreError::InvalidUrl { id });
    }
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(CoreError::InvalidUrl { id });
    }
    if url.starts_with('-') {
        return Err(CoreError::InvalidUrl { id });
    }
    // مضيفٌ فارغ (`https://`) ليس عنوانًا.
    if lower.trim_start_matches("https://").trim_start_matches("http://").is_empty() {
        return Err(CoreError::InvalidUrl { id });
    }
    Ok(url.to_owned())
}

#[cfg(test)]
impl Inputs {
    /// بناء مباشر للاختبارات وحدها، يتجاوز التحقّق.
    pub fn from_pairs(pairs: Vec<(&'static str, Value)>) -> Self {
        Inputs { values: pairs.into_iter().collect(), as_given: BTreeMap::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_http_and_https_are_accepted_as_urls() {
        for good in ["http://example.com", "https://example.com/a?b=1", "HTTPS://EXAMPLE.COM"] {
            assert!(sanitize_url(good, "url").is_ok(), "{good} must be accepted");
        }
        for bad in [
            "file:///etc/passwd",
            "ftp://example.com",
            "scp://host/x",
            "smb://host/share",
            "//example.com",
            "example.com",
            "",
            "https://",
            "-o/etc/passwd",
            "https://example.com/\nHost: evil",
            "https://example.com/a b",
        ] {
            assert!(
                matches!(sanitize_url(bad, "url"), Err(CoreError::InvalidUrl { id: "url" })),
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn an_over_long_url_is_refused_before_it_becomes_an_argument() {
        let long = format!("https://example.com/{}", "a".repeat(MAX_URL_LEN));
        assert!(matches!(sanitize_url(&long, "url"), Err(CoreError::InvalidUrl { .. })));
    }

    #[test]
    fn the_url_is_trimmed_but_never_rewritten() {
        assert_eq!(
            sanitize_url("  https://example.com/x  ", "url").unwrap(),
            "https://example.com/x"
        );
    }
}
