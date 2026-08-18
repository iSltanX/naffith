// @vitest-environment jsdom
import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AR } from "./i18n";
import ResultView, {
  type ResultCategory,
  type ResultContract,
} from "./result-view";

afterEach(cleanup);

const writeText = vi.fn<(value: string) => Promise<void>>();

beforeEach(() => {
  writeText.mockReset();
  writeText.mockResolvedValue();
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText },
  });
});

const FAMILY_RESULTS = {
  artifact: {
    category: "artifact",
    semantic: "completed",
    type: "artifact",
    path: "/Users/sara/أرشيف/النتيجة.zip",
    name: "النتيجة.zip",
    parent: "/Users/sara/أرشيف",
    output: [],
    reveal: "file",
  },
  acknowledgement: {
    category: "acknowledgement",
    semantic: "completed",
    type: "acknowledgement",
    message_key: "result.ack.commit_created",
    details: [],
    notices: [],
  },
  collection: {
    category: "collection",
    semantic: "matches",
    type: "collection",
    kind: "file_matches",
    columns: ["result.column.value"],
    rows: [{ cells: ["/Users/sara/نتيجة.txt"], stream: "stdout" }],
    notices: [],
  },
  properties_report: {
    category: "properties_report",
    semantic: "completed",
    type: "properties_report",
    kind: "image",
    properties: [{ label_key: "result.report.image.row", value: "1920 × 1080", stream: "stdout" }],
    notices: [],
  },
  metrics: {
    category: "metrics",
    semantic: "completed",
    type: "metrics",
    kind: "network_latency",
    metrics: [{ label_key: "result.metric.network_latency.row", value: "24", unit: "ms", stream: "stdout" }],
    notices: [],
  },
  digest: {
    category: "digest",
    semantic: "completed",
    type: "digest",
    algorithm: "sha256",
    value: "a".repeat(64),
    details: [],
    notices: [],
  },
  comparison: {
    category: "comparison",
    semantic: "no_differences",
    type: "comparison",
    kind: "sha256",
    reference: "a".repeat(64),
    comparison: "a".repeat(64),
    equal: true,
    details: [],
    notices: [],
  },
  verdict: {
    category: "verdict",
    semantic: "unsigned",
    type: "verdict",
    kind: "code_signature",
    value: "unsigned",
    details: [{ value: "code object is not signed", stream: "stderr" }],
    notices: [],
  },
  diff_search: {
    category: "diff_search",
    semantic: "differences",
    type: "diff_search",
    kind: "diff",
    items: [{ value: "- قديم\n+ جديد", stream: "stdout" }],
    notices: [],
  },
  diagnostic: {
    category: "diagnostic",
    semantic: "failed",
    type: "diagnostic",
    lines: [{ stream: "stderr", line: "permission denied" }],
  },
  raw_output: {
    category: "raw_output",
    semantic: "completed",
    type: "raw_output",
    lines: [{ stream: "stdout", line: "opaque tool output" }],
  },
} satisfies Record<ResultCategory, ResultContract>;

const TITLES: Record<ResultCategory, keyof typeof AR> = {
  artifact: "result.artifact.title",
  acknowledgement: "result.acknowledgement.title",
  collection: "result.collection.title",
  properties_report: "result.properties.title",
  metrics: "result.metrics.title",
  digest: "result.digest.title",
  comparison: "result.comparison.title",
  verdict: "result.verdict.title",
  diff_search: "result.diff_search.title",
  diagnostic: "result.diagnostic.title",
  raw_output: "result.raw_output.title",
};

function view(result: ResultContract, over: Partial<Parameters<typeof ResultView>[0]> = {}) {
  const props = {
    result,
    onReveal: vi.fn(),
    onRunAgain: vi.fn(),
    onLibrary: vi.fn(),
    ...over,
  };
  return { ...render(<ResultView {...props} />), props };
}

