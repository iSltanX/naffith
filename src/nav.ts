/**
 * آلة حالات التنقّل.
 *
 * ## لماذا آلة حالات لا مجموعة رايات
 *
 * ثلاث رايات منطقية (`showOnboarding`، `selectedOp`، `showSettings`) تصف ثماني
 * تركيبات، ستٌّ منها لا معنى لها — «ترحيبٌ وعمليةٌ مختارة معًا» ليست شاشة.
 * النوع هنا يجعل تلك الست غير قابلة للتعبير عنها أصلًا، فلا يبقى شرطٌ في
 * الشاشة يسأل «وماذا لو الاثنان معًا؟».
 *
 * ## لماذا الانتقال دالّة نقيّة تعيد قرارًا لا شاشة
 *
 * «رجوع» أثناء تشغيلٍ نشط ليس انتقالًا ولا رفضًا: هو سؤال. لو أعادت الدالة
 * شاشةً لكان على المستدعي أن يفحص حالة التشغيل بنفسه قبل الاستدعاء وبعده،
 * فينتقل قرارُ السلامة إلى موضعين. هنا القرار واحد: الدالة تعرف كلفة المغادرة
 * فتعيد إمّا انتقالًا وإمّا طلبَ تأكيد، والشاشة تعرض السؤال ولا تخترعه.
 */

export type Screen =
  | { name: 'onboarding' }
  /** شاشة الفئات: جذر المكتبة. */
  | { name: 'operations-list' }
  /** قسمٌ مفتوح، وعملياته معروضة. */
  | { name: 'category-view'; categoryId: string }
  | { name: 'operation-view'; opId: string }
  | { name: 'run-log' }
  | { name: 'settings' };

export type ScreenName = Screen['name'];

/**
 * كلفة مغادرة الشاشة الحالية.
 *
 * تُحسب في الشاشة التي تملك الحالة، وتُمرَّر إلى الآلة. الآلة لا تعرف حقول
 * النموذج ولا دورة حياة التشغيل، وهذا مقصود: إضافة حقلٍ غدًا لا تلمس هذا الملف.
 */
export type ExitCost =
  /** لا شيء يُفقد بالمغادرة. */
  | 'free'
  /** حقول مملوءة لم تُنفَّذ. تُفقد بالمغادرة، فيُسأل المستخدم. */
  | 'dirty'
  /** تشغيلٌ جارٍ. المغادرة قرارٌ لا انزلاق. */
  | 'busy';

export type NavEvent =
  /** «ابدأ الآن». */
  | { type: 'onboarding.finished' }
  | { type: 'category.selected'; categoryId: string }
  | { type: 'operation.selected'; opId: string }
  | { type: 'log.opened' }
  | { type: 'settings.opened' }
  | { type: 'back' }
  /** الرجوع إلى جذر المكتبة مهما بعُدت الشاشة. */
  | { type: 'library.opened' }
  /** إعادة عرض الترحيب من الإعدادات. */
  | { type: 'onboarding.replay' };

export type NavResult =
  | { kind: 'navigate'; screen: Screen }
  /** المغادرة مكلفة: تُعرض على المستخدم ثم تُعاد بـ `confirmNavigation`. */
  | { kind: 'confirm'; reason: Exclude<ExitCost, 'free'>; pending: Screen }
  /** لا وجهة لهذا الحدث من هذه الشاشة. لا صوت ولا أثر. */
  | { kind: 'ignore' };

/**
 * الشاشة الأولى.
 *
 * أول تشغيل يبدأ من الترحيب، وما بعده من قائمة العمليات. لا شرط ثالث: تعذّرُ
 * قراءة الإعداد يُعامَل «لم يُتمّ بعد» في `settings.ts` فيصل هنا قرارًا واحدًا.
 */
export function initialScreen(showOnboarding: boolean): Screen {
  return showOnboarding ? { name: 'onboarding' } : { name: 'operations-list' };
}

/**
 * إلى أين يعود «رجوع» من كل شاشة. `null` يعني «لا شيء خلفها».
 *
 * ## لماذا العودة من شاشة العملية تحتاج سياقًا
 *
 * المسار المقصود هو: الفئات ← القسم ← العملية ← رجوع. لكن العملية تُفتح من
 * موضعين آخرين كذلك — من نتائج البحث، ومن «المستخدَمة حديثًا» — وكلاهما في
 * شاشة الفئات لا في قسم. فعودةٌ ثابتة إلى القسم كانت ستُنزل من جاء من البحث
 * في شاشةٍ لم يرها، وعودةٌ ثابتة إلى الفئات كانت تُخرج من دخل من قسمٍ منه.
 *
 * `origin` هو الجواب: الشاشة التي فُتحت العملية منها، تُمرَّر من `App` الذي
 * يعرفها. وهي دائمًا الفئات أو قسمٌ فيها، فلا يمكن أن تصير سلسلةً تنمو.
 */
