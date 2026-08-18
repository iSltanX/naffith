/**
 * مخزن الإعداد وحالة أول تشغيل.
 *
 * الوعد المكتوب في رأس `settings.ts` وعدٌ واحد: **لا تسقط الشاشة من أجل تفضيل
 * عرض**. لذلك أكثر ما هنا مسارات فشل لا مسار نجاح: قيمةٌ عدّلها أحدهم بيده،
 * مخزنٌ يرمي عند القراءة، حصةٌ ممتلئة عند الكتابة، وإصدارٌ كُتب بنسخة أحدث من
 * التطبيق. كلها يجب أن تنتهي إلى إعدادٍ صالح وسببٍ مسجَّل، لا إلى استثناء.
 *
 * `SettingsStorage` واجهة لا `localStorage` — وهذه الاختبارات هي السبب الذي
 * كُتبت من أجله: محاكاة قرصٍ ممتلئ عبر المخزن الحقيقي غير ممكنة هنا.
 */
import { describe, expect, it } from 'vitest';
import {
  SETTINGS_SCHEMA_VERSION,
  SETTINGS_STORAGE_KEY,
  defaultSettings,
  loadSettings,
  saveSettings,
  shouldShowOnboarding,
  withAutoUpdate,
  withCargoPath,
  withConfirmBeforeExecute,
  withDefaultWorkingPath,
  withNodePath,
  withNotificationSound,
  withOnboardingCompleted,
  withOnboardingReset,
  withSidebarIconSize,
  withTheme,
  type Settings,
  type SettingsStorage,
} from './settings';

interface FakeStorage extends SettingsStorage {
  /** كل ما كُتب، بمفتاحه. يمسك الكتابات التي لا يجوز أن تقع أصلًا. */
  readonly writes: Array<[string, string]>;
  /** كل ما قُرئ، بمفتاحه. يحرس أن المفتاح المعلَن هو المستعمل فعلًا. */
  readonly reads: string[];
  peek(): string | null;
}

function fakeStorage(seed: string | null = null): FakeStorage {
  let value = seed;
  const writes: Array<[string, string]> = [];
  const reads: string[] = [];
  return {
    getItem(key: string) {
      reads.push(key);
      return key === SETTINGS_STORAGE_KEY ? value : null;
    },
    setItem(key: string, next: string) {
      writes.push([key, next]);
      if (key === SETTINGS_STORAGE_KEY) value = next;
    },
    removeItem(key: string) {
      if (key === SETTINGS_STORAGE_KEY) value = null;
    },
    writes,
    reads,
    peek: () => value,
  };
}

/** مخزنٌ ترمي قراءتُه — وضع خصوصية يمنع الوصول أصلًا. */
const throwsOnRead: SettingsStorage = {
  getItem(): string | null {
    throw new Error('SecurityError: access to storage is denied');
  },
  setItem(): void {},
  removeItem(): void {},
};

/** مخزنٌ ترمي كتابتُه — حصة ممتلئة. القراءة منه سليمة. */
function throwsOnWrite(): SettingsStorage {
  return {
    getItem: () => null,
    setItem(): void {
      throw new Error('QuotaExceededError');
    },
    removeItem(): void {},
  };
}

function storedValue(store: FakeStorage): Record<string, unknown> {
  const raw = store.peek();
  expect(raw).not.toBeNull();
  return JSON.parse(raw ?? '{}') as Record<string, unknown>;
}

const COMPLETED_AT = '2026-07-29T12:34:56.789Z';

function valid(overrides: Record<string, unknown> = {}): string {
  return JSON.stringify({
    schemaVersion: SETTINGS_SCHEMA_VERSION,
    onboardingCompletedAt: COMPLETED_AT,
    ...overrides,
  });
}

