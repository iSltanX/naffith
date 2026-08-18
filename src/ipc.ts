/**
 * الواجهة المطبوعة لنواة Rust.
 *
 * لاحظ ما لا يوجد هنا: لا دالة تقبل أمرًا أو وسائط أو مسار أداة. أقصى ما
 * تستطيع الواجهة قوله هو «خطّط العملية س بهذه المدخلات» ثم «نفّذ الخطة ذات
 * الرمز ص». بناء `argv` والتحقّق من المسارات يقعان كاملين في النواة.
 *
 * `argv_display` نصوص **للعرض في سَطْر فقط**. لا يوجد أمر IPC يقبلها، فلا
 * سبيل لإعادة إرسالها.
 *
 * و`reveal` يأخذ معرّف تشغيل لا مسارًا: النواة تُخرج المسار من سجلّها هي.
 */
import { Channel, invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';

// ── ما ترسله الواجهة ───────────────────────────────────────────────────

export type RawValue =
  | { kind: 'path'; value: string }
  | { kind: 'text'; value: string }
  | { kind: 'flag'; value: boolean };

export type Danger = 'safe' | 'creates' | 'modifies' | 'destructive';

/** خيار واحد في قائمة مغلقة. القيمة تدخل الأمر، والمفتاح يُترجَم. */
export interface ChoiceOption {
  value: string;
  label_key: string;
}

export type InputKind =
  | { kind: 'existing_dir' }
  | { kind: 'existing_file' }
  | { kind: 'existing_path' }
  | { kind: 'target_dir' }
  | { kind: 'new_name'; ext: string | null }
  | { kind: 'new_dir_name' }
  | { kind: 'text'; max_len: number }
  | { kind: 'choice'; options: ChoiceOption[] }
  | { kind: 'number'; min: number; max: number; default: number }
  | { kind: 'url' }
  | { kind: 'flag' };

/** مدخلٌ مسمّى كما يسلسله Rust: اتحادٌ موسوم لا سجلٌ مفتوح. */
export type InputSummary = InputKind & {
  id: string;
  required: boolean;
};

/**
 * معرّفات الأقسام كما تعلنها النواة.
 *
 * مغلقة عمدًا: خريطة الأيقونات وحالات العرض تُبنى منها، و`tsc` يمسك القسم
 * الذي يُضاف في Rust ولا يُذكر هنا — وهو بالضبط الشكل الذي يتقادم صامتًا.
 */
export type CategoryId =
  | 'files'
  | 'compress'
  | 'images'
  | 'text'
  | 'disk'
  | 'network'
  | 'security'
  | 'git'
  | 'system'
  | 'developer'
  | 'history'
  | 'internal';

/**
 * حالة إتاحة العملية على **هذا الجهاز**.
 *
 * ليست حالةً في الفهرس — الفهرس يُدرج العملية دائمًا. هذه تجيب: «هل أداة هذه
 * العملية موجودة هنا؟». `git` تأتي مع أدوات Xcode وقد تغيب.
 */
export type CoreAvailability =
  | { state: 'available' }
  | { state: 'tool_missing'; tool: string };

export interface OperationSummary {
  id: string;
  title_key: string;
  description_key: string;
  category: CategoryId;
  danger: Danger;
  conflict: Conflict;
  /** معرّف الأداة، كي يجدها البحث ويسمّيها سببُ التعطيل. */
  tool: string;
  availability: CoreAvailability;
  sort_order: number;
  search_terms: string[];
  inputs: InputSummary[];
}

/** من أين يأتي محتوى القسم. */
export type CategoryKind = 'operations' | 'journal' | 'hidden';

/**
 * قسمٌ كما تعلنه النواة، بعدديه محسوبين من الفهرس لا مكتوبين بيد.
 *
 * عددان لا واحد: قسمٌ فيه ستّ عمليات تعمل منها أربع يقول ذلك صراحةً بدل أن
 * يَعِد بستّ ثم يعرض اثنتين معطّلتين في الشاشة التي بعده.
 */
export interface CategorySummary {
  id: CategoryId;
  title_key: string;
  description_key: string;
  /** معرّفٌ في لوحة الرموز، جاء من النواة. */
  icon: string;
  sort_order: number;
  kind: CategoryKind;
  operation_count: number;
  available_count: number;
}

// ── ما تستقبله ─────────────────────────────────────────────────────────

export type TokenRole = 'tool' | 'flag' | 'path' | 'value';

export interface ExplainToken {
  token: string;
  /** مفتاح في `i18n.ts`، أو null إن كان الرمز بيانات لا راية. */
  key: string | null;
  role: TokenRole;
}

/** ما تفعله العملية إن كان الاسم النهائي مشغولًا. تعلنها النواة. */
export type Conflict = 'refuse' | 'no_artifact';

export interface ToolView {
  id: string;
  /** المسار المطلق المُتحقَّق منه لحظة التخطيط — لا ما في PATH. */
  path: string;
}

/**
 * تقدير لا قياس. أسماء الحقول تحمل هذا المعنى عمدًا: لا يوجد حقل اسمه
 * `size` كي لا يُعرض الرقم كأنه حجم الأرشيف الناتج.
 */
export interface EstimateView {
  /** مجموع أحجام ملفات المصدر **قبل** الضغط. الناتج أصغر عادةً. */
  approx_source_bytes: number;
  scanned_entries: number;
  /** false يعني أن المسح توقّف عند حدّ: الرقمان حدّ أدنى لا مجموع. */
  complete: boolean;
}

export interface PlanResponse {
  token: string;
  /** معرّف عام يربط قيود السجل. ليس قدرة. */
  plan_id: string;
  op_id: string;
  title_key: string;
  description_key: string;
  /** القسم الذي تنتمي إليه العملية. يُقيَّد في السجل ويُصفّى به لاحقًا. */
  category: CategoryId;
  danger: Danger;
  /** للعرض فقط. لا يمكن إعادة إرساله. */
  argv_display: string[];
  explain: ExplainToken[];
  warnings: string[];
  /** الأداة التي ستُنفِّذ. */
  tool: ToolView;
  /** السياسة المعلَنة عند تضارب الاسم. تُعرض قبل التنفيذ. */
  conflict: Conflict;
  /** حجم تقديري للمصدر، أو null إن كانت العملية لا تمسح شجرة. */
  estimate: EstimateView | null;
  /** الاسم النهائي المتوقّع. */
  produces: string | null;
  /** المسار المؤقّت الذي يكتب إليه الأمر فعلًا (الوسيط الأخير). */
  writes_to: string | null;
  working_directory: string | null;
}

export type Outcome =
  | { status: 'success'; produced: string | null }
  | { status: 'failed'; code: number | null }
  | { status: 'signalled'; signal: number | null }
  | { status: 'cancelled' }
  | { status: 'error'; key: string };

/** Stable ResultView family selected by Rust for every production operation. */
export type ResultCategory =
  | 'artifact'
  | 'acknowledgement'
  | 'collection'
  | 'properties_report'
  | 'metrics'
  | 'digest'
  | 'comparison'
  | 'verdict'
  | 'diff_search'
  | 'diagnostic'
  | 'raw_output';

/** Domain answer classified from the operation and exit status in Rust. */
export type ResultSemantic =
  | 'completed'
  | 'matches'
  | 'no_matches'
  | 'differences'
  | 'no_differences'
  | 'accepted'
  | 'rejected'
  | 'signed'
  | 'unsigned'
  | 'failed'
  | 'cancelled';

export type RevealKind = 'file' | 'directory';

/** A bounded, already-classified raw line; the frontend never parses its text. */
export type RawOutputLine =
  | { stream: 'stdout'; line: string }
  | { stream: 'stderr'; line: string }
  | { stream: 'omitted'; line: { dropped: number } }
  | { stream: 'truncated'; line: { dropped: number } };

export type ResultStream = 'stdout' | 'stderr';

/** A line whose stream has already been separated from terminal notices. */
export interface StructuredLine {
  value: string;
  stream: ResultStream;
}

/** Why a structured payload contains only a bounded subset of its source. */
export interface OutputNotice {
  kind: 'omitted' | 'truncated';
  dropped: number;
}

export type CollectionKind =
  | 'file_matches'
  | 'directory_sizes'
  | 'archive_entries'
  | 'filesystem_usage'
  | 'storage_devices'
  | 'dns_records'
  | 'listening_ports'
  | 'git_status'
  | 'merged_branches'
  | 'processes'
  | 'git_log'
  | 'git_blame'
  | 'file_content'
  | 'directory_entries'
  | 'process_matches';

export type ReportKind =
  | 'image'
  | 'http_headers'
  | 'permissions'
  | 'extended_attributes'
  | 'system_version'
  | 'system_profile'
  | 'git_version'
  | 'file_type'
  | 'architecture';

export type MetricsKind = 'network_latency' | 'system_uptime';
export type ComparisonKind = 'sha256' | 'git_diff' | 'bytes';
export type VerdictKind = 'archive_integrity' | 'gatekeeper' | 'code_signature' | 'code_integrity';
export type DiffSearchKind = 'diff' | 'search';

export interface CollectionRow {
  cells: string[];
  stream: ResultStream;
}

export interface ResultProperty {
  label_key: string;
  value: string;
  stream: ResultStream;
}

export interface ResultMetric {
  label_key: string;
  value: string;
  unit?: string;
  stream: ResultStream;
}

/**
 * Exact flattened `ResultContract` wire shape.
 *
 * The repeated common fields keep this a useful discriminated union: checking
 * `type` proves whether `path`/`output` or `lines` is present.
 */
export type ResultContract =
  | {
      category: 'artifact';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'artifact';
      path: string;
      /** اسم الناتج، مشتقٌّ من المسار في Rust. */
      name: string;
      /** المجلد الحاوي، مشتقٌّ من المسار في Rust. */
      parent: string;
      /** الحجم بالبايت حين تقيسه النواة؛ يغيب حين يتعذّر. */
      size_bytes?: number;
      /** عدد ما يحويه مجلدُ الناتج مباشرةً؛ يغيب للملف. */
      entries?: number;
      output?: RawOutputLine[];
    }
  | {
      category: 'acknowledgement';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'acknowledgement';
      message_key: string;
      details: StructuredLine[];
      notices: OutputNotice[];
    }
  | {
      category: 'collection';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'collection';
      kind: CollectionKind;
      columns: string[];
      rows: CollectionRow[];
      notices: OutputNotice[];
    }
  | {
      category: 'properties_report';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'properties_report';
      kind: ReportKind;
      properties: ResultProperty[];
      notices: OutputNotice[];
    }
  | {
      category: 'metrics';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'metrics';
      kind: MetricsKind;
      metrics: ResultMetric[];
      notices: OutputNotice[];
    }
  | {
      category: 'digest';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'digest';
      algorithm: 'sha256';
      value: string | null;
      details: StructuredLine[];
      notices: OutputNotice[];
    }
  | {
      category: 'comparison';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'comparison';
      kind: ComparisonKind;
      reference: string | null;
      comparison: string | null;
      equal: boolean | null;
      details: StructuredLine[];
      notices: OutputNotice[];
    }
  | {
      category: 'verdict';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'verdict';
      kind: VerdictKind;
      value: ResultSemantic;
      details: StructuredLine[];
      notices: OutputNotice[];
    }
  | {
      category: 'diff_search';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'diff_search';
      kind: DiffSearchKind;
      items: StructuredLine[];
      notices: OutputNotice[];
    }
  | {
      category: 'diagnostic';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'diagnostic';
      lines: RawOutputLine[];
    }
  | {
      category: 'raw_output';
      semantic: ResultSemantic;
      reveal?: RevealKind;
      type: 'raw_output';
      lines: RawOutputLine[];
    };

/**
 * سطر خرج واحد كما يبثّه `run://output`.
 *
 * ثلاث صيغ لا صيغتان. `truncated` تُبَثّ مرّة واحدة حين تبلغ النواة سقف
 * الأسطر المعلَن، ومعناها «ما تراه ليس كل الخرج» — وهي الحالة الوحيدة التي
 * لا تحمل فيها `line` نصًّا يُعرض بل عدد ما سقط. اتحادٌ موسوم لا واجهة
 * واحدة، لأن `stream` هو ما يقرّر نوع `line`: بواجهة واحدة كان `line: string`
 * كذبًا يمرّ من `tsc`، فتُطبع حمولة القصّ في الشاشة كأنها سطر من الأداة.
 */
export type RunOutputEvent =
  | { run_id: string; stream: 'stdout'; line: string }
  | { run_id: string; stream: 'stderr'; line: string }
  | { run_id: string; stream: 'omitted'; line: { dropped: number } }
  | { run_id: string; stream: 'truncated'; line: { dropped: number } };

export interface RunFinishedEvent extends Record<string, unknown> {
  run_id: string;
  result: ResultContract;
  status: Outcome['status'];
  produced?: string | null;
  code?: number | null;
  /**
   * رقم الإشارة التي أنهت الأداة، مع `status: 'signalled'` وحدها.
   *
   * الحدث هو ما يصل المستمع فعلًا، لا `Outcome` المجرّد، فحقلٌ يعلنه ذاك
   * ويغفله هذا لا سبيل لقراءته: شاشةٌ تريد التفريق بين خروجٍ بغير صفر
   * وإشارةٍ قاتلة كانت تقرأ `unknown` من الشكل المفتوح بلا نوع يحرسها.
   */
  signal?: number | null;
  key?: string;
}

export type JournalState = 'planned' | 'running' | 'succeeded' | 'failed' | 'cancelled';

/**
 * مدخلٌ واحد كما قُيِّد، لإعادة ملء النموذج.
 *
 * `value: null` تعني «حقلٌ كان، وقيمته لا تُكتب» لا «حقلٌ فارغ». زرّ «أعد
 * بهذه القيم» يترك مثله للمستخدم بدل أن يملأه بفراغٍ ويبدو كأنه استعاد كل شيء.
 */
export interface JournalInput {
  id: string;
  value: string | null;
}

export interface JournalEntry {
  id: string;
  op_id: string;
  /** القسم وقت التشغيل. يغيب عن القيود القديمة، فهو اختياري. */
  category?: CategoryId | null;
  at: number;
  /**
   * زمن التنفيذ، في القيد النهائي وحده.
   *
   * ‏`skip_serializing_if` في النواة يعني أن المفتاح **يغيب** عن السلك حين لا
   * قيمة له، لا أنه يصل `null`. فالاختيارية هنا ليست تجميلًا: بدونها يَعِد
   * ‏`tsc` بمفتاحٍ حاضرٍ دومًا، وتصير `=== null` مقارنةً لا تصدق أبدًا، ويمرّ
   * قيد `planned` — الذي لا يحمل الحقل أصلًا — على أنه يحمله.
   */
  duration_ms?: number | null;
  program: string;
  args: string[];
  /** مجلد العمل. يغيب عن السلك عند الغياب كما يغيب `duration_ms`. */
  cwd?: string | null;
  state: JournalState;
  produced?: string | null;
  reason?: string;
  code?: number | null;
  /** المدخلات بعد تنقيح السرّي. تغيب حين لا مدخلات لها. */
  inputs?: JournalInput[];
  /** آخر ما طبعته الأداة، محدودًا في النواة. في القيد النهائي وحده. */
  tail?: string[];
  /** النتيجة المهيكلة. تغيب عن القيود القديمة وعن planned/running. */
  result?: ResultContract;
}

/** خطأ من النواة: مفتاح ترجمة، والحقل المسؤول، وتفصيل اختياري. */
export interface CoreErrorShape {
  key: string;
  input: string | null;
  detail: unknown;
}

export function asCoreError(e: unknown): CoreErrorShape {
  if (typeof e === 'object' && e !== null && 'key' in e) {
    const shape = e as Partial<CoreErrorShape>;
    return {
      key: String(shape.key),
      input: typeof shape.input === 'string' ? shape.input : null,
      detail: shape.detail ?? null,
    };
  }
  return { key: 'err.unknown', input: null, detail: null };
}

// ── الأوامر ────────────────────────────────────────────────────────────

export const listOperations = () => invoke<OperationSummary[]>('list_operations');

/** الأقسام وأعدادها، محسوبةً في النواة من الفهرس نفسه. */
export const listCategories = () => invoke<CategorySummary[]>('list_categories');

export const plan = (opId: string, inputs: Record<string, RawValue>) =>
  invoke<PlanResponse>('plan', { opId, inputs });

/** لا يقبل إلا رمزًا صادرًا عن النواة. يعيد معرّف التشغيل ويبثّ الباقي. */
export const execute = (token: string) => invoke<string>('execute', { token });

export const cancel = (runId: string) => invoke<void>('cancel', { runId });

export const recentRuns = () => invoke<JournalEntry[]>('recent_runs');

/** يحذف كل قيود تشغيلٍ واحد. لا يمسّ ما أنتجه ذلك التشغيل على القرص. */
export const journalDelete = (runId: string) => invoke<void>('journal_delete', { runId });

/** يمسح السجلّ كله. الشاشة تسأل قبله. */
export const journalClear = () => invoke<void>('journal_clear');

/** يُظهر ناتج تشغيلٍ ناجح في Finder. لا يقبل مسارًا. */
export const reveal = (runId: string) => invoke<void>('reveal', { runId });

// ── الأحداث ────────────────────────────────────────────────────────────

export const onRunOutput = (fn: (e: RunOutputEvent) => void): Promise<UnlistenFn> =>
  listen<RunOutputEvent>('run://output', (e) => fn(e.payload));

export const onRunFinished = (fn: (e: RunFinishedEvent) => void): Promise<UnlistenFn> =>
  listen<RunFinishedEvent>('run://finished', (e) => fn(e.payload));

// ── اختيار مجلد ────────────────────────────────────────────────────────

/**
 * يفتح حوار النظام لاختيار مجلد، ويعيد مساره نصًّا.
 *
 * النص العائد **غير موثوق** تمامًا كالنص المكتوب يدويًا: يعبر الحدّ داخل
 * `RawValue::Path` ويمرّ بـ `paths.rs` كاملًا. الحوار راحةٌ للمستخدم لا
 * مصدرُ ثقة، ولذلك يبقى الحقل قابلًا للكتابة واللصق بلوحة المفاتيح وحدها.
 */
export async function pickDirectory(): Promise<string | null> {
  const chosen = await openDialog({ directory: true, multiple: false });
  return typeof chosen === 'string' ? chosen : null;
}

/**
 * يفتح حوار macOS لاختيار ملف قائم.
 *
 * مثل اختيار المجلد تمامًا، هذه راحة إدخال فقط: المسار لا يكتسب ثقةً من
 * الحوار، ويُعاد التحقّق منه في Rust وفق نوع الحقل وسياسة الجذور المحمية.
 */
export async function pickFile(): Promise<string | null> {
  const chosen = await openDialog({ directory: false, multiple: false });
  return typeof chosen === 'string' ? chosen : null;
}

// ── التحديث ────────────────────────────────────────────────────────────
//
// أوامر إضافة التحديث ليست من أوامر هذا التطبيق: هي أوامر مُلحقةٍ تُسجَّل
// باسمٍ مُنَطَّق (`plugin:updater|…`) خارج `generate_handler!` في `lib.rs`.
// ولذلك يبقى سطح أوامر التطبيق تسعةً كما يثبته `security.rs` — الإضافة توسّع
// الصلاحيات (‏`updater:default` في ملف القدرات) لا قائمةَ الأوامر.

/** رقم إصدار التطبيق، محقونًا وقت البناء من `package.json`. */
export const APP_VERSION: string = __APP_VERSION__;

/**
 * هل ضُبطت وجهة التحديث ومفتاح التوقيع في هذا البناء؟
 *
 * تُقرأ من `tauri.conf.json` وقت البناء. حين تكون `false` لا تُسأل النواة
 * أصلًا: الجواب معروفٌ سلفًا، وسؤالٌ يُعرف فشله قبل طرحه يُنتج انتظارًا ثم
 * رسالةَ خطأ عن شيءٍ لم ينكسر.
 */
export const UPDATER_CONFIGURED: boolean = __UPDATER_CONFIGURED__;

/** ما تعيده إضافة التحديث حين يوجد تحديث. `null` يعني «لا جديد». */
export interface UpdateInfo {
  /** معرّف المورد في النواة. يُمرَّر إلى التنزيل، ولا معنى له خارج الجلسة. */
  rid: number;
  currentVersion: string;
  version: string;
  date?: string | null;
  body?: string | null;
}

/**
 * يسأل عن وجود تحديث. يرمي إن تعذّر السؤال.
 *
 * **الرمي حالةٌ متوقّعة لا خلل**: ما دامت `endpoints` فارغة في
 * `tauri.conf.json` — وهي كذلك حتى تُضبط نقطة التحديث الحقيقية ومفتاح
 * التوقيع — فستُعيد النواة `Updater does not have any endpoints set`. الشاشة
 * تعرض ذلك بوصفه حالة «تعذّر التحقّق» المرسومة في التصميم، ولا تدّعي أن
 * النسخة محدَّثة. أن يقول المنتج «لا أعرف» أصدقُ من أن يقول «أنت على أحدث
 * إصدار» وهو لم يسأل أحدًا.
 */
export const checkForUpdate = () => invoke<UpdateInfo | null>('plugin:updater|check');

/**
 * ينزّل التحديث ويثبّته. لا يُستدعى إلا بعد `checkForUpdate` ناجحة.
 *
 * `onEvent` قناةٌ تطلبها النواة ولو لم نقرأها: توقيع الأمر يشترطها، وتمريرُ
 * قناةٍ صامتة أصدق من ادّعاء تقدّمٍ لا نعرضه.
 */
export async function downloadAndInstallUpdate(rid: number): Promise<void> {
  const onEvent = new Channel<unknown>();
  await invoke<void>('plugin:updater|download_and_install', { rid, onEvent });
}