function backTarget(from: Screen, origin: Screen | null): Screen | null {
  switch (from.name) {
    case 'operation-view':
      // الرجوع إلى ما جاء منه، أو إلى الجذر إن لم يُعرف — لا إلى العدم.
      return origin && origin.name !== 'operation-view' ? origin : { name: 'operations-list' };
    case 'category-view':
    case 'run-log':
    case 'settings':
      return { name: 'operations-list' };
    // شاشة الفئات هي الجذر، والترحيب يُغادَر بـ«ابدأ الآن» لا بالرجوع.
    case 'operations-list':
    case 'onboarding':
      return null;
  }
}

/** الوجهة المطلوبة لحدثٍ ما، بصرف النظر عن كلفة المغادرة. */
function destination(current: Screen, event: NavEvent, origin: Screen | null): Screen | null {
  // الترحيب مغلق: «ابدأ الآن» وحده يُغادره. حدثٌ آخر يُخرج منه إلى داخل التطبيق
  // قبل إتمامه يترك الإعداد «لم يُتمّ»، فيعود الترحيب في التشغيل القادم بلا سبب
  // يفهمه المستخدم. الحارس هنا لا في الشاشات كي لا يُنسى في واحدةٍ منها.
  if (current.name === 'onboarding' && event.type !== 'onboarding.finished') return null;

  switch (event.type) {
    case 'onboarding.finished':
      // الترحيب وحده يُنهى. الحدث نفسه من شاشة أخرى لا معنى له.
      return current.name === 'onboarding' ? { name: 'operations-list' } : null;
    case 'category.selected':
      return { name: 'category-view', categoryId: event.categoryId };
    case 'operation.selected':
      return { name: 'operation-view', opId: event.opId };
    case 'library.opened':
      return { name: 'operations-list' };
    case 'log.opened':
      return { name: 'run-log' };
    case 'settings.opened':
      return { name: 'settings' };
    case 'onboarding.replay':
      return { name: 'onboarding' };
    case 'back':
      return backTarget(current, origin);
  }
}

/**
 * الانتقال.
 *
 * `exitCost` كلفة مغادرة `current` لا كلفة الوصول. الشاشة التي لا تملك حالة
 * تمرّر `'free'` دائمًا.
 */
export function navigate(
  current: Screen,
  event: NavEvent,
  exitCost: ExitCost = 'free',
  origin: Screen | null = null,
): NavResult {
  const next = destination(current, event, origin);
  if (!next) return { kind: 'ignore' };

  // الانتقال إلى نفس الشاشة نفسها ليس انتقالًا، ولا يستحق سؤالًا عن فقد حالة
  // لن تُفقد. اختيارُ العملية المفتوحة من قائمةٍ خلفها يقع فعلًا.
  if (sameScreen(current, next)) return { kind: 'ignore' };

  if (exitCost !== 'free') {
    return { kind: 'confirm', reason: exitCost, pending: next };
  }
  return { kind: 'navigate', screen: next };
}

/** تأكيد المستخدم بعد `confirm`. ينقل بلا سؤالٍ ثانٍ. */
export function confirmNavigation(pending: Screen): NavResult {
  return { kind: 'navigate', screen: pending };
}

export function sameScreen(a: Screen, b: Screen): boolean {
  if (a.name !== b.name) return false;
  if (a.name === 'operation-view' && b.name === 'operation-view') return a.opId === b.opId;
  if (a.name === 'category-view' && b.name === 'category-view') {
    return a.categoryId === b.categoryId;
  }
  return true;
}

/**
 * هويّة الشاشة نصًّا.
 *
 * الشاشة كائنٌ يُبنى من جديد في كل انتقال، فمقارنتُه بمرجعه تعلن تبدّلًا حيث
 * لا تبدّل. والمعرّف داخلٌ فيه لأن الانتقال من قسمٍ إلى آخر انتقالُ شاشةٍ كامل
 * وإن بقي الاسم واحدًا.
 *
 * هنا لا في `app.tsx`: موضع التمرير والبؤرة كلاهما يُفهرَس بهذا المفتاح، وقد
 * كان محسوبًا في موضعين — فشاشةٌ تُضاف تُنسى في أحدهما.
 */
export function screenKey(screen: Screen): string {
  if (screen.name === 'operation-view') return `${screen.name}:${screen.opId}`;
  if (screen.name === 'category-view') return `${screen.name}:${screen.categoryId}`;
  return screen.name;
}