describe('القراءة', () => {
  it('غياب القيمة أول تشغيل حقيقي لا خطأ', () => {
    const result = loadSettings(fakeStorage(null));
    expect(result).toEqual({
      status: 'fallback',
      settings: defaultSettings(),
      reason: 'missing',
    });
    expect(shouldShowOnboarding(result.settings)).toBe(true);
  });

  it('القيمة الصالحة تُقرأ كما كُتبت', () => {
    const result = loadSettings(fakeStorage(valid()));
    expect(result).toEqual({
      status: 'loaded',
      settings: { ...defaultSettings(), onboardingCompletedAt: COMPLETED_AT },
      // لا ترقية وقعت: الإصدار هو إصدار هذا البناء.
      migratedFrom: null,
    });
    expect(shouldShowOnboarding(result.settings)).toBe(false);
  });

  it('يقرأ بالمفتاح المعلَن لا بغيره', () => {
    // المفتاح يحمل اسم المنتج كي لا يشتبه بغيره في نفس الأصل، فتغييره صامتًا
    // يعني فقدان إعداد كل مستخدم قائم.
    const store = fakeStorage(valid());
    loadSettings(store);
    expect(store.reads).toEqual([SETTINGS_STORAGE_KEY]);
    expect(SETTINGS_STORAGE_KEY).toBe('naffith.settings');
  });
});

describe('مسار Node.js', () => {
  it('قيمةٌ من الإصدار ٣ تُقرأ كما كُتبت', () => {
    const result = loadSettings(
      fakeStorage(valid({ nodePath: '/Users/سارة/.nvm/versions/node/v20/bin/node' })),
    );
    expect(result.settings.nodePath).toBe('/Users/سارة/.nvm/versions/node/v20/bin/node');
  });

  it('إصدارٌ ٢ لا يحمل الحقل، فيُقرأ null لا خطأً', () => {
    // الترقية تحتفظ بما كان وتضيف افتراضي ما استُجدّ — نفس منطق `lastCategoryId`
    // عند الترقية من ١ إلى ٢.
    const v2 = JSON.stringify({ schemaVersion: 2, onboardingCompletedAt: COMPLETED_AT });
    const result = loadSettings(fakeStorage(v2));
    expect(result).toMatchObject({ status: 'loaded', migratedFrom: 2 });
    expect(result.settings.nodePath).toBeNull();
  });

  it('قيمةٌ عدّلها أحدٌ بيده — لا نصّ، أو مسارٌ نسبي، أو فارغ — تُقرأ null', () => {
    for (const bad of [42, { path: '/x' }, 'ليس-مسارًا', '', '   ']) {
      const result = loadSettings(fakeStorage(valid({ nodePath: bad })));
      expect(result.settings.nodePath, JSON.stringify(bad)).toBeNull();
    }
  });

  it('withNodePath يحفظ القيمة ويتخطّى الكتابة إن لم تتغيّر', () => {
    const s1 = withNodePath(defaultSettings(), '/usr/local/bin/node');
    expect(s1.nodePath).toBe('/usr/local/bin/node');

    const s2 = withNodePath(s1, '/usr/local/bin/node');
    expect(s2).toBe(s1); // نفس المرجع: لا تغيير فلا كتابة.

    const s3 = withNodePath(s1, null);
    expect(s3.nodePath).toBeNull();
  });
});

describe('مسار Cargo', () => {
  it('قيمةٌ من الإصدار ٤ تُقرأ كما كُتبت', () => {
    const result = loadSettings(fakeStorage(valid({ cargoPath: '/Users/سارة/.cargo/bin/cargo' })));
    expect(result.settings.cargoPath).toBe('/Users/سارة/.cargo/bin/cargo');
  });

  it('إصدارٌ ٣ لا يحمل الحقل، فيُقرأ null لا خطأً', () => {
    const v3 = JSON.stringify({
      schemaVersion: 3,
      onboardingCompletedAt: COMPLETED_AT,
      nodePath: '/usr/local/bin/node',
    });
    const result = loadSettings(fakeStorage(v3));
    expect(result).toMatchObject({ status: 'loaded', migratedFrom: 3 });
    expect(result.settings.nodePath).toBe('/usr/local/bin/node'); // يُحتفَظ بما كان
    expect(result.settings.cargoPath).toBeNull(); // ويُضاف افتراض ما استُجدّ
  });

  it('withCargoPath يحفظ القيمة ويتخطّى الكتابة إن لم تتغيّر', () => {
    const s1 = withCargoPath(defaultSettings(), '/Users/x/.cargo/bin/cargo');
    expect(s1.cargoPath).toBe('/Users/x/.cargo/bin/cargo');

    const s2 = withCargoPath(s1, '/Users/x/.cargo/bin/cargo');
    expect(s2).toBe(s1);

    const s3 = withCargoPath(s1, null);
    expect(s3.cargoPath).toBeNull();
  });
});

