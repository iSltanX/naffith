// @vitest-environment node
/**
 * وعاء الصفحة يعطي حشوةً وعرضًا أقصى — لا صنفًا فارغًا.
 *
 * `app.test.tsx` يثبت أن كل شاشة تُلفّ بـ`.page`؛ وهذا يثبت أن اللفّ يعني
 * شيئًا. الاثنان معًا يسدّان الطريقين إلى العطل نفسه: شاشةٌ خارج الوعاء،
 * ووعاءٌ فقد ما يجعله وعاءً. ولا يكفي أحدهما — أنماط الملفّات لا تُحمَّل في
 * jsdom، فاختبار العرض يرى البنية ولا يرى قيمة حشوةٍ واحدة.
 *
 * لذلك يُقرأ الملف نصًّا لا يُستورد. وبيئة الاختبار `node` لأن `jsdom` يعيد
 * كتابة أصل `import.meta.url` إلى عنوان http فيصير المسار من جذر القرص.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const SHELL = readFileSync(new URL('./app-shell.css', import.meta.url).pathname, 'utf8');
const OPERATION = readFileSync(
  new URL('./operation-layout.css', import.meta.url).pathname,
  'utf8',
);
const TOKENS = readFileSync(
  new URL('./design-system/tokens.css', import.meta.url).pathname,
  'utf8',
);
const TAURI = JSON.parse(
  readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url).pathname, 'utf8'),
) as {
  app: {
    windows: Array<{
      minWidth: number;
      minHeight: number;
      titleBarStyle: string;
      trafficLightPosition?: { x: number; y: number };
    }>;
  };
};

function tokenPx(name: string): number {
  const value = TOKENS.match(new RegExp(`${name}:\\s*(\\d+)px`))?.[1];
  expect(value, `الرمز ${name} بلا قيمة px`).toBeTruthy();
  return Number(value);
}

/** جسم قاعدةٍ واحدة بمحدّدها الحرفي. */
function rule(css: string, selector: string): string {
  const head = `${selector} {`;
  const at = css.indexOf(head);
  expect(at, `القاعدة ${selector} غير موجودة`).toBeGreaterThan(-1);
  const body = css.slice(at + head.length);
  return body.slice(0, body.indexOf('}'));
}

