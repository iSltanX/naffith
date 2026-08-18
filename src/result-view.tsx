import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { errorText, t, tOptional } from "./i18n";
import type {
  OutputNotice,
  Outcome,
  RawOutputLine,
  ResultCategory,
  ResultContract,
  ResultSemantic,
  StructuredLine,
  VerdictKind,
} from "./ipc";
import "./result-view.css";

/**
 * The i18n key for a verdict's headline/body — scoped by `kind`, not just by
 * `value`.
 *
 * The same `ResultSemantic` means something different depending on which
 * tool produced it: `accepted`/`rejected` from `security.gatekeeper` is a
 * macOS *policy* decision, while the same two values from
 * `security.codesign.verify` are a *cryptographic* integrity check, and from
 * `compress.zip.test` they describe an archive, not a signed binary at all.
 * Rendering one generic string for every kind was the bug — see H-1 in the
 * branch audit — so every verdict site must key its copy by `(kind, value)`,
 * never by `value` alone.
 */
function verdictSemanticKey(kind: VerdictKind, value: ResultSemantic): string {
  return `result.semantic.${kind}.${value}`;
}

export type { ResultCategory, ResultContract } from "./ipc";

interface CategoryMeta {
  title: string;
  body: string;
}

/** Exhaustive visual mapping: adding a Rust family must fail TypeScript here. */
const CATEGORY: Record<ResultCategory, CategoryMeta> = {
  artifact: { title: "result.artifact.title", body: "result.artifact.body" },
  acknowledgement: {
    title: "result.acknowledgement.title",
    body: "result.acknowledgement.body",
  },
  collection: { title: "result.collection.title", body: "result.collection.body" },
  properties_report: {
    title: "result.properties.title",
    body: "result.properties.body",
  },
  metrics: { title: "result.metrics.title", body: "result.metrics.body" },
  digest: { title: "result.digest.title", body: "result.digest.body" },
  comparison: { title: "result.comparison.title", body: "result.comparison.body" },
  verdict: { title: "result.verdict.title", body: "result.verdict.body" },
  diff_search: {
    title: "result.diff_search.title",
    body: "result.diff_search.body",
  },
  diagnostic: {
    title: "result.diagnostic.title",
    body: "result.diagnostic.body",
  },
  raw_output: {
    title: "result.raw_output.title",
    body: "result.raw_output.body",
  },
};

const SEMANTIC_ICON: Record<ResultSemantic, string> = {
  completed: "#i-success",
  matches: "#i-success",
  no_matches: "#i-info",
  differences: "#i-warning",
  no_differences: "#i-success",
  accepted: "#i-success",
  rejected: "#i-warning",
  signed: "#i-success",
  unsigned: "#i-warning",
  failed: "#i-error",
  cancelled: "#i-cancelled",
};

type Tone = "success" | "info" | "warning" | "danger" | "neutral";

const SEMANTIC_TONE: Record<ResultSemantic, Tone> = {
  completed: "success",
  matches: "success",
  no_matches: "info",
  differences: "warning",
  no_differences: "success",
  accepted: "success",
  rejected: "warning",
  signed: "success",
  unsigned: "warning",
  failed: "danger",
  cancelled: "neutral",
};

const STATUS_LABEL: Record<Tone, string> = {
  success: "result.status.success",
  info: "result.status.info",
  warning: "result.status.warning",
  danger: "result.status.failure",
  neutral: "result.status.cancelled",
};

function markerText(line: Extract<RawOutputLine, { stream: "omitted" | "truncated" }>) {
  return `${t(line.stream === "omitted" ? "stream.omitted" : "stream.truncated")} ${line.line.dropped}`;
}

function rawLineText(line: RawOutputLine): string {
  return line.stream === "stdout" || line.stream === "stderr" ? line.line : markerText(line);
}

function structuredText(lines: readonly StructuredLine[]): string {
  return lines.map((line) => line.value).join("\n");
}

