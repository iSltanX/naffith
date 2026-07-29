/**
 * طبقة عرض العمليات.
 *
 * ## القاعدة الوحيدة
 *
 * **لا توجد في الواجهة قائمة عمليات.** الفهرس يأتي من `list_operations()`،
 * وهذا الملف لا يضيف إليه ولا يحذف منه ولا يعيد ترتيبه بأسماء مكتوبة هنا. كل
 * ما فيه دوالُّ اشتقاق: أيقونةٌ من الفئة، ونصٌّ من مفتاح، وتوفّرٌ من شكل
 * المدخلات. عمليةٌ تُضاف في النواة غدًا تظهر في القائمة بلا سطرٍ واحد هنا.
 *
 * الاختبار الذي يحرس هذا يحقن عملياتٍ لا وجود لها في هذا البناء ويتحقّق أنها
 * تُعرض كاملةً — فلو عاد أحدٌ يومًا فكتب `if (op.id === '…')` سقط الاختبار.
 *
 * ## لماذا الأيقونة من الفئة لا من المعرّف
 *
 * خريطةٌ من `op_id` إلى أيقونة هي قائمة العمليات نفسها متنكّرة: تنمو مع كل
 * عملية، وتُنسى فتظهر عمليةٌ بلا أيقونة. الفئة مغلقة ومعلَنة في النواة، فالخريطة
 * منها ثابتة الحجم مهما بلغ عدد العمليات.
 */
import type { InputSummary, OperationSummary, RawValue } from './ipc';

/** أنواع المدخلات التي يعرف هذا البناء كيف يرسمها حقلًا. */
const RENDERABLE_INPUT_KINDS = [
  'existing_dir',
  'existing_file',
  'target_dir',
  'new_name',
  'text',
  'flag',
] as const;

/**
 * أيقونة الفئة. الفئات معلَنة في `spec.rs` ومغلقة، فالخريطة كاملة بالبناء.
 *
 * `internal` مذكورة كي يكون النوع شاملًا لا لأنها تُعرض: النواة لا تُدرج
 * العمليات الداخلية في الفهرس في أي بناء.
 */
const CATEGORY_ICON: Record<OperationSummary['category'], string> = {
  files: '#i-file',
  compress: '#i-compress',
  system: '#i-system',
  internal: '#i-admin',
};

export function iconForCategory(category: OperationSummary['category']): string {
  return CATEGORY_ICON[category] ?? '#i-file';
}

/**
 * حالة توفّر العملية في هذا البناء.
 *
 * ليست حالةً في النواة — النواة إمّا تُدرج العملية أو لا. هذه تجيب سؤالًا
 * مختلفًا: «هل تعرف هذه الواجهة كيف ترسم نموذج هذه العملية؟». تطبيقٌ يشحن
 * نواةً أحدث من واجهته يعرض العملية معطّلةً بسببٍ مفهوم، بدل أن يفتح لها شاشةً
 * فارغة أو يسقط.
 */
export type Availability =
  | { state: 'available' }
  | { state: 'unsupported'; unknownKinds: string[] };

export function availabilityOf(op: OperationSummary): Availability {
  const unknown = op.inputs
    .map((i) => String((i as InputSummary & { kind?: unknown }).kind))
    .filter((kind) => !(RENDERABLE_INPUT_KINDS as readonly string[]).includes(kind));

  if (unknown.length > 0) {
    // مجموعة مرتّبة بلا تكرار: الرسالة تُعرض للمستخدم فلا تتكرّر فيها كلمة.
    return { state: 'unsupported', unknownKinds: [...new Set(unknown)].sort() };
  }
  return { state: 'available' };
}

/** بطاقة عملية جاهزة للعرض. كل حقولها مشتقّة، ولا واحد منها مكتوب بيد. */
export interface OperationCard {
  id: string;
  titleKey: string;
  descriptionKey: string;
  category: OperationSummary['category'];
  danger: OperationSummary['danger'];
  icon: string;
  availability: Availability;
}

export function toCard(op: OperationSummary): OperationCard {
  return {
    id: op.id,
    titleKey: op.title_key,
    descriptionKey: op.description_key,
    category: op.category,
    danger: op.danger,
    icon: iconForCategory(op.category),
    availability: availabilityOf(op),
  };
}

/**
 * يحوّل الفهرس إلى بطاقات.
 *
 * الترتيب هو ترتيب النواة كما وصل. فرضُ ترتيبٍ هنا (أبجديًا مثلًا) كان سيجعل
 * الواجهة تقرّر أولويةَ عرضٍ ليست لها، وتنفصل عن نيّة الفهرس حين يُرتَّب يومًا.
 */
export function toCards(ops: OperationSummary[]): OperationCard[] {
  return ops.map(toCard);
}

/** يبحث عن بطاقة بمعرّفها. `undefined` إن لم تعد العملية موجودة في الفهرس. */
export function findCard(cards: OperationCard[], opId: string): OperationCard | undefined {
  return cards.find((c) => c.id === opId);
}