describe("نصوص Page 17 المعتمدة", () => {
  it("يثبت عناوين WarningPartial وحالات التنفيذ وسياقات الأفعال حرفيًا", () => {
    expect({
      partialEyebrow: AR["result.partial.eyebrow"],
      partialTitle: AR["result.partial.title"],
      partialBody: AR["result.partial.body"],
      signalledTitle: AR["result.execution.signalled"],
      signalledBody: AR["result.execution.signalled.body"],
      coreErrorTitle: AR["result.execution.core_error"],
      coreErrorBody: AR["result.execution.core_error.body"],
      artifactActions: AR["result.actions.artifact"],
      informationActions: AR["result.actions.result"],
      followupActions: AR["result.actions.followup"],
      recoveryActions: AR["result.actions.recovery"],
      diagnosticCopy: AR["action.copy.diagnostic"],
      unavailableLocation: AR["result.artifact.location.unavailable"],
    }).toEqual({
      partialEyebrow: "تحذير — نتيجة جزئية",
      partialTitle: "اكتملت مع تنبيه",
      partialBody: "النتيجة قابلة للاستخدام، مع تفاصيل تحتاج مراجعة.",
      signalledTitle: "توقفت العملية بإشارة",
      signalledBody: "أنهى النظام العملية قبل اكتمالها.",
      coreErrorTitle: "تعذر بدء التنفيذ الآمن",
      coreErrorBody: "رفضت طبقة التشغيل الطلب قبل تشغيل الأمر.",
      artifactActions: "إجراءات الملف",
      informationActions: "إجراءات النتيجة",
      followupActions: "إجراءات المتابعة",
      recoveryActions: "إجراءات الاسترداد",
      diagnosticCopy: "نسخ التشخيص",
      unavailableLocation: "الموقع غير متاح حاليًا",
    });
  });
});

