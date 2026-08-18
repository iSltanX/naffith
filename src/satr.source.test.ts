// @vitest-environment node
/**
 * لوحة «سَطْر»: تطوي عرضها حين لا محتوى، ويبقى شريطها مقروءًا.
 *
 * ## العقد المحروس هنا
 *
 * 1. **الطيّ حقيقيّ.** عرضُ اللوحة المطويّة قيمةٌ مطلقة صغيرة، والمتوسّعة نسبةٌ
 *    من الصفّ في نطاق ‎30–35%‎. الفرق بينهما لا يقلّ عن الضِعف: طيٌّ يوفّر ‎20%‎ من
 *    العرض ليس طيًّا بل تضييقًا، وهو ما وقع ثلاث مرّات قبل هذا التنفيذ.
 * 2. **الانتقال من رموز الحركة.** مدّةٌ من النظام لا رقمٌ مكتوب، فتنهار إلى
 *    ‎1ms‎ تحت `prefers-reduced-motion` بلا سطرٍ إضافي في الملف.
 * 3. **الشريط مقروء.** نصوصه تبلغ حدّ AA على أرضية اللوحة في الوضعين، بلا
 *    شفافيةٍ تخفض الحبر قبل أن يصل إلى العين.
 *
 * ## ما سقط من هذا الملف، ولماذا صار أشدّ لا أهون
 *
 * كان يحرس **كتابةً مائية**: شفافيةٌ ‎≤ 0.7‎ ووزنٌ خفيف وعنوانٌ ‎≥ 24px‎ يستحقّ حدّ
 * النصّ الكبير (‏3:1) بدل حدّ المتن (‏4.5). وقد سقطت الكتابة المائية نفسها: هي
 * كانت محتوًى بديلًا يشغل مكان الغائب، والعلاج ألّا يُحجز مكان.
 *
 * وما حلّ محلّه أضيق:
 *
 * - **حدّ ‎4.5‎ لكل نصوص الشريط** بلا استثناءٍ للنصّ الكبير: لم يبقَ فيه نصٌّ كبير.
 * - **بلا شفافية**: كانت ‎0.65‎ تضرب الحبر قبل العين فيُقاس المزيج لا الرمز. صار
 *   الشريط عنصر واجهةٍ بدرجاته كاملة، فالقياس على الرمز مباشرةً — وهو أعلى.
 * - **الأرضية تُقرأ من قاعدة اللوحة** لا مكتوبةً هنا. كان الملف يقيس على
 *   `--bg-canvas` بينما أرضية اللوحة `--bg-sunken` فعلًا، فيحرس تباينًا على سطحٍ
 *   لا وجود له في الشاشة.
 * - **والطيّ نفسه محروس** — وهو الشرط الذي لم يكن محروسًا أصلًا فسقط ثلاث مرّات.
 *
 * وبيئته `node` لأن الأنماط لا تُحمَّل في jsdom أصلًا: لا يوجد في شجرة الاختبار
 * لونٌ محسوب يُسأل عنه، فالمصدر هو المرجع الوحيد.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const OPERATION = readFileSync(
  new URL('./operation-layout.css', import.meta.url).pathname,
  'utf8',
);
const TOKENS = readFileSync(
  new URL('./design-system/tokens.css', import.meta.url).pathname,
  'utf8',
);

/** حدّ AA لنصّ المتن. لا شيء في هذه الكتلة يستحقّ استثناء النصّ الكبير. */
const AA_BODY = 4.5;

/** جسم قاعدةٍ واحدة بمحدّدها الحرفي. */
function rule(css: string, selector: string): string {
  const at = css.indexOf(`\n${selector} {`);
  expect(at, `القاعدة ${selector} غير موجودة`).toBeGreaterThan(-1);
  const body = css.slice(at + selector.length + 3);
  return body.slice(0, body.indexOf('}'));
}

/** تصريحات كتلةٍ ما، رمزًا وقيمة. */
function declarations(body: string): Map<string, string> {
  const out = new Map<string, string>();
  for (const [, name, value] of body.matchAll(/(--[\w-]+):\s*([^;]+);/g)) {
    if (name && value) out.set(name, value.trim());
  }
  return out;
}

/** قيمة رمزٍ بعد تتبّع سلسلة `var()` إلى منتهاها. */
function resolve(vars: Map<string, string>, name: string): string {
  let value = vars.get(name);
  for (let hop = 0; hop < 8; hop += 1) {
    if (value === undefined) break;
    const ref = /^var\((--[\w-]+)\)$/.exec(value.trim());
    if (!ref?.[1]) return value.trim();
    value = vars.get(ref[1]);
  }
  throw new Error(`تعذّر حلّ الرمز ${name}`);
}

