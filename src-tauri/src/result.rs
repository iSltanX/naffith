//! Typed result metadata for the ResultView boundary.
//!
//! The frontend must not infer what an operation means by parsing stdout or
//! stderr.  This module is the single, closed mapping from every production
//! operation to its result presentation category, and the single place where
//! tool-specific exit codes are translated into domain answers.
//!
//! `RawOutput` is deliberately retained as the fail-safe for internal/future
//! operations and for registered tools whose human output has no stable grammar
//! we can validate. A diagnostic is also a first-class category: output that is
//! useful but cannot safely be structured remains honest text rather than a
//! guessed table.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;

/// Maximum output rows duplicated into the terminal `run://finished` result.
///
/// The live `run://output` stream is unaffected. ResultView retains the same
/// bounded tail it can render, instead of receiving the executor's much larger
/// safety ceiling a second time in one terminal event.
pub const MAX_EVENT_OUTPUT_LINES: usize = 1_000;

/// The stable presentation family used by ResultView.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultCategory {
    /// A new file or directory is the primary answer.
    Artifact,
    /// Completion is the answer and there is no artifact to inspect.
    Acknowledgement,
    /// A streamed set of findings, rows, branches, files, or processes.
    Collection,
    /// Named metadata or measurements about one subject.
    PropertiesReport,
    /// Numeric measurements such as bytes or free space.
    Metrics,
    /// A cryptographic digest.
    Digest,
    /// Two inputs or states were compared.
    Comparison,
    /// A bounded yes/no trust or integrity decision.
    Verdict,
    /// Diff/search output with exit-code-defined empty/difference semantics.
    DiffSearch,
    /// A failed or cancelled execution with diagnostic output.
    Diagnostic,
    /// Unstructured output with no operation-specific presentation contract.
    RawOutput,
}

/// What the completed operation established.
///
/// This is intentionally independent of `executor::Outcome`: `Outcome` says
/// whether the process infrastructure completed, while this enum says what the
/// operation answered.  For example, grep exit code 1 is a completed answer
/// (`NoMatches`), not an execution failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultSemantic {
    Completed,
    Matches,
    NoMatches,
    Differences,
    NoDifferences,
    Accepted,
    Rejected,
    Signed,
    Unsigned,
    Failed,
    Cancelled,
}

/// The kind of run-owned target that the existing `reveal(run_id)` command may
/// resolve.  This is descriptive metadata, never an authority: the frontend
/// still sends only the run id and `reveal.rs` revalidates the recorded path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevealKind {
    File,
    Directory,
}

/// A bounded line captured by Rust while it was already forwarding the stream.
///
/// This deliberately mirrors `run://output`. ResultView receives an explicitly
/// typed raw fallback and never needs to inspect a string to rediscover its
/// stream or truncation state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "stream", content = "line")]
pub enum RawOutputLine {
    Stdout(String),
    Stderr(String),
    /// Older emitted rows were omitted from a bounded terminal result.
    Omitted {
        dropped: usize,
    },
    /// The executor hit its broadcast ceiling and stopped emitting source rows.
    Truncated {
        dropped: usize,
    },
}

/// Streaming tail for the terminal result event.
///
/// When the head is omitted, the first retained row states how many emitted
/// rows were removed. An executor-provided truncation row is itself ordinary
/// typed data here and remains at the tail, so the event reports both layers:
/// rows omitted from this event and rows never emitted by the executor.
pub(crate) struct EventOutputTail {
    lines: VecDeque<RawOutputLine>,
    dropped: usize,
    limit: usize,
}

impl EventOutputTail {
    pub(crate) fn new() -> Self {
        Self::with_limit(MAX_EVENT_OUTPUT_LINES)
    }

    /// Use the same omission-aware tail for the much smaller persisted result.
    /// A plain `VecDeque` silently hid the fact that Run Log retained only its
    /// final rows.
    pub(crate) fn with_limit(limit: usize) -> Self {
        assert!(limit > 0, "an output tail must retain at least its omission marker");
        Self { lines: VecDeque::with_capacity(limit), dropped: 0, limit }
    }

    pub(crate) fn push(&mut self, line: RawOutputLine) {
        if self.lines.len() == self.limit {
            self.lines.pop_front();
            self.dropped += 1;
        }
        self.lines.push_back(line);
    }

    pub(crate) fn into_lines(mut self) -> Vec<RawOutputLine> {
        if self.dropped > 0 {
            if self.lines.len() == self.limit {
                self.lines.pop_front();
                self.dropped += 1;
            }
            self.lines.push_front(RawOutputLine::Omitted { dropped: self.dropped });
        }
        self.lines.into_iter().collect()
    }
}

/// The source of a display value inside a structured result.
///
/// A stderr value is not automatically an error: several macOS tools (notably
/// `codesign`) write their successful report there. The semantic answer comes
/// from the exit contract, while this field preserves provenance for styling
/// and copying.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// One opaque value selected by Rust for a structured family.
///
/// The value remains the tool's text. Its position and family are already
/// typed, so the frontend never splits, matches, or otherwise reinterprets it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredLine {
    pub value: String,
    pub stream: OutputStream,
}

