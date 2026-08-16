/**
 * الإعداد المحلي — وفيه حالة أول تشغيل.
 *
 * ## لماذا هنا لا في النواة
 *
 * سطح IPC ستة أوامر، ولا سابع. «هل عُرض الترحيب؟» سؤالٌ عن تفضيل عرض لا عن
 * ملفات المستخدم ولا عن أمر يُنفَّذ، فتوسيع الحدّ الأمني من أجله كان سيدفع
 * ثمنًا معماريًا بلا مقابل. WKWebView يحفظ `localStorage` في حاوية التطبيق
 * ويبقى بين التشغيلات، وهذا كل ما يلزم.
 *
 * ## لماذا مخزَّن بإصدار مخطط
 *
 * الإعداد يعيش أطول من الشيفرة التي كتبته. حقلٌ يُضاف غدًا سيجد ملفات كُتبت
 * اليوم، فالإصدار مكتوبٌ داخل القيمة منذ أول كتابة، والقراءة تمرّ بسلسلة
 * ترقيات. بدونه يكون الخيار الوحيد أمام قيمة قديمة هو مسحها.
 *
 * ## لماذا لا ترمي القراءة أبدًا
 *
 * `localStorage` يفشل فعلًا: وضع خصوصية، حصة ممتلئة، قيمة عدّلها أحدهم بيده.
 * تطبيقٌ يسقط في شاشة بيضاء لأن تفضيل عرضٍ تالف هو تطبيق أسوأ من تطبيق يعرض
 * الترحيب مرّةً زائدة. كل مسارات الفشل هنا تنتهي إلى إعداد افتراضي صالح،
 * والسبب يُعاد إلى المستدعي كي يُسجَّل لا كي يُسقط الشاشة.
 */

/**
 * إصدار مخطط الإعداد. يُرفع عند كل تغيير غير متوافق في شكل القيمة.
 *
 * الإصدار ٢ أضاف المفضّلة وآخر قسمٍ مفتوح. والترقية من ١ ليست حذفًا: من أتمّ
 * الترحيب في نسخةٍ أقدم يجب ألّا يراه مرّةً أخرى لأننا أضفنا حقلًا لا يعرفه.
 */
export const SETTINGS_SCHEMA_VERSION = 2;

/**
 * أقصى عدد مفضّلات تُحفظ.
 *
 * حدٌّ لا لأن الذاكرة تضيق، بل لأن `localStorage` قيمةٌ واحدة نكتبها كاملةً في
 * كل تغيير — وقائمةٌ بلا حدّ تعني أن قيمةً تالفة أو مُعدَّلة بيد يمكن أن تكبر
 * حتى تفشل الكتابة، فيضيع الإعداد كلّه لا المفضّلة وحدها.
 */
export const MAX_FAVOURITES = 40;

/** مفتاح التخزين. يحمل اسم المنتج صراحةً كي لا يشتبه بغيره في نفس الأصل. */
export const SETTINGS_STORAGE_KEY = 'naffith.settings';

export interface Settings {
  schemaVersion: number;
  /**
   * لحظة إتمام الترحيب بصيغة ISO، أو `null` إن لم يُتمّ بعد.
   *
   * وقتٌ لا رايةٌ منطقية: «متى؟» تجيب عن «هل؟» وتزيد، وهي مفيدة حين نحتاج
   * يومًا أن نعرض ترحيبًا محدَّثًا لمن أتمّ القديم قبل تاريخ ما.
   */
  onboardingCompletedAt: string | null;
  /**
   * معرّفات العمليات المفضّلة، بلا ترتيبٍ ذي معنى.
   *
   * تُعرض بترتيب المكتبة لا بترتيب الإضافة (انظر `library.ts`)، فالترتيب هنا
   * تفصيلُ تخزينٍ لا قرار عرض.
   */
  favourites: string[];
  /**
   * آخر قسمٍ فُتح، أو `null`.
   *
   * يُحفظ لأن المستخدم يعود إلى ما كان فيه: من يعمل في الضغط طوال اليوم لا
   * يريد أن يبدأ كل مرّةٍ من شبكة الأقسام. **ولا يُفتح تلقائيًا** — الجذر يبقى
   * شاشة الفئات — بل يُبرَز بوصفه «آخر ما فتحت»، فالقرار يبقى للمستخدم ولا
   * يجد نفسه في شاشةٍ لم يطلبها.
   */
  lastCategoryId: string | null;
}

export function defaultSettings(): Settings {
  return {
    schemaVersion: SETTINGS_SCHEMA_VERSION,
    onboardingCompletedAt: null,
    favourites: [],
    lastCategoryId: null,
  };
}