function outputNotices(result: ResultContract): readonly OutputNotice[] {
  if (
    result.type === "acknowledgement" ||
    result.type === "collection" ||
    result.type === "properties_report" ||
    result.type === "metrics" ||
    result.type === "digest" ||
    result.type === "comparison" ||
    result.type === "verdict" ||
    result.type === "diff_search"
  ) {
    return result.notices;
  }
  const lines = result.type === "artifact" ? (result.output ?? []) : result.lines;
  return lines.flatMap((line) =>
    line.stream === "omitted" || line.stream === "truncated"
      ? [{ kind: line.stream, dropped: line.line.dropped }]
      : [],
  );
}

function copyText(result: ResultContract): string {
  switch (result.type) {
    case "artifact":
      return result.path;
    case "acknowledgement":
      return [t(result.message_key), structuredText(result.details)].filter(Boolean).join("\n");
    case "collection":
      return [result.columns.map(t).join("\t"), ...result.rows.map((row) => row.cells.join("\t"))]
        .filter(Boolean)
        .join("\n");
    case "properties_report":
      return result.properties
        .map((item) => `${t(item.label_key)}: ${item.value}`)
        .join("\n");
    case "metrics":
      return result.metrics
        .map((item) => {
          const value = `${item.value}${item.unit ? ` ${item.unit}` : ""}`;
          return `${t(item.label_key)}: ${value}`;
        })
        .join("\n");
    case "digest":
      return result.value ?? structuredText(result.details);
    case "comparison":
      return [result.reference, result.comparison, structuredText(result.details)]
        .filter(Boolean)
        .join("\n");
    case "verdict":
      return [t(verdictSemanticKey(result.kind, result.value)), structuredText(result.details)]
        .filter(Boolean)
        .join("\n");
    case "diff_search":
      return structuredText(result.items);
    case "diagnostic":
    case "raw_output":
      return result.lines.map(rawLineText).join("\n");
  }
}

export interface ResultDiagnostic {
  label: string;
  value?: string | number;
}

export interface ResultExecutionDetails {
  runId: string;
  status: Outcome["status"];
  exitCode?: number | null;
  signal?: number | null;
  executable?: string;
  warnings?: string[];
}

export interface ResultViewProps {
  result: ResultContract;
  /**
   * معرّف العملية، لصياغة عنوان النتيجة بلسانها: «تم إنشاء الأرشيف» بدل
   * «ملف أو مجلد جاهز». وهو **للنصّ وحده** — الاختيار البنيوي (أي قالبٍ
   * يُرسم، وأي إجراءٍ يظهر) يبقى مشتقًّا من العقد الذي أرسلته النواة، فلا
   * تعود الواجهة تقرّر شيئًا من معرّف عملية.
   */
  operationId?: string | undefined;
  diagnostic?: ResultDiagnostic | undefined;
  execution?: ResultExecutionDetails | undefined;
  onReveal?: (() => void) | undefined;
  onRunAgain: () => void;
  onLibrary: () => void;
}

type ActionContext = "artifact" | "information" | "state-change" | "failure";

const ACTION_HEADING: Record<ActionContext, string> = {
  artifact: "result.actions.artifact",
  information: "result.actions.result",
  "state-change": "result.actions.followup",
  failure: "result.actions.recovery",
};

function actionContext(result: ResultContract): ActionContext {
  if (result.semantic === "failed") return "failure";
  if (result.type === "acknowledgement" || result.semantic === "cancelled") {
    return "state-change";
  }
  return result.type === "artifact" ? "artifact" : "information";
}

interface StatusPresentation {
  tone: Tone;
  icon: string;
  titleKey: string;
  bodyKey: string;
  badgeKey: string;
  eyebrowKey: string;
}

/**
 * The transport outcome owns execution failures while `ResultSemantic` owns
 * successful domain answers. In particular, `signalled` and wire `error`
 * must not collapse into the generic failed header. Truncation only promotes
 * an otherwise successful result to Page 17's WarningPartial presentation.
 */
