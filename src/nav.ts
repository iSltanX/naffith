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
  | { name: 'operations-list' }
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
  | { type: 'operation.selected'; opId: string }
  | { type: 'log.opened' }
  | { type: 'settings.opened' }
  | { type: 'back' }
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

/** إلى أين يعود «رجوع» من كل شاشة. `null` يعني «لا شيء خلفها». */
function backTarget(from: Screen): Screen | null {
  switch (from.name) {
    case 'operation-view':
    case 'run-log':
    case 'settings':
      return { name: 'operations-list' };
    // قائمة العمليات هي الجذر، والترحيب يُغادَر بـ«ابدأ الآن» لا بالرجوع.
    case 'operations-list':
    case 'onboarding':
      return null;
  }
}

/** الوجهة المطلوبة لحدثٍ ما، بصرف النظر عن كلفة المغادرة. */
function destination(current: Screen, event: NavEvent): Screen | null {
  switch (event.type) {
    case 'onboarding.finished':
      // الترحيب وحده يُنهى. الحدث نفسه من شاشة أخرى لا معنى له.
      return current.name === 'onboarding' ? { name: 'operations-list' } : null;
    case 'operation.selected':
      return { name: 'operation-view', opId: event.opId };
    case 'log.opened':
      return { name: 'run-log' };
    case 'settings.opened':
      return { name: 'settings' };
    case 'onboarding.replay':
      return { name: 'onboarding' };
    case 'back':
      return backTarget(current);
  }
}

/**
 * الانتقال.
 *
 * `exitCost` كلفة مغادرة `current` لا كلفة الوصول. الشاشة التي لا تملك حالة
 * تمرّر `'free'` دائمًا.
 */
export function navigate(current: Screen, event: NavEvent, exitCost: ExitCost = 'free'): NavResult {
  const next = destination(current, event);
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
  return true;
}