describe("عائلات النتيجة المهيكلة", () => {
  it("ترسم قالبًا إنتاجيًا مميزًا لكل عائلة من العقد المغلق", () => {
    for (const category of Object.keys(FAMILY_RESULTS) as ResultCategory[]) {
      const result = FAMILY_RESULTS[category];
      const rendered = view(result);
      expect(screen.getByText(AR[TITLES[category]])).toBeTruthy();
      expect(rendered.container.querySelector(`.result-content--${result.type}`)).toBeTruthy();
      rendered.unmount();
    }
  });

  it("تعرض البيانات المهيكلة بحسب نوعها", () => {
    view(FAMILY_RESULTS.collection);
    const table = screen.getByRole("table");
    expect(within(table).getByText(AR["result.column.value"])).toBeTruthy();
    expect(within(table).getByText("/Users/sara/نتيجة.txt")).toBeTruthy();
    cleanup();

    view(FAMILY_RESULTS.properties_report);
    expect(screen.getByText(AR["result.report.image.row"])).toBeTruthy();
    expect(screen.getByText("1920 × 1080")).toBeTruthy();
    cleanup();

    view(FAMILY_RESULTS.metrics);
    expect(screen.getByText(AR["result.metric.network_latency.row"])).toBeTruthy();
    expect(screen.getByText("24 ms")).toBeTruthy();
    cleanup();

    view(FAMILY_RESULTS.digest);
    expect(screen.getByText("a".repeat(64))).toBeTruthy();
    cleanup();

    view(FAMILY_RESULTS.acknowledgement);
    expect(screen.getByText(AR["result.ack.commit_created"])).toBeTruthy();
  });

  it("تعرض دلالة النواة ولا تستنتجها من نص النتيجة", () => {
    const result: ResultContract = {
      category: "diff_search",
      semantic: "no_matches",
      type: "diff_search",
      kind: "search",
      items: [{ value: "MATCHES=999; accepted=true", stream: "stdout" }],
      notices: [],
    };
    const { container } = view(result);

    expect(screen.getAllByText(AR["result.semantic.no_matches"]).length).toBeGreaterThan(0);
    expect(screen.getByText("MATCHES=999; accepted=true")).toBeTruthy();
    expect(container.querySelector(".result-output")).toBeNull();
    expect(container.querySelector(".result-lines--framed")).toBeTruthy();
  });

  it("يعرض no matches وno diff والأحكام كإجابات دلالية لا كفشل عام", () => {
    const cases: ResultContract[] = [
      { ...FAMILY_RESULTS.diff_search, semantic: "no_matches", items: [] },
      { ...FAMILY_RESULTS.comparison, semantic: "no_differences", equal: true },
      { ...FAMILY_RESULTS.verdict, kind: "gatekeeper", semantic: "rejected", value: "rejected" },
      { ...FAMILY_RESULTS.verdict, semantic: "unsigned", value: "unsigned" },
    ];
    for (const result of cases) {
      const rendered = view(result);
      expect(rendered.container.querySelector(".result-status--danger")).toBeNull();
      rendered.unmount();
    }
  });

  /**
   * H-1 regression: the same `ResultSemantic` value must render different
   * copy depending on `kind` — a macOS *policy* decision (Gatekeeper) reads
   * nothing like a *cryptographic* integrity check (codesign --verify) or
   * an *archive* integrity check (unzip -t), even though all three can
   * carry `accepted`/`rejected`. Before the fix, every kind shared one
   * generic string, so a validly-signed-but-unnotarised binary — or an
   * intact ZIP — displayed "مقبول وفق سياسة macOS" (accepted per macOS
   * policy), which is not what either tool actually checked.
   */
  it("يفرد لكل نوع حكمٍ نصّه الخاص، لا نصّ Gatekeeper العام للجميع", () => {
    // النصّ يظهر مرّتين على الشاشة (شارة الرأس ومحتوى البطاقة معًا)، تمامًا
    // كما تفعل بقيّة العائلات — لذلك `getAllByText` لا `getByText`.
    const gatekeeper = view({
      ...FAMILY_RESULTS.verdict,
      kind: "gatekeeper",
      semantic: "accepted",
      value: "accepted",
    });
    expect(
      screen.getAllByText(AR["result.semantic.gatekeeper.accepted"]).length,
    ).toBeGreaterThan(0);
    gatekeeper.unmount();

    const codeIntegrity = view({
      ...FAMILY_RESULTS.verdict,
      kind: "code_integrity",
      semantic: "accepted",
      value: "accepted",
    });
    expect(
      screen.getAllByText(AR["result.semantic.code_integrity.accepted"]).length,
    ).toBeGreaterThan(0);
    expect(screen.queryByText(AR["result.semantic.gatekeeper.accepted"])).toBeNull();
    codeIntegrity.unmount();

    const archiveIntegrity = view({
      ...FAMILY_RESULTS.verdict,
      kind: "archive_integrity",
      semantic: "rejected",
      value: "rejected",
    });
    expect(
      screen.getAllByText(AR["result.semantic.archive_integrity.rejected"]).length,
    ).toBeGreaterThan(0);
    expect(screen.queryByText(AR["result.semantic.gatekeeper.rejected"])).toBeNull();
    archiveIntegrity.unmount();
  });
});