/** اسم الرمز في تصريحٍ من شكل `prop: var(--token)`. */
function tokenOf(body: string, property: string): string {
  const found = new RegExp(`${property}:\\s*var\\((--[\\w-]+)\\)`).exec(body);
  expect(found?.[1], `${property} ليست رمزًا من النظام`).toBeTruthy();
  return found?.[1] ?? '';
}

type Rgb = readonly [number, number, number];

function rgb(hex: string): Rgb {
  const clean = hex.trim().replace('#', '');
  expect(clean, `لون غير سداسي: ${hex}`).toMatch(/^[0-9a-fA-F]{6}$/);
  return [
    parseInt(clean.slice(0, 2), 16),
    parseInt(clean.slice(2, 4), 16),
    parseInt(clean.slice(4, 6), 16),
  ];
}

// كان هنا `composite`: تمزج الحبر بأرضيته بنسبة الشفافية، لأن الكتابة المائية
// كانت تُضرب في ٠٫٦٥ قبل أن تصل إلى العين فيُقاس المزيج لا الرمز. وقد سقطت مع
// الشفافية نفسها — انظر «بلا شفافيةٍ تخفض الحبر قبل العين» أدناه — فالقياس الآن
// على الرمز مباشرةً، وهو أعلى ممّا كان.