/** سبب اللجوء إلى الإعداد الافتراضي. يُسجَّل ولا يُعرض للمستخدم. */
export type FallbackReason =
  /** لا قيمة مخزّنة — أول تشغيل حقيقي. */
  | 'missing'
  /** القيمة موجودة لكنها ليست JSON صالحًا أو ليست بالشكل المتوقّع. */
  | 'corrupt'
  /** تعذّر الوصول إلى المخزن أصلًا (خصوصية، حصة، أو منع). */
  | 'unreadable'
  /** إصدار من المستقبل: كُتب بنسخة أحدث من التطبيق. */
  | 'from_future';

export type SettingsLoad =
  | { status: 'loaded'; settings: Settings; migratedFrom: number | null }
  | { status: 'fallback'; settings: Settings; reason: FallbackReason };

/**
 * الحدّ الأدنى من `Storage`.
 *
 * واجهة لا `localStorage` مباشرة كي تُختبر مسارات الفشل باستهزاء يرمي فعلًا؛
 * محاكاة قرصٍ ممتلئ عبر `localStorage` الحقيقي غير ممكنة في اختبار وحدة.
 */
export interface SettingsStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
  removeItem(key: string): void;
}

/** يعيد `localStorage` إن كان متاحًا فعلًا، أو `null` إن كان الوصول يرمي. */
export function browserStorage(): SettingsStorage | null {
  try {
    // مجرّد وجود `window.localStorage` لا يكفي: القراءة نفسها ترمي تحت بعض
    // إعدادات الخصوصية، فالفحص يقرأ فعلًا.
    const probe = globalThis.localStorage;
    probe.getItem(SETTINGS_STORAGE_KEY);
    return probe;
  } catch {
    return null;
  }
}

/** قائمة معرّفات نظيفة من قيمةٍ مخزَّنة قد تكون أي شيء. */
function readIdList(value: unknown, limit: number): string[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  for (const item of value) {
    if (typeof item !== 'string') continue;
    const id = item.trim();
    // المعرّفات في هذا المنتج نقاطٌ وحروفٌ لاتينية. أي شيءٍ آخر ليس معرّفًا،
    // والقيمة المخزَّنة قابلةٌ للتعديل بيد.
    if (id === '' || id.length > 120 || !/^[a-z0-9._-]+$/.test(id)) continue;
    seen.add(id);
    if (seen.size >= limit) break;
  }
  return [...seen];
}

function readNullableId(value: unknown): string | null {
  const [only] = readIdList([value], 1);
  return only ?? null;
}

/**
 * يرقّي قيمة من إصدار أقدم. تُضاف حلقة لكل إصدار جديد.
 *
 * الترقية من ١ إلى ٢ **تحتفظ بما كان** وتضيف الافتراضي لما استُجدّ. الاختصار
 * — إعادة الافتراضي كاملًا — كان سيعرض شاشة الترحيب على كل من رقّى، وهي أول
 * ما يراه من التحديث: رسالةٌ تقول له إن التطبيق نسيه.
 */
function migrate(raw: Record<string, unknown>, from: number): Settings | null {
  if (from > SETTINGS_SCHEMA_VERSION || from < 1) return null;

  const at = raw['onboardingCompletedAt'];
  const onboardingCompletedAt = typeof at === 'string' && at !== '' ? at : null;

  // الإصدار ١ لا يحمل الحقلين، فيقرآن غيابًا ويصيران افتراضيهما.
  return {
    schemaVersion: SETTINGS_SCHEMA_VERSION,
    onboardingCompletedAt,
    favourites: readIdList(raw['favourites'], MAX_FAVOURITES),
    lastCategoryId: readNullableId(raw['lastCategoryId']),
  };
}

/**
 * يقرأ الإعداد. لا يرمي أبدًا.
 *
 * القيمة التالفة تُعامل معاملة الغياب لا معاملة الخطأ: النتيجة إعدادٌ صالح
 * في الحالتين، والفرق مسجَّلٌ في `reason` وحده.
 */