describe('القيمة التالفة', () => {
  /** كل صورة تلفٍ رأيناها أو نتوقّعها، وكلها يجب أن تنتهي إلى نفس الجواب. */
  const corrupt: Array<[string, string]> = [
    ['ليست JSON أصلًا', 'not json at all {{{'],
    ['نصّ فارغ', ''],
    ['مصفوفة', '[1,2,3]'],
    ['مصفوفة فارغة', '[]'],
    ['رقم مجرّد', '42'],
    ['نصّ JSON مجرّد', '"naffith"'],
    ['قيمة منطقية', 'true'],
    ['null حرفيّة', 'null'],
    ['كائن بلا إصدار', JSON.stringify({ onboardingCompletedAt: COMPLETED_AT })],
    ['إصدار نصّي', JSON.stringify({ schemaVersion: '1', onboardingCompletedAt: null })],
    ['إصدار null', JSON.stringify({ schemaVersion: null })],
    ['إصدار صفر', JSON.stringify({ schemaVersion: 0 })],
    ['إصدار سالب', JSON.stringify({ schemaVersion: -1 })],
    ['إصدار كسريّ', JSON.stringify({ schemaVersion: 1.5 })],
  ];

  for (const [name, raw] of corrupt) {
    it(`${name}: إعدادٌ افتراضي وسببٌ مسجَّل، بلا استثناء`, () => {
      const store = fakeStorage(raw);
      // الشرط الأول أن لا ترمي: شاشةٌ بيضاء بسبب تفضيل عرضٍ تالف أسوأ من ترحيبٍ
      // يُعرض مرّةً زائدة.
      expect(() => loadSettings(store)).not.toThrow();
      const result = loadSettings(store);
      expect(result).toEqual({
        status: 'fallback',
        settings: defaultSettings(),
        reason: 'corrupt',
      });
      expect(shouldShowOnboarding(result.settings)).toBe(true);
    });
  }

  it('لا تُصلَح القيمة التالفة بالكتابة فوقها أثناء القراءة', () => {
    // القراءة قراءة. مسحُ ما لا نفهمه قرارٌ يخصّ المستدعي لا هذه الدالة.
    const store = fakeStorage('{{{');
    loadSettings(store);
    expect(store.writes).toEqual([]);
    expect(store.peek()).toBe('{{{');
  });

  it('الحقل الصالح بجانب حقلٍ تالف يبقى صالحًا', () => {
    // التلف في `onboardingCompletedAt` وحده لا يُسقط القيمة كلها: الإصدار سليم
    // فالقيمة مقروءة، والحقل وحده يُطبَّع.
    expect(loadSettings(fakeStorage(valid({ onboardingCompletedAt: '' })))).toEqual({
      status: 'loaded',
      settings: { ...defaultSettings(), onboardingCompletedAt: null },
      migratedFrom: null,
    });
  });

  it('لحظة الإتمام بنوع غير نصّي تُطبَّع إلى null', () => {
    for (const bad of [0, 1, true, {}, [], null]) {
      const result = loadSettings(fakeStorage(valid({ onboardingCompletedAt: bad })));
      expect(result.settings.onboardingCompletedAt).toBeNull();
      expect(shouldShowOnboarding(result.settings)).toBe(true);
    }
  });

  it('غياب حقل لحظة الإتمام ليس تلفًا', () => {
    const raw = JSON.stringify({ schemaVersion: SETTINGS_SCHEMA_VERSION });
    const result = loadSettings(fakeStorage(raw));
    expect(result.status).toBe('loaded');
    expect(result.settings.onboardingCompletedAt).toBeNull();
  });

  it('الحقول التي لا نعرفها لا تمنع القراءة', () => {
    // نسخةٌ ثانية على نفس الأصل قد تكتب حقلًا لا يعنينا؛ تجاهله أهون من رفض
    // القيمة كلها.
    //
    // كان المثال هنا `theme` حتى صار حقلًا معروفًا في الإصدار ٥ — فاختبار
    // «المجهول يُتجاهل» صار يقرأ قيمةً حقيقية ويسقط. المثال الآن حقلٌ لا
    // يعرفه هذا البناء ولا يُتوقّع أن يعرفه.
    const result = loadSettings(fakeStorage(valid({ soundscape: 'rain' })));
    expect(result.status).toBe('loaded');
    expect(result.settings).toEqual({
      ...defaultSettings(),
      onboardingCompletedAt: COMPLETED_AT,
    });
  });
});

