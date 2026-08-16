/**
 * مكتبة العمليات — الاشتقاق والبحث.
 *
 * ## القاعدة نفسها التي تحكم `operations.ts`
 *
 * **لا توجد في هذا الملف قائمة أقسام ولا قائمة عمليات ولا عددٌ مكتوب.** كل ما
 * فيه دوالُّ اشتقاقٍ نقيّة تعمل على ما وصل من النواة: بطاقةٌ من ملخّص، وعددٌ
 * من طول مصفوفة، وترتيبٌ من `sort_order` المعلَن. قسمٌ يُضاف في Rust غدًا يظهر
 * على الشاشة بلا سطرٍ واحد هنا.
 *
 * وهو ملفٌّ بلا React عمدًا: البحث والتصفية والترتيب قرارات تُختبر بلا رسم،
 * ووضعُها في مكوّن كان سيجعل اختبارها يمرّ عبر DOM بلا سبب.
 *
 * ## البحث العربي ليس `includes`
 *
 * «الملفات» و«الملفّات» و«ملفات» ثلاث كتاباتٍ لشيء واحد، ومن يكتب إحداها
 * يتوقّع أن يجد الأخريات. المطابقة الحرفية كانت تعني أن البحث يعمل حين يكتب
 * المستخدم النصّ **كما كُتب في القاموس** حرفًا بحرف، وهو ما لا يفعله أحد.
 * `fold` أدناه توحّد الهمزات والألفات والتاء المربوطة والياء، وتُسقط التشكيل
 * والتطويل، وتحوّل الأرقام العربية-الهندية إلى نظيراتها.
 */
import type {
  CategoryId,
  CategorySummary,
  CoreAvailability,
  InputSummary,
  JournalEntry,
  OperationSummary,
} from './ipc';
import { t } from './i18n';

/** أنواع المدخلات التي يعرف هذا البناء كيف يرسمها حقلًا. */
export const RENDERABLE_INPUT_KINDS = [
  'existing_dir',
  'existing_file',
  'existing_path',
  'target_dir',
  'new_name',
  'new_dir_name',
  'text',
  'choice',
  'number',
  'url',
  'flag',
] as const;

/**
 * حالة توفّر العملية كما تراها الشاشة.
 *
 * مصدران لا واحد، وهما يجيبان عن سؤالين مختلفين:
 *
 * - `tool_missing` تأتي من **النواة**: أداة العملية غير موجودة على هذا الجهاز.
 * - `unsupported` تُشتقّ **هنا**: العملية تعلن مدخلًا لا تعرف هذه النسخة من
 *   الواجهة كيف ترسمه. تطبيقٌ يشحن نواةً أحدث من واجهته يعرض العملية معطّلةً
 *   بسببٍ مفهوم بدل أن يفتح لها شاشةً فارغة.
 *
 * وترتيب الأولوية مقصود: غيابُ الأداة يسبق، لأنه السبب الذي يستطيع المستخدم
 * أن يفعل شيئًا حياله.
 */
export type Availability =
  | { state: 'available' }
  | { state: 'tool_missing'; tool: string }
  | { state: 'unsupported'; unknownKinds: string[] };

function kindOf(input: InputSummary): string {
  return String((input as InputSummary & { kind?: unknown }).kind);
}

export function availabilityOf(op: OperationSummary): Availability {
  const core: CoreAvailability = op.availability ?? { state: 'available' };
  if (core.state === 'tool_missing') {
    return { state: 'tool_missing', tool: core.tool };
  }

  const unknown = op.inputs
    .map(kindOf)
    .filter((kind) => !(RENDERABLE_INPUT_KINDS as readonly string[]).includes(kind));

  if (unknown.length > 0) {
    // مجموعة مرتّبة بلا تكرار: الرسالة تُعرض فلا تتكرّر فيها كلمة.
    return { state: 'unsupported', unknownKinds: [...new Set(unknown)].sort() };
  }
  return { state: 'available' };
}

export function isAvailable(a: Availability): boolean {
  return a.state === 'available';
}

// ── البطاقات ───────────────────────────────────────────────────────────

/** بطاقة عملية جاهزة للعرض. كل حقولها مشتقّة، ولا واحد منها مكتوب بيد. */
export interface OperationCard {
  id: string;
  titleKey: string;
  descriptionKey: string;
  category: CategoryId;
  danger: OperationSummary['danger'];
  conflict: OperationSummary['conflict'];
  tool: string;
  icon: string;
  sortOrder: number;
  searchTerms: string[];
  availability: Availability;
}

/** بطاقة قسم. العددان يأتيان محسوبين من النواة. */
export interface CategoryCard {
  id: CategoryId;
  titleKey: string;
  descriptionKey: string;
  icon: string;
  kind: CategorySummary['kind'];
  sortOrder: number;
  operationCount: number;
  availableCount: number;
}

/**
 * أيقونة العملية: أيقونة قسمها.
 *
 * خريطةٌ من `op_id` إلى أيقونة كانت ستكون قائمة العمليات نفسها متنكّرة: تنمو
 * مع كل عملية، وتُنسى فتظهر عمليةٌ بلا أيقونة. والقسم يعلن أيقونته في النواة،
 * فالخريطة منه ثابتة الحجم مهما بلغ عدد العمليات.
 */