/// A bounded-output condition, kept separate from real result rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum OutputNotice {
    Omitted { dropped: usize },
    Truncated { dropped: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionRow {
    pub cells: Vec<String>,
    pub stream: OutputStream,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultProperty {
    /// Operation-owned localization key for the row's role. The frontend may
    /// translate this key, but never has to parse the tool's prose to invent a
    /// label.
    pub label_key: String,
    pub value: String,
    pub stream: OutputStream,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricValue {
    /// Operation-owned localization key for the measurement's role.
    pub label_key: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub stream: OutputStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionKind {
    FileMatches,
    DirectorySizes,
    ArchiveEntries,
    FilesystemUsage,
    StorageDevices,
    DnsRecords,
    ListeningPorts,
    GitStatus,
    MergedBranches,
    Processes,
    GitLog,
    GitBlame,
    FileContent,
    DirectoryEntries,
    ProcessMatches,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportKind {
    Image,
    HttpHeaders,
    Permissions,
    ExtendedAttributes,
    SystemVersion,
    SystemProfile,
    GitVersion,
    FileType,
    Architecture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricsKind {
    NetworkLatency,
    SystemUptime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    Sha256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonKind {
    Sha256,
    GitDiff,
    Bytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerdictKind {
    ArchiveIntegrity,
    Gatekeeper,
    CodeSignature,
    CodeIntegrity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffSearchKind {
    Diff,
    Search,
}

/// The data portion of a result.
///
/// Every approved ResultView family has a distinct payload. Tool prose is only
/// promoted into a richer field where the format is deterministic (SHA-256's
/// 64 hexadecimal characters); everywhere else Rust supplies safe generic
/// rows/values and an operation-owned kind. Unknown operations and known tools
/// whose human output cannot be validated against a stable grammar use the raw
/// fallback instead of receiving a polished but invented structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ResultPayload {
    Artifact {
        path: String,
        /// اسمُ الناتج وحده — مشتقٌّ من المسار لا مقروءٌ من خرج الأداة.
        name: String,
        /// المجلد الحاوي. «أين وقع الناتج؟» جوابٌ مستقلٌّ عن «ما اسمه؟».
        parent: String,
        /// الحجم بالبايت، و`None` حين يتعذّر قياسه.
        ///
        /// اختياريٌّ عن قصد: القياس قراءةٌ من نظام الملفات بعد الترقية، وقد
        /// يفشل (سباقٌ مع حذفٍ خارجي، أو صلاحية). و`None` أصدق من صفرٍ يُعرض
        /// حجمًا. وللمجلد: مجموعُ ما تحته حتى عمقٍ محدود.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        size_bytes: Option<u64>,
        /// عددُ ما يحويه الناتج مباشرةً حين يكون مجلدًا، و`None` لملف.
        ///
        /// يُعدّ من نظام الملفات لا من خرج الأداة، فيصحّ لعمليات الاستخراج
        /// والتقسيم. ولا يُخمَّن لأرشيفٍ مضغوط: عددُ ما بداخله لا يُعرف إلا
        /// بقراءته، وهي قراءةٌ لا تقع هنا — فيبقى `None` بدل رقمٍ مُختلَق.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entries: Option<usize>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        output: Vec<RawOutputLine>,
    },
    Acknowledgement {
        message_key: String,
        details: Vec<StructuredLine>,
        notices: Vec<OutputNotice>,
    },
    Collection {
        kind: CollectionKind,
        columns: Vec<String>,
        rows: Vec<CollectionRow>,
        notices: Vec<OutputNotice>,
    },
    PropertiesReport {
        kind: ReportKind,
        properties: Vec<ResultProperty>,
        notices: Vec<OutputNotice>,
    },
    Metrics {
        kind: MetricsKind,
        metrics: Vec<MetricValue>,
        notices: Vec<OutputNotice>,
    },
    Digest {
        algorithm: DigestAlgorithm,
        value: Option<String>,
        details: Vec<StructuredLine>,
        notices: Vec<OutputNotice>,
    },
    Comparison {
        kind: ComparisonKind,
        reference: Option<String>,
        comparison: Option<String>,
        equal: Option<bool>,
        details: Vec<StructuredLine>,
        notices: Vec<OutputNotice>,
    },
    Verdict {
        kind: VerdictKind,
        value: ResultSemantic,
        details: Vec<StructuredLine>,
        notices: Vec<OutputNotice>,
    },
    DiffSearch {
        kind: DiffSearchKind,
        items: Vec<StructuredLine>,
        notices: Vec<OutputNotice>,
    },
    Diagnostic {
        lines: Vec<RawOutputLine>,
    },
    RawOutput {
        lines: Vec<RawOutputLine>,
    },
}

impl ResultPayload {
    fn category(&self) -> ResultCategory {
        match self {
            ResultPayload::Artifact { .. } => ResultCategory::Artifact,
            ResultPayload::Acknowledgement { .. } => ResultCategory::Acknowledgement,
            ResultPayload::Collection { .. } => ResultCategory::Collection,
            ResultPayload::PropertiesReport { .. } => ResultCategory::PropertiesReport,
            ResultPayload::Metrics { .. } => ResultCategory::Metrics,
            ResultPayload::Digest { .. } => ResultCategory::Digest,
            ResultPayload::Comparison { .. } => ResultCategory::Comparison,
            ResultPayload::Verdict { .. } => ResultCategory::Verdict,
            ResultPayload::DiffSearch { .. } => ResultCategory::DiffSearch,
            ResultPayload::Diagnostic { .. } => ResultCategory::Diagnostic,
            ResultPayload::RawOutput { .. } => ResultCategory::RawOutput,
        }
    }
}

/// Result metadata carried by a terminal run event and terminal journal entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultContract {
    pub category: ResultCategory,
    pub semantic: ResultSemantic,
    #[serde(flatten)]
    pub payload: ResultPayload,
    /// Omitted when this run has no currently safe, existing target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reveal: Option<RevealKind>,
}

#[derive(Deserialize)]
struct ResultContractWire {
    category: ResultCategory,
    semantic: ResultSemantic,
    #[serde(flatten)]
    payload: ResultPayload,
    #[serde(default)]
    reveal: Option<RevealKind>,
}

impl<'de> Deserialize<'de> for ResultContract {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;

        let wire = ResultContractWire::deserialize(deserializer)?;
        if wire.category != wire.payload.category() {
            return Err(D::Error::custom("result category does not match its payload type"));
        }

        let diagnostic = matches!(wire.payload, ResultPayload::Diagnostic { .. });
        let terminal_problem =
            matches!(wire.semantic, ResultSemantic::Failed | ResultSemantic::Cancelled);
        if diagnostic != terminal_problem {
            return Err(D::Error::custom(
                "failed/cancelled semantics require a diagnostic payload and vice versa",
            ));
        }
        if terminal_problem && wire.reveal.is_some() {
            return Err(D::Error::custom("a failed or cancelled result cannot be revealable"));
        }
        if let ResultPayload::Verdict { value, .. } = &wire.payload {
            if *value != wire.semantic {
                return Err(D::Error::custom("verdict value does not match result semantic"));
            }
        }

        Ok(Self {
            category: wire.category,
            semantic: wire.semantic,
            payload: wire.payload,
            reveal: wire.reveal,
        })
    }
}

impl ResultContract {
    pub fn for_operation(
        op_id: &str,
        semantic: ResultSemantic,
        produced: Option<&str>,
        output: Vec<RawOutputLine>,
        reveal: Option<RevealKind>,
    ) -> Self {
        if matches!(semantic, ResultSemantic::Failed | ResultSemantic::Cancelled) {
            return Self {
                category: ResultCategory::Diagnostic,
                semantic,
                payload: ResultPayload::Diagnostic { lines: output },
                // A diagnostic result never authorizes or advertises a
                // filesystem action. Production callers already derive this
                // as `None` from the failed outcome; enforcing it here keeps
                // the contract fail-closed for every caller and round-trip.
                reveal: None,
            };
        }

        let spec = spec_for(op_id);
        let Some(payload) = payload_for(spec.category, op_id, semantic, produced, output) else {
            // A successful artifact operation without a committed path violates
            // the core contract. Keep the wire shape honest and diagnosable
            // instead of emitting a mismatched category/payload pair.
            return Self {
                category: ResultCategory::Diagnostic,
                semantic: ResultSemantic::Failed,
                payload: ResultPayload::Diagnostic { lines: Vec::new() },
                reveal: None,
            };
        };
        // The mapping describes the approved presentation when Rust can prove
        // the tool output has that shape. A strict parser may instead return
        // RawOutput; the wire category must follow the payload actually sent,
        // never the aspirational template.
        let category = payload.category();
        Self { category, semantic, payload, reveal }
    }
}

/// حقائق الناتج التي تستطيع النواة إثباتها بنفسها.
///
/// كلّها من المسار ومن نظام الملفات — **لا شيء منها مقروءٌ من خرج الأداة**.
/// وهذا هو الشرط: ما لا يُثبَت يبقى `None` فتطويه الشاشة، بدل رقمٍ مصدره
/// تخمينُ نصٍّ بشريّ لم يُصمَّم ليُقرأ آليًا.
struct ArtifactFacts {
    name: String,
    parent: String,
    size_bytes: Option<u64>,
    entries: Option<usize>,
}

/// سقفٌ لعبور المجلد عند القياس.
///
/// حجمُ مجلدٍ عميق قد يكلّف آلاف قراءات القرص، والنتيجة سطرٌ واحد في الشاشة.
/// فيُقاس ما يُقاس بحدٍّ معلَن، وما تجاوزه يعود `None` — «لا أعرف» أصدق من
/// إبطاء الشاشة أو من رقمٍ ناقص يُعرض كاملًا.
const MAX_MEASURED_ENTRIES: usize = 20_000;

impl ArtifactFacts {
    fn measure(path: &Path) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let parent = path.parent().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();

        // `symlink_metadata` لا `metadata`: الناتج لا يُتبَع رابطُه عند القياس،
        // فلا يُنسب إلى العملية حجمُ شيءٍ لم تُنشئه.
        let meta = std::fs::symlink_metadata(path).ok();
        let is_dir = meta.as_ref().is_some_and(|m| m.is_dir());

        if !is_dir {
            return Self { name, parent, size_bytes: meta.map(|m| m.len()), entries: None };
        }

        let entries =
            std::fs::read_dir(path).ok().map(|dir| dir.take(MAX_MEASURED_ENTRIES).count());
        Self { name, parent, size_bytes: directory_size(path), entries }
    }
}

/// مجموع أحجام ما تحت المجلد، أو `None` إن تجاوز السقف أو تعذّرت القراءة.
fn directory_size(root: &Path) -> Option<u64> {
    let mut total = 0_u64;
    let mut seen = 0_usize;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).ok()? {
            let entry = entry.ok()?;
            seen += 1;
            if seen > MAX_MEASURED_ENTRIES {
                return None;
            }
            let meta = entry.metadata().ok()?;
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                total = total.saturating_add(meta.len());
            }
        }
    }
    Some(total)
}

fn payload_for(
    category: ResultCategory,
    op_id: &str,
    semantic: ResultSemantic,
    produced: Option<&str>,
    output: Vec<RawOutputLine>,
) -> Option<ResultPayload> {
    match category {
        ResultCategory::Artifact => {
            let path = produced?.to_owned();
            let facts = ArtifactFacts::measure(Path::new(&path));
            Some(ResultPayload::Artifact {
                name: facts.name,
                parent: facts.parent,
                size_bytes: facts.size_bytes,
                entries: facts.entries,
                path,
                output,
            })
        }
        ResultCategory::Acknowledgement => {
            let message_key = acknowledgement_message(op_id)?;
            let (details, notices) = structure_output(output);
            Some(ResultPayload::Acknowledgement {
                message_key: message_key.to_owned(),
                details,
                notices,
            })
        }
        ResultCategory::Collection => Some(
            collection_payload(op_id, &output)
                .unwrap_or(ResultPayload::RawOutput { lines: output }),
        ),
        ResultCategory::PropertiesReport => Some(
            report_payload(op_id, &output).unwrap_or(ResultPayload::RawOutput { lines: output }),
        ),
        ResultCategory::Metrics => Some(
            metrics_payload(op_id, &output).unwrap_or(ResultPayload::RawOutput { lines: output }),
        ),
        ResultCategory::Digest => {
            Some(digest_payload(&output).unwrap_or(ResultPayload::RawOutput { lines: output }))
        }
        ResultCategory::Comparison => {
            let kind = comparison_kind(op_id)?;
            if kind == ComparisonKind::Sha256 {
                Some(
                    hash_comparison_payload(&output)
                        .unwrap_or(ResultPayload::RawOutput { lines: output }),
                )
            } else {
                let equal = match semantic {
                    ResultSemantic::NoDifferences => Some(true),
                    ResultSemantic::Differences => Some(false),
                    _ => None,
                };
                let (details, notices) = structure_output(output);
                Some(ResultPayload::Comparison {
                    kind,
                    reference: None,
                    comparison: None,
                    equal,
                    details,
                    notices,
                })
            }
        }
        ResultCategory::Verdict => {
            let kind = verdict_kind(op_id)?;
            let (details, notices) = structure_output(output);
            Some(ResultPayload::Verdict { kind, value: semantic, details, notices })
        }
        ResultCategory::DiffSearch => {
            let kind = diff_search_kind(op_id)?;
            let (items, notices) = structure_output(output);
            Some(ResultPayload::DiffSearch { kind, items, notices })
        }
        ResultCategory::Diagnostic => Some(ResultPayload::Diagnostic { lines: output }),
        ResultCategory::RawOutput => Some(ResultPayload::RawOutput { lines: output }),
    }
}

fn structure_output(output: Vec<RawOutputLine>) -> (Vec<StructuredLine>, Vec<OutputNotice>) {
    let mut lines = Vec::new();
    let mut notices = Vec::new();
    for line in output {
        match line {
            RawOutputLine::Stdout(value) => {
                lines.push(StructuredLine { value, stream: OutputStream::Stdout });
            }
            RawOutputLine::Stderr(value) => {
                lines.push(StructuredLine { value, stream: OutputStream::Stderr });
            }
            RawOutputLine::Omitted { dropped } => {
                notices.push(OutputNotice::Omitted { dropped });
            }
            RawOutputLine::Truncated { dropped } => {
                notices.push(OutputNotice::Truncated { dropped });
            }
        }
    }
    (lines, notices)
}

/// Return only stdout values and bounded-output notices.
///
/// A successful tool can still write a warning to stderr. Until an operation
/// owns a stable schema for that warning, presenting it as a path, property, or
/// metric would be false. The caller therefore falls back to RawOutput when a
/// stderr value is present.
fn strict_stdout(output: &[RawOutputLine]) -> Option<(Vec<&str>, Vec<OutputNotice>)> {
    let mut lines = Vec::new();
    let mut notices = Vec::new();
    for line in output {
        match line {
            RawOutputLine::Stdout(value) => lines.push(value.as_str()),
            RawOutputLine::Stderr(_) => return None,
            RawOutputLine::Omitted { dropped } => {
                notices.push(OutputNotice::Omitted { dropped: *dropped });
            }
            RawOutputLine::Truncated { dropped } => {
                notices.push(OutputNotice::Truncated { dropped: *dropped });
            }
        }
    }
    Some((lines, notices))
}

fn row(cells: impl IntoIterator<Item = impl Into<String>>) -> CollectionRow {
    CollectionRow {
        cells: cells.into_iter().map(Into::into).collect(),
        stream: OutputStream::Stdout,
    }
}

fn collection(
    kind: CollectionKind,
    columns: &[&str],
    rows: Vec<CollectionRow>,
    notices: Vec<OutputNotice>,
) -> ResultPayload {
    ResultPayload::Collection {
        kind,
        columns: columns.iter().map(|column| (*column).to_owned()).collect(),
        rows,
        notices,
    }
}

/// Split a whitespace-delimited record into exactly `count` fields while
/// preserving whitespace in the final field (mount points and commands may
/// contain it). Repeated delimiter whitespace is ignored between fields.
fn split_fields(line: &str, count: usize) -> Option<Vec<&str>> {
    if count == 0 {
        return None;
    }
    let mut fields = Vec::with_capacity(count);
    let mut rest = line.trim();
    for _ in 1..count {
        let end = rest.find(char::is_whitespace)?;
        let field = &rest[..end];
        if field.is_empty() {
            return None;
        }
        fields.push(field);
        rest = rest[end..].trim_start();
    }
    if rest.is_empty() {
        return None;
    }
    fields.push(rest);
    Some(fields)
}

fn collection_payload(op_id: &str, output: &[RawOutputLine]) -> Option<ResultPayload> {
    let (lines, notices) = strict_stdout(output)?;
    match op_id {
        "files.find.large" | "files.find.stale" | "files.find.name" => {
            let mut rows = Vec::with_capacity(lines.len());
            for path in lines {
                if path.is_empty() || !path.starts_with('/') {
                    return None;
                }
                rows.push(row([path]));
            }
            Some(collection(CollectionKind::FileMatches, &["result.column.path"], rows, notices))
        }
        "files.tree.size" => {
            let mut rows = Vec::with_capacity(lines.len());
            for line in lines {
                let (size, path) = line.split_once('\t')?;
                if !valid_human_size(size) || path.is_empty() || !path.starts_with('/') {
                    return None;
                }
                rows.push(row([size, path]));
            }
            if rows.is_empty() {
                return None;
            }
            Some(collection(
                CollectionKind::DirectorySizes,
                &["result.column.size", "result.column.path"],
                rows,
                notices,
            ))
        }
        "disk.free" => parse_df(lines, notices),
        "net.dns" => {
            let mut rows = Vec::with_capacity(lines.len());
            for line in lines {
                if line.trim().is_empty() {
                    continue;
                }
                let fields = split_fields(line, 5)?;
                if fields[1].parse::<u64>().is_err()
                    || !fields[2].bytes().all(|byte| byte.is_ascii_alphabetic())
                    || !fields[3].bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                {
                    return None;
                }
                rows.push(row(fields));
            }
            Some(collection(
                CollectionKind::DnsRecords,
                &[
                    "result.column.dns.name",
                    "result.column.dns.ttl",
                    "result.column.dns.class",
                    "result.column.dns.type",
                    "result.column.dns.value",
                ],
                rows,
                notices,
            ))
        }
        "git.status" => parse_git_status(lines, notices),
        "git.branches.merged" => {
            let mut rows = Vec::with_capacity(lines.len());
            for line in lines {
                let (marker, branch) = line.get(..2).zip(line.get(2..))?;
                if !matches!(marker, "* " | "  ") || branch.trim().is_empty() {
                    return None;
                }
                rows.push(row([marker.trim(), branch.trim()]));
            }
            Some(collection(
                CollectionKind::MergedBranches,
                &["result.column.git.current", "result.column.git.branch"],
                rows,
                notices,
            ))
        }
        "system.processes" => parse_processes(lines, notices),
        "git.log" => parse_git_log(lines, notices),
        "git.blame" => parse_git_blame(lines, notices),
        // Any line is valid file content, including blank ones: there is no
        // grammar to validate here, unlike every other collection above.
        "git.show.file" => Some(collection(
            CollectionKind::FileContent,
            &["result.column.content"],
            lines.into_iter().map(|line| row([line])).collect(),
            notices,
        )),
        // Same reasoning as `git.show.file`: `ls -1Ap` output is one name per
        // line with no further grammar to validate, including on an empty
        // folder (zero rows is a legitimate answer, not a parse failure).
        "files.list" => Some(collection(
            CollectionKind::DirectoryEntries,
            &["result.column.name"],
            lines.into_iter().map(|line| row([line])).collect(),
            notices,
        )),
        "system.process.find" => parse_process_matches(lines, notices),
        // These commands intentionally request human reports whose field
        // layout is not a stable interface. Keep their approved future family
        // in MAPPINGS, but do not manufacture cells from prose today.
        //
        // The `lsof` trio and `log show` join that list for the same reason:
        // `lsof`'s column set shifts with what it finds (a socket row and a
        // regular-file row do not carry the same fields), and `log show`
        // prints human prose after its timestamp. Both stay honest text.
        "compress.zip.list"
        | "compress.tar.list"
        | "disk.list"
        | "net.ports"
        | "net.port.owner"
        | "system.process.open_files"
        | "disk.directory.open_handles"
        | "system.log.recent" => None,
        _ => None,
    }
}

/// Parses `pgrep -l` output: one match per line, `<pid> <name>`.
///
/// Unlike the `lsof` tables above this really is a stable two-field grammar —
/// `pgrep` prints the pid, one space, then the process name — so it becomes a
/// table rather than raw text. A line whose first field is not a number, or
/// which carries no name at all, drops the whole result to `RawOutput` rather
/// than emit a half-right table.
fn parse_process_matches(lines: Vec<&str>, notices: Vec<OutputNotice>) -> Option<ResultPayload> {
    let mut rows = Vec::with_capacity(lines.len());
    for line in lines {
        let (pid, name) = line.split_once(' ')?;
        if pid.parse::<u32>().is_err() || name.trim().is_empty() {
            return None;
        }
        rows.push(row([pid, name.trim()]));
    }
    if rows.is_empty() {
        return None;
    }
    Some(collection(
        CollectionKind::ProcessMatches,
        &["result.column.process.pid", "result.column.process.name"],
        rows,
        notices,
    ))
}

/// Parses `git log --format=%H%x09%ad%x09%an%x09%s --date=short` output: one
/// row per commit, four tab-separated fields. Any line that does not split
/// into exactly four fields, or whose hash/date/author look wrong, falls the
/// whole result back to `RawOutput` rather than emit a partially-wrong table.
fn parse_git_log(lines: Vec<&str>, notices: Vec<OutputNotice>) -> Option<ResultPayload> {
    let mut rows = Vec::with_capacity(lines.len());
    for line in lines {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 4 {
            return None;
        }
        let (hash, date, author, subject) = (fields[0], fields[1], fields[2], fields[3]);
        if !valid_git_hash(hash) || !valid_short_date(date) || author.is_empty() {
            return None;
        }
        rows.push(row([hash, date, author, subject]));
    }
    Some(collection(
        CollectionKind::GitLog,
        &[
            "result.column.git.hash",
            "result.column.git.date",
            "result.column.git.author",
            "result.column.git.subject",
        ],
        rows,
        notices,
    ))
}

/// Parses `git blame --line-porcelain` output.
///
/// Verified empirically against real output, not assumed from `--porcelain`
/// (no `--line-`) prose: `--line-porcelain` repeats the **full** metadata
/// block (`author `, `author-time `, `summary `, …) before every single
/// source line, even consecutive lines owned by the same commit — it does
/// not compress repeats down to a bare header the way plain `--porcelain`
/// does. This parser still caches each hash's author in a `HashMap` and
/// keys every content line off `current_hash` regardless, because that
/// approach is correct under *both* shapes: `.or_insert` on a value that is
/// always identical for a given hash is a harmless no-op on repeat, so
/// nothing breaks if a future `git` version reintroduces the compressed
/// form. See `git_blame.rs` for why `--line-porcelain` was chosen over the
/// default human format.
///
/// A header line is recognized structurally (its first token is exactly 40
/// lowercase hex characters), not by an exhaustive list of metadata line
/// prefixes: every other metadata line (`author-mail `, `committer `,
/// `summary `, `filename `, `previous `, …) is simply not one of the three
/// recognized shapes and is skipped, so a future `git` version that adds a
/// new metadata line degrades to being ignored, not to a wrong parse.
fn parse_git_blame(lines: Vec<&str>, notices: Vec<OutputNotice>) -> Option<ResultPayload> {
    let mut rows = Vec::with_capacity(lines.len());
    let mut authors: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
    let mut current_hash: Option<&str> = None;

    for line in &lines {
        if let Some(content) = line.strip_prefix('\t') {
            let hash = current_hash?;
            let author = *authors.get(hash)?;
            rows.push(row([&hash[..8], author, content]));
            continue;
        }
        if let Some(author) = line.strip_prefix("author ") {
            if let Some(hash) = current_hash {
                // `--line-porcelain` in fact repeats this line for every
                // source line, but `.or_insert` makes a second sighting of
                // the same hash a no-op rather than relying on that.
                authors.entry(hash).or_insert(author);
            }
            continue;
        }
        let Some(candidate) = line.split(' ').next() else { continue };
        if valid_git_hash(candidate) {
            current_hash = Some(candidate);
        }
        // Any other metadata line (`author-mail `, `committer `, `summary `,
        // `filename `, `previous `, …) carries nothing this table shows.
    }

    // Unlike `parse_git_status`/`parse_processes`, an empty result here is
    // not necessarily a sign that nothing matched the expected grammar: a
    // tracked file that is genuinely 0 bytes long produces zero blame lines
    // on a real, successful run, so an empty table is a legitimate answer,
    // not a fallback to `RawOutput`.
    Some(collection(
        CollectionKind::GitBlame,
        &["result.column.git.hash", "result.column.git.author", "result.column.content"],
        rows,
        notices,
    ))
}

fn valid_git_hash(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_short_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

fn valid_human_size(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().any(|byte| byte.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_digit()
                || matches!(
                    byte,
                    b'.' | b'B' | b'K' | b'M' | b'G' | b'T' | b'P' | b'E' | b'i' | b'k'
                )
        })
}

fn parse_df(lines: Vec<&str>, notices: Vec<OutputNotice>) -> Option<ResultPayload> {
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        if words
            == [
                "Filesystem",
                "Size",
                "Used",
                "Avail",
                "Capacity",
                "iused",
                "ifree",
                "%iused",
                "Mounted",
                "on",
            ]
        {
            continue;
        }
        rows.push(row(parse_df_record(line)?));
    }
    if rows.is_empty() {
        return None;
    }
    Some(collection(
        CollectionKind::FilesystemUsage,
        &[
            "result.column.filesystem",
            "result.column.size",
            "result.column.used",
            "result.column.available",
            "result.column.capacity",
            "result.column.files_used",
            "result.column.files_free",
            "result.column.files_capacity",
            "result.column.mount",
        ],
        rows,
        notices,
    ))
}

fn parse_df_record(line: &str) -> Option<Vec<String>> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    let mut parsed = None;
    // Three byte-size columns, capacity, two inode counts, and inode capacity
    // form a stable numeric island. Parse around it so both a filesystem name
    // (`map auto_home`) and a mount point may legitimately contain spaces.
    for start in 1..fields.len().saturating_sub(7) {
        let numeric = &fields[start..start + 7];
        if valid_human_size(numeric[0])
            && valid_human_size(numeric[1])
            && valid_human_size(numeric[2])
            && valid_percent(numeric[3])
            && valid_count(numeric[4])
            && valid_count(numeric[5])
            && valid_percent_or_dash(numeric[6])
        {
            let filesystem = fields[..start].join(" ");
            let mount = fields[start + 7..].join(" ");
            if filesystem.is_empty() || !mount.starts_with('/') || parsed.is_some() {
                return None;
            }
            parsed = Some(vec![
                filesystem,
                numeric[0].to_owned(),
                numeric[1].to_owned(),
                numeric[2].to_owned(),
                numeric[3].to_owned(),
                numeric[4].to_owned(),
                numeric[5].to_owned(),
                numeric[6].to_owned(),
                mount,
            ]);
        }
    }
    parsed
}