describe("ناتج الملف والأفعال", () => {
  const ARTIFACT = FAMILY_RESULTS.artifact;

  it("يعزل المسار ويعرض Reveal داخل الهدف وفي شريط الأفعال حين تعلنه النواة", async () => {
    const user = userEvent.setup();
    const { container, props } = view(ARTIFACT);
    const path = container.querySelector(".result-artifact__path");
    expect(path?.getAttribute("dir")).toBe("ltr");
    expect(path?.textContent).toBe(ARTIFACT.type === "artifact" ? ARTIFACT.path : "");
    const revealActions = screen.getAllByRole("button", { name: AR["action.reveal"] });
    expect(revealActions).toHaveLength(2);
    const inlineReveal = revealActions[0];
    const barReveal = revealActions[1];
    if (!inlineReveal || !barReveal) throw new Error("expected both Page 17 reveal controls");
    expect(container.querySelector(".result-artifact__reveal")).toBe(inlineReveal);

    await user.click(inlineReveal);
    await user.click(barReveal);
    expect(props.onReveal).toHaveBeenCalledTimes(2);
    expect(props.onReveal).toHaveBeenNthCalledWith(1);
    expect(props.onReveal).toHaveBeenNthCalledWith(2);

    cleanup();
    const withoutReveal: ResultContract = { ...ARTIFACT };
    delete withoutReveal.reveal;
    view(withoutReveal);
    expect(screen.queryByRole("button", { name: AR["action.reveal"] })).toBeNull();
    expect(screen.getByText(AR["result.artifact.location.unavailable"])).toBeTruthy();
  });

  it("لا يحوّل مسار الملف إلى قدرة كشف حين لا تتوفر دالة التشغيل", () => {
    view(ARTIFACT, { onReveal: undefined });

    expect(screen.queryByRole("button", { name: AR["action.reveal"] })).toBeNull();
    expect(screen.getByText(ARTIFACT.type === "artifact" ? ARTIFACT.path : "")).toBeTruthy();
    expect(screen.getByText(AR["result.artifact.location.unavailable"])).toBeTruthy();
  });

  it("ينسخ المسار ويستدعي التشغيل مجددًا والمكتبة ولا يعرض Open", async () => {
    const user = userEvent.setup();
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    const { props } = view(ARTIFACT);

    await user.click(screen.getByRole("button", { name: AR["action.copy.path"] }));
    expect(writeText).toHaveBeenCalledWith(
      ARTIFACT.type === "artifact" ? ARTIFACT.path : "",
    );
    expect(screen.getAllByRole("button", { name: AR["action.copied"] })).toHaveLength(2);

    await user.click(screen.getAllByRole("button", { name: AR["action.again"] })[0] as HTMLElement);
    await user.click(screen.getByRole("button", { name: AR["action.return.library"] }));
    expect(props.onRunAgain).toHaveBeenCalledTimes(1);
    expect(props.onLibrary).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("button", { name: /فتح|Open/i })).toBeNull();
  });

  it("يعرض فشل الحافظة كحالة قابلة للمراجعة", async () => {
    const user = userEvent.setup();
    writeText.mockRejectedValueOnce(new Error("clipboard unavailable"));
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    view(FAMILY_RESULTS.digest);

    await user.click(screen.getByRole("button", { name: AR["action.copy.result"] }));
    expect(screen.getByRole("alert").textContent).toContain(AR["result.copy.failed"]);
  });

  it("يطبق سياسات أفعال Page 17 بحسب السياق لا بحسب وجود نص عابر", () => {
    let rendered = view(FAMILY_RESULTS.acknowledgement);
    let actionBar = rendered.container.querySelector(".result-actions");
    expect(actionBar).toBeTruthy();
    expect(screen.getByText(AR["result.actions.followup"])).toBeTruthy();
    expect(screen.queryByRole("button", { name: AR["action.copy.result"] })).toBeNull();
    expect(
      within(actionBar as HTMLElement).getByRole("button", { name: AR["action.again"] })
        .classList,
    ).toContain("btn--primary");
    rendered.unmount();

    rendered = view(FAMILY_RESULTS.diagnostic);
    actionBar = rendered.container.querySelector(".result-actions");
    expect(actionBar).toBeTruthy();
    expect(screen.getByText(AR["result.actions.recovery"])).toBeTruthy();
    expect(
      within(actionBar as HTMLElement).getByRole("button", {
        name: AR["action.copy.diagnostic"],
      }).classList,
    ).toContain("result-actions__copy--failure");
    expect(
      within(actionBar as HTMLElement).getByRole("button", { name: AR["action.again"] })
        .classList,
    ).toContain("btn--outline");
    rendered.unmount();

    rendered = view({ ...FAMILY_RESULTS.diagnostic, semantic: "cancelled" });
    actionBar = rendered.container.querySelector(".result-actions");
    expect(actionBar).toBeTruthy();
    expect(screen.queryByRole("button", { name: AR["action.copy.result"] })).toBeNull();
    expect(
      within(actionBar as HTMLElement).getByRole("button", { name: AR["action.again"] })
        .classList,
    ).toContain("btn--primary");
    rendered.unmount();

    view(FAMILY_RESULTS.collection);
    expect(screen.getByText(AR["result.actions.result"])).toBeTruthy();
    expect(screen.getByRole("button", { name: AR["action.copy.result"] }).classList).toContain(
      "btn--primary",
    );
  });

  it.each(Object.keys(FAMILY_RESULTS) as ResultCategory[])(
    "يعرض أفعال القالب الداخلية لعائلة %s إضافةً إلى شريط الأفعال",
    (category) => {
      const { container } = view(FAMILY_RESULTS[category]);
      const inline = container.querySelector(".result-template__actions");
      expect(inline).toBeTruthy();
      const buttons = within(inline as HTMLElement).getAllByRole("button");
      expect(buttons).toHaveLength(2);
      expect(buttons[0]?.classList.contains("result-template__action")).toBe(true);
      expect(buttons[0]?.classList.contains("btn")).toBe(false);
      expect(buttons[1]?.classList.contains("result-template__action--accent")).toBe(true);

      if (category === "diagnostic") {
        expect(
          within(inline as HTMLElement).getByRole("button", {
            name: AR["action.copy.diagnostic"],
          }),
        ).toBeTruthy();
        expect(
          within(inline as HTMLElement).getByRole("button", {
            name: AR["action.return.library"],
          }),
        ).toBeTruthy();
        expect(
          within(inline as HTMLElement).queryByRole("button", { name: AR["action.again"] }),
        ).toBeNull();
      } else {
        expect(
          within(inline as HTMLElement).getByRole("button", { name: AR["action.copy.short"] }),
        ).toBeTruthy();
        expect(
          within(inline as HTMLElement).getByRole("button", { name: AR["action.again"] }),
        ).toBeTruthy();
      }
    },
  );

  it("يسمي سياق الملف بإجراءات الملف حتى إن تعذّر Reveal", () => {
    const artifact: ResultContract = { ...ARTIFACT };
    delete artifact.reveal;
    view(artifact);

    expect(screen.getByText(AR["result.actions.artifact"])).toBeTruthy();
    expect(screen.queryByText(AR["result.actions.result"])).toBeNull();
  });

  it("يبقي شريط الأفعال نصيًا كما في Figma ولا يكرر أيقونات زخرفية", () => {
    const { container } = view(ARTIFACT);
    expect(container.querySelector(".result-actions svg")).toBeNull();
    expect(container.querySelector(".result-content__head svg")).toBeNull();
    expect(container.querySelector(".result-status__badge")?.getAttribute("aria-hidden")).toBe(
      "true",
    );
  });
});

