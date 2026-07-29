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
    TargetDir(PathBuf),
    /// اسم ملف مُنقّى، بلا فاصل مسار.
    Name(String),
    Text(String),
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

    pub fn flag(&self, id: &'static str) -> bool {
        matches!(self.values.get(id), Some(Value::Flag(true)))
    }

    /// المجلدات والملفات القائمة التي شارك بها المستخدم. تُستخدم لبصمة
    /// الشروط المسبقة قبل التنفيذ.
    pub fn existing_paths(&self) -> Vec<&Path> {
        self.values
            .values()
            .filter_map(|v| match v {
                Value::Dir(p) | Value::File(p) | Value::TargetDir(p) => Some(p.as_path()),
                _ => None,
            })
            .collect()
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
            (InputKind::Text { max_len }, RawValue::Text(s)) => {
                if s.len() > max_len {
                    return Err(CoreError::WrongInputType { id: spec.id });
                }
                Value::Text(s.clone())
            }
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

#[cfg(test)]
impl Inputs {
    /// بناء مباشر للاختبارات وحدها، يتجاوز التحقّق.
    pub fn from_pairs(pairs: Vec<(&'static str, Value)>) -> Self {
        Inputs { values: pairs.into_iter().collect(), as_given: BTreeMap::new() }
    }
}
