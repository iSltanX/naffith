/**
 * آلة حالات التنقّل.
 *
 * ما يُختبر هنا ليس «هل تعمل الدوال؟» بل **هل تبقى الشاشات المستحيلة مستحيلة؟**.
 * الآلة كُتبت كي لا يحتاج أحد أن يسأل في مكوّنٍ ما «وماذا لو كان الترحيب معروضًا
 * وعمليةٌ مختارة؟»، فكل اختبار هنا يمسك طرفًا واحدًا من ذلك الوعد: حدثٌ لا وجهة
 * له لا يخترع وجهة، وانتقالٌ إلى المكان نفسه ليس انتقالًا، وكلفة المغادرة تُقرَّر
 * في موضع واحد لا في كل شاشة.
 *
 * الاختبارات لا تستدعي React ولا DOM: الملف نقيّ، والبيئة الافتراضية `node` تكفيه.
 */
import { describe, expect, it } from 'vitest';
import {
  confirmNavigation,
  initialScreen,
  navigate,
  sameScreen,
  type ExitCost,
  type NavEvent,
  type Screen,
} from './nav';

const ONBOARDING: Screen = { name: 'onboarding' };
const LIST: Screen = { name: 'operations-list' };
const LOG: Screen = { name: 'run-log' };
const SETTINGS: Screen = { name: 'settings' };
const view = (opId: string): Screen => ({ name: 'operation-view', opId });

/** كل الشاشات، كي لا يُنسى موضعٌ حين يُفحص حدثٌ على كلٍّ منها. */
const EVERY_SCREEN: Screen[] = [ONBOARDING, LIST, view('any.op'), LOG, SETTINGS];

describe('الشاشة الأولى', () => {
  it('يبدأ من الترحيب في أول تشغيل', () => {
    expect(initialScreen(true)).toEqual(ONBOARDING);
  });

  it('يبدأ من قائمة العمليات فيما بعده', () => {
    // لا شرط ثالث: تعذّرُ قراءة الإعداد صار «لم يُتمّ بعد» قبل أن يصل إلى هنا.
    expect(initialScreen(false)).toEqual(LIST);
  });
});

describe('الترحيب', () => {
  it('«ابدأ الآن» ينقل إلى قائمة العمليات', () => {
    expect(navigate(ONBOARDING, { type: 'onboarding.finished' })).toEqual({
      kind: 'navigate',
      screen: LIST,
    });
  });

  it('«ابدأ الآن» من غير شاشة الترحيب لا وجهة له', () => {
    // الحدث يصف إنهاء شيءٍ معروض. إطلاقه من شاشة أخرى خطأ برمجي لا انتقال،
    // والجواب صمتٌ لا قفزة إلى القائمة.
    for (const from of [LIST, view('some.op'), LOG, SETTINGS]) {
      expect(navigate(from, { type: 'onboarding.finished' })).toEqual({ kind: 'ignore' });
    }
  });

  it('الرجوع من الترحيب لا يفعل شيئًا', () => {
    // لا شيء خلف أول شاشة يراها المستخدم.
    expect(navigate(ONBOARDING, { type: 'back' })).toEqual({ kind: 'ignore' });
  });

  it('الرجوع من الترحيب لا يُسأل عنه ولو كانت المغادرة مكلفة', () => {
    expect(navigate(ONBOARDING, { type: 'back' }, 'busy')).toEqual({ kind: 'ignore' });
  });

  /**
   * عيبٌ مرصود لا سلوكٌ مطلوب — ولذلك `it.fails`.
   *
   * تعليق `backTarget` يقول إن الترحيب «يُغادَر بابدأ الآن لا بالرجوع»، لكن
   * `destination` لا تحرس إلا الرجوع: `settings.opened` و`log.opened`
   * و`operation.selected` تُخرج من الترحيب إلى داخل التطبيق قبل إتمامه، فيبقى
   * الإعداد «لم يُتمّ» ويعود الترحيب في التشغيل القادم بلا سبب مفهوم للمستخدم.
   * الحارس الصحيح: من `onboarding` لا وجهة إلا `onboarding.finished`.
   *
   * حين يُسدّ العيب سيصير هذا الاختبار ناجحًا، فيسقط بوصفه `it.fails` — وهذا
   * هو المقصود: إشارةٌ صريحة إلى تحويله إلى `it` عاديًّا.
   */
  it.fails('الترحيب مغلق: لا يُغادَر بحدثٍ آخر', () => {
    expect(navigate(ONBOARDING, { type: 'settings.opened' })).toEqual({ kind: 'ignore' });
    expect(navigate(ONBOARDING, { type: 'log.opened' })).toEqual({ kind: 'ignore' });
    expect(navigate(ONBOARDING, { type: 'operation.selected', opId: 'x.y' })).toEqual({
      kind: 'ignore',
    });
  });
});