describe("الخرج والتفاصيل الآمنة", () => {
  it("يميّز القناتين والقص بوسم ونص لا باللون وحده", () => {
    const result: ResultContract = {
      category: "diagnostic",
      semantic: "failed",
      type: "diagnostic",
      lines: [
        { stream: "stdout", line: "ok" },
        { stream: "stderr", line: "permission denied" },
        { stream: "omitted", line: { dropped: 7 } },
        { stream: "truncated", line: { dropped: 42 } },
      ],
    };
    view(result);

    const output = screen.getByRole("list", { name: AR["result.raw_output.title"] });
    expect(within(output).getByText(AR["stream.stdout"])).toBeTruthy();
    expect(within(output).getByText(AR["stream.stderr"])).toBeTruthy();
    expect(within(output).getByText("7", { exact: false })).toBeTruthy();
    expect(within(output).getByText("42", { exact: false })).toBeTruthy();
    expect(screen.getByRole("list", { name: AR["result.partial"] })).toBeTruthy();
    expect(screen.getAllByRole("alert").length).toBeGreaterThan(0);
  });

  it("يعرض النتيجة القابلة للاستخدام بتحذير حين تكون جزئية أو مقصوصة", () => {
    view({
      ...FAMILY_RESULTS.collection,
      notices: [{ kind: "omitted", dropped: 9 }],
    });

    expect(screen.getByText(AR["result.partial.eyebrow"])).toBeTruthy();
    expect(screen.getByRole("heading", { name: AR["result.partial.title"] })).toBeTruthy();
    expect(screen.getByText(AR["result.partial.body"])).toBeTruthy();
    expect(screen.getByText(AR["result.status.warning"])).toBeTruthy();
    expect(document.querySelector(".result-status--warning")).toBeTruthy();
  });

  it("يفرّق رؤوس Signalled وCoreError عن فشل الأداة العام بأسماء السلك الدقيقة", () => {
    let rendered = view(FAMILY_RESULTS.diagnostic, {
      execution: { runId: "run-signalled", status: "signalled", signal: 9 },
    });
    expect(
      screen.getByRole("heading", { name: AR["result.execution.signalled"] }),
    ).toBeTruthy();
    expect(screen.getByText(AR["result.execution.signalled.body"])).toBeTruthy();
    expect(screen.getByText(AR["result.status.signalled"])).toBeTruthy();
    expect(screen.queryByText(AR["result.semantic.failed"])).toBeNull();
    rendered.unmount();

    rendered = view(FAMILY_RESULTS.diagnostic, {
      execution: { runId: "run-core-error", status: "error" },
    });
    expect(
      screen.getByRole("heading", { name: AR["result.execution.core_error"] }),
    ).toBeTruthy();
    expect(screen.getByText(AR["result.execution.core_error.body"])).toBeTruthy();
    expect(screen.getByText(AR["result.status.core_error"])).toBeTruthy();
    expect(screen.queryByText(AR["result.semantic.failed"])).toBeNull();
    rendered.unmount();

    view(FAMILY_RESULTS.diagnostic, {
      execution: { runId: "run-failed", status: "failed", exitCode: 1 },
    });
    expect(
      screen.getByRole("heading", { name: AR["result.semantic.failed"] }),
    ).toBeTruthy();
    expect(screen.getByText(AR["result.status.failure"])).toBeTruthy();
  });

  it("لا يسمح لعلامة قص أن تخفي إنهاءً بإشارة خلف WarningPartial", () => {
    view(
      {
        ...FAMILY_RESULTS.diagnostic,
        lines: [{ stream: "truncated", line: { dropped: 4 } }],
      },
      { execution: { runId: "run-signalled", status: "signalled", signal: 9 } },
    );

    expect(
      screen.getByRole("heading", { name: AR["result.execution.signalled"] }),
    ).toBeTruthy();
    expect(screen.queryByText(AR["result.partial.title"])).toBeNull();
  });

  it("يفرّق بين جواب بلا مطابقات ونجاح صامت", () => {
    view({ ...FAMILY_RESULTS.diff_search, semantic: "no_matches", items: [] });
    expect(screen.getByText(AR["result.empty"])).toBeTruthy();
    cleanup();
    view({ ...FAMILY_RESULTS.raw_output, lines: [] });
    expect(screen.getByText(AR["result.silent"])).toBeTruthy();
  });

  it("لا يصف التشخيص الفارغ عند الفشل كنجاح صامت", () => {
    view({ ...FAMILY_RESULTS.diagnostic, lines: [] });
    expect(screen.getByText(AR["result.fallback"])).toBeTruthy();
    expect(screen.queryByText(AR["result.silent"])).toBeNull();
  });

  it("يكشف بيانات التنفيذ الآمنة ويحجب الوسائط دائمًا", async () => {
    const user = userEvent.setup();
    const { container } = view(FAMILY_RESULTS.raw_output, {
      execution: {
        runId: "run-safe-id",
        status: "success",
        exitCode: 0,
        executable: "/usr/bin/find",
        warnings: ["plan.warning.example"],
      },
    });
    await user.click(screen.getByText(AR["result.technical"]));

    expect(screen.getByText(AR["result.technical.hide"])).toBeTruthy();

    expect(container.textContent).toContain("run-safe-id");
    expect(container.textContent).toContain("/usr/bin/find");
    expect(container.textContent).toContain(AR["result.technical.arguments.redacted"]);
    expect(container.textContent).not.toContain("--password");
    expect(container.textContent).toContain(AR["result.technical.redacted"]);
  });

  it("ينسخ الصفوف بمسمّياتها الدلالية المترجمة", async () => {
    const user = userEvent.setup();
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    view({
      ...FAMILY_RESULTS.properties_report,
      properties: [{
        label_key: "result.report.image.row",
        value: "1920 × 1080",
        stream: "stdout",
      }],
    });

    await user.click(screen.getByRole("button", { name: AR["action.copy.result"] }));
    expect(writeText).toHaveBeenCalledWith(`${AR["result.report.image.row"]}: 1920 × 1080`);
  });
});

