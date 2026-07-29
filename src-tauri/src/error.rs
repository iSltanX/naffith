//! أخطاء النواة.
//!
//! كل خطأ يحمل مفتاحًا نصيًا (`key`) تترجمه الواجهة من `locales/ar.json`، ولا
//! يحمل نصًا عربيًا داخل Rust. السبب: النواة تُختبر بلا واجهة، والنصوص تتغيّر
//! أكثر مما يتغيّر المنطق.

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    // ── الفهرس والسياسة ────────────────────────────────────────────────
    #[error("unknown operation: {0}")]
    UnknownOperation(String),

    /// عملية داخلية طُلبت في وضع الإنتاج. ليست "غير موجودة" — بل ممنوعة.
    #[error("operation is not available in this build: {0}")]
    OperationNotAvailable(String),

    // ── المدخلات ───────────────────────────────────────────────────────
    #[error("missing required input: {0}")]
    MissingInput(&'static str),

    #[error("input `{id}` has the wrong type")]
    WrongInputType { id: &'static str },

    #[error("unexpected input: {0}")]
    UnexpectedInput(String),

    #[error("invalid file name: {reason:?}")]
    InvalidName { reason: NameRejection },

    // ── المسارات ───────────────────────────────────────────────────────
    #[error("path must be absolute")]
    PathNotAbsolute,

    #[error("path escapes its parent (`..` is not allowed)")]
    PathTraversal,

    #[error("path is outside the allowed roots")]
    PathOutsideAllowedRoots,

    #[error("path is inside a protected system location")]
    PathProtected,

    #[error("path does not exist")]
    PathMissing,

    #[error("path is not a directory")]
    NotADirectory,

    #[error("destination already exists")]
    DestinationExists,

    #[error("destination is not writable")]
    DestinationNotWritable,

    /// الوجهة تقع داخل المصدر، فالأرشيف يُكتب داخل الشجرة التي يقرؤها.
    #[error("the destination lies inside the source")]
    DestinationInsideSource,

    /// لا ناتج مسجَّل لهذا التشغيل يمكن إظهاره.
    #[error("this run produced nothing to reveal")]
    NothingToReveal,

    // ── الأدوات ────────────────────────────────────────────────────────
    #[error("required tool `{id}` not found at its expected absolute path")]
    ToolMissing { id: &'static str },

    #[error("required tool `{id}` is not an executable regular file")]
    ToolNotExecutable { id: &'static str },

    // ── الخطط ──────────────────────────────────────────────────────────
    #[error("no such plan, or it has already been used or expired")]
    PlanNotFound,

    #[error("the plan is no longer valid: the filesystem changed since it was made")]
    PlanStale { detail: StaleReason },

    #[error("too many plans are open at once")]
    PlanLimitReached,

    // ── التنفيذ ────────────────────────────────────────────────────────
    #[error("no such run")]
    RunNotFound,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    // ── نسبة الخطأ إلى حقل ─────────────────────────────────────────────
    /// خطأٌ منسوبٌ إلى مدخل بعينه.
    ///
    /// «المسار غير موجود» رسالةٌ لا تُصلح شيئًا إن لم يعرف المستخدم أيّ حقل
    /// تعنيه. المفتاح يبقى مفتاح الخطأ الأصلي — الغلاف يضيف *أين* لا *ماذا*.
    #[error("input `{id}`: {source}")]
    OnInput {
        id: &'static str,
        #[source]
        source: Box<CoreError>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NameRejection {
    Empty,
    TooLong,
    ContainsSeparator,
    ContainsNul,
    ContainsControl,
    DotOrDotDot,
    LeadingDot,
    TrailingSpaceOrDot,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StaleReason {
    SourceGone,
    SourceReplaced,
    DestinationGone,
    DestinationNotWritable,
    /// ظهر ملف بالاسم النهائي بين التخطيط والتنفيذ.
    FinalPathAppeared,
    /// اختفت الأداة أو لم تعد تنفيذية بين التخطيط والتنفيذ.
    ToolGone,
}

impl CoreError {
    /// مفتاح ثابت تترجمه الواجهة. لا يتغيّر بتغيّر نص الخطأ.
    pub fn key(&self) -> &'static str {
        use CoreError::*;
        match self {
            UnknownOperation(_) => "err.op.unknown",
            OperationNotAvailable(_) => "err.op.unavailable",
            MissingInput(_) => "err.input.missing",
            WrongInputType { .. } => "err.input.type",
            UnexpectedInput(_) => "err.input.unexpected",
            InvalidName { .. } => "err.name.invalid",
            PathNotAbsolute => "err.path.relative",
            PathTraversal => "err.path.traversal",
            PathOutsideAllowedRoots => "err.path.outside",
            PathProtected => "err.path.protected",
            PathMissing => "err.path.missing",
            NotADirectory => "err.path.not_dir",
            DestinationExists => "err.dest.exists",
            DestinationNotWritable => "err.dest.readonly",
            DestinationInsideSource => "err.dest.inside_source",
            NothingToReveal => "err.reveal.nothing",
            ToolMissing { .. } => "err.tool.missing",
            ToolNotExecutable { .. } => "err.tool.not_exec",
            PlanNotFound => "err.plan.not_found",
            PlanStale { .. } => "err.plan.stale",
            PlanLimitReached => "err.plan.limit",
            RunNotFound => "err.run.not_found",
            Io(_) => "err.io",
            // الغلاف لا يغيّر ما حدث، بل يقول أين حدث.
            OnInput { source, .. } => source.key(),
        }
    }

    /// ينسب الخطأ إلى مدخل. لا يغلّف مرتين كي لا يضيع الحقل الأصلي.
    pub fn on_input(self, id: &'static str) -> Self {
        match self {
            CoreError::OnInput { .. } => self,
            other => CoreError::OnInput { id, source: Box::new(other) },
        }
    }

    /// المدخل الذي يخصّه هذا الخطأ، إن كان يخصّ مدخلًا.
    pub fn input(&self) -> Option<&'static str> {
        match self {
            CoreError::OnInput { id, .. } => Some(id),
            CoreError::MissingInput(id) => Some(id),
            CoreError::WrongInputType { id } => Some(id),
            _ => None,
        }
    }

    /// شكل الخطأ على السلك. مصدر واحد يستعمله `From` و`Serialize` معًا، فلا
    /// يفترق تمثيلان لنفس الخطأ.
    fn wire(&self) -> WireError {
        let inner = match self {
            CoreError::OnInput { source, .. } => source.as_ref(),
            other => other,
        };
        let detail = match inner {
            CoreError::InvalidName { reason } => serde_json::to_value(reason).ok(),
            CoreError::PlanStale { detail } => serde_json::to_value(detail).ok(),
            CoreError::ToolMissing { id } | CoreError::ToolNotExecutable { id } => {
                Some(serde_json::json!({ "tool": id }))
            }
            _ => None,
        };
        WireError { key: self.key(), input: self.input(), detail }
    }
}

/// شكل الخطأ كما يصل الواجهة. لا يحمل مسارات مطلقة ولا رسائل نظام خام.
#[derive(Debug, Serialize)]
pub struct WireError {
    pub key: &'static str,
    /// المدخل الذي يخصّه الخطأ، كي تعرضه الواجهة بجوار حقله لا في أعلى الشاشة.
    pub input: Option<&'static str>,
    pub detail: Option<serde_json::Value>,
}

impl From<CoreError> for WireError {
    fn from(e: CoreError) -> Self {
        e.wire()
    }
}

impl Serialize for CoreError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        self.wire().serialize(s)
    }
}

pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attributing_an_error_to_a_field_keeps_its_key() {
        let e = CoreError::PathMissing.on_input("source");
        assert_eq!(e.key(), "err.path.missing", "the wrapper says where, not what");
        assert_eq!(e.input(), Some("source"));
    }

    #[test]
    fn an_error_is_never_attributed_twice() {
        let e = CoreError::PathMissing.on_input("source").on_input("destination");
        assert_eq!(e.input(), Some("source"), "the innermost attribution is the true one");
    }

    #[test]
    fn the_wire_form_carries_the_field_and_the_original_detail() {
        let e = CoreError::InvalidName { reason: NameRejection::Empty }.on_input("archive_name");
        let wire = e.wire();
        assert_eq!(wire.key, "err.name.invalid");
        assert_eq!(wire.input, Some("archive_name"));
        assert_eq!(wire.detail, Some(serde_json::json!("empty")));
    }

    #[test]
    fn an_unattributed_error_names_no_field() {
        assert_eq!(CoreError::PlanNotFound.wire().input, None);
    }

    #[test]
    fn missing_and_mistyped_inputs_name_their_field_without_wrapping() {
        assert_eq!(CoreError::MissingInput("source").wire().input, Some("source"));
        assert_eq!(
            CoreError::WrongInputType { id: "archive_name" }.wire().input,
            Some("archive_name")
        );
    }

    #[test]
    fn the_two_serialisation_paths_agree() {
        // `From` و`Serialize` كانا نسختين متطابقتين يدويًا، وكان بينهما فرق.
        let a = CoreError::PathProtected.on_input("source");
        let b = CoreError::PathProtected.on_input("source");
        let from: WireError = a.into();
        let direct = serde_json::to_value(&b).unwrap();
        assert_eq!(serde_json::to_value(&from).unwrap(), direct);
    }

    #[test]
    fn no_error_key_leaks_a_filesystem_path() {
        // المفاتيح ثابتة معلومة سلفًا؛ لا تُبنى من مدخلات المستخدم.
        for e in [
            CoreError::PathMissing,
            CoreError::PathProtected,
            CoreError::DestinationExists,
            CoreError::DestinationInsideSource,
            CoreError::NothingToReveal,
        ] {
            assert!(e.key().starts_with("err."), "{:?}", e.key());
            assert!(!e.key().contains('/'), "an error key must not carry a path");
        }
    }
}
