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
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';

// ── ما ترسله الواجهة ───────────────────────────────────────────────────

export type RawValue =
  | { kind: 'path'; value: string }
  | { kind: 'text'; value: string }
  | { kind: 'flag'; value: boolean };

export type Danger = 'safe' | 'creates' | 'modifies' | 'destructive';

export type InputKind =
  | { kind: 'existing_dir' }
  | { kind: 'existing_file' }
  | { kind: 'target_dir' }
  | { kind: 'new_name'; ext: string | null }
  | { kind: 'text'; max_len: number }
  | { kind: 'flag' };

export interface InputSummary extends Record<string, unknown> {
  id: string;
  required: boolean;
}

export interface OperationSummary {
  id: string;
  title_key: string;
  description_key: string;
  category: 'files' | 'compress' | 'system' | 'internal';
  danger: Danger;
  inputs: InputSummary[];
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
  | { run_id: string; stream: 'truncated'; line: { dropped: number } };

export interface RunFinishedEvent extends Record<string, unknown> {
  run_id: string;
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

export interface JournalEntry {
  id: string;
  op_id: string;
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

export const plan = (opId: string, inputs: Record<string, RawValue>) =>
  invoke<PlanResponse>('plan', { opId, inputs });

/** لا يقبل إلا رمزًا صادرًا عن النواة. يعيد معرّف التشغيل ويبثّ الباقي. */
export const execute = (token: string) => invoke<string>('execute', { token });

export const cancel = (runId: string) => invoke<void>('cancel', { runId });

export const recentRuns = () => invoke<JournalEntry[]>('recent_runs');

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