describe('الفتح والرجوع', () => {
  it('اختيار عملية يفتح شاشتها بمعرّفها كما وصل', () => {
    // المعرّف يمرّ نصًّا كما هو: الآلة لا تعرف الفهرس ولا تتحقّق من وجود العملية.
    expect(navigate(LIST, { type: 'operation.selected', opId: 'compress.folder.zip' })).toEqual({
      kind: 'navigate',
      screen: view('compress.folder.zip'),
    });
  });

  it('يفتح السجل والإعدادات من القائمة', () => {
    expect(navigate(LIST, { type: 'log.opened' })).toEqual({ kind: 'navigate', screen: LOG });
    expect(navigate(LIST, { type: 'settings.opened' })).toEqual({
      kind: 'navigate',
      screen: SETTINGS,
    });
  });

  it('الرجوع من شاشة العملية إلى القائمة', () => {
    expect(navigate(view('files.rename'), { type: 'back' })).toEqual({
      kind: 'navigate',
      screen: LIST,
    });
  });

  it('الرجوع من السجل وَمن الإعدادات إلى القائمة', () => {
    // القائمة هي الجذر الوحيد، فلا تتفرّع شجرةُ رجوعٍ يحفظها المستدعي.
    expect(navigate(LOG, { type: 'back' })).toEqual({ kind: 'navigate', screen: LIST });
    expect(navigate(SETTINGS, { type: 'back' })).toEqual({ kind: 'navigate', screen: LIST });
  });

  it('الرجوع من القائمة لا يفعل شيئًا: هي الجذر', () => {
    expect(navigate(LIST, { type: 'back' })).toEqual({ kind: 'ignore' });
  });

  it('إعادة عرض الترحيب تأتي من الإعدادات', () => {
    expect(navigate(SETTINGS, { type: 'onboarding.replay' })).toEqual({
      kind: 'navigate',
      screen: ONBOARDING,
    });
  });
});

describe('كلفة المغادرة', () => {
  const target: NavEvent = { type: 'back' };

  it('«لا كلفة» ينتقل مباشرة، وهو الافتراضي حين لا تُمرَّر', () => {
    expect(navigate(view('a.b'), target, 'free')).toEqual({ kind: 'navigate', screen: LIST });
    expect(navigate(view('a.b'), target)).toEqual({ kind: 'navigate', screen: LIST });
  });

  it('«حقول مملوءة» تسأل وتحمل الوجهة المعلّقة معها', () => {
    // الوجهة تعود في `pending` كي لا تحسبها الشاشة مرّةً ثانية بعد التأكيد،
    // فينفرد موضعٌ واحد بمعرفة «إلى أين».
    expect(navigate(view('a.b'), target, 'dirty')).toEqual({
      kind: 'confirm',
      reason: 'dirty',
      pending: LIST,
    });
  });

  it('«تشغيل جارٍ» تسأل بسببها هي لا بسبب الحقول', () => {
    // السبب يُعاد كي تختار الشاشة نصّ السؤال: فقدُ حقولٍ ليس كإيقاف تشغيل.
    expect(navigate(LOG, target, 'busy')).toEqual({
      kind: 'confirm',
      reason: 'busy',
      pending: LIST,
    });
  });

  it('الكلفة تُسأل عند كل حدثٍ له وجهة لا عند الرجوع وحده', () => {
    expect(navigate(view('a.b'), { type: 'settings.opened' }, 'busy')).toEqual({
      kind: 'confirm',
      reason: 'busy',
      pending: SETTINGS,
    });
    expect(navigate(view('a.b'), { type: 'operation.selected', opId: 'c.d' }, 'dirty')).toEqual({
      kind: 'confirm',
      reason: 'dirty',
      pending: view('c.d'),
    });
  });

  it('حدثٌ بلا وجهة يُهمَل مهما بلغت الكلفة', () => {
    // «لا وجهة» يسبق «كلفة»: سؤالٌ عن مغادرةٍ لن تقع سؤالٌ مربك.
    const costs: ExitCost[] = ['free', 'dirty', 'busy'];
    for (const cost of costs) {
      expect(navigate(LIST, { type: 'back' }, cost)).toEqual({ kind: 'ignore' });
      expect(navigate(LIST, { type: 'onboarding.finished' }, cost)).toEqual({ kind: 'ignore' });
    }
  });

  it('التأكيد ينقل بلا سؤالٍ ثانٍ', () => {
    const asked = navigate(view('a.b'), { type: 'back' }, 'dirty');
    expect(asked.kind).toBe('confirm');
    if (asked.kind !== 'confirm') return;
    expect(confirmNavigation(asked.pending)).toEqual({ kind: 'navigate', screen: LIST });
  });

  it('التأكيد لا يعيد فحص الكلفة: هو جواب المستخدم لا سؤال جديد', () => {
    // بعد «نعم» لا يبقى ما يُسأل عنه، ولو كان التشغيل ما زال جاريًا.
    expect(confirmNavigation(view('z.z'))).toEqual({ kind: 'navigate', screen: view('z.z') });
  });
});