fn valid_count(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().any(|byte| byte.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_digit()
                || matches!(byte, b'.' | b'k' | b'K' | b'M' | b'G' | b'T' | b'P' | b'E')
        })
}

fn valid_percent(value: &str) -> bool {
    value
        .strip_suffix('%')
        .is_some_and(|number| !number.is_empty() && number.parse::<f64>().is_ok())
}

fn valid_percent_or_dash(value: &str) -> bool {
    value == "-" || valid_percent(value)
}

fn parse_git_status(lines: Vec<&str>, notices: Vec<OutputNotice>) -> Option<ResultPayload> {
    let mut rows = Vec::with_capacity(lines.len());
    for line in lines {
        let status = line.get(..2)?;
        let separator = line.as_bytes().get(2)?;
        let path = line.get(3..)?.trim();
        if *separator != b' ' || path.is_empty() || !valid_git_status(status) {
            return None;
        }
        rows.push(row([status, path]));
    }
    if rows.is_empty() {
        return None;
    }
    Some(collection(
        CollectionKind::GitStatus,
        &["result.column.git.status", "result.column.path"],
        rows,
        notices,
    ))
}

fn valid_git_status(status: &str) -> bool {
    status == "##"
        || status == "??"
        || status == "!!"
        || status
            .bytes()
            .all(|byte| matches!(byte, b' ' | b'M' | b'T' | b'A' | b'D' | b'R' | b'C' | b'U'))
}

fn parse_processes(lines: Vec<&str>, notices: Vec<OutputNotice>) -> Option<ResultPayload> {
    let mut rows = Vec::new();
    for line in lines {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words == ["PID", "PPID", "%CPU", "%MEM", "COMM"] {
            continue;
        }
        let fields = split_fields(line, 5)?;
        if fields[0].parse::<u32>().is_err()
            || fields[1].parse::<u32>().is_err()
            || fields[2].parse::<f64>().is_err()
            || fields[3].parse::<f64>().is_err()
        {
            return None;
        }
        rows.push(row(fields));
    }
    if rows.is_empty() {
        return None;
    }
    Some(collection(
        CollectionKind::Processes,
        &[
            "result.column.process.pid",
            "result.column.process.ppid",
            "result.column.process.cpu",
            "result.column.process.memory",
            "result.column.process.command",
        ],
        rows,
        notices,
    ))
}

fn report_payload(op_id: &str, output: &[RawOutputLine]) -> Option<ResultPayload> {
    let (lines, notices) = strict_stdout(output)?;
    let (kind, properties) = match op_id {
        "image.info" => (ReportKind::Image, parse_image_properties(lines)?),
        "net.headers" => (ReportKind::HttpHeaders, parse_http_headers(lines)?),
        "system.info" => (ReportKind::SystemVersion, parse_system_version(lines)?),
        "git.version" => (ReportKind::GitVersion, parse_git_version(lines)?),
        "files.identify" => {
            (ReportKind::FileType, parse_single_line(lines, "result.property.file_type")?)
        }
        "system.architecture" => (
            ReportKind::Architecture,
            parse_single_line(lines, "result.property.system.architecture")?,
        ),
        // `ls -le@d`, `xattr -l`, and `system_profiler` expose human layouts,
        // not versioned data formats. Their values remain useful RawOutput.
        "security.permissions" | "security.xattr" | "system.report" => return None,
        _ => return None,
    };
    Some(ResultPayload::PropertiesReport { kind, properties, notices })
}

/// Parses output that is exactly one non-empty line — `file -b`'s type
/// description, `uname -m`'s architecture word — into a single named
/// property. More than one non-empty line, or none at all, is not this
/// shape and falls back to `RawOutput`.
fn parse_single_line(lines: Vec<&str>, label_key: &'static str) -> Option<Vec<ResultProperty>> {
    let mut nonempty = lines.into_iter().filter(|line| !line.trim().is_empty());
    let line = nonempty.next()?.trim();
    if nonempty.next().is_some() || line.is_empty() {
        return None;
    }
    Some(vec![property(label_key, line)])
}

/// Parses `git --version` output: exactly one non-empty line, always
/// starting with the literal prefix `git version `. Apple's bundled Git
/// appends a vendor suffix (`git version 2.39.3 (Apple Git-146)`), which is
/// kept as part of the value rather than stripped — it is real information
/// about which build is installed, not noise.
fn parse_git_version(lines: Vec<&str>) -> Option<Vec<ResultProperty>> {
    let mut nonempty = lines.into_iter().filter(|line| !line.trim().is_empty());
    let line = nonempty.next()?;
    if nonempty.next().is_some() {
        return None;
    }
    let version = line.strip_prefix("git version ")?.trim();
    if version.is_empty() {
        return None;
    }
    Some(vec![property("result.property.git.version", version)])
}

fn property(label_key: impl Into<String>, value: impl Into<String>) -> ResultProperty {
    ResultProperty {
        label_key: label_key.into(),
        value: value.into(),
        stream: OutputStream::Stdout,
    }
}

fn parse_image_properties(lines: Vec<&str>) -> Option<Vec<ResultProperty>> {
    let mut nonempty = lines.into_iter().filter(|line| !line.trim().is_empty());
    let source = nonempty.next()?.trim();
    if !source.starts_with('/') {
        return None;
    }
    let mut properties = vec![property("result.property.source", source)];
    for line in nonempty {
        let (label, value) = line.trim().split_once(':')?;
        if label.is_empty() || value.trim().is_empty() || !label.bytes().all(valid_property_byte) {
            return None;
        }
        properties.push(property(image_label_key(label), value.trim()));
    }
    (properties.len() > 1).then_some(properties)
}

fn valid_property_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

fn image_label_key(label: &str) -> &str {
    match label {
        "pixelWidth" => "result.property.image.pixel_width",
        "pixelHeight" => "result.property.image.pixel_height",
        "typeIdentifier" => "result.property.image.type_identifier",
        "format" => "result.property.image.format",
        "formatOptions" => "result.property.image.format_options",
        "dpiWidth" => "result.property.image.dpi_width",
        "dpiHeight" => "result.property.image.dpi_height",
        "samplesPerPixel" => "result.property.image.samples_per_pixel",
        "bitsPerSample" => "result.property.image.bits_per_sample",
        "hasAlpha" => "result.property.image.has_alpha",
        "space" => "result.property.image.color_space",
        "profile" => "result.property.image.profile",
        // A future sips property is already a meaningful technical label. The
        // frontend's translation helper deliberately displays unknown keys.
        other => other,
    }
}

fn parse_http_headers(lines: Vec<&str>) -> Option<Vec<ResultProperty>> {
    let mut properties = Vec::new();
    for line in lines {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        if line.starts_with("HTTP/") {
            let mut fields = line.split_whitespace();
            let version = fields.next()?;
            let status = fields.next()?;
            if !version.strip_prefix("HTTP/").is_some_and(|value| {
                !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit() || b == b'.')
            }) || status.len() != 3
                || !status.bytes().all(|byte| byte.is_ascii_digit())
            {
                return None;
            }
            properties.push(property("result.property.http.status", line));
            continue;
        }
        let (name, value) = line.split_once(':')?;
        if name.is_empty()
            || !name.bytes().all(|byte| {
                byte.is_ascii_alphanumeric()
                    || matches!(
                        byte,
                        b'!' | b'#'
                            | b'$'
                            | b'%'
                            | b'&'
                            | b'\''
                            | b'*'
                            | b'+'
                            | b'-'
                            | b'.'
                            | b'^'
                            | b'_'
                            | b'`'
                            | b'|'
                            | b'~'
                    )
            })
        {
            return None;
        }
        properties.push(property(name, value.trim()));
    }
    (!properties.is_empty()).then_some(properties)
}