/**
 * «تم إنشاء الأرشيف» بدل «ملف أو مجلد جاهز».
 *
 * النصّ وحده يتبع العملية؛ والتصنيف يبقى تفصيلًا داخليًا لا يُعرض. والشرط
 * الحاسم أن العنوان لا يُصاغ إلا لنتيجةٍ نجحت: عنوانُ إنجازٍ فوق تشخيصِ فشلٍ
 * كذبٌ صريح، ولا يمسكه شيء إن لم يُختبَر — فكلا الحالتين تعرض القالب نفسه.
 */
describe("عنوان النتيجة بلسان العملية", () => {
  const artifact = FAMILY_RESULTS.artifact as ResultContract;

  it("يستعمل عنوان العملية حين تعلنه", () => {
    view(artifact, { operationId: "compress.folder.zip" });
    expect(screen.getByText(AR["op.compress.folder.zip.result"] as string)).toBeTruthy();
  });

  it("يعود إلى عنوان العائلة حين لا تعلن العملية عنوانًا", () => {
    view(artifact, { operationId: "compress.zip.list" });
    expect(screen.getByText(AR["result.artifact.title"] as string)).toBeTruthy();
  });

  it("لا يعلن إنجازًا فوق نتيجةٍ فاشلة", () => {
    const failed = { ...artifact, semantic: "failed" } as ResultContract;
    view(failed, { operationId: "compress.folder.zip" });
    expect(screen.queryByText(AR["op.compress.folder.zip.result"] as string)).toBeNull();
  });

  it("لا يعلن إنجازًا فوق نتيجةٍ أُلغيت", () => {
    const cancelled = { ...artifact, semantic: "cancelled" } as ResultContract;
    view(cancelled, { operationId: "compress.folder.zip" });
    expect(screen.queryByText(AR["op.compress.folder.zip.result"] as string)).toBeNull();
  });
});