describe('الانتقال إلى نفس الشاشة', () => {
  it('يُهمَل لأنه ليس انتقالًا', () => {
    expect(navigate(LOG, { type: 'log.opened' })).toEqual({ kind: 'ignore' });
    expect(navigate(SETTINGS, { type: 'settings.opened' })).toEqual({ kind: 'ignore' });
    expect(navigate(ONBOARDING, { type: 'onboarding.replay' })).toEqual({ kind: 'ignore' });
  });

  it('اختيار العملية المفتوحة نفسها يُهمَل', () => {
    expect(navigate(view('same.op'), { type: 'operation.selected', opId: 'same.op' })).toEqual({
      kind: 'ignore',
    });
  });

  it('اختيار عملية أخرى يقع فعلًا', () => {
    // القائمة تبقى خلف الشاشة، فاختيار عمليةٍ ثانية منها انتقالٌ حقيقي.
    expect(navigate(view('same.op'), { type: 'operation.selected', opId: 'other.op' })).toEqual({
      kind: 'navigate',
      screen: view('other.op'),
    });
  });

  it('لا يُسأل عن فقد حالةٍ لن تُفقد', () => {
    // الشاشة نفسها لا تُغادَر، فالكلفة لا معنى لها هنا مهما كانت.
    expect(navigate(SETTINGS, { type: 'settings.opened' }, 'busy')).toEqual({ kind: 'ignore' });
    expect(navigate(view('same.op'), { type: 'operation.selected', opId: 'same.op' }, 'dirty'))
      .toEqual({ kind: 'ignore' });
  });
});

describe('sameScreen', () => {
  it('اسمان مختلفان ليسا شاشة واحدة', () => {
    expect(sameScreen(LIST, SETTINGS)).toBe(false);
    expect(sameScreen(ONBOARDING, LOG)).toBe(false);
    expect(sameScreen(LIST, view('a.b'))).toBe(false);
  });

  it('الشاشات بلا حمولة تتساوى باسمها وحده', () => {
    for (const screen of EVERY_SCREEN) {
      expect(sameScreen(screen, screen)).toBe(true);
    }
    expect(sameScreen(LIST, { name: 'operations-list' })).toBe(true);
  });

  it('شاشة العملية تتساوى بمعرّفها لا باسمها', () => {
    expect(sameScreen(view('a.b'), view('a.b'))).toBe(true);
    expect(sameScreen(view('a.b'), view('a.c'))).toBe(false);
  });

  it('المقارنة متماثلة الطرفين', () => {
    // لو اعتمدت على `a` وحدها في فحص النوع لاختلّ أحد الاتجاهين.
    expect(sameScreen(view('a.b'), view('a.c'))).toBe(sameScreen(view('a.c'), view('a.b')));
    expect(sameScreen(LIST, view('a.b'))).toBe(sameScreen(view('a.b'), LIST));
  });
});