export function iconForCategory(id: CategoryId, categories: CategoryCard[]): string {
  return categories.find((c) => c.id === id)?.icon ?? '#i-file';
}

export function toOperationCard(op: OperationSummary, categories: CategoryCard[]): OperationCard {
  return {
    id: op.id,
    titleKey: op.title_key,
    descriptionKey: op.description_key,
    category: op.category,
    danger: op.danger,
    conflict: op.conflict,
    tool: op.tool,
    icon: iconForCategory(op.category, categories),
    sortOrder: op.sort_order ?? 0,
    searchTerms: op.search_terms ?? [],
    availability: availabilityOf(op),
  };
}

export function toCategoryCard(c: CategorySummary): CategoryCard {
  return {
    id: c.id,
    titleKey: c.title_key,
    descriptionKey: c.description_key,
    icon: c.icon,
    kind: c.kind,
    sortOrder: c.sort_order,
    operationCount: c.operation_count,
    availableCount: c.available_count,
  };
}

/**
 * عمليات قسمٍ بعينه، بترتيب النواة كما وصل.
 *
 * لا يُعاد الترتيب هنا: النواة رتّبت بـ(ترتيب القسم، ترتيب العملية، المعرّف)،
 * وفرضُ ترتيبٍ ثانٍ في الواجهة كان سيجعلها تقرّر أولويةَ عرضٍ ليست لها.
 */
export function operationsIn(cards: OperationCard[], category: CategoryId): OperationCard[] {
  return cards.filter((c) => c.category === category);
}

export function findCategory(cards: CategoryCard[], id: string): CategoryCard | undefined {
  return cards.find((c) => c.id === id);
}

export function findOperation(cards: OperationCard[], id: string): OperationCard | undefined {
  return cards.find((c) => c.id === id);
}

// ── البحث ──────────────────────────────────────────────────────────────

/**
 * يوحّد نصًّا عربيًا قبل المقارنة.
 *
 * كل تحويلٍ هنا يعالج خطأً وقع فعلًا في بحثٍ نصّي مباشر:
 *
 * * **التشكيل والتطويل** (`ً-ْ`، `ـ`) يُسقطان: من يكتب «نفّذ»
 *   بالشدّة لا يجد «نفذ»، والعكس.
 * * **الألفات** (`أ إ آ ٱ`) تصير `ا`: «الأرشيف» و«الارشيف» كلمة واحدة عمليًا،
 *   ولا أحد يميّز بينهما وهو يكتب بسرعة.
 * * **التاء المربوطة** `ة` تصير `ه`، و**الياء** `ى` تصير `ي`: الفرق بينهما
 *   إملائيّ لا دلاليّ، وكثيرٌ من لوحات المفاتيح تُنتج أحدهما دون الآخر.
 * * **الهمزات المتوسّطة** (`ؤ ئ`) تُردّ إلى حرفها.
 * * **الأرقام العربية-الهندية** (`٠-٩`) تصير لاتينية: النصوص تعرض «‏٢٥٦»
 *   والمستخدم يكتب «256» أو العكس.
 *
 * ولا يُحوَّل شيءٌ في الاتجاه المعاكس: هذه دالّة مقارنة لا دالّة عرض، وما
 * يُعرض على الشاشة يبقى كما كُتب.
 */
export function fold(text: string): string {
  return text
    .toLowerCase()
    // بخاصّية يونيكود لا بمدًى مكتوب: `\p{Mn}` هي «علامة غير متباعدة»، وهي
    // تعريف التشكيل العربي نفسه لا تقريبٌ له — فتشمل ما لم يخطر ببال من كتب
    // المدى، ولا تشمل حرفًا بالخطأ. والمدى المكتوب كان يُقرأ وحداتِ ترميز
    // فيحذّر المدقّق من أن علامةً قد تلتصق بما قبلها.
    .replace(/\p{Mn}/gu, '')
    // والتطويل ليس علامة بل حرفُ مدٍّ زخرفيّ، فيُحذف على حدة.
    .replace(/\u0640/gu, '')
    .replace(/[آأإٱ]/g, 'ا')
    .replace(/ة/g, 'ه')
    .replace(/ى/g, 'ي')
    .replace(/ؤ/g, 'و')
    .replace(/ئ/g, 'ي')
    .replace(/[٠-٩]/g, (d) => String(d.charCodeAt(0) - 0x0660))
    .replace(/[۰-۹]/g, (d) => String(d.charCodeAt(0) - 0x06f0))
    .replace(/\s+/g, ' ')
    .trim();
}

/** كل ما يُبحث فيه داخل عملية، مطويًّا مرّةً واحدة. */
function haystackForOperation(card: OperationCard, categories: CategoryCard[]): string {
  const category = findCategory(categories, card.category);
  return fold(
    [
      t(card.titleKey),
      t(card.descriptionKey),
      card.tool,
      card.id,
      card.searchTerms.join(' '),
      category ? t(category.titleKey) : '',
    ].join(' '),
  );
}