/**
 * حقائق الناتج تُعرض حين تثبتها النواة، وتُطوى حين تغيب.
 *
 * الغياب هو الحالة الحسّاسة: حقلٌ اختياري يُعرض بقيمةٍ افتراضية يقول للمستخدم
 * «صفر بايت» أو «لا عناصر» عن شيءٍ لم يُقَس أصلًا.
 */
describe("حقائق الناتج في ResultView", () => {
  const base = FAMILY_RESULTS.artifact as ResultContract & { type: "artifact" };

  it("يعرض الحجم والعدد والوجهة حين ترسلها النواة", () => {
    view({ ...base, size_bytes: 2048, entries: 18 } as ResultContract);
    expect(screen.getByText(AR["result.artifact.size"] as string)).toBeTruthy();
    expect(screen.getByText(AR["result.artifact.entries"] as string)).toBeTruthy();
    expect(screen.getByText(AR["result.artifact.destination"] as string)).toBeTruthy();
  });

  it("يصوغ الحجم بأكبر وحدةٍ لا يقلّ فيها الرقم عن واحد", () => {
    const { unmount } = view({ ...base, size_bytes: 2048 } as ResultContract);
    expect(screen.getByText(`2 ${AR["unit.kib"]}`)).toBeTruthy();
    unmount();
    view({ ...base, size_bytes: 900 } as ResultContract);
    expect(screen.getByText(`900 ${AR["unit.byte"]}`)).toBeTruthy();
  });

  it("يطوي ما لم تقسه النواة بدل أن يعرض صفرًا", () => {
    // لا `size_bytes` ولا `entries` — كما يصل عن مسارٍ تعذّر قياسه.
    view(base);
    expect(screen.queryByText(AR["result.artifact.size"] as string)).toBeNull();
    expect(screen.queryByText(AR["result.artifact.entries"] as string)).toBeNull();
  });

  it("يعرض صفرًا حقيقيًا حين يكون الصفر هو القياس", () => {
    // ملفٌ فارغ قِيس فعلًا: `0` ليس غيابًا، فيُعرض.
    view({ ...base, size_bytes: 0 } as ResultContract);
    expect(screen.getByText(`0 ${AR["unit.byte"]}`)).toBeTruthy();
  });
});
