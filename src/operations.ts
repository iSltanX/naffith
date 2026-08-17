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
import type { ChoiceOption, InputSummary, OperationSummary, RawValue } from './ipc';


/**
 * ## أين ذهبت البطاقات والأيقونات
 *
 * `toCards` و`iconForCategory` و`availabilityOf` كانت هنا، وانتقلت إلى
 * `library.ts` حين صار للأقسام كيانٌ في النواة. السبب أن أيقونة العملية صارت
 * **أيقونة قسمها كما تعلنها النواة** لا خريطةً في الواجهة، والاشتقاق صار
 * يحتاج قائمة الأقسام — وهي ليست من شأن هذا الملف. ما بقي هنا هو ما يخصّ
 * النموذج وحده: قراءة أنواع المدخلات، وتحويل القيم إلى `RawValue`.
 */

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

/** نموذج ابتدائي لعملية: الأرقام تبدأ بالقيمة الافتراضية التي أعلنتها النواة. */
export function emptyValues(op: OperationSummary): FormValues {
  return Object.fromEntries(
    op.inputs.map((input) => {
      const number = numberSpec(input);
      return [input.id, number ? String(number.default) : ''];
    }),
  );
}

/** النوع المسطّح كما يسلسله Rust بجانب `id` و`required`. */
export function inputKind(input: InputSummary): string {
  return String((input as InputSummary & { kind?: unknown }).kind);
}

/** هل هذا المدخل مسار؟ يحدّد زرّ الاختيار واتجاه النصّ LTR. */
export function isPathInput(input: InputSummary): boolean {
  const kind = inputKind(input);
  return (
    kind === 'existing_dir' ||
    kind === 'existing_file' ||
    kind === 'existing_path' ||
    kind === 'target_dir'
  );
}

/** هل يُختار بحوار مجلد؟ الملف القائم لا يُختار بحوار المجلدات. */
export function isDirectoryInput(input: InputSummary): boolean {
  const kind = inputKind(input);
  return kind === 'existing_dir' || kind === 'target_dir';
}

export function isFlagInput(input: InputSummary): boolean {
  return inputKind(input) === 'flag';
}

export function isChoiceInput(input: InputSummary): boolean {
  return inputKind(input) === 'choice';
}

export function choiceOptions(input: InputSummary): ChoiceOption[] {
  if (!isChoiceInput(input)) return [];
  const options = (input as InputSummary & { options?: unknown }).options;
  if (!Array.isArray(options)) return [];
  return options.filter(
    (option): option is ChoiceOption =>
      typeof option === 'object' &&
      option !== null &&
      typeof (option as Partial<ChoiceOption>).value === 'string' &&
      typeof (option as Partial<ChoiceOption>).label_key === 'string',
  );
}

export interface NumberSpec {
  min: number;
  max: number;
  default: number;
}

export function isNumberInput(input: InputSummary): boolean {
  return inputKind(input) === 'number';
}

export function numberSpec(input: InputSummary): NumberSpec | null {
  if (!isNumberInput(input)) return null;
  const candidate = input as InputSummary & Partial<NumberSpec>;
  if (
    typeof candidate.min !== 'number' ||
    typeof candidate.max !== 'number' ||
    typeof candidate.default !== 'number'
  ) {
    return null;
  }
  return { min: candidate.min, max: candidate.max, default: candidate.default };
}

export function isUrlInput(input: InputSummary): boolean {
  return inputKind(input) === 'url';
}

/** لاحقة تُعرض بجانب حقل الاسم (‏.zip)، أو `null`. تأتي من المواصفة لا من نصّ. */
export function extensionHint(input: InputSummary): string | null {
  if (inputKind(input) !== 'new_name') return null;
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