export function loadSettings(storage: SettingsStorage | null): SettingsLoad {
  if (!storage) {
    return { status: 'fallback', settings: defaultSettings(), reason: 'unreadable' };
  }

  let raw: string | null;
  try {
    raw = storage.getItem(SETTINGS_STORAGE_KEY);
  } catch {
    return { status: 'fallback', settings: defaultSettings(), reason: 'unreadable' };
  }

  if (raw === null) {
    return { status: 'fallback', settings: defaultSettings(), reason: 'missing' };
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return { status: 'fallback', settings: defaultSettings(), reason: 'corrupt' };
  }

  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    return { status: 'fallback', settings: defaultSettings(), reason: 'corrupt' };
  }

  const record = parsed as Record<string, unknown>;
  const version = record['schemaVersion'];
  if (typeof version !== 'number' || !Number.isInteger(version) || version < 1) {
    return { status: 'fallback', settings: defaultSettings(), reason: 'corrupt' };
  }

  // إصدارٌ أحدث من هذا البناء: لا نعرف شكله ولا يجوز أن نكتب فوقه بصمت —
  // المستخدم قد يعود إلى النسخة الأحدث. نعمل على افتراضيٍّ في الذاكرة.
  if (version > SETTINGS_SCHEMA_VERSION) {
    return { status: 'fallback', settings: defaultSettings(), reason: 'from_future' };
  }

  const migrated = migrate(record, version);
  if (!migrated) {
    return { status: 'fallback', settings: defaultSettings(), reason: 'corrupt' };
  }

  return {
    status: 'loaded',
    settings: migrated,
    migratedFrom: version === SETTINGS_SCHEMA_VERSION ? null : version,
  };
}

/**
 * يكتب الإعداد. يعيد `false` إن فشلت الكتابة ولا يرمي.
 *
 * فشل الكتابة لا يُبطل الانتقال الجاري: من ضغط «ابدأ الآن» ينتقل إلى قائمة
 * العمليات حتى لو تعذّر حفظ ذلك — أسوأ ما يقع أن يرى الترحيب مرّةً أخرى في
 * التشغيل القادم، وهو أهون من أن يُحبس في شاشة لا تُغادَر.
 */
export function saveSettings(storage: SettingsStorage | null, settings: Settings): boolean {
  if (!storage) return false;
  try {
    // الإصدار يُبصم هنا لا يُنقل كما وصل: مستدعٍ يبني الإعداد بيده — أو يمرّر
    // قيمةً نجت من بناءٍ أقدم — كان سيكتب إصدارًا لا تقبله `loadSettings`، فتضيع
    // القيمة صامتةً في التشغيل التالي بوصفها «تالفة». الكتابة وحدها تعرف إصدار
    // هذا البناء، فهي الموضع الوحيد القادر على ضمان الوعد.
    storage.setItem(
      SETTINGS_STORAGE_KEY,
      JSON.stringify({ ...settings, schemaVersion: SETTINGS_SCHEMA_VERSION }),
    );
    return true;
  } catch {
    return false;
  }
}

/** هل يُعرض الترحيب؟ سؤالٌ واحد بإجابة واحدة، فلا يتفرّع الشرط في الشاشات. */
export function shouldShowOnboarding(settings: Settings): boolean {
  return settings.onboardingCompletedAt === null;
}

/** يعلّم الترحيب مُتمًّا. `now` مُمرَّر كي يكون الاختبار حتميًا. */
export function withOnboardingCompleted(settings: Settings, now: Date): Settings {
  return { ...settings, onboardingCompletedAt: now.toISOString() };
}

/** يعيد ضبط الترحيب كي يُعرض في التشغيل القادم — ومن الإعدادات، فورًا. */
export function withOnboardingReset(settings: Settings): Settings {
  return { ...settings, onboardingCompletedAt: null };
}

/**
 * يقلب حالة التفضيل لعملية.
 *
 * الحدّ يُفرض عند **الإضافة** لا عند القراءة: قراءةٌ تقصّ كانت ستُسقط مفضّلةً
 * موجودة بلا أن يفعل المستخدم شيئًا، وهو فقدٌ صامت. والإضافة فوق الحدّ تُهمَل
 * بلا خطأ — أربعون عمليةً مفضّلة ليست قائمةً مفضّلة أصلًا.
 */
export function withFavouriteToggled(settings: Settings, opId: string): Settings {
  const has = settings.favourites.includes(opId);
  if (!has && settings.favourites.length >= MAX_FAVOURITES) return settings;
  return {
    ...settings,
    favourites: has
      ? settings.favourites.filter((id) => id !== opId)
      : [...settings.favourites, opId],
  };
}

/** يحفظ آخر قسمٍ فُتح. لا يُفتح تلقائيًا — انظر تعليق `lastCategoryId`. */
export function withLastCategory(settings: Settings, categoryId: string | null): Settings {
  if (settings.lastCategoryId === categoryId) return settings;
  return { ...settings, lastCategoryId: categoryId };
}