describe('تفضيلات صفحة ٢٠', () => {
  it('الافتراضات هي ما يرسمه التصميم', () => {
    const d = defaultSettings();
    expect(d.theme).toBe('system');
    expect(d.sidebarIconSize).toBe('medium');
    expect(d.notificationSound).toBe(true);
    expect(d.confirmBeforeExecute).toBe(true);
    expect(d.autoUpdate).toBe(true);
    expect(d.defaultWorkingPath).toBeNull();
  });

  it('قيمةٌ خارج المجموعة المغلقة تعود إلى الافتراضي', () => {
    // القيمة المخزّنة قابلةٌ للتعديل بيد. «سمة» لا يعرفها هذا البناء يجب ألّا
    // تصل إلى عنصر الجذر كما هي.
    const result = loadSettings(fakeStorage(valid({ theme: 'neon', sidebarIconSize: 3 })));
    expect(result.status).toBe('loaded');
    expect(result.settings.theme).toBe('system');
    expect(result.settings.sidebarIconSize).toBe('medium');
  });

  it('رايةٌ ليست منطقيةً تعود إلى الافتراضي', () => {
    const result = loadSettings(fakeStorage(valid({ notificationSound: 'yes', autoUpdate: null })));
    expect(result.settings.notificationSound).toBe(true);
    expect(result.settings.autoUpdate).toBe(true);
  });

  it('الترقية من ٤ تُبقي ما كان وتضيف الافتراضي لما استُجدّ', () => {
    const older = JSON.stringify({
      schemaVersion: 4,
      onboardingCompletedAt: COMPLETED_AT,
      favourites: [],
      lastCategoryId: null,
      nodePath: '/usr/local/bin/node',
      cargoPath: null,
    });
    const result = loadSettings(fakeStorage(older));
    expect(result.status).toBe('loaded');
    expect(result.settings.nodePath).toBe('/usr/local/bin/node');
    expect(result.settings.theme).toBe('system');
    expect(result.settings.confirmBeforeExecute).toBe(true);
    expect(result.settings.schemaVersion).toBe(SETTINGS_SCHEMA_VERSION);
  });

  it('المُعدِّلات لا تُنشئ كائنًا جديدًا حين لا تتغيّر القيمة', () => {
    const s = defaultSettings();
    expect(withTheme(s, 'system')).toBe(s);
    expect(withAutoUpdate(s, true)).toBe(s);
    expect(withTheme(s, 'dark')).not.toBe(s);
    expect(withTheme(s, 'dark').theme).toBe('dark');
    expect(withSidebarIconSize(s, 'small').sidebarIconSize).toBe('small');
    expect(withDefaultWorkingPath(s, '/tmp').defaultWorkingPath).toBe('/tmp');
    expect(withConfirmBeforeExecute(s, false).confirmBeforeExecute).toBe(false);
    expect(withNotificationSound(s, false).notificationSound).toBe(false);
  });
});