fn parse_system_version(lines: Vec<&str>) -> Option<Vec<ResultProperty>> {
    let mut properties = Vec::new();
    let mut seen = [false; 3];
    for line in lines.into_iter().filter(|line| !line.trim().is_empty()) {
        let (label, value) = line.split_once(':')?;
        let (index, key) = match label.trim() {
            "ProductName" => (0, "result.property.system.product_name"),
            "ProductVersion" => (1, "result.property.system.product_version"),
            "BuildVersion" => (2, "result.property.system.build_version"),
            _ => return None,
        };
        if seen[index] || value.trim().is_empty() {
            return None;
        }
        seen[index] = true;
        properties.push(property(key, value.trim()));
    }
    seen.into_iter().all(|value| value).then_some(properties)
}

fn metrics_payload(op_id: &str, output: &[RawOutputLine]) -> Option<ResultPayload> {
    match op_id {
        "net.ping" => parse_ping_metrics(output),
        // `uptime` has no machine-readable mode on macOS, and its human phrase
        // changes with duration and locale. Do not label the entire sentence a
        // metric merely because it contains numbers.
        "system.uptime" => None,
        _ => None,
    }
}

fn metric(label_key: &str, value: impl Into<String>, unit: Option<&str>) -> MetricValue {
    MetricValue {
        label_key: label_key.to_owned(),
        value: value.into(),
        unit: unit.map(str::to_owned),
        stream: OutputStream::Stdout,
    }
}

fn parse_ping_metrics(output: &[RawOutputLine]) -> Option<ResultPayload> {
    let (lines, notices) = strict_stdout(output)?;
    let mut metrics = Vec::new();
    let mut saw_packets = false;
    let mut saw_timing = false;
    for line in lines {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with("PING ")
            || line.starts_with("--- ") && line.ends_with(" ping statistics ---")
            || is_ping_reply(line)
        {
            continue;
        }
        if let Some((sent, received, loss)) = parse_ping_packets(line) {
            if saw_packets {
                return None;
            }
            saw_packets = true;
            metrics.extend([
                metric("result.metric.ping.transmitted", sent, Some("packets")),
                metric("result.metric.ping.received", received, Some("packets")),
                metric("result.metric.ping.packet_loss", loss, Some("%")),
            ]);
            continue;
        }
        if let Some(values) = parse_ping_timing(line) {
            if saw_timing {
                return None;
            }
            saw_timing = true;
            for (label, value) in [
                ("result.metric.ping.minimum", values[0]),
                ("result.metric.ping.average", values[1]),
                ("result.metric.ping.maximum", values[2]),
                ("result.metric.ping.stddev", values[3]),
            ] {
                metrics.push(metric(label, value, Some("ms")));
            }
            continue;
        }
        return None;
    }
    (saw_packets && saw_timing).then_some(ResultPayload::Metrics {
        kind: MetricsKind::NetworkLatency,
        metrics,
        notices,
    })
}

fn is_ping_reply(line: &str) -> bool {
    let Some((bytes, rest)) = line.split_once(" bytes from ") else {
        return false;
    };
    bytes.parse::<u64>().is_ok() && rest.contains("time=")
}

fn parse_ping_packets(line: &str) -> Option<(&str, &str, &str)> {
    let parts: Vec<&str> = line.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return None;
    }
    let sent = parts[0].strip_suffix(" packets transmitted")?;
    let received = parts[1].strip_suffix(" packets received")?;
    let loss = parts[2].strip_suffix("% packet loss")?.trim();
    if sent.parse::<u64>().is_err()
        || received.parse::<u64>().is_err()
        || loss.parse::<f64>().is_err()
    {
        return None;
    }
    Some((sent, received, loss))
}

fn parse_ping_timing(line: &str) -> Option<[&str; 4]> {
    let values = line.strip_prefix("round-trip min/avg/max/stddev = ")?.strip_suffix(" ms")?;
    let mut values = values.split('/');
    let parsed = [values.next()?, values.next()?, values.next()?, values.next()?];
    if values.next().is_some() || parsed.iter().any(|value| value.parse::<f64>().is_err()) {
        return None;
    }
    Some(parsed)
}

fn digest_payload(output: &[RawOutputLine]) -> Option<ResultPayload> {
    let (lines, notices) = strict_stdout(output)?;
    let lines: Vec<&str> = lines.into_iter().filter(|line| !line.is_empty()).collect();
    if lines.len() != 1 {
        return None;
    }
    let value = sha256_text(lines[0])?;
    Some(ResultPayload::Digest {
        algorithm: DigestAlgorithm::Sha256,
        value: Some(value),
        details: vec![StructuredLine { value: lines[0].to_owned(), stream: OutputStream::Stdout }],
        notices,
    })
}

fn hash_comparison_payload(output: &[RawOutputLine]) -> Option<ResultPayload> {
    let (lines, notices) = strict_stdout(output)?;
    let lines: Vec<&str> = lines.into_iter().filter(|line| !line.is_empty()).collect();
    if lines.len() != 2 {
        return None;
    }
    let reference = sha256_text(lines[0])?;
    let comparison = sha256_text(lines[1])?;
    let equal = reference.eq_ignore_ascii_case(&comparison);
    Some(ResultPayload::Comparison {
        kind: ComparisonKind::Sha256,
        reference: Some(reference),
        comparison: Some(comparison),
        equal: Some(equal),
        details: lines
            .into_iter()
            .map(|value| StructuredLine { value: value.to_owned(), stream: OutputStream::Stdout })
            .collect(),
        notices,
    })
}

/// Parse only the format Naffith itself selected: `shasum -a 256` begins with
/// exactly 64 hexadecimal ASCII bytes followed by whitespace. No filename or
/// locale-sensitive prose is interpreted.
fn sha256_text(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    let digest = bytes.get(..64)?;
    if !digest.iter().all(u8::is_ascii_hexdigit)
        || !bytes.get(64).is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }
    Some(String::from_utf8_lossy(digest).into_owned())
}

fn acknowledgement_message(op_id: &str) -> Option<&'static str> {
    match op_id {
        "files.open" => Some("result.ack.opened"),
        "git.init" => Some("result.ack.repository_initialized"),
        "git.commit" => Some("result.ack.commit_created"),
        "system.dns.flush" => Some("result.ack.dns_flushed"),
        // «أُرسلت الإشارة» لا «أُنهيت العملية»: `kill` تخرج بصفرٍ حين تُرسَل
        // الإشارة، وبرنامجٌ يلتقط `SIGTERM` ويتجاهلها يبقى حيًّا والرمز صفر.
        // انظر رأس `system_process_kill.rs`.
        "system.process.kill" => Some("result.ack.signal_sent"),
        "dev.npm.typecheck" => Some("result.ack.typecheck_passed"),
        "dev.npm.lint" => Some("result.ack.lint_passed"),
        "dev.npm.test" => Some("result.ack.tests_passed"),
        "dev.npm.install" => Some("result.ack.packages_installed"),
        "dev.npm.dev" | "dev.tauri.dev" => Some("result.ack.dev_server_stopped"),
        "dev.tauri.build" => Some("result.ack.tauri_build_completed"),
        "dev.cargo.test" => Some("result.ack.cargo_tests_passed"),
        "dev.cargo.check" => Some("result.ack.cargo_check_passed"),
        "dev.cargo.clippy" => Some("result.ack.cargo_clippy_passed"),
        "dev.cargo.fmt.check" => Some("result.ack.cargo_fmt_check_passed"),
        "dev.cargo.fmt" => Some("result.ack.cargo_fmt_applied"),
        "dev.cargo.build.release" => Some("result.ack.cargo_build_completed"),
        "dev.cargo.clean" => Some("result.ack.cargo_cleaned"),
        _ => None,
    }
}

fn comparison_kind(op_id: &str) -> Option<ComparisonKind> {
    match op_id {
        "disk.compare.hash" => Some(ComparisonKind::Sha256),
        "git.diff" | "git.diff.commits" => Some(ComparisonKind::GitDiff),
        "disk.compare.bytes" => Some(ComparisonKind::Bytes),
        _ => None,
    }
}

fn verdict_kind(op_id: &str) -> Option<VerdictKind> {
    match op_id {
        "compress.zip.test" => Some(VerdictKind::ArchiveIntegrity),
        "security.gatekeeper" => Some(VerdictKind::Gatekeeper),
        "security.codesign" => Some(VerdictKind::CodeSignature),
        "security.codesign.verify" => Some(VerdictKind::CodeIntegrity),
        _ => None,
    }
}