function statusPresentation(
  result: ResultContract,
  execution: ResultExecutionDetails | undefined,
  notices: readonly OutputNotice[],
): StatusPresentation {
  switch (execution?.status) {
    case "signalled":
      return {
        tone: "danger",
        icon: "#i-error",
        titleKey: "result.execution.signalled",
        bodyKey: "result.execution.signalled.body",
        badgeKey: "result.status.signalled",
        eyebrowKey: "result.heading",
      };
    case "error":
      return {
        tone: "danger",
        icon: "#i-error",
        titleKey: "result.execution.core_error",
        bodyKey: "result.execution.core_error.body",
        badgeKey: "result.status.core_error",
        eyebrowKey: "result.heading",
      };
    case "failed":
      return {
        tone: "danger",
        icon: "#i-error",
        titleKey: "result.semantic.failed",
        bodyKey: "result.semantic.failed.body",
        badgeKey: "result.status.failure",
        eyebrowKey: "result.heading",
      };
    case "cancelled":
      return {
        tone: "neutral",
        icon: "#i-cancelled",
        titleKey: "result.semantic.cancelled",
        bodyKey: "result.semantic.cancelled.body",
        badgeKey: "result.status.cancelled",
        eyebrowKey: "result.heading",
      };
    case "success":
    case undefined:
      break;
  }

  const partial =
    notices.length > 0 && result.semantic !== "failed" && result.semantic !== "cancelled";
  if (partial) {
    return {
      tone: "warning",
      icon: "#i-warning",
      titleKey: "result.partial.title",
      bodyKey: "result.partial.body",
      badgeKey: "result.status.warning",
      eyebrowKey: "result.partial.eyebrow",
    };
  }

  const tone = SEMANTIC_TONE[result.semantic];
  // Verdict copy is kind-scoped — see `verdictSemanticKey`. `result.semantic`
  // and `result.value` are the same value for every verdict (the wire
  // contract's own deserializer rejects a payload where they differ), so
  // reading `result.semantic` here is exactly the value the inner content
  // renders too.
  const semanticKey =
    result.type === "verdict"
      ? verdictSemanticKey(result.kind, result.semantic)
      : `result.semantic.${result.semantic}`;
  return {
    tone,
    icon: SEMANTIC_ICON[result.semantic],
    titleKey: semanticKey,
    bodyKey: `${semanticKey}.body`,
    badgeKey: STATUS_LABEL[tone],
    eyebrowKey: "result.heading",
  };
}