describe('إصدار من المستقبل', () => {
  const future = JSON.stringify({
    schemaVersion: SETTINGS_SCHEMA_VERSION + 1,
    onboardingCompletedAt: COMPLETED_AT,
    somethingNew: true,
  });

  it('يُعامَل معاملةً خاصة لا معاملة التلف', () => {
    expect(loadSettings(fakeStorage(future))).toEqual({
      status: 'fallback',
      settings: defaultSettings(),
      reason: 'from_future',
    });
  });

  it('لا يُكتَب فوقه بصمت', () => {
    // المستخدم قد يعود إلى النسخة الأحدث، فمسحُ إعدادها لأننا لا نفهمه خسارةٌ
    // لا مبرّر لها. نعمل على افتراضيٍّ في الذاكرة وحدها.
    const store = fakeStorage(future);
    loadSettings(store);
    expect(store.writes).toEqual([]);
    expect(store.peek()).toBe(future);
  });
});

describe('تعذّر الوصول', () => {
  it('مخزنٌ ترمي قراءتُه يُعطي «غير مقروء» لا استثناءً', () => {
    expect(() => loadSettings(throwsOnRead)).not.toThrow();
    expect(loadSettings(throwsOnRead)).toEqual({
      status: 'fallback',
      settings: defaultSettings(),
      reason: 'unreadable',
    });
  });

  it('غياب المخزن أصلًا هو الحالة نفسها', () => {
    // `browserStorage()` تعيد null حين يمنع وضع الخصوصية الوصول، فتصل هنا null.
    expect(loadSettings(null)).toEqual({
      status: 'fallback',
      settings: defaultSettings(),
      reason: 'unreadable',
    });
  });

  it('«غير مقروء» يعرض الترحيب: أهون من حبس المستخدم', () => {
    expect(shouldShowOnboarding(loadSettings(null).settings)).toBe(true);
  });
});

describe('الكتابة', () => {
  it('الكتابة الناجحة تعود صحيحةً وتُقرأ كما كُتبت', () => {
    const store = fakeStorage();
    const settings = withOnboardingCompleted(defaultSettings(), new Date(COMPLETED_AT));
    expect(saveSettings(store, settings)).toBe(true);
    expect(loadSettings(store)).toEqual({ status: 'loaded', settings, migratedFrom: null });
  });

  it('الكتابة تحمل إصدار المخطط', () => {
    // الإصدار مكتوبٌ داخل القيمة منذ أول كتابة: بدونه يكون الخيار الوحيد أمام
    // قيمةٍ قديمة يومًا هو مسحها.
    const store = fakeStorage();
    saveSettings(store, defaultSettings());
    expect(storedValue(store)['schemaVersion']).toBe(SETTINGS_SCHEMA_VERSION);
  });

  it('فشل الكتابة يعود كذبًا ولا يرمي', () => {
    // من ضغط «ابدأ الآن» ينتقل حتى لو تعذّر الحفظ؛ أسوأ ما يقع أن يرى الترحيب
    // مرّةً أخرى، وهو أهون من شاشةٍ لا تُغادَر.
    const store = throwsOnWrite();
    expect(() => saveSettings(store, defaultSettings())).not.toThrow();
    expect(saveSettings(store, defaultSettings())).toBe(false);
  });

  it('الكتابة إلى مخزنٍ معدوم تعود كذبًا', () => {
    expect(saveSettings(null, defaultSettings())).toBe(false);
  });

  it('يكتب بالمفتاح المعلَن وحده', () => {
    const store = fakeStorage();
    saveSettings(store, defaultSettings());
    expect(store.writes.map(([key]) => key)).toEqual([SETTINGS_STORAGE_KEY]);
  });

  /**
   * يحرس أن تبصم الكتابة إصدار البناء الحالي لا الإصدار الممرَّر.
   *
   * مستدعٍ يبني الإعداد بيده (أو يمرّر قيمةً نجت من نسخة أقدم) كان سيكتب
   * إصدارًا لا يقبله `loadSettings`، فتضيع القيمة صامتةً في التشغيل التالي
   * بوصفها «تالفة». الوعد المعلَن أن كل قيمة مكتوبة تحمل إصدار البناء الحالي،
   * والموضع الوحيد القادر على ضمانه هو الكتابة نفسها:
   *
   *     JSON.stringify({ ...settings, schemaVersion: SETTINGS_SCHEMA_VERSION })
   */
  it('تبصم الإصدار الحالي مهما كان الإصدار الممرَّر', () => {
    const store = fakeStorage();
    const stale: Settings = { ...defaultSettings(), schemaVersion: 0, onboardingCompletedAt: COMPLETED_AT };
    expect(saveSettings(store, stale)).toBe(true);
    expect(storedValue(store)['schemaVersion']).toBe(SETTINGS_SCHEMA_VERSION);
    expect(loadSettings(store).status).toBe('loaded');
  });
});