function haystackForCategory(card: CategoryCard): string {
  return fold([t(card.titleKey), t(card.descriptionKey), card.id].join(' '));
}

export interface SearchResults {
  /** هل كان هناك استعلام أصلًا؟ الشاشة تعرض المكتبة كاملةً حين لا يوجد. */
  active: boolean;
  categories: CategoryCard[];
  operations: OperationCard[];
}

/**
 * يبحث في المكتبة كلها: الأقسام والعمليات والأدوات والأوصاف.
 *
 * ## كل كلمة يجب أن تُوجد، لا إحداها
 *
 * «ضغط مجلد» يجب أن يجد عملية الضغط لا كل ما فيه «مجلد». المطابقة على تقاطع
 * الكلمات (AND) لا اتحادها، لأن إضافة كلمةٍ في مربّع البحث فعلُ **تضييق** في
 * ذهن من يكتبها — ونتيجةٌ تتّسع كلما كتب أكثر تجعل المربّع يبدو معطّلًا.
 *
 * ## وغير المتاح يظهر في النتائج
 *
 * البحث لا يصفّي بالإتاحة. من يبحث عن «git» وأدوات Xcode غير مثبّتة يجب أن
 * يجد العمليات معطّلةً بسببها المكتوب، لا أن يجد لا شيء ويظنّ أن المنتج لا
 * يعرف Git أصلًا. الترتيب وحده يقدّم المتاح.
 */
export function search(
  query: string,
  operations: OperationCard[],
  categories: CategoryCard[],
): SearchResults {
  const needle = fold(query);
  if (needle === '') {
    return { active: false, categories, operations };
  }
  const words = needle.split(' ').filter((w) => w !== '');

  const matches = (hay: string) => words.every((w) => hay.includes(w));

  const ops = operations
    .filter((card) => matches(haystackForOperation(card, categories)))
    // المتاح أولًا، ثم ترتيب المكتبة كما وصل. `sort` في JS مستقرّ، فالثاني
    // يبقى محفوظًا بلا حاجة إلى مفتاحٍ ثانٍ.
    .sort((a, b) => Number(isAvailable(b.availability)) - Number(isAvailable(a.availability)));

  const cats = categories.filter((card) => matches(haystackForCategory(card)));

  return { active: true, categories: cats, operations: ops };
}

// ── المستخدَمة حديثًا والمفضّلة ─────────────────────────────────────────

/**
 * آخر العمليات المستخدَمة، مشتقّةً من **سجلٍّ حقيقي**.
 *
 * ثلاثة قيود تجعل هذه القائمة صادقة:
 *
 * 1. **قيود `planned` تُهمَل.** الواجهة تعيد التخطيط بعد كل سكونٍ في الكتابة،
 *    فتعديل اسم ملفٍ واحد يولّد عشرات المعاينات. عدّها «استخدامًا» كان يجعل
 *    «المستخدَمة حديثًا» تعني «آخر ما لمسته لوحة المفاتيح».
 * 2. **بلا تكرار.** تشغيل العملية نفسها خمس مرّات يملأ القائمة بها وحدها.
 * 3. **ما لم يعد في الفهرس يُسقَط.** عمليةٌ حُذفت في إصدارٍ لاحق لا تُعرض
 *    بطاقةً تفتح شاشةً لا وجود لها.
 */
export function recentOperations(
  entries: JournalEntry[],
  cards: OperationCard[],
  limit = 6,
): OperationCard[] {
  const seen = new Set<string>();
  const out: OperationCard[] = [];
  for (let i = entries.length - 1; i >= 0 && out.length < limit; i -= 1) {
    const entry = entries[i];
    if (!entry || entry.state === 'planned') continue;
    if (seen.has(entry.op_id)) continue;
    seen.add(entry.op_id);
    const card = findOperation(cards, entry.op_id);
    if (card) out.push(card);
  }
  return out;
}

/**
 * المفضّلة، بترتيب المكتبة لا بترتيب الإضافة.
 *
 * القرار مقصود: قائمةٌ تتبع ترتيب الإضافة تتحرّك تحت اليد كلما أُضيف شيء،
 * فيفقد المستخدم موضع ما يعرفه. وترتيب المكتبة ثابتٌ ومعروفٌ له أصلًا.
 *
 * والمعرّف الذي لم يعد في الفهرس يُسقَط صامتًا — لا يُعرض ولا يُحذف من
 * الإعداد: نواةٌ أقدم قد لا تحمل عمليةً يحملها الإعداد، وحذفُها كان سيُفقدها
 * حين يعود المستخدم إلى إصدارٍ أحدث.
 */
export function favouriteOperations(ids: string[], cards: OperationCard[]): OperationCard[] {
  const wanted = new Set(ids);
  return cards.filter((c) => wanted.has(c.id));
}

export function toggleFavourite(ids: string[], opId: string): string[] {
  return ids.includes(opId) ? ids.filter((id) => id !== opId) : [...ids, opId];
}