export default function ResultView({
  result,
  operationId,
  diagnostic,
  execution,
  onReveal,
  onRunAgain,
  onLibrary,
}: ResultViewProps) {
  const heading = useRef<HTMLHeadingElement>(null);
  const meta = CATEGORY[result.category];
  // عنوان النتيجة بلسان العملية حين تعلنه، وإلا عنوان العائلة. ولا يُصاغ إلا
  // لنتيجةٍ ناجحة: «تم إنشاء الأرشيف» فوق تشخيصِ فشلٍ كذبٌ صريح.
  const succeeded = result.semantic !== "failed" && result.semantic !== "cancelled";
  const operationTitle =
    operationId && succeeded ? tOptional(`op.${operationId}.result`) : null;
  const notices = outputNotices(result);
  const status = statusPresentation(result, execution, notices);
  const tone = status.tone;
  const actions = actionContext(result);
  const [copyState, setCopyState] = useState<"idle" | "copied" | "failed">("idle");

  useEffect(() => {
    heading.current?.focus();
  }, []);

  const valueToCopy = useMemo(() => copyText(result), [result]);

  async function copy() {
    try {
      await navigator.clipboard.writeText(valueToCopy);
      setCopyState("copied");
      window.setTimeout(() => setCopyState("idle"), 1600);
    } catch {
      setCopyState("failed");
    }
  }

  // Reveal remains artifact-only and run-scoped. The callback closes over the
  // current run id; neither control receives or re-submits the displayed path.
  const revealAction =
    result.type === "artifact" && result.reveal !== undefined ? onReveal : undefined;
  const canReveal = revealAction !== undefined;
  const contentIsCopyable = actions !== "state-change" && valueToCopy.trim().length > 0;
  const copyClass =
    actions === "failure"
      ? "btn result-actions__copy--failure"
      : canReveal
        ? "btn btn--outline"
        : "btn btn--primary";
  const runAgainClass = actions === "state-change" ? "btn btn--primary" : "btn btn--outline";

  return (
    <section className={`result-workspace result-workspace--${tone}`} aria-labelledby="result-view-heading">
      <p className="result-workspace__eyebrow t-label">
        {t(status.eyebrowKey)}
      </p>

      <header
        className={`result-status result-status--${tone}`}
        role={tone === "danger" ? "alert" : "status"}
      >
        <span className="result-status__icon" aria-hidden="true">
          <svg viewBox="0 0 24 24"><use href={status.icon} /></svg>
        </span>
        <div className="result-status__copy">
          <h2
            id="result-view-heading"
            ref={heading}
            tabIndex={-1}
            className="t-card-title result-status__title"
          >
            {t(status.titleKey)}
          </h2>
          <p className="t-body-sec result-status__body">
            {t(status.bodyKey)}
          </p>
        </div>
        <span
          className={`result-status__badge result-status__badge--${tone}`}
          aria-hidden="true"
          dir="ltr"
          lang="en"
        >
          {t(status.badgeKey)}
        </span>
      </header>

      <article className={`result-content result-content--${result.type}`}>
        <header className="result-content__head">
          <div className="result-content__heading-copy">
            <h3 className="t-card-title result-content__title">
              {operationTitle ?? t(meta.title)}
            </h3>
            <p className="t-body-sec result-content__summary">{t(meta.body)}</p>
          </div>
          <span className="result-content__contract">
            {t(
              result.type === "raw_output" || result.type === "diagnostic"
                ? "result.contract.raw"
                : "result.contract.structured",
            )}
          </span>
        </header>

        <div className="result-content__divider" />
        <ResultBody
          result={result}
          onReveal={revealAction}
          onCopy={copy}
          copyState={copyState}
          onRunAgain={onRunAgain}
          onLibrary={onLibrary}
          canCopy={valueToCopy.trim().length > 0}
        />

        {notices.length > 0 && (
          <ul className="result-notices" aria-label={t("result.partial")}>
            {notices.map((notice, index) => (
              <li key={`${notice.kind}:${index}`}>
                <svg viewBox="0 0 24 24" aria-hidden="true"><use href="#i-warning" /></svg>
                <span>
                  {t(notice.kind === "omitted" ? "stream.omitted" : "stream.truncated")}{" "}
                  <bdi className="num">{notice.dropped}</bdi>
                </span>
              </li>
            ))}
          </ul>
        )}

        {diagnostic && (
          <p className="result-diagnostic t-body-sec">
            <span>{diagnostic.label}</span>
            {diagnostic.value !== undefined && <bdi className="num">{diagnostic.value}</bdi>}
          </p>
        )}
      </article>

      <div className="result-actions">
        <p className="t-label result-actions__label">
          {t(ACTION_HEADING[actions])}
        </p>
        <div className="result-actions__buttons">
          {canReveal && (
            <button
              type="button"
              className="btn btn--primary"
              onClick={() => revealAction()}
            >
              {t("action.reveal")}
            </button>
          )}
          {contentIsCopyable && (
            <button
              type="button"
              className={copyClass}
              onClick={copy}
            >
              {t(
                copyState === "copied"
                  ? "action.copied"
                  : actions === "failure"
                    ? "action.copy.diagnostic"
                  : result.type === "artifact"
                    ? "action.copy.path"
                    : "action.copy.result",
              )}
            </button>
          )}
          <button type="button" className={runAgainClass} onClick={onRunAgain}>
            {t("action.again")}
          </button>
          <button type="button" className="btn btn--outline" onClick={onLibrary}>
            {t("action.return.library")}
          </button>
        </div>
        {copyState === "failed" && (
          <p className="result-actions__error" role="alert">{t("result.copy.failed")}</p>
        )}
      </div>

      <TechnicalDetails execution={execution} notices={notices} />
    </section>
  );
}