/**
 * مفاتيح نصوص الحقل، بأخصّ ما يوجد.
 *
 * عمليةٌ تريد صياغةً خاصة بحقلها تعلن `field.<op>.<input>.label`؛ وما لم تفعل
 * يُستعمل النصّ العام `field.<input>.label`. هكذا لا يحتاج حقلٌ مألوف (مصدر،
 * وجهة، اسم) إلى نصٍّ مكرّر في كل عملية، ويبقى التخصيص ممكنًا حين يلزم.
 */
export function fieldKeys(opId: string, inputId: string) {
  return {
    label: [`field.${opId}.${inputId}.label`, `field.${inputId}.label`],
    help: [`field.${opId}.${inputId}.help`, `field.${inputId}.help`],
    placeholder: [`field.${opId}.${inputId}.placeholder`, `field.${inputId}.placeholder`],
  };
}

// ── النموذج ────────────────────────────────────────────────────────────

/**
 * قيم النموذج، مفتاحُها معرّف المدخل كما أعلنته النواة.
 *
 * سجلٌّ لا بنية بحقول مسمّاة: بنيةٌ تسمّي `source` و`destination` تكون قائمةَ
 * حقولٍ ثانية بجانب `op.inputs`، وتحتاج تعديلًا يدويًا لكل عملية جديدة. هنا
 * الحقول تُشتقّ من المواصفة، والرايات تُخزَّن نصًّا (`''` | `'1'`) كي يبقى
 * السجلّ متجانسًا ويبقى التحويل في موضع واحد أدناه.
 */
export type FormValues = Record<string, string>;

/** نموذج فارغ لعملية: كل مدخلاتها المعلَنة بقيمة فارغة. */
export function emptyValues(op: OperationSummary): FormValues {
  return Object.fromEntries(op.inputs.map((i) => [i.id, '']));
}

function kindOf(input: InputSummary): string {
  return String((input as InputSummary & { kind?: unknown }).kind);
}

/** هل هذا المدخل مسار؟ يحدّد زرّ الاختيار واتجاه النصّ LTR. */
export function isPathInput(input: InputSummary): boolean {
  const kind = kindOf(input);
  return kind === 'existing_dir' || kind === 'existing_file' || kind === 'target_dir';
}

/** هل يُختار بحوار مجلد؟ الملف القائم لا يُختار بحوار المجلدات. */
export function isDirectoryInput(input: InputSummary): boolean {
  const kind = kindOf(input);
  return kind === 'existing_dir' || kind === 'target_dir';
}

export function isFlagInput(input: InputSummary): boolean {
  return kindOf(input) === 'flag';
}

/** لاحقة تُعرض بجانب حقل الاسم (‏.zip)، أو `null`. تأتي من المواصفة لا من نصّ. */
export function extensionHint(input: InputSummary): string | null {
  if (kindOf(input) !== 'new_name') return null;
  const ext = (input as InputSummary & { ext?: unknown }).ext;
  return typeof ext === 'string' && ext !== '' ? `.${ext.replace(/^\./, '')}` : null;
}

/**
 * هل اكتملت الحقول المطلوبة؟
 *
 * الراية لا تكون «ناقصة» أبدًا: غيابها قيمةٌ صالحة (مطفأة). أما النصّ والمسار
 * فالفراغ فيهما نقص. هذا ما يقرّر ظهور المعاينة وتعطيل «نفِّذ».
 */
export function isComplete(op: OperationSummary, values: FormValues): boolean {
  return op.inputs.every((input) => {
    if (!input.required) return true;
    if (isFlagInput(input)) return true;
    return (values[input.id] ?? '').trim() !== '';
  });
}

/**
 * يحوّل قيم النموذج إلى `RawValue` بحسب نوع كل مدخل.
 *
 * الموضع الوحيد الذي يعرف كيف تعبر قيمة الحدَّ إلى النواة. المسار يُشذَّب من
 * الفراغ المحيط لأن اللصق يجرّه معه؛ والاسم **لا يُشذَّب** لأن الفراغ في طرفه
 * خطأٌ يجب أن تراه النواة وترفضه برسالتها، لا أن تُخفيه الواجهة فيُنشأ ملفٌ
 * باسمٍ غير الذي كُتب.
 */
export function toRawValues(op: OperationSummary, values: FormValues): Record<string, RawValue> {
  const out: Record<string, RawValue> = {};
  for (const input of op.inputs) {
    const raw = values[input.id] ?? '';
    if (isFlagInput(input)) {
      out[input.id] = { kind: 'flag', value: raw === '1' };
    } else if (isPathInput(input)) {
      out[input.id] = { kind: 'path', value: raw.trim() };
    } else {
      out[input.id] = { kind: 'text', value: raw };
    }
  }
  return out;
}

/** هل في النموذج قيمةٌ تُفقد بالمغادرة؟ يغذّي `ExitCost` في `nav.ts`. */
export function isDirty(values: FormValues): boolean {
  return Object.values(values).some((v) => v.trim() !== '');
}