describe('وعاء الصفحة', () => {
  it('يثبّت الحد الأدنى وإشارات macOS على اليسار الفيزيائي', () => {
    const window = TAURI.app.windows[0];
    expect(window).toBeTruthy();
    expect(window?.minWidth).toBe(680);
    expect(window?.minHeight).toBe(520);
    expect(window?.titleBarStyle).toBe('Overlay');
    // `trafficLightPosition` غائبٌ عمدًا — انظر «إشارات النافذة الأصلية»
    // أدناه لسبب غيابه، لا حضوره.
    expect(window?.trafficLightPosition).toBeUndefined();
  });

  it('يحمل حشوة المحتوى وسقف القراءة والتوسيط', () => {
    const body = rule(SHELL, '.page');
    // الحشوة هي ما كان غائبًا عن الشاشات الثلاث: بلا هذه القيم يعود العنوان
    // ملاصقًا للحافة وتُقصّ حلقةُ التركيز على البطاقة المحاذية لها.
    expect(body, 'وعاء بلا حشوة ليس وعاءً').toMatch(/padding:\s*var\(--space-/);
    expect(body).toMatch(/max-width:\s*1120px/);
    expect(body).toMatch(/margin-inline:\s*auto/);
  });

  it('لا يُكتب مرّتين: الوعاء مِلك الغلاف لا مِلك شاشة «نَفِّذ»', () => {
    // نسخةٌ ثانية في تخطيط العملية هي ما وقع أصلًا: شاشةٌ واحدة تملك الحشوة
    // وأخواتها بلا شيء. الصنف الذي كان يحملها هناك اسمه `.app`.
    expect(OPERATION).not.toMatch(/^\.app\s*\{/m);
    expect(OPERATION).not.toMatch(/^\.page\s*\{/m);
  });

  it('يضبط نافذة 680px إلى محتوى 576px وعنوان y=51', () => {
    const minimumStart = SHELL.indexOf('@media (max-width: 759px)');
    const minimumEnd = SHELL.indexOf('@media (prefers-reduced-motion: reduce)', minimumStart);
    const minimum = SHELL.slice(minimumStart, minimumEnd);
    expect(minimum).toMatch(
      /\.page\s*\{[^}]*padding:\s*calc\(var\(--space-14\) \+ var\(--border-hairline\)\) var\(--space-24\)\s*var\(--space-24\)/,
    );

    const titleY =
      tokenPx('--titlebar-h') + tokenPx('--space-14') + tokenPx('--border-hairline');
    const contentWidth =
      680 - tokenPx('--sidebar-w-collapsed') - 2 * tokenPx('--space-24');
    expect(titleY).toBe(51);
    expect(contentWidth).toBe(576);
  });

  it('يترك 768px داخل نافذة 1080px للتركيب الافتراضي', () => {
    const body = rule(SHELL, '.page');
    expect(body).toMatch(
      /padding:\s*var\(--space-20\) var\(--space-40\) var\(--space-40\)/,
    );
    const contentWidth = 1080 - tokenPx('--sidebar-w') - 2 * tokenPx('--space-40');
    expect(contentWidth).toBe(768);
  });
});

describe('الإيقاع الرأسي للتنقّل', () => {
  it('يستخدم سطح الاختيار ونص Ember مع مؤشّر الصفحة', () => {
    const selected = rule(SHELL, ".app-nav__item[aria-current='page']");
    expect(selected).toMatch(/background:\s*var\(--bg-selected\)/);
    expect(selected).toMatch(/border-inline-start-color:\s*var\(--action-primary\)/);
    expect(selected).toMatch(/color:\s*var\(--text-accent\)/);
    expect(selected).toMatch(/font-weight:\s*var\(--fw-bold\)/);
  });

  // 14.3 يعلن `State=Active` لصيغ عنصر التنقّل الأربع (موسَّع/شريط ×
  // رئيسي/فئة). وهي حالةٌ واحدة في الورقة لأن الصيغ الأربع تتقاسم `.app-nav__item`.
  it('يعلن حالة الضغط بسطح --bg-active لكل صيغ عنصر التنقّل', () => {
    expect(rule(SHELL, '.app-nav__item:active')).toMatch(
      /background:\s*var\(--bg-active\)/,
    );
  });

  // الضغط لحظيّ: يغيّر السطح ولا يعيد رسم هوية العنصر الحالي. ولأن خصوصيته
  // تساوي خصوصية `[aria-current='page']`، فترتيبُه بعده هو ما يجعله يفوز.
  it('يعلن الضغط بعد الحالي والسالف، وبـbackground وحده', () => {
    const pressed = rule(SHELL, '.app-nav__item:active');
    expect(pressed).not.toMatch(/color:/);
    expect(pressed).not.toMatch(/font-weight:/);
    expect(SHELL.indexOf('.app-nav__item:active')).toBeGreaterThan(
      SHELL.indexOf('.app-nav__item.is-ancestor'),
    );
  });

  it('يضع المكتبة عند y=88 وأول فئة عند y=140 في الشريطين', () => {
    expect(rule(SHELL, '.app-nav')).toMatch(
      /padding:\s*var\(--space-20\) var\(--space-12\) var\(--space-24\)/,
    );
    expect(rule(SHELL, '.app-nav__primary')).toMatch(
      /margin-block-start:\s*var\(--space-12\)/,
    );
    expect(rule(SHELL, '.app-nav__categories')).toMatch(
      /padding-block-start:\s*var\(--space-8\)/,
    );

    const libraryY = tokenPx('--space-20') + tokenPx('--space-56') + tokenPx('--space-12');
    const categoryY = libraryY + tokenPx('--control-h-nav') + tokenPx('--space-8');
    expect(libraryY).toBe(88);
    expect(categoryY).toBe(140);
  });

  it('يثبّت الإعدادات على 24px والسجل على 80px بفجوة 12px', () => {
    expect(rule(SHELL, '.app-nav__utility')).toMatch(/gap:\s*var\(--space-12\)/);
    const settingsBottom = tokenPx('--space-24');
    const logBottom = settingsBottom + tokenPx('--control-h-nav') + tokenPx('--space-12');
    expect(settingsBottom).toBe(24);
    expect(logBottom).toBe(80);
  });

  it('يحافظ على حدّ الشريط المضغوط 959/960 وحشوة القاع نفسها', () => {
    const compactStart = SHELL.indexOf('@media (max-width: 959px)');
    const compactEnd = SHELL.indexOf('@media (max-width: 759px)', compactStart);
    const compact = SHELL.slice(compactStart, compactEnd);
    expect(compact).toMatch(
      /\.app-frame\s*\{[^}]*var\(--sidebar-w-collapsed\)/,
    );
    expect(compact).toMatch(
      /\.app-nav\s*\{[^}]*padding:\s*var\(--space-20\) var\(--space-4\) var\(--space-24\)/,
    );
    expect(rule(SHELL, '.app-frame')).toMatch(/var\(--sidebar-w\)/);
  });
});

describe('حوار Page 18 عند النافذة الدنيا', () => {
  it('يبقي القرارين 180×40 جنبًا إلى جنب عند 680px', () => {
    const actions = rule(SHELL, '.dialog__actions');
    const button = rule(SHELL, '.dialog__actions .btn');
    expect(actions).toMatch(/grid-template-columns:\s*repeat\(2, 180px\)/);
    expect(actions).toMatch(/gap:\s*var\(--space-12\)/);
    expect(actions).toMatch(/justify-content:\s*center/);
    expect(button).toMatch(/width:\s*180px/);
    expect(button).toMatch(/height:\s*var\(--control-h-md\)/);
    expect(tokenPx('--control-h-md')).toBe(40);

    const minimumStart = SHELL.indexOf('@media (max-width: 759px)');
    const minimumEnd = SHELL.indexOf('@media (prefers-reduced-motion: reduce)', minimumStart);
    const minimum = SHELL.slice(minimumStart, minimumEnd);
    expect(minimum).not.toMatch(/\.dialog__actions[^}]*grid-template-columns/);
  });
});

describe('تلميح شريط التنقّل المضغوط', () => {
  const compactStart = SHELL.indexOf('@media (max-width: 959px)');
  const compactEnd = SHELL.indexOf('@media (max-width: 759px)', compactStart);
  const compact = SHELL.slice(compactStart, compactEnd);

  it('يفتح تسميةً مصمّمةً عند التحويم والتركيز', () => {
    expect(compactStart).toBeGreaterThan(-1);
    expect(compactEnd).toBeGreaterThan(compactStart);
    expect(compact).toMatch(/\.app-nav__item:hover \.app-nav__tooltip,/);
    expect(compact).toMatch(/\.app-nav__item:focus \.app-nav__tooltip\s*\{/);
    expect(compact).toMatch(/visibility:\s*visible/);
    expect(compact).toMatch(/opacity:\s*1/);
  });

  it('يفتح إلى يسار الشريط الأيمن ويحمل سطح الهوية لا سطح المتصفّح', () => {
    const tooltip = compact.match(/\.app-nav__tooltip\s*\{([^}]*)}/)?.[1] ?? '';
    expect(tooltip).not.toBe('');
    expect(tooltip).toMatch(/inset-inline-start:\s*calc\(100% \+ var\(--space-8\)\)/);
    expect(tooltip).toMatch(/background:\s*var\(--bg-inverse\)/);
    expect(tooltip).toMatch(/color:\s*var\(--text-on-inverse\)/);
    expect(tooltip).toMatch(/z-index:\s*var\(--z-tooltip\)/);
  });

  it('يحافظ على دعم WebKit في macOS 12', () => {
    expect(SHELL).not.toMatch(/:has\(|color-mix\(|:focus-visible/);
  });
});

/**
 * إشارات macOS تُترك لموضعها الأصلي — لا تُعاد بـ`trafficLightPosition`.
 *
 * ‏`tao` (وراء Tauri) يبني موضع الأزرار الثلاثة بضربٍ متزايد:
 * `x + i * (miniaturize.x - close.x)`. وتحت تخطيط RTL يصير ذلك الفارق
 * **سالبًا**، فيمشي الحساب يسارًا بدل يمين: زرّ التكبير يخرج عن النافذة
 * كليًا، والتصغير يُقصّ عند الحافة، والإغلاق وحده يظهر — وهذا عطلٌ رآه هذا
 * المشروع فعلًا حين كان `trafficLightPosition` مضبوطًا (جُرِّب `{x:12,y:10}`
 * ثم `{x:20,y:12}` لمحاولة توسيط الأزرار، وكلاهما وقع في العطل نفسه).
 *
 * والإصلاح الوحيد الصحيح لهذا هو **عدم إعادة تعريف الموضع أصلًا**: بلا
 * `trafficLightPosition`، تضع macOS الأزرار في موضعها القياسي بنفسها بلا
 * المرور بحساب `tao` المكسور تحت RTL. فالاختبار هنا حارسٌ على الغياب، لا على
 * قيمة — أي عودةٍ لهذا المفتاح تُعيد العطل ولو بأرقامٍ مختلفة.
 */
describe('إشارات النافذة الأصلية', () => {
  it('لا يُعاد تعريف موضع الإشارات — تُترك لتوسيط macOS القياسي', () => {
    expect(
      TAURI.app.windows[0]?.trafficLightPosition,
      'إعادة `trafficLightPosition` تحت RTL تكسر ترتيب الأزرار — انظر توثيق الوصف أعلاه',
    ).toBeUndefined();
  });
});