function ResultBody({
  result,
  onReveal,
  onCopy,
  copyState,
  onRunAgain,
  onLibrary,
  canCopy,
}: {
  result: ResultContract;
  onReveal?: (() => void) | undefined;
  onCopy: () => void;
  copyState: "idle" | "copied" | "failed";
  onRunAgain: () => void;
  onLibrary: () => void;
  canCopy: boolean;
}) {
  const diagnostic = result.type === "diagnostic";
  return (
    <div className="result-template">
      <div className="result-template__body">
        <ResultBodyContent result={result} onReveal={onReveal} />
      </div>
      <div className="result-template__actions">
        <button
          type="button"
          className="result-template__action"
          disabled={!canCopy}
          onClick={onCopy}
        >
          {t(
            copyState === "copied"
              ? "action.copied"
              : diagnostic
                ? "action.copy.diagnostic"
                : "action.copy.short",
          )}
        </button>
        <button
          type="button"
          className="result-template__action result-template__action--accent"
          onClick={diagnostic ? onLibrary : onRunAgain}
        >
          {t(diagnostic ? "action.return.library" : "action.again")}
        </button>
      </div>
    </div>
  );
}

function ResultBodyContent({
  result,
  onReveal,
}: {
  result: ResultContract;
  onReveal?: (() => void) | undefined;
}) {
  switch (result.type) {
    case "artifact":
      return (
        <div className="result-artifact">
          <div className="result-artifact__target">
            <div className="result-artifact__name">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <use href={result.reveal === "directory" ? "#i-folder" : "#i-file"} />
              </svg>
              <strong>{result.name}</strong>
            </div>
            {onReveal ? (
              <button
                type="button"
                className="btn btn--ghost btn--sm result-artifact__reveal"
                onClick={() => onReveal()}
              >
                {t("action.reveal")}
              </button>
            ) : (
              <span className="result-artifact__unavailable">
                {t("result.artifact.location.unavailable")}
              </span>
            )}
          </div>
          <ArtifactFacts
            sizeBytes={result.size_bytes}
            entries={result.entries}
            parent={result.parent}
          />
          <bdi dir="ltr" className="result-artifact__path">{result.path}</bdi>
        </div>
      );
    case "acknowledgement":
      return (
        <div className="result-acknowledgement">
          <strong>{t(result.message_key)}</strong>
          <p>{t("result.acknowledgement.empty")}</p>
          <StructuredLines lines={result.details} />
        </div>
      );
    case "collection":
      return result.rows.length === 0 ? <EmptyResult semantic={result.semantic} /> : (
        <div className="result-table-wrap">
          <table className="result-table" aria-label={t("result.collection.title")}>
            <thead>
              <tr>{result.columns.map((column, index) => <th key={`${column}:${index}`}>{t(column)}</th>)}</tr>
            </thead>
            <tbody>
              {result.rows.map((row, index) => (
                <tr key={index} className={row.stream === "stderr" ? "result-row--stderr" : undefined}>
                  {row.cells.map((cell, cellIndex) => <td key={cellIndex}><bdi dir="auto">{cell}</bdi></td>)}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      );
    case "properties_report":
      return result.properties.length === 0 ? <EmptyResult semantic={result.semantic} /> : (
        <dl className="result-properties">
          {result.properties.map((property, index) => (
            <div
              key={index}
              className={[
                property.stream === "stderr" ? "result-row--stderr" : "",
              ].filter(Boolean).join(" ") || undefined}
            >
              <dt>{t(property.label_key)}</dt>
              <dd><bdi dir="auto">{property.value}</bdi></dd>
            </div>
          ))}
        </dl>
      );
    case "metrics":
      return result.metrics.length === 0 ? <EmptyResult semantic={result.semantic} /> : (
        <dl className="result-metrics">
          {result.metrics.map((metric, index) => (
            <div key={index} className={metric.stream === "stderr" ? "result-row--stderr" : undefined}>
              <dd><bdi dir="auto">{metric.value}{metric.unit ? ` ${metric.unit}` : ""}</bdi></dd>
              <dt>{t(metric.label_key)}</dt>
            </div>
          ))}
        </dl>
      );
    case "digest":
      return (
        <div className="result-digest">
          <span>SHA-256</span>
          {result.value ? (
            <code dir="ltr">{result.value}</code>
          ) : (
            <StructuredLines lines={result.details} empty />
          )}
        </div>
      );
    case "comparison": {
      const equal = result.equal ?? result.semantic === "no_differences";
      const isGitDiff = result.kind === "git_diff";
      const referenceValue = result.reference ?? t(
        isGitDiff ? "result.comparison.git.reference" : equal ? "result.equal" : "result.unavailable",
      );
      const comparisonValue = result.comparison ?? t(
        isGitDiff ? "result.comparison.git.current" : `result.semantic.${result.semantic}`,
      );
      return (
        <>
          <div className="result-comparison">
            <div className={equal ? "result-comparison__side result-comparison__side--success" : "result-comparison__side"}>
              <span>{t("result.comparison.reference")}</span>
              <strong>
                <bdi dir="auto">{referenceValue}</bdi>
              </strong>
            </div>
            <div className={equal ? "result-comparison__side result-comparison__side--success" : "result-comparison__side result-comparison__side--warning"}>
              <span>{t("result.comparison.compared")}</span>
              <strong>
                <bdi dir="auto">{comparisonValue}</bdi>
              </strong>
            </div>
          </div>
          <StructuredLines lines={result.details} />
        </>
      );
    }
    case "verdict": {
      const semanticKey = verdictSemanticKey(result.kind, result.value);
      return (
        <div className={`result-verdict result-verdict--${SEMANTIC_TONE[result.value]}`}>
          <strong>{t(semanticKey)}</strong>
          <p>{t(`${semanticKey}.body`)}</p>
          <StructuredLines lines={result.details} />
        </div>
      );
    }
    case "diff_search":
      return result.items.length === 0 ? (
        <EmptyResult semantic={result.semantic} />
      ) : (
        <StructuredLines lines={result.items} framed />
      );
    case "diagnostic":
    case "raw_output":
      return result.lines.length === 0 ? (
        <EmptyResult semantic={result.semantic} />
      ) : (
        <RawOutput lines={result.lines} />
      );
  }
}

function StructuredLines({
  lines,
  empty = false,
  framed = false,
}: {
  lines: readonly StructuredLine[];
  empty?: boolean;
  framed?: boolean;
}) {
  if (lines.length === 0) return empty ? <EmptyResult semantic="completed" /> : null;
  return (
    <ol className={framed ? "result-lines result-lines--framed" : "result-lines"}>
      {lines.map((line, index) => (
        <li key={index} className={line.stream === "stderr" ? "result-row--stderr" : undefined}>
          <code dir="auto">{line.value}</code>
        </li>
      ))}
    </ol>
  );
}

const ARABIC_NUMBER = new Intl.NumberFormat("ar");

/** حجمٌ مقروء: أكبر وحدةٍ لا يقلّ فيها الرقم عن واحد، بمنزلةٍ عشرية واحدة. */
function formatBytes(bytes: number): string {
  const units = ["unit.byte", "unit.kib", "unit.mib", "unit.gib", "unit.tib"] as const;
  let value = bytes;
  let step = 0;
  while (value >= 1024 && step < units.length - 1) {
    value /= 1024;
    step += 1;
  }
  // البايتات أعدادٌ صحيحة؛ وما فوقها منزلةٌ واحدة تكفي للحكم ولا تُثقل السطر.
  const rounded = step === 0 ? value : Math.round(value * 10) / 10;
  return `${ARABIC_NUMBER.format(rounded)} ${t(units[step] as string)}`;
}

/**
 * حقائق الناتج التي أثبتتها النواة.
 *
 * كلٌّ منها يظهر **إن وُجد وحده**: النواة تُرسل ما تستطيع إثباته، والشاشة تطوي
 * ما غاب بدل أن تعرض «غير معروف» صفًّا بعد صفّ. ولا حساب هنا سوى صياغة الوحدة.
 */
function ArtifactFacts({
  sizeBytes,
  entries,
  parent,
}: {
  sizeBytes?: number | undefined;
  entries?: number | undefined;
  parent: string;
}) {
  const facts: Array<{ key: string; label: string; value: ReactNode }> = [];
  if (entries !== undefined) {
    facts.push({
      key: "entries",
      label: t("result.artifact.entries"),
      value: <span className="num">{ARABIC_NUMBER.format(entries)}</span>,
    });
  }
  if (sizeBytes !== undefined) {
    facts.push({
      key: "size",
      label: t("result.artifact.size"),
      value: <span className="num">{formatBytes(sizeBytes)}</span>,
    });
  }
  if (parent) {
    facts.push({
      key: "parent",
      label: t("result.artifact.destination"),
      value: (
        <bdi dir="ltr" className="result-artifact__destination">
          {parent}
        </bdi>
      ),
    });
  }
  if (facts.length === 0) return null;

  return (
    <dl className="result-artifact__facts">
      {facts.map((fact) => (
        <div key={fact.key} className="result-artifact__fact">
          <dt className="t-label">{fact.label}</dt>
          <dd>{fact.value}</dd>
        </div>
      ))}
    </dl>
  );
}

function EmptyResult({ semantic }: { semantic: ResultSemantic }) {
  const key =
    semantic === "no_matches"
      ? "result.empty"
      : semantic === "failed"
        ? "result.fallback"
        : semantic === "cancelled"
          ? "result.semantic.cancelled.body"
          : "result.silent";
  return (
    <p className="result-empty t-body-sec">
      {t(key)}
    </p>
  );
}

function RawOutput({ lines }: { lines: readonly RawOutputLine[] }) {
  return (
    <ol className="result-output" aria-label={t("result.raw_output.title")}>
      {lines.map((line, index) => (
        <li key={index} className={`result-output__line result-output__line--${line.stream}`}>
          {line.stream === "omitted" || line.stream === "truncated" ? (
            <span className="result-output__notice">{markerText(line)}</span>
          ) : (
            <>
              <span className="result-output__stream">
                {t(line.stream === "stderr" ? "stream.stderr" : "stream.stdout")}
              </span>
              <code dir="auto">{line.line}</code>
            </>
          )}
        </li>
      ))}
    </ol>
  );
}

function TechnicalDetails({
  execution,
  notices,
}: {
  execution?: ResultExecutionDetails | undefined;
  notices: readonly OutputNotice[];
}) {
  const warnings = execution?.warnings ?? [];
  const [open, setOpen] = useState(false);
  return (
    <details
      className={`result-technical${
        warnings.length > 0 || notices.length > 0 ? " result-technical--warning" : ""
      }`}
      onToggle={(event) => setOpen(event.currentTarget.open)}
    >
      <summary>
        <span>{t(open ? "result.technical.hide" : "result.technical")}</span>
        <small>{t("result.execution.data")}</small>
      </summary>
      <div className="result-technical__body">
        {execution ? (
          <dl>
            <TechnicalRow label={t("result.technical.run_id")} value={execution.runId} />
            <TechnicalRow label={t("result.technical.status")} value={execution.status} />
            {execution.exitCode != null && (
              <TechnicalRow label={t("result.technical.exit_code")} value={execution.exitCode} />
            )}
            {execution.signal != null && (
              <TechnicalRow label={t("result.technical.signal")} value={execution.signal} />
            )}
            {execution.executable && (
              <TechnicalRow label={t("result.technical.executable")} value={execution.executable} />
            )}
            <TechnicalRow
              label={t("result.technical.arguments")}
              value={t("result.technical.arguments.redacted")}
              direction="auto"
            />
          </dl>
        ) : (
          <p className="t-body-sec">{t("result.execution.unavailable")}</p>
        )}
        {warnings.length > 0 && (
          <ul className="result-technical__warnings">
            {warnings.map((warning, index) => (
              <li key={`${warning}:${index}`}>{t(warning)}</li>
            ))}
          </ul>
        )}
        <p className="result-technical__redaction">{t("result.technical.redacted")}</p>
      </div>
    </details>
  );
}

function TechnicalRow({
  label,
  value,
  direction = "ltr",
}: {
  label: string;
  value: ReactNode;
  direction?: "auto" | "ltr";
}) {
  return (
    <div>
      <dt>{label}</dt>
      <dd><bdi dir={direction}>{value}</bdi></dd>
    </div>
  );
}

/** Outcome detail remains visible inside the typed diagnostic presentation. */
export function diagnosticFor(outcome: {
  status: string;
  signal?: number | null;
  code?: number | null;
  key?: string;
}): ResultDiagnostic | undefined {
  if (outcome.status === "signalled" && outcome.signal != null) {
    return { label: t("state.failed.signal"), value: outcome.signal };
  }
  if (outcome.code != null) return { label: t("state.failed.code"), value: outcome.code };
  if (outcome.key) return { label: errorText(outcome.key) };
  return undefined;
}
