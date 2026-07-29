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

/** إصدار مخطط الإعداد. يُرفع عند كل تغيير غير متوافق في شكل القيمة. */
export const SETTINGS_SCHEMA_VERSION = 1;

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
}

export function defaultSettings(): Settings {
  return { schemaVersion: SETTINGS_SCHEMA_VERSION, onboardingCompletedAt: null };
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

/** يرقّي قيمة من إصدار أقدم. تُضاف حلقة لكل إصدار جديد. */
function migrate(raw: Record<string, unknown>, from: number): Settings | null {
  // لا إصدار أقدم من 1 بعد. الدالة موجودة كي تكون الترقية موضعًا واحدًا
  // معروفًا حين تلزم، لا بحثًا في الشيفرة يوم يلزم.
  if (from === SETTINGS_SCHEMA_VERSION) {
    const at = raw['onboardingCompletedAt'];
    return {
      schemaVersion: SETTINGS_SCHEMA_VERSION,
      onboardingCompletedAt: typeof at === 'string' && at !== '' ? at : null,
    };
  }
  return null;
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
    storage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
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