function luminance([r, g, b]: Rgb): number {
  const channel = (raw: number): number => {
    const c = raw / 255;
    return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function contrast(a: Rgb, b: Rgb): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return ((hi ?? 0) + 0.05) / ((lo ?? 0) + 0.05);
}

/** ‏px من رمز مقاسٍ في النظام. */
function px(vars: Map<string, string>, token: string): number {
  return Number.parseFloat(resolve(vars, token));
}

// الوضعان: الأساس في `:root`، والليلي يغلبه بما يعيد تعريفه. والمُقروء هنا هو
// كتلة `[data-theme="dark"]` الصريحة لا الاستعلام، وهما نسختان متطابقتان.
const LIGHT = declarations(rule(TOKENS, ':root'));
const DARK = new Map([...LIGHT, ...declarations(rule(TOKENS, ':root[data-theme="dark"]'))]);
const MODES: ReadonlyArray<readonly [string, Map<string, string>]> = [
  ['النهاري', LIGHT],
  ['الليلي', DARK],
];

const PANES = rule(OPERATION, '.op__panes');
const PANEL = rule(OPERATION, '.satr');
const LIVE = rule(OPERATION, '.satr--live');
const RAIL = rule(OPERATION, '.satr__rail');
const NAME = rule(OPERATION, '.satr__rail-name');
const STATE = rule(OPERATION, '.satr__rail-state');

/** رمز أرضية اللوحة، مقروءًا من قاعدتها لا مكتوبًا هنا. */
const GROUND_TOKEN = tokenOf(PANEL, 'background');

/** قيمة رمزٍ معرَّفٍ في جسم قاعدةٍ ما (‏`--satr-w-folded` وأختها). */
function localVar(body: string, name: string): string {
  const found = new RegExp(`${name}:\\s*([^;]+);`).exec(body);
  expect(found?.[1], `${name} غير معرَّف`).toBeTruthy();
  return (found?.[1] ?? '').trim();
}

describe('اللوحة تطوي عرضها', () => {
  it('عرضُ المطويّة يساوي 96px من سلّم الهوية', () => {
    // نسبةٌ للمطويّة تجعلها تكبر مع النافذة، فتعود المساحة المحجوزة من بابٍ
    // آخر. والقيمة محسوبةٌ من رمزٍ في النظام لا رقمًا مكتوبًا.
    const folded = localVar(PANES, '--satr-w-folded');
    expect(folded).toContain('var(--sidebar-w-collapsed)');
    expect(folded, 'عرضُ المطويّة يقرأ الصفّ أو النافذة').not.toMatch(/%|vw|vh/);
    const foldedPx = [...folded.matchAll(/var\((--[\w-]+)\)/g)]
      .reduce((total, match) => total + px(LIGHT, match[1] as string), 0);
    expect(foldedPx).toBe(96);
    expect(PANEL).toContain('inline-size: var(--satr-w-folded)');
  });

  it('عرضُ المتوسّعة ثابت 328px كما في Page 16', () => {
    const live = localVar(PANES, '--satr-w-live');
    expect(live, 'عرضُ المتوسّعة يقرأ الصفّ أو النافذة').not.toMatch(/%|vw|vh/);
    const livePx = [...live.matchAll(/var\((--[\w-]+)\)/g)]
      .reduce((total, match) => total + px(LIGHT, match[1] as string), 0);
    expect(livePx).toBe(328);
    expect(LIVE).toContain('inline-size: var(--satr-w-live)');
  });

  it('الفرق بين الحالتين طيٌّ لا تضييق', () => {
    const value = (name: string) => [...localVar(PANES, name).matchAll(/var\((--[\w-]+)\)/g)]
      .reduce((total, match) => total + px(LIGHT, match[1] as string), 0);
    expect(value('--satr-w-live')).toBeGreaterThanOrEqual(value('--satr-w-folded') * 3);
  });

  it('الانتقال بمدّةٍ من رموز الحركة، فيُطاع تقليلُ الحركة', () => {
    // رقمٌ مكتوب هنا يفلت من انهيار المدد إلى ‎1ms‎ في `tokens.css`، فتبقى
    // حركةٌ لمن طلب ألّا تكون.
    expect(PANEL).toMatch(/transition:\s*inline-size var\(--duration-[\w-]+\)/);
    expect(PANEL).not.toMatch(/transition:[^;]*\d+ms/);
  });
});

describe('علاقة اللوحتين تستجيب لنافذة المنتج', () => {
  it('تتكدّسان تحت 900px وتبقيان منقسمتين عند 900px', () => {
    const compactAt = OPERATION.indexOf('@media (max-width: 899px)');
    expect(compactAt, 'قاعدة التكديس تحت 900px غير موجودة').toBeGreaterThan(-1);
    const compact = OPERATION.slice(compactAt);

    expect(compact).toMatch(/\.op__panes\s*\{[^}]*flex-direction:\s*column/s);
    expect(compact).toMatch(/\.op__panes > \.naffith\s*\{[^}]*flex:\s*none/s);
    expect(compact).toMatch(/\.satr,\s*\.satr--live\s*\{[^}]*inline-size:\s*100%/s);
    expect(OPERATION).not.toContain('@media (max-width: 900px)');

    // قاعدة الحالة الضيقة تأتي بعد قواعد الانقسام، وإلا يغلبها ترتيب المصدر.
    expect(compactAt).toBeGreaterThan(OPERATION.lastIndexOf('\n.satr {'));
  });
});

describe('الشريط المطويّ يبقى مقروءًا', () => {
  it('لا يعيد رقعة أيقونة إلى حالة «لا أمر بعد»', () => {
    expect(OPERATION).not.toContain('.satr__rail-glyph');
  });

  it('يقيس على أرضية اللوحة نفسها', () => {
    // حارسٌ على الاختبار: كان يقيس على `--bg-canvas` مكتوبًا هنا بينما أرضية
    // اللوحة `--bg-sunken`، فكان يحرس تباينًا على سطحٍ لا وجود له في الشاشة.
    expect(GROUND_TOKEN, 'أرضية اللوحة ليست رمزًا من النظام').toMatch(/^--bg-/);
  });

  it('بلا شفافيةٍ تخفض الحبر قبل العين', () => {
    // الشفافية كانت لازمةً لكتابةٍ مائية تحتلّ نصف الشاشة. وشريطٌ بعرض ‎96px‎
    // عنصرُ واجهة: يأخذ درجته من النظام كاملة، ويُقاس عليها مباشرةً.
    expect(RAIL).not.toMatch(/opacity:/);
  });

  it('بلا لونٍ خام: كل قيمة رمزٌ من النظام', () => {
    for (const body of [RAIL, NAME, STATE]) {
      expect(body).not.toMatch(/#[0-9a-fA-F]{3,8}/);
      expect(body).not.toMatch(/\brgba?\(/);
    }
  });

  for (const [mode, vars] of MODES) {
    const ground = rgb(resolve(vars, GROUND_TOKEN));
    const ink = (body: string): Rgb => rgb(resolve(vars, tokenOf(body, 'color')));

    it(`الاسم والحالة يبلغان حدّ AA في الوضع ${mode}`, () => {
      // حدٌّ واحد للاثنين: لا نصّ كبير في الشريط يستحقّ استثناء ‎3:1‎.
      expect(contrast(ink(NAME), ground)).toBeGreaterThanOrEqual(AA_BODY);
      expect(contrast(ink(STATE), ground)).toBeGreaterThanOrEqual(AA_BODY);
    });

    it(`الاسم أظهر من الحالة في الوضع ${mode}`, () => {
      expect(contrast(ink(NAME), ground)).toBeGreaterThanOrEqual(contrast(ink(STATE), ground));
    });
  }
});