fn diff_search_kind(op_id: &str) -> Option<DiffSearchKind> {
    match op_id {
        "text.diff" => Some(DiffSearchKind::Diff),
        "text.search" | "git.grep" => Some(DiffSearchKind::Search),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitSemantics {
    Standard,
    Search,
    Diff,
    Gatekeeper,
    CodeSignature,
    CodeIntegrity,
    /// `lsof +D`: exit 1 means "failed to locate *some* search item", and
    /// `+D` turns every file in the tree into a search item — so 1 is the
    /// ordinary outcome even on a run that printed matching rows. Measured:
    /// a tree with one held file printed its row and still exited 1; only a
    /// tree whose every entry is open exits 0.
    ///
    /// So the exit code carries no verdict for this shape, and 0 and 1 are
    /// both "the command ran". Treating 1 as failure marked every real run
    /// red; treating it as `NoMatches` would have claimed "nothing found"
    /// over a list that found something. Anything above 1 is still failure.
    PartialLookup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OperationResultSpec {
    id: &'static str,
    category: ResultCategory,
    exit: ExitSemantics,
}

impl OperationResultSpec {
    const fn new(id: &'static str, category: ResultCategory) -> Self {
        Self { id, category, exit: ExitSemantics::Standard }
    }

    const fn with_exit(mut self, exit: ExitSemantics) -> Self {
        self.exit = exit;
        self
    }
}

use ResultCategory::{
    Acknowledgement, Artifact, Collection, Comparison, DiffSearch, Digest, Metrics,
    PropertiesReport, Verdict,
};

/// One explicit mapping for every production operation in `registry::ALL`.
///
/// Keeping this as data, instead of a prefix heuristic, makes omission visible
/// in the coverage test and prevents a newly added operation from silently
/// inheriting a polished-but-wrong ResultView.
const MAPPINGS: &[OperationResultSpec] = &[
    // Files and folders (10)
    OperationResultSpec::new("files.copy", Artifact),
    OperationResultSpec::new("files.move", Artifact),
    OperationResultSpec::new("files.mkdir", Artifact),
    OperationResultSpec::new("files.find.large", Collection),
    OperationResultSpec::new("files.find.stale", Collection),
    OperationResultSpec::new("files.find.name", Collection),
    OperationResultSpec::new("files.tree.size", Collection),
    OperationResultSpec::new("files.open", Acknowledgement),
    OperationResultSpec::new("files.list", Collection),
    OperationResultSpec::new("files.identify", PropertiesReport),
    // Compression (7)
    OperationResultSpec::new("compress.folder.zip", Artifact),
    OperationResultSpec::new("compress.zip.list", Collection),
    OperationResultSpec::new("compress.zip.extract", Artifact),
    OperationResultSpec::new("compress.zip.test", Verdict),
    OperationResultSpec::new("compress.tar.create", Artifact),
    OperationResultSpec::new("compress.tar.extract", Artifact),
    OperationResultSpec::new("compress.tar.list", Collection),
    // Images (4)
    OperationResultSpec::new("image.convert", Artifact),
    OperationResultSpec::new("image.resize", Artifact),
    OperationResultSpec::new("image.rotate", Artifact),
    OperationResultSpec::new("image.info", PropertiesReport),
    // Text (5)
    OperationResultSpec::new("text.merge", Artifact),
    OperationResultSpec::new("text.split", Artifact),
    OperationResultSpec::new("text.encoding.utf8", Artifact),
    OperationResultSpec::new("text.diff", DiffSearch).with_exit(ExitSemantics::Diff),
    OperationResultSpec::new("text.search", DiffSearch).with_exit(ExitSemantics::Search),
    // Disks (6)
    OperationResultSpec::new("disk.free", Collection),
    OperationResultSpec::new("disk.hash.sha256", Digest),
    OperationResultSpec::new("disk.compare.hash", Comparison),
    OperationResultSpec::new("disk.compare.bytes", Comparison).with_exit(ExitSemantics::Diff),
    OperationResultSpec::new("disk.list", Collection),
    // `+D` تخرج بـ1 حتى حين تجد وتطبع — مقيسٌ تجريبيًا. انظر
    // `ExitSemantics::PartialLookup`: بلا هذا كان كل تشغيلٍ واقعي يُعرض فشلًا
    // أحمر ويُسجَّل كذلك، بما فيه الناجح.
    OperationResultSpec::new("disk.directory.open_handles", Collection)
        .with_exit(ExitSemantics::PartialLookup),
    // Network (6)
    OperationResultSpec::new("net.ping", Metrics),
    OperationResultSpec::new("net.dns", Collection),
    OperationResultSpec::new("net.ports", Collection),
    // `lsof -i` تخرج بـ1 حين لا يكون على المنفذ شيء — «لا أحد يشغله» جوابٌ
    // مكتمل لا فشل. مقيسٌ تجريبيًا، خلافًا لـ`-p` و`+D` أدناه.
    OperationResultSpec::new("net.port.owner", Collection).with_exit(ExitSemantics::Search),
    OperationResultSpec::new("net.download", Artifact),
    OperationResultSpec::new("net.headers", PropertiesReport),
    // Security (5)
    OperationResultSpec::new("security.permissions", PropertiesReport),
    OperationResultSpec::new("security.xattr", PropertiesReport),
    OperationResultSpec::new("security.gatekeeper", Verdict).with_exit(ExitSemantics::Gatekeeper),
    OperationResultSpec::new("security.codesign", Verdict).with_exit(ExitSemantics::CodeSignature),
    OperationResultSpec::new("security.codesign.verify", Verdict)
        .with_exit(ExitSemantics::CodeIntegrity),
    // Git (12)
    OperationResultSpec::new("git.init", Acknowledgement),
    OperationResultSpec::new("git.status", Collection),
    OperationResultSpec::new("git.commit", Acknowledgement),
    OperationResultSpec::new("git.diff", Comparison).with_exit(ExitSemantics::Diff),
    OperationResultSpec::new("git.branches.merged", Collection),
    OperationResultSpec::new("git.archive", Artifact),
    OperationResultSpec::new("git.log", Collection),
    OperationResultSpec::new("git.diff.commits", Comparison).with_exit(ExitSemantics::Diff),
    OperationResultSpec::new("git.show.file", Collection),
    OperationResultSpec::new("git.blame", Collection),
    OperationResultSpec::new("git.grep", DiffSearch).with_exit(ExitSemantics::Search),
    OperationResultSpec::new("git.version", PropertiesReport),
    // System (10)
    OperationResultSpec::new("system.processes", Collection),
    // `pgrep` تخرج بـ1 حين لا تجد — «لا عملية بهذا الاسم» جوابٌ مكتمل.
    OperationResultSpec::new("system.process.find", Collection).with_exit(ExitSemantics::Search),
    // مقيسٌ تجريبيًا: `-p` تخرج بـ1 حين لا تسرد شيئًا — رقمٌ لا وجود له، أو
    // عمليةٌ قائمة لا يملكها المستخدم — وبصفرٍ حين تسرد. والحالتان الأوليان
    // لا تُفرَّقان برمز الخروج، فكلتاهما «لا نتيجة» لا فشلًا.
    OperationResultSpec::new("system.process.open_files", Collection)
        .with_exit(ExitSemantics::Search),
    OperationResultSpec::new("system.info", PropertiesReport),
    OperationResultSpec::new("system.architecture", PropertiesReport),
    OperationResultSpec::new("system.uptime", Metrics),
    OperationResultSpec::new("system.log.recent", Collection),
    OperationResultSpec::new("system.dns.flush", Acknowledgement),
    OperationResultSpec::new("system.report", PropertiesReport),
    // الإقرار يقول «أُرسلت الإشارة» لا «أُنهيت العملية»، ورمزٌ غير صفري فشلٌ
    // صادق (لا عملية بهذا الرقم، أو لا صلاحية عليها) لا جوابٌ آخر.
    OperationResultSpec::new("system.process.kill", Acknowledgement),
    // Developer tools — N1 (Node/npm/Tauri)
    OperationResultSpec::new("dev.npm.typecheck", Acknowledgement),
    OperationResultSpec::new("dev.npm.lint", Acknowledgement),
    OperationResultSpec::new("dev.npm.test", Acknowledgement),
    OperationResultSpec::new("dev.npm.install", Acknowledgement),
    OperationResultSpec::new("dev.npm.dev", Acknowledgement),
    OperationResultSpec::new("dev.tauri.dev", Acknowledgement),
    OperationResultSpec::new("dev.tauri.build", Acknowledgement),
    // Developer tools — N2 (Cargo/Rust)
    OperationResultSpec::new("dev.cargo.test", Acknowledgement),
    OperationResultSpec::new("dev.cargo.check", Acknowledgement),
    OperationResultSpec::new("dev.cargo.clippy", Acknowledgement),
    OperationResultSpec::new("dev.cargo.fmt.check", Acknowledgement),
    OperationResultSpec::new("dev.cargo.fmt", Acknowledgement),
    OperationResultSpec::new("dev.cargo.build.release", Acknowledgement),
    OperationResultSpec::new("dev.cargo.clean", Acknowledgement),
];

const FALLBACK: OperationResultSpec = OperationResultSpec {
    id: "",
    category: ResultCategory::RawOutput,
    exit: ExitSemantics::Standard,
};

fn spec_for(op_id: &str) -> OperationResultSpec {
    MAPPINGS.iter().find(|mapping| mapping.id == op_id).copied().unwrap_or(FALLBACK)
}

/// Interpretation of a process exit after signals have already been handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitMeaning {
    Answer(ResultSemantic),
    Failure,
}

/// Translate an exit code into the operation's domain without reading output.
pub fn classify_exit(op_id: &str, code: Option<i32>) -> ExitMeaning {
    use ExitMeaning::{Answer, Failure};
    use ResultSemantic::{
        Accepted, Completed, Differences, Matches, NoDifferences, NoMatches, Rejected, Signed,
        Unsigned,
    };

    match (spec_for(op_id).exit, code) {
        (ExitSemantics::Search, Some(0)) => Answer(Matches),
        (ExitSemantics::Search, Some(1)) => Answer(NoMatches),
        (ExitSemantics::Diff, Some(0)) => Answer(NoDifferences),
        (ExitSemantics::Diff, Some(1)) => Answer(Differences),
        (ExitSemantics::Gatekeeper, Some(0)) => Answer(Accepted),
        (ExitSemantics::Gatekeeper, Some(3)) => Answer(Rejected),
        (ExitSemantics::CodeSignature, Some(0)) => Answer(Signed),
        (ExitSemantics::CodeSignature, Some(1)) => Answer(Unsigned),
        (ExitSemantics::CodeIntegrity, Some(0)) => Answer(Accepted),
        (ExitSemantics::CodeIntegrity, Some(1)) => Answer(Rejected),
        (ExitSemantics::PartialLookup, Some(0 | 1)) => Answer(Completed),
        (_, Some(0)) => Answer(Completed),
        _ => Failure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::Policy;
    use std::collections::{BTreeMap, BTreeSet};

    fn ping_output() -> Vec<RawOutputLine> {
        [
            "PING example.test (192.0.2.1): 56 data bytes",
            "64 bytes from 192.0.2.1: icmp_seq=0 ttl=64 time=12.250 ms",
            "",
            "--- example.test ping statistics ---",
            "1 packets transmitted, 1 packets received, 0.0% packet loss",
            "round-trip min/avg/max/stddev = 12.250/12.250/12.250/0.000 ms",
        ]
        .into_iter()
        .map(|line| RawOutputLine::Stdout(line.to_owned()))
        .collect()
    }

    #[test]
    fn every_production_operation_has_exactly_one_approved_mapping() {
        let approved = [
            ("files.copy", Artifact),
            ("files.move", Artifact),
            ("files.mkdir", Artifact),
            ("files.find.large", Collection),
            ("files.find.stale", Collection),
            ("files.find.name", Collection),
            ("files.tree.size", Collection),
            ("files.open", Acknowledgement),
            ("files.list", Collection),
            ("files.identify", PropertiesReport),
            ("compress.folder.zip", Artifact),
            ("compress.zip.list", Collection),
            ("compress.zip.extract", Artifact),
            ("compress.zip.test", Verdict),
            ("compress.tar.create", Artifact),
            ("compress.tar.extract", Artifact),
            ("compress.tar.list", Collection),
            ("image.convert", Artifact),
            ("image.resize", Artifact),
            ("image.rotate", Artifact),
            ("image.info", PropertiesReport),
            ("text.merge", Artifact),
            ("text.split", Artifact),
            ("text.encoding.utf8", Artifact),
            ("text.diff", DiffSearch),
            ("text.search", DiffSearch),
            ("disk.free", Collection),
            ("disk.hash.sha256", Digest),
            ("disk.compare.hash", Comparison),
            ("disk.compare.bytes", Comparison),
            ("disk.list", Collection),
            ("disk.directory.open_handles", Collection),
            ("net.ping", Metrics),
            ("net.dns", Collection),
            ("net.ports", Collection),
            ("net.port.owner", Collection),
            ("net.download", Artifact),
            ("net.headers", PropertiesReport),
            ("security.permissions", PropertiesReport),
            ("security.xattr", PropertiesReport),
            ("security.gatekeeper", Verdict),
            ("security.codesign", Verdict),
            ("security.codesign.verify", Verdict),
            ("git.init", Acknowledgement),
            ("git.status", Collection),
            ("git.commit", Acknowledgement),
            ("git.diff", Comparison),
            ("git.branches.merged", Collection),
            ("git.archive", Artifact),
            ("git.log", Collection),
            ("git.diff.commits", Comparison),
            ("git.show.file", Collection),
            ("git.blame", Collection),
            ("git.grep", DiffSearch),
            ("git.version", PropertiesReport),
            ("system.processes", Collection),
            ("system.process.find", Collection),
            ("system.process.open_files", Collection),
            ("system.info", PropertiesReport),
            ("system.architecture", PropertiesReport),
            ("system.uptime", Metrics),
            ("system.log.recent", Collection),
            ("system.dns.flush", Acknowledgement),
            ("system.report", PropertiesReport),
            ("system.process.kill", Acknowledgement),
            ("dev.npm.typecheck", Acknowledgement),
            ("dev.npm.lint", Acknowledgement),
            ("dev.npm.test", Acknowledgement),
            ("dev.npm.install", Acknowledgement),
            ("dev.npm.dev", Acknowledgement),
            ("dev.tauri.dev", Acknowledgement),
            ("dev.tauri.build", Acknowledgement),
            ("dev.cargo.test", Acknowledgement),
            ("dev.cargo.check", Acknowledgement),
            ("dev.cargo.clippy", Acknowledgement),
            ("dev.cargo.fmt.check", Acknowledgement),
            ("dev.cargo.fmt", Acknowledgement),
            ("dev.cargo.build.release", Acknowledgement),
            ("dev.cargo.clean", Acknowledgement),
        ];
        let registered: BTreeSet<&str> =
            crate::registry::list(Policy::production()).iter().map(|op| op.id).collect();
        let mapped: BTreeSet<&str> = MAPPINGS.iter().map(|mapping| mapping.id).collect();
        let approved_by_id: BTreeMap<&str, ResultCategory> = approved.into_iter().collect();
        let mapped_by_id: BTreeMap<&str, ResultCategory> =
            MAPPINGS.iter().map(|mapping| (mapping.id, mapping.category)).collect();

        assert_eq!(MAPPINGS.len(), 79, "the result contract must enumerate all 79 operations");
        assert_eq!(mapped.len(), MAPPINGS.len(), "result mappings must not contain duplicates");
        assert_eq!(mapped, registered, "the result contract and production registry diverged");
        assert_eq!(mapped_by_id, approved_by_id, "an operation changed approved result family");
        assert!(MAPPINGS.iter().all(|mapping| {
            !matches!(mapping.category, ResultCategory::Diagnostic | ResultCategory::RawOutput)
        }));

        for (category, expected) in [
            (ResultCategory::Artifact, 15),
            (ResultCategory::Acknowledgement, 19),
            (ResultCategory::Collection, 22),
            (ResultCategory::PropertiesReport, 9),
            (ResultCategory::Metrics, 2),
            (ResultCategory::Digest, 1),
            (ResultCategory::Comparison, 4),
            (ResultCategory::Verdict, 4),
            (ResultCategory::DiffSearch, 3),
        ] {
            assert_eq!(
                MAPPINGS.iter().filter(|mapping| mapping.category == category).count(),
                expected,
                "wrong mapping count for {category:?}"
            );
        }
    }

    #[test]
    fn unknown_and_internal_operations_use_the_raw_fallback() {
        let result = ResultContract::for_operation(
            "internal.echo",
            ResultSemantic::Completed,
            None,
            Vec::new(),
            None,
        );
        assert_eq!(result.category, ResultCategory::RawOutput);
        assert_eq!(
            classify_exit("future.operation", Some(0)),
            ExitMeaning::Answer(ResultSemantic::Completed)
        );
        assert_eq!(classify_exit("future.operation", Some(7)), ExitMeaning::Failure);
    }

    #[test]
    fn expected_nonzero_codes_are_domain_answers_not_failures() {
        assert_eq!(
            classify_exit("text.search", Some(1)),
            ExitMeaning::Answer(ResultSemantic::NoMatches)
        );
        assert_eq!(
            classify_exit("text.diff", Some(1)),
            ExitMeaning::Answer(ResultSemantic::Differences)
        );
        assert_eq!(
            classify_exit("git.diff", Some(1)),
            ExitMeaning::Answer(ResultSemantic::Differences)
        );
        assert_eq!(
            classify_exit("git.diff.commits", Some(1)),
            ExitMeaning::Answer(ResultSemantic::Differences)
        );
        assert_eq!(
            classify_exit("git.grep", Some(1)),
            ExitMeaning::Answer(ResultSemantic::NoMatches)
        );
        assert_eq!(
            classify_exit("security.gatekeeper", Some(3)),
            ExitMeaning::Answer(ResultSemantic::Rejected)
        );
        assert_eq!(
            classify_exit("security.codesign", Some(1)),
            ExitMeaning::Answer(ResultSemantic::Unsigned)
        );
        assert_eq!(
            classify_exit("disk.compare.bytes", Some(1)),
            ExitMeaning::Answer(ResultSemantic::Differences)
        );
        // Verified empirically (`codesign --verify --deep --strict`) against
        // both an unsigned file and a tampered signed binary: both fail
        // cases share exit code 1, same as `security.codesign`'s `-d -vv`.
        assert_eq!(
            classify_exit("security.codesign.verify", Some(1)),
            ExitMeaning::Answer(ResultSemantic::Rejected)
        );
        // كلاهما مقيسٌ تجريبيًا: `pgrep` و`lsof -i` تخرجان بـ1 حين لا تجدان،
        // فـ«لا نتيجة» جوابٌ مكتمل لا فشلٌ أحمر.
        assert_eq!(
            classify_exit("system.process.find", Some(1)),
            ExitMeaning::Answer(ResultSemantic::NoMatches)
        );
        assert_eq!(
            classify_exit("net.port.owner", Some(1)),
            ExitMeaning::Answer(ResultSemantic::NoMatches)
        );
        assert_eq!(
            classify_exit("system.process.open_files", Some(1)),
            ExitMeaning::Answer(ResultSemantic::NoMatches)
        );
    }

    /// `system.process.kill` وحدها هنا: رمزٌ غير صفريّ يعني «لا عملية بهذا
    /// الرقم» أو «لا صلاحية عليها» — وكلاهما فشلٌ في تنفيذ ما طُلب، لا جوابٌ
    /// آخر. تُنهي أو لا تُنهي، ولا حالة ثالثة.
    ///
    /// ولا تُجمَع معها عمليتا `lsof`: نسخةٌ أولى من هذا الاختبار جمعت الثلاث
    /// تحت `Standard` **فثبّتت خطأً بوصفه صحيحًا** — القياس اللاحق أثبت أن
    /// `-p` و`+D` تخرجان بـ1 في مجرى تشغيلهما الطبيعي، فكان كل تشغيلٍ منهما
    /// يُعرض أحمر. لكلٍّ منهما الآن دلالتها، والاختباران أدناه يحرسانها.
    #[test]
    fn ending_a_process_either_worked_or_failed_with_no_third_answer() {
        assert_eq!(
            classify_exit("system.process.kill", Some(0)),
            ExitMeaning::Answer(ResultSemantic::Completed)
        );
        assert_eq!(classify_exit("system.process.kill", Some(1)), ExitMeaning::Failure);
    }

    /// `lsof -p` تخرج بـ1 حين لا تسرد شيئًا — رقمٌ لا وجود له أو عمليةٌ لا
    /// يملكها المستخدم — وهو جوابٌ مكتمل لا فشل. وبلا هذا كان أقصر مسارٍ في
    /// الشاشة (القيمة المبدئية `1`، وهي `launchd` المملوكة لـroot) يُعرض
    /// فشلًا أحمر.
    #[test]
    fn listing_the_files_of_an_unlistable_process_is_no_matches_not_failure() {
        assert_eq!(
            classify_exit("system.process.open_files", Some(0)),
            ExitMeaning::Answer(ResultSemantic::Matches)
        );
        assert_eq!(
            classify_exit("system.process.open_files", Some(1)),
            ExitMeaning::Answer(ResultSemantic::NoMatches)
        );
        assert_eq!(classify_exit("system.process.open_files", Some(2)), ExitMeaning::Failure);
    }

    /// `lsof +D` تخرج بـ1 **حتى حين تجد وتطبع**: `+D` تجعل كل ملفٍّ في الشجرة
    /// عنصر بحث، وأيّ ملفٍّ غير مفتوح يُعدّ عنصرًا لم يُعثر عليه. مقيسٌ
    /// تجريبيًا: شجرةٌ فيها ملفٌّ ممسوك طبعت سطره وخرجت بـ1.
    ///
    /// فلا `Standard` (تجعل كل تشغيلٍ فشلًا) ولا `Search` (تدّعي «لم أجد» فوق
    /// قائمةٍ وجدت). الصفر والواحد كلاهما «جرى الأمر»، وما فوقهما فشل.
    #[test]
    fn holding_handles_reports_completion_on_both_zero_and_one() {
        for code in [0, 1] {
            assert_eq!(
                classify_exit("disk.directory.open_handles", Some(code)),
                ExitMeaning::Answer(ResultSemantic::Completed),
                "exit {code} must not be read as a failure"
            );
        }
        assert_eq!(classify_exit("disk.directory.open_handles", Some(2)), ExitMeaning::Failure);
        assert_eq!(classify_exit("disk.directory.open_handles", None), ExitMeaning::Failure);
    }

    /// نصّ الإقرار يقول «أُرسلت الإشارة» لا «أُنهيت العملية». الفرق ليس
    /// تجميلًا: `kill` تخرج بصفرٍ حين تُرسَل الإشارة، وبرنامجٌ يلتقط
    /// `SIGTERM` ويتجاهلها يبقى حيًّا والرمز صفر.
    #[test]
    fn killing_acknowledges_sending_a_signal_not_ending_a_process() {
        let result = ResultContract::for_operation(
            "system.process.kill",
            ResultSemantic::Completed,
            None,
            Vec::new(),
            None,
        );
        let ResultPayload::Acknowledgement { message_key, .. } = result.payload else {
            panic!("system.process.kill did not produce an acknowledgement")
        };
        assert_eq!(message_key, "result.ack.signal_sent");
    }

    #[test]
    fn unexpected_codes_still_fail_closed() {
        for op_id in [
            "text.search",
            "text.diff",
            "git.diff",
            "git.diff.commits",
            "git.grep",
            "security.gatekeeper",
            "security.codesign",
            "disk.compare.bytes",
            "security.codesign.verify",
            "system.process.find",
            "net.port.owner",
        ] {
            assert_eq!(classify_exit(op_id, Some(2)), ExitMeaning::Failure, "{op_id}");
            assert_eq!(classify_exit(op_id, None), ExitMeaning::Failure, "{op_id}");
        }
    }

    /// حقائق الناتج تُقاس من نظام الملفات، لا من خرج الأداة.
    ///
    /// الاختبار يكتب شجرةً حقيقية لأن المقيس هو القياس نفسه: مُحاكاةٌ للقرص
    /// كانت ستثبت أن الدالّة تُعيد ما لُقِّنته، لا أنها تقرأ ما هناك.
    #[test]
    fn artifact_facts_are_measured_from_the_filesystem_not_from_output() {
        let root = std::env::temp_dir().join(format!("naffith-facts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("nested")).unwrap();
        std::fs::write(root.join("a.txt"), b"12345").unwrap();
        std::fs::write(root.join("b.txt"), b"678").unwrap();
        std::fs::write(root.join("nested/c.txt"), b"90").unwrap();

        // مجلد: العدّ للمباشر وحده (‏a، b، nested)، والحجم لكل ما تحته.
        let dir = ArtifactFacts::measure(&root);
        assert_eq!(dir.entries, Some(3));
        assert_eq!(dir.size_bytes, Some(10));
        assert_eq!(dir.parent, root.parent().unwrap().to_string_lossy());

        // ملف: حجمٌ بلا عدد — لا «عناصر» داخل ملف.
        let file = ArtifactFacts::measure(&root.join("a.txt"));
        assert_eq!(file.name, "a.txt");
        assert_eq!(file.size_bytes, Some(5));
        assert_eq!(file.entries, None);

        // غائب: يبقى الاسم والأب مشتقَّين، ويصمت القياس بدل أن يخترع صفرًا.
        let missing = ArtifactFacts::measure(&root.join("nope.bin"));
        assert_eq!(missing.name, "nope.bin");
        assert_eq!(missing.size_bytes, None);
        assert_eq!(missing.entries, None);

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_wire_contract_is_stable_and_omits_absent_reveal() {
        let result = ResultContract::for_operation(
            "text.search",
            ResultSemantic::NoMatches,
            None,
            Vec::new(),
            None,
        );
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            serde_json::json!({
                "category": "diff_search",
                "semantic": "no_matches",
                "type": "diff_search",
                "kind": "search",
                "items": [],
                "notices": []
            })
        );

        let result = ResultContract::for_operation(
            "compress.folder.zip",
            ResultSemantic::Completed,
            Some("/Users/x/archive.zip"),
            vec![RawOutputLine::Stdout("done".into())],
            Some(RevealKind::File),
        );
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            serde_json::json!({
                "category": "artifact",
                "semantic": "completed",
                "type": "artifact",
                // مشتقّان من المسار فيَحضران دائمًا؛ و`size_bytes`/`entries`
                // قياسان من نظام الملفات فيغيبان هنا لأن المسار غير موجود.
                // وهذا هو العقد: ما لا يُثبَت لا يُرسَل.
                "name": "archive.zip",
                "parent": "/Users/x",
                "path": "/Users/x/archive.zip",
                "output": [{ "stream": "stdout", "line": "done" }],
                "reveal": "file"
            })
        );
    }

    #[test]
    fn category_and_domain_semantic_wire_names_are_exact() {
        let categories = [
            (ResultCategory::Artifact, "artifact"),
            (ResultCategory::Acknowledgement, "acknowledgement"),
            (ResultCategory::Collection, "collection"),
            (ResultCategory::PropertiesReport, "properties_report"),
            (ResultCategory::Metrics, "metrics"),
            (ResultCategory::Digest, "digest"),
            (ResultCategory::Comparison, "comparison"),
            (ResultCategory::Verdict, "verdict"),
            (ResultCategory::DiffSearch, "diff_search"),
            (ResultCategory::Diagnostic, "diagnostic"),
            (ResultCategory::RawOutput, "raw_output"),
        ];
        for (category, wire) in categories {
            assert_eq!(serde_json::to_value(category).unwrap(), serde_json::json!(wire));
        }

        let semantics = [
            (ResultSemantic::Matches, "matches"),
            (ResultSemantic::NoMatches, "no_matches"),
            (ResultSemantic::Differences, "differences"),
            (ResultSemantic::NoDifferences, "no_differences"),
            (ResultSemantic::Accepted, "accepted"),
            (ResultSemantic::Rejected, "rejected"),
            (ResultSemantic::Signed, "signed"),
            (ResultSemantic::Unsigned, "unsigned"),
        ];
        for (semantic, wire) in semantics {
            assert_eq!(serde_json::to_value(semantic).unwrap(), serde_json::json!(wire));
        }
    }

    #[test]
    fn deserialization_rejects_internally_contradictory_result_contracts() {
        for invalid in [
            serde_json::json!({
                "category": "artifact",
                "semantic": "completed",
                "type": "raw_output",
                "lines": []
            }),
            serde_json::json!({
                "category": "diagnostic",
                "semantic": "completed",
                "type": "diagnostic",
                "lines": []
            }),
            serde_json::json!({
                "category": "verdict",
                "semantic": "accepted",
                "type": "verdict",
                "kind": "gatekeeper",
                "value": "rejected",
                "details": [],
                "notices": []
            }),
            serde_json::json!({
                "category": "diagnostic",
                "semantic": "failed",
                "type": "diagnostic",
                "lines": [],
                "reveal": "file"
            }),
        ] {
            assert!(
                serde_json::from_value::<ResultContract>(invalid.clone()).is_err(),
                "accepted contradictory result contract: {invalid}"
            );
        }
    }

    #[test]
    fn execution_failures_use_the_diagnostic_category() {
        let result = ResultContract::for_operation(
            "compress.folder.zip",
            ResultSemantic::Failed,
            None,
            vec![RawOutputLine::Stderr("failed".into())],
            Some(RevealKind::File),
        );
        assert_eq!(result.category, ResultCategory::Diagnostic);
        assert!(matches!(result.payload, ResultPayload::Diagnostic { .. }));
        assert_eq!(result.reveal, None, "diagnostics must fail closed even for a bad caller");
    }

    #[test]
    fn every_result_family_has_a_distinct_stable_json_shape() {
        let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let cases = vec![
            (
                ResultContract::for_operation(
                    "files.open",
                    ResultSemantic::Completed,
                    None,
                    Vec::new(),
                    None,
                ),
                "acknowledgement",
            ),
            (
                ResultContract::for_operation(
                    "files.find.name",
                    ResultSemantic::Completed,
                    None,
                    vec![RawOutputLine::Stdout("/tmp/value".into())],
                    None,
                ),
                "collection",
            ),
            (
                ResultContract::for_operation(
                    "image.info",
                    ResultSemantic::Completed,
                    None,
                    vec![
                        RawOutputLine::Stdout("/tmp/a.png".into()),
                        RawOutputLine::Stdout("  pixelWidth: 128".into()),
                    ],
                    None,
                ),
                "properties_report",
            ),
            (
                ResultContract::for_operation(
                    "net.ping",
                    ResultSemantic::Completed,
                    None,
                    ping_output(),
                    None,
                ),
                "metrics",
            ),
            (
                ResultContract::for_operation(
                    "disk.hash.sha256",
                    ResultSemantic::Completed,
                    None,
                    vec![RawOutputLine::Stdout(format!("{hash}  /tmp/a"))],
                    None,
                ),
                "digest",
            ),
            (
                ResultContract::for_operation(
                    "git.diff",
                    ResultSemantic::NoDifferences,
                    None,
                    Vec::new(),
                    None,
                ),
                "comparison",
            ),
            (
                ResultContract::for_operation(
                    "security.gatekeeper",
                    ResultSemantic::Accepted,
                    None,
                    Vec::new(),
                    None,
                ),
                "verdict",
            ),
            (
                ResultContract::for_operation(
                    "text.search",
                    ResultSemantic::Matches,
                    None,
                    vec![RawOutputLine::Stdout("/tmp/a:1:value".into())],
                    None,
                ),
                "diff_search",
            ),
            (
                ResultContract::for_operation(
                    "internal.echo",
                    ResultSemantic::Completed,
                    None,
                    vec![RawOutputLine::Stdout("value".into())],
                    None,
                ),
                "raw_output",
            ),
        ];

        for (contract, expected_type) in cases {
            let encoded = serde_json::to_value(&contract).unwrap();
            assert_eq!(encoded["type"], expected_type);
            assert_eq!(contract.category, contract.payload.category());
            let decoded: ResultContract = serde_json::from_value(encoded).unwrap();
            assert_eq!(decoded, contract);
        }

        let diagnostic = ResultContract::for_operation(
            "net.ping",
            ResultSemantic::Failed,
            None,
            vec![RawOutputLine::Stderr("failed".into())],
            None,
        );
        assert_eq!(serde_json::to_value(&diagnostic).unwrap()["type"], "diagnostic");
        assert_eq!(diagnostic.category, diagnostic.payload.category());

        let artifact = ResultContract::for_operation(
            "files.copy",
            ResultSemantic::Completed,
            Some("/tmp/copied"),
            Vec::new(),
            Some(RevealKind::File),
        );
        assert_eq!(serde_json::to_value(&artifact).unwrap()["type"], "artifact");
        assert_eq!(artifact.category, artifact.payload.category());
    }

    #[test]
    fn structured_rows_keep_streams_and_separate_output_bounds() {
        let result = ResultContract::for_operation(
            "files.find.large",
            ResultSemantic::Completed,
            None,
            vec![
                RawOutputLine::Stdout("/tmp/a".into()),
                RawOutputLine::Omitted { dropped: 9 },
                RawOutputLine::Truncated { dropped: 11 },
            ],
            None,
        );
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            serde_json::json!({
                "category": "collection",
                "semantic": "completed",
                "type": "collection",
                "kind": "file_matches",
                "columns": ["result.column.path"],
                "rows": [
                    { "cells": ["/tmp/a"], "stream": "stdout" }
                ],
                "notices": [
                    { "kind": "omitted", "dropped": 9 },
                    { "kind": "truncated", "dropped": 11 }
                ]
            })
        );
    }

    #[test]
    fn deterministic_sha256_output_is_parsed_only_in_rust() {
        let first = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let second = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let digest = ResultContract::for_operation(
            "disk.hash.sha256",
            ResultSemantic::Completed,
            None,
            vec![RawOutputLine::Stdout(format!("{first}  /tmp/a"))],
            None,
        );
        let encoded = serde_json::to_value(digest).unwrap();
        assert_eq!(encoded["algorithm"], "sha256");
        assert_eq!(encoded["value"], first);

        let comparison = ResultContract::for_operation(
            "disk.compare.hash",
            ResultSemantic::Completed,
            None,
            vec![
                RawOutputLine::Stdout(format!("{first}  /tmp/a")),
                RawOutputLine::Stdout(format!("{second}  /tmp/b")),
            ],
            None,
        );
        let encoded = serde_json::to_value(comparison).unwrap();
        assert_eq!(encoded["reference"], first);
        assert_eq!(encoded["comparison"], second);
        assert_eq!(encoded["equal"], false);

        let malformed = ResultContract::for_operation(
            "disk.hash.sha256",
            ResultSemantic::Completed,
            None,
            vec![RawOutputLine::Stdout("not a digest".into())],
            None,
        );
        let malformed = serde_json::to_value(malformed).unwrap();
        assert_eq!(malformed["category"], "raw_output");
        assert_eq!(malformed["type"], "raw_output");
    }

    #[test]
    fn stable_collection_formats_produce_meaningful_columns_and_cells() {
        let cases = [
            (
                "files.tree.size",
                vec![RawOutputLine::Stdout("4.0K\t/tmp/folder".into())],
                vec!["result.column.size", "result.column.path"],
                vec!["4.0K", "/tmp/folder"],
            ),
            (
                "disk.free",
                vec![
                    RawOutputLine::Stdout(
                        "Filesystem Size Used Avail Capacity iused ifree %iused Mounted on".into(),
                    ),
                    RawOutputLine::Stdout(
                        "map auto_home 0Bi 0Bi 0Bi 100% 0 0 - /System/Volumes/Data/home".into(),
                    ),
                ],
                vec![
                    "result.column.filesystem",
                    "result.column.size",
                    "result.column.used",
                    "result.column.available",
                    "result.column.capacity",
                    "result.column.files_used",
                    "result.column.files_free",
                    "result.column.files_capacity",
                    "result.column.mount",
                ],
                vec![
                    "map auto_home",
                    "0Bi",
                    "0Bi",
                    "0Bi",
                    "100%",
                    "0",
                    "0",
                    "-",
                    "/System/Volumes/Data/home",
                ],
            ),
            (
                "net.dns",
                vec![RawOutputLine::Stdout("example.com. 300 IN MX 10 mail.example.com.".into())],
                vec![
                    "result.column.dns.name",
                    "result.column.dns.ttl",
                    "result.column.dns.class",
                    "result.column.dns.type",
                    "result.column.dns.value",
                ],
                vec!["example.com.", "300", "IN", "MX", "10 mail.example.com."],
            ),
            (
                "git.status",
                vec![RawOutputLine::Stdout(" M src/result.rs".into())],
                vec!["result.column.git.status", "result.column.path"],
                vec![" M", "src/result.rs"],
            ),
            (
                "git.branches.merged",
                vec![RawOutputLine::Stdout("* main".into())],
                vec!["result.column.git.current", "result.column.git.branch"],
                vec!["*", "main"],
            ),
            (
                "system.processes",
                vec![
                    RawOutputLine::Stdout("PID PPID %CPU %MEM COMM".into()),
                    RawOutputLine::Stdout("42 1 12.5 0.4 /usr/bin/example process".into()),
                ],
                vec![
                    "result.column.process.pid",
                    "result.column.process.ppid",
                    "result.column.process.cpu",
                    "result.column.process.memory",
                    "result.column.process.command",
                ],
                vec!["42", "1", "12.5", "0.4", "/usr/bin/example process"],
            ),
            (
                "git.log",
                vec![RawOutputLine::Stdout(
                    "0123456789abcdef0123456789abcdef01234567\t2024-01-15\tAmira Yousef\tFix the thing"
                        .into(),
                )],
                vec![
                    "result.column.git.hash",
                    "result.column.git.date",
                    "result.column.git.author",
                    "result.column.git.subject",
                ],
                vec![
                    "0123456789abcdef0123456789abcdef01234567",
                    "2024-01-15",
                    "Amira Yousef",
                    "Fix the thing",
                ],
            ),
            (
                "system.process.find",
                vec![RawOutputLine::Stdout("405 loginwindow".into())],
                vec!["result.column.process.pid", "result.column.process.name"],
                vec!["405", "loginwindow"],
            ),
            (
                "git.blame",
                vec![
                    RawOutputLine::Stdout(
                        "0123456789abcdef0123456789abcdef01234567 1 1 1".into(),
                    ),
                    RawOutputLine::Stdout("author Amira Yousef".into()),
                    RawOutputLine::Stdout("author-mail <amira@example.test>".into()),
                    RawOutputLine::Stdout("author-time 1700000000".into()),
                    RawOutputLine::Stdout("summary Fix the thing".into()),
                    RawOutputLine::Stdout("filename src/main.rs".into()),
                    RawOutputLine::Stdout("\tfn main() {}".into()),
                ],
                vec!["result.column.git.hash", "result.column.git.author", "result.column.content"],
                vec!["01234567", "Amira Yousef", "fn main() {}"],
            ),
        ];

        for (op_id, output, expected_columns, expected_cells) in cases {
            let result =
                ResultContract::for_operation(op_id, ResultSemantic::Completed, None, output, None);
            let ResultPayload::Collection { columns, rows, .. } = result.payload else {
                panic!("{op_id} did not produce a structured collection")
            };
            assert_eq!(columns, expected_columns, "{op_id}");
            assert_eq!(rows.len(), 1, "{op_id}");
            assert_eq!(rows[0].cells, expected_cells, "{op_id}");
        }

        for op_id in ["files.find.large", "files.find.stale", "files.find.name"] {
            let result = ResultContract::for_operation(
                op_id,
                ResultSemantic::Completed,
                None,
                vec![RawOutputLine::Stdout("/tmp/a file".into())],
                None,
            );
            let ResultPayload::Collection { columns, rows, .. } = result.payload else {
                panic!("{op_id} did not produce a path collection")
            };
            assert_eq!(columns, ["result.column.path"]);
            assert_eq!(rows[0].cells, ["/tmp/a file"]);
        }
    }

    /// `ls -1Ap` output: bare names, one per line, directories marked with a
    /// trailing `/`. Unlike `files.find.*` above these are not full paths, so
    /// this gets its own column key rather than reusing `result.column.path`.
    #[test]
    fn files_list_keeps_every_entry_including_directory_markers() {
        let result = ResultContract::for_operation(
            "files.list",
            ResultSemantic::Completed,
            None,
            vec![
                RawOutputLine::Stdout("subdir/".into()),
                RawOutputLine::Stdout("readme.txt".into()),
            ],
            None,
        );
        let ResultPayload::Collection { columns, rows, .. } = result.payload else {
            panic!("files.list did not produce a structured collection")
        };
        assert_eq!(columns, ["result.column.name"]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].cells, ["subdir/"]);
        assert_eq!(rows[1].cells, ["readme.txt"]);
    }

    /// An empty folder is a legitimate answer, not a parse failure — mirrors
    /// `git_show_file_keeps_every_line_including_blank_ones_as_its_own_row`.
    #[test]
    fn files_list_of_an_empty_folder_is_an_empty_table_not_a_raw_fallback() {
        let result = ResultContract::for_operation(
            "files.list",
            ResultSemantic::Completed,
            None,
            Vec::new(),
            None,
        );
        let ResultPayload::Collection { rows, .. } = result.payload else {
            panic!("files.list did not produce a structured (if empty) collection")
        };
        assert_eq!(rows, Vec::new());
    }

    /// `git blame --line-porcelain` in fact repeats the full metadata block
    /// before *every* source line, even consecutive lines from the same
    /// commit — verified against a real run, not assumed. This asserts the
    /// parser produces the right author for both lines when metadata is
    /// (correctly) repeated in full, guarding the exact misunderstanding a
    /// docstring here once stated as fact.
    #[test]
    fn git_blame_reads_the_author_correctly_when_metadata_repeats_on_every_line() {
        let hash = "0123456789abcdef0123456789abcdef01234567";
        let output = vec![
            RawOutputLine::Stdout(format!("{hash} 1 1 2")),
            RawOutputLine::Stdout("author Amira Yousef".into()),
            RawOutputLine::Stdout("summary Fix the thing".into()),
            RawOutputLine::Stdout("filename src/main.rs".into()),
            RawOutputLine::Stdout("\tfn main() {".into()),
            RawOutputLine::Stdout(format!("{hash} 2 2")),
            RawOutputLine::Stdout("author Amira Yousef".into()),
            RawOutputLine::Stdout("summary Fix the thing".into()),
            RawOutputLine::Stdout("filename src/main.rs".into()),
            RawOutputLine::Stdout("\t}".into()),
        ];
        let result = ResultContract::for_operation(
            "git.blame",
            ResultSemantic::Completed,
            None,
            output,
            None,
        );
        let ResultPayload::Collection { rows, .. } = result.payload else {
            panic!("git.blame did not produce a structured collection")
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].cells, ["01234567", "Amira Yousef", "fn main() {"]);
        assert_eq!(rows[1].cells, ["01234567", "Amira Yousef", "}"]);
    }

    /// A commit hash cache keyed by `HashMap::entry(..).or_insert(..)` is a
    /// no-op on a repeat, so the same parser is also correct if a future
    /// `git` version ever compresses metadata to a bare header line for a
    /// repeated commit — the shape `--porcelain` (without `--line-`) uses
    /// today, though `--line-porcelain` itself does not.
    #[test]
    fn git_blame_would_still_read_the_author_correctly_if_metadata_were_ever_compressed() {
        let hash = "0123456789abcdef0123456789abcdef01234567";
        let output = vec![
            RawOutputLine::Stdout(format!("{hash} 1 1 2")),
            RawOutputLine::Stdout("author Amira Yousef".into()),
            RawOutputLine::Stdout("summary Fix the thing".into()),
            RawOutputLine::Stdout("filename src/main.rs".into()),
            RawOutputLine::Stdout("\tfn main() {".into()),
            RawOutputLine::Stdout(format!("{hash} 2 2")),
            RawOutputLine::Stdout("\t}".into()),
        ];
        let result = ResultContract::for_operation(
            "git.blame",
            ResultSemantic::Completed,
            None,
            output,
            None,
        );
        let ResultPayload::Collection { rows, .. } = result.payload else {
            panic!("git.blame did not produce a structured collection")
        };
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].cells, ["01234567", "Amira Yousef", "}"]);
    }

    /// Unlike `parse_git_status`/`parse_processes`/`parse_git_log`, a line
    /// that matches none of the three recognized shapes is silently skipped
    /// rather than rejecting the whole result to `RawOutput` — because a
    /// genuinely empty table is a legitimate answer here (a 0-byte tracked
    /// file produces zero blame lines on a real, successful run), so there
    /// is no reliable way to tell "nothing matched" apart from "there was
    /// nothing to match" from line content alone. This test makes that
    /// choice explicit and load-bearing, not an untested accident.
    #[test]
    fn git_blame_of_output_matching_no_recognized_shape_is_an_empty_table_not_a_raw_fallback() {
        let result = ResultContract::for_operation(
            "git.blame",
            ResultSemantic::Completed,
            None,
            vec![RawOutputLine::Stdout("not blame porcelain at all".into())],
            None,
        );
        let ResultPayload::Collection { rows, .. } = result.payload else {
            panic!("git.blame did not produce a structured (if empty) collection")
        };
        assert_eq!(rows, Vec::new(), "an unrecognized line is skipped, not rejected");
    }

    /// Unlike every other collection parsed above, any text is valid file
    /// content — including a blank line — so this never falls back to
    /// `RawOutput` the way a malformed `git.log`/`git.blame` line would.
    #[test]
    fn git_show_file_keeps_every_line_including_blank_ones_as_its_own_row() {
        let output = vec![
            RawOutputLine::Stdout("fn main() {".into()),
            RawOutputLine::Stdout(String::new()),
            RawOutputLine::Stdout("}".into()),
        ];
        let result = ResultContract::for_operation(
            "git.show.file",
            ResultSemantic::Completed,
            None,
            output,
            None,
        );
        let ResultPayload::Collection { columns, rows, .. } = result.payload else {
            panic!("git.show.file did not produce a structured collection")
        };
        assert_eq!(columns, ["result.column.content"]);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].cells, ["fn main() {"]);
        assert_eq!(rows[1].cells, [""]);
        assert_eq!(rows[2].cells, ["}"]);
    }

    #[test]
    fn stable_reports_use_real_property_names_not_one_generic_row_label() {
        let image = ResultContract::for_operation(
            "image.info",
            ResultSemantic::Completed,
            None,
            vec![
                RawOutputLine::Stdout("/tmp/a.png".into()),
                RawOutputLine::Stdout("  pixelWidth: 128".into()),
                RawOutputLine::Stdout("  format: png".into()),
            ],
            None,
        );
        let ResultPayload::PropertiesReport { properties, .. } = image.payload else {
            panic!("image metadata was not structured")
        };
        assert_eq!(
            properties.iter().map(|item| item.label_key.as_str()).collect::<Vec<_>>(),
            [
                "result.property.source",
                "result.property.image.pixel_width",
                "result.property.image.format",
            ]
        );
        assert_eq!(properties[1].value, "128");

        let headers = ResultContract::for_operation(
            "net.headers",
            ResultSemantic::Completed,
            None,
            vec![
                RawOutputLine::Stdout("HTTP/2 200".into()),
                RawOutputLine::Stdout("Content-Type: text/plain; charset=utf-8".into()),
                RawOutputLine::Stdout(String::new()),
            ],
            None,
        );
        let ResultPayload::PropertiesReport { properties, .. } = headers.payload else {
            panic!("HTTP headers were not structured")
        };
        assert_eq!(properties[0].label_key, "result.property.http.status");
        assert_eq!(properties[1].label_key, "Content-Type");
        assert_eq!(properties[1].value, "text/plain; charset=utf-8");

        let version = ResultContract::for_operation(
            "system.info",
            ResultSemantic::Completed,
            None,
            vec![
                RawOutputLine::Stdout("ProductName:\t\tmacOS".into()),
                RawOutputLine::Stdout("ProductVersion:\t\t26.6.1".into()),
                RawOutputLine::Stdout("BuildVersion:\t\t25G76".into()),
            ],
            None,
        );
        let ResultPayload::PropertiesReport { properties, .. } = version.payload else {
            panic!("system version was not structured")
        };
        assert_eq!(
            properties.iter().map(|item| item.label_key.as_str()).collect::<Vec<_>>(),
            [
                "result.property.system.product_name",
                "result.property.system.product_version",
                "result.property.system.build_version",
            ]
        );

        let git_version = ResultContract::for_operation(
            "git.version",
            ResultSemantic::Completed,
            None,
            vec![RawOutputLine::Stdout("git version 2.39.3 (Apple Git-146)".into())],
            None,
        );
        let ResultPayload::PropertiesReport { properties, .. } = git_version.payload else {
            panic!("git version was not structured")
        };
        assert_eq!(properties.len(), 1);
        assert_eq!(properties[0].label_key, "result.property.git.version");
        // The Apple vendor suffix is kept, not stripped: it is real
        // information about which build is installed, not noise.
        assert_eq!(properties[0].value, "2.39.3 (Apple Git-146)");

        // `files.identify` and `system.architecture` share one parser
        // (`parse_single_line`): both are `file -b`/`uname -m` — exactly one
        // non-empty line, no fixed prefix to strip (unlike `git --version`).
        let file_type = ResultContract::for_operation(
            "files.identify",
            ResultSemantic::Completed,
            None,
            vec![RawOutputLine::Stdout("ASCII text".into())],
            None,
        );
        let ResultPayload::PropertiesReport { properties, .. } = file_type.payload else {
            panic!("file type was not structured")
        };
        assert_eq!(properties, [property("result.property.file_type", "ASCII text")]);

        let architecture = ResultContract::for_operation(
            "system.architecture",
            ResultSemantic::Completed,
            None,
            vec![RawOutputLine::Stdout("arm64".into())],
            None,
        );
        let ResultPayload::PropertiesReport { properties, .. } = architecture.payload else {
            panic!("architecture was not structured")
        };
        assert_eq!(properties, [property("result.property.system.architecture", "arm64")]);
    }

    #[test]
    fn ping_summary_becomes_numeric_metrics_with_units() {
        let result = ResultContract::for_operation(
            "net.ping",
            ResultSemantic::Completed,
            None,
            ping_output(),
            None,
        );
        let ResultPayload::Metrics { metrics, .. } = result.payload else {
            panic!("ping summary was not structured")
        };
        assert_eq!(metrics.len(), 7);
        assert_eq!(metrics[0].label_key, "result.metric.ping.transmitted");
        assert_eq!(metrics[0].value, "1");
        assert_eq!(metrics[0].unit.as_deref(), Some("packets"));
        assert_eq!(metrics[2].label_key, "result.metric.ping.packet_loss");
        assert_eq!(metrics[2].unit.as_deref(), Some("%"));
        assert_eq!(metrics[4].label_key, "result.metric.ping.average");
        assert_eq!(metrics[4].value, "12.250");
        assert_eq!(metrics[4].unit.as_deref(), Some("ms"));
    }

    #[test]
    fn unstable_or_unrecognized_formats_use_the_honest_raw_fallback() {
        for op_id in [
            "compress.zip.list",
            "compress.tar.list",
            "disk.list",
            "net.ports",
            "security.permissions",
            "security.xattr",
            "system.report",
            "system.uptime",
        ] {
            let result = ResultContract::for_operation(
                op_id,
                ResultSemantic::Completed,
                None,
                vec![RawOutputLine::Stdout("human tool prose".into())],
                None,
            );
            assert_eq!(result.category, ResultCategory::RawOutput, "{op_id}");
            assert!(matches!(result.payload, ResultPayload::RawOutput { .. }), "{op_id}");
        }

        for (op_id, malformed) in [
            ("files.tree.size", "not a du row"),
            ("disk.free", "not a df row"),
            ("net.dns", "not a DNS answer"),
            ("git.status", "not porcelain"),
            ("system.processes", "not a process row"),
            ("image.info", "not sips metadata"),
            ("net.headers", "not an HTTP header"),
            ("system.info", "not sw_vers"),
            ("net.ping", "not ping output"),
            ("git.log", "not four tab-separated fields"),
            ("git.version", "not git --version output"),
            ("system.process.find", "not-a-pid loginwindow"),
        ] {
            let result = ResultContract::for_operation(
                op_id,
                ResultSemantic::Completed,
                None,
                vec![RawOutputLine::Stdout(malformed.into())],
                None,
            );
            assert_eq!(result.category, ResultCategory::RawOutput, "{op_id}");
        }

        // `files.identify`/`system.architecture` accept any single non-empty
        // line (there is no fixed prefix to validate, unlike `git --version`),
        // so the only way to fall outside their grammar is more than one
        // line — mirrors the same invariant `parse_git_version` enforces.
        for op_id in ["files.identify", "system.architecture"] {
            let result = ResultContract::for_operation(
                op_id,
                ResultSemantic::Completed,
                None,
                vec![
                    RawOutputLine::Stdout("first line".into()),
                    RawOutputLine::Stdout("second line".into()),
                ],
                None,
            );
            assert_eq!(result.category, ResultCategory::RawOutput, "{op_id}");
        }

        let warning = ResultContract::for_operation(
            "files.find.large",
            ResultSemantic::Completed,
            None,
            vec![
                RawOutputLine::Stdout("/tmp/a".into()),
                RawOutputLine::Stderr("permission warning".into()),
            ],
            None,
        );
        assert_eq!(warning.category, ResultCategory::RawOutput);
    }

    #[test]
    fn git_diff_comparison_is_driven_by_exit_semantics_not_output_prose() {
        // `git.diff.commits` reuses `ComparisonKind::GitDiff`: same `--stat`
        // output shape, same exit-code-driven semantics as `git.diff` — the
        // two operations differ in what they compare, not in how the result
        // is classified.
        for op_id in ["git.diff", "git.diff.commits"] {
            for (semantic, expected_equal) in
                [(ResultSemantic::NoDifferences, true), (ResultSemantic::Differences, false)]
            {
                let encoded = serde_json::to_value(ResultContract::for_operation(
                    op_id,
                    semantic,
                    None,
                    vec![RawOutputLine::Stdout("arbitrary --stat prose".into())],
                    None,
                ))
                .unwrap();
                assert_eq!(encoded["kind"], "git_diff", "{op_id}");
                assert_eq!(encoded["equal"], expected_equal, "{op_id}");
                assert!(encoded["reference"].is_null(), "{op_id}");
                assert!(encoded["comparison"].is_null(), "{op_id}");
            }
        }
    }

    /// `disk.compare.bytes` reuses `ExitSemantics::Diff` (same 0/1 split as
    /// `git.diff`), but is its own `ComparisonKind::Bytes` — a different
    /// tool (`cmp`) with a different output shape, not a `git.diff` alias.
    #[test]
    fn disk_compare_bytes_is_driven_by_exit_semantics_not_output_prose() {
        for (semantic, expected_equal) in
            [(ResultSemantic::NoDifferences, true), (ResultSemantic::Differences, false)]
        {
            let encoded = serde_json::to_value(ResultContract::for_operation(
                "disk.compare.bytes",
                semantic,
                None,
                vec![RawOutputLine::Stdout("/a /b differ: char 3, line 1".into())],
                None,
            ))
            .unwrap();
            assert_eq!(encoded["kind"], "bytes");
            assert_eq!(encoded["equal"], expected_equal);
        }
    }

    #[test]
    fn verdict_value_is_the_core_classified_semantic_not_parsed_prose() {
        let result = ResultContract::for_operation(
            "security.gatekeeper",
            ResultSemantic::Rejected,
            None,
            vec![RawOutputLine::Stderr("arbitrary tool prose".into())],
            None,
        );
        let encoded = serde_json::to_value(result).unwrap();
        assert_eq!(encoded["kind"], "gatekeeper");
        assert_eq!(encoded["value"], "rejected");
        assert_eq!(encoded["details"][0]["value"], "arbitrary tool prose");

        // `security.codesign.verify` is its own `VerdictKind::CodeIntegrity`
        // — a distinct question from `security.codesign`'s `CodeSignature`
        // kind (signed-at-all vs. still-intact), reusing `Accepted`/
        // `Rejected` the same way `Gatekeeper` does.
        let integrity = ResultContract::for_operation(
            "security.codesign.verify",
            ResultSemantic::Rejected,
            None,
            vec![RawOutputLine::Stderr("main executable failed strict validation".into())],
            None,
        );
        let encoded = serde_json::to_value(integrity).unwrap();
        assert_eq!(encoded["kind"], "code_integrity");
        assert_eq!(encoded["value"], "rejected");
    }

    #[test]
    fn terminal_event_output_is_a_bounded_tail_with_an_omitted_head_marker() {
        let mut output = EventOutputTail::new();
        for index in 0..(MAX_EVENT_OUTPUT_LINES + 5) {
            output.push(RawOutputLine::Stdout(index.to_string()));
        }

        let lines = output.into_lines();
        assert_eq!(lines.len(), MAX_EVENT_OUTPUT_LINES);
        assert_eq!(lines.first(), Some(&RawOutputLine::Omitted { dropped: 6 }));
        assert_eq!(lines.get(1), Some(&RawOutputLine::Stdout("6".into())));
        assert_eq!(
            lines.last(),
            Some(&RawOutputLine::Stdout((MAX_EVENT_OUTPUT_LINES + 4).to_string()))
        );
    }

    #[test]
    fn persisted_result_tail_announces_rows_it_omits() {
        let mut output = EventOutputTail::with_limit(3);
        for index in 0..5 {
            output.push(RawOutputLine::Stdout(index.to_string()));
        }

        assert_eq!(
            output.into_lines(),
            vec![
                RawOutputLine::Omitted { dropped: 3 },
                RawOutputLine::Stdout("3".into()),
                RawOutputLine::Stdout("4".into()),
            ]
        );
    }

    #[test]
    fn terminal_event_tail_preserves_the_executor_truncation_marker() {
        let mut output = EventOutputTail::new();
        for index in 0..(MAX_EVENT_OUTPUT_LINES + 5) {
            output.push(RawOutputLine::Stdout(index.to_string()));
        }
        output.push(RawOutputLine::Truncated { dropped: 77 });

        let lines = output.into_lines();
        assert_eq!(lines.len(), MAX_EVENT_OUTPUT_LINES);
        assert_eq!(lines.first(), Some(&RawOutputLine::Omitted { dropped: 7 }));
        assert_eq!(lines.last(), Some(&RawOutputLine::Truncated { dropped: 77 }));
        assert_eq!(
            serde_json::to_value(lines.first().unwrap()).unwrap(),
            serde_json::json!({ "stream": "omitted", "line": { "dropped": 7 } })
        );
        assert_eq!(
            serde_json::to_value(lines.last().unwrap()).unwrap(),
            serde_json::json!({ "stream": "truncated", "line": { "dropped": 77 } })
        );
    }
}