describe('حالة الترحيب', () => {
  it('لا لحظة إتمام يعني: اعرض الترحيب', () => {
    expect(shouldShowOnboarding(defaultSettings())).toBe(true);
  });

  it('الإتمام يكتب لحظةً بصيغة ISO من التاريخ المحقون', () => {
    // `now` مُمرَّر لا مأخوذ من الساعة كي يكون هذا الاختبار حتميًا.
    const now = new Date(Date.UTC(2026, 6, 29, 12, 34, 56, 789));
    const after = withOnboardingCompleted(defaultSettings(), now);
    expect(after.onboardingCompletedAt).toBe(COMPLETED_AT);
    expect(shouldShowOnboarding(after)).toBe(false);
  });

  it('«متى» تجيب عن «هل» وتزيد', () => {
    // وقتٌ لا رايةٌ منطقية: يبقى قابلًا للمقارنة بتاريخٍ يومًا نعرض فيه ترحيبًا
    // محدَّثًا لمن أتمّ القديم.
    const at = withOnboardingCompleted(defaultSettings(), new Date(COMPLETED_AT))
      .onboardingCompletedAt;
    expect(at).not.toBeNull();
    expect(Date.parse(at ?? '')).toBe(Date.parse(COMPLETED_AT));
  });

  it('الإعادة تُظهر الترحيب من جديد', () => {
    const done = withOnboardingCompleted(defaultSettings(), new Date(COMPLETED_AT));
    const reset = withOnboardingReset(done);
    expect(reset.onboardingCompletedAt).toBeNull();
    expect(shouldShowOnboarding(reset)).toBe(true);
  });

  it('الإعادة تنجو من دورة كتابةٍ وقراءة', () => {
    // «أعد عرض الترحيب» من الإعدادات وعدٌ للتشغيل القادم لا لهذه الجلسة وحدها.
    const store = fakeStorage(valid());
    const loaded = loadSettings(store);
    saveSettings(store, withOnboardingReset(loaded.settings));
    expect(shouldShowOnboarding(loadSettings(store).settings)).toBe(true);
  });

  it('التعديل لا يمسّ الإعداد الأصلي', () => {
    // القيمة تُنسخ لا تُحوَّر: حالةٌ في React قد تكون ممسكةً بالأصل.
    const before = defaultSettings();
    withOnboardingCompleted(before, new Date(COMPLETED_AT));
    withOnboardingReset(before);
    expect(before).toEqual(defaultSettings());
  });

  it('الإتمام يحفظ بقية الحقول', () => {
    const done = withOnboardingCompleted(
      { ...defaultSettings(), onboardingCompletedAt: null },
      new Date(COMPLETED_AT),
    );
    expect(done.schemaVersion).toBe(SETTINGS_SCHEMA_VERSION);
  });

  it('الافتراضي كائنٌ جديد في كل مرّة', () => {
    // افتراضيٌّ مشترك يعني أن تحويرًا في شاشةٍ يظهر في أخرى.
    const a = defaultSettings();
    const b = defaultSettings();
    expect(a).toEqual(b);
    expect(a).not.toBe(b);
  });
});
