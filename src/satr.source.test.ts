// @vitest-environment node
/**
 * الكتابة المائية: خافتةٌ ومقروءة معًا، لا واحدة منهما.
 *
 * العقد على هذه الكتلة شرطان يشدّ كلٌّ منهما في جهة: أن تكون **كتابةً في
 * الخلفية** — بلا حدّ ولا بطاقة، بشفافيةٍ منخفضة ووزنٍ خفيف — وأن تكون
 * **تعليمةً تُقرأ**. سقط الثاني مرّة: كان السطران يأخذان `--text-secondary`،
 * وهي درجةٌ صحيحة على الأرضية وحدها لكنها تُضرب في ٠٫٦٥ قبل أن تصل إلى العين،
 * فتنزل إلى 3.20:1 في الوضع النهاري و3.96:1 في الليلي — دون حدّ AA (‏4.5)
 * لنصّ المتن في الوضعين معًا.
 *
 * ولذلك يحسب هذا الملف ما تراه العين لا ما يقوله الرمز: يقرأ قيمة الرمز من
 * نظام التصميم، ويمزجها بأرضيتها بنسبة الشفافية المعلنة، ثم يقيس التباين.
 * فرفعُ الشفافية أو خفضُ الحبر يسقط هنا قبل أن يصل إلى مستخدم.
 *
 * وبيئته `node` لأن الأنماط لا تُحمَّل في jsdom أصلًا: لا يوجد في شجرة الاختبار
 * لونٌ محسوب يُسأل عنه، فالمصدر هو المرجع الوحيد.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const APP = readFileSync(new URL('./app.css', import.meta.url).pathname, 'utf8');
const TOKENS = readFileSync(
  new URL('./design-system/tokens.css', import.meta.url).pathname,
  'utf8',
);

/** حدّ AA لنصّ المتن، وحدّه للنصّ الكبير (‏≥ 24px بوزنٍ عادي أو أخفّ). */
const AA_BODY = 4.5;
const AA_LARGE = 3;

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

/** ما تراه العين فعلًا: الحبر ممزوجًا بأرضيته بنسبة الشفافية. */
function composite(ink: Rgb, ground: Rgb, alpha: number): Rgb {
  return [
    alpha * ink[0] + (1 - alpha) * ground[0],
    alpha * ink[1] + (1 - alpha) * ground[1],
    alpha * ink[2] + (1 - alpha) * ground[2],
  ];
}

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

const MARK = rule(APP, '.satr__watermark');
const TITLE = rule(APP, '.satr__watermark-title');
const LINE = rule(APP, '.satr__watermark-line');

describe('الكتابة المائية تبقى خافتة', () => {
  it('شفافيةٌ منخفضة من رمزٍ في النظام، لا رقمًا مكتوبًا هنا', () => {
    const alpha = Number.parseFloat(resolve(LIGHT, tokenOf(MARK, 'opacity')));
    expect(alpha).toBeGreaterThan(0);
    // «كتابة مائية» شرطٌ بصري لا زينة: بلا خفوتٍ محسوس تصير الكتلة نصًّا
    // ثالثًا ينافس النموذج على الانتباه.
    expect(alpha).toBeLessThanOrEqual(0.7);
  });

  it('وزنٌ خفيف في الأسطر الثلاثة', () => {
    expect(TITLE).toContain('font-weight: var(--fw-light)');
    expect(LINE).toContain('font-weight: var(--fw-light)');
  });

  it('بلا لونٍ خام: كل قيمة رمزٌ من النظام', () => {
    for (const body of [MARK, TITLE, LINE]) {
      expect(body).not.toMatch(/#[0-9a-fA-F]{3,8}/);
      expect(body).not.toMatch(/\brgba?\(/);
    }
  });
});

describe('الكتابة المائية تبقى مقروءة', () => {
  const alpha = Number.parseFloat(resolve(LIGHT, tokenOf(MARK, 'opacity')));

  for (const [mode, vars] of MODES) {
    const ground = rgb(resolve(vars, '--bg-canvas'));
    const ink = (body: string): Rgb =>
      composite(rgb(resolve(vars, tokenOf(body, 'color'))), ground, alpha);

    it(`السطران يبلغان حدّ AA في الوضع ${mode}`, () => {
      // ما يُقاس هو المزيج لا قيمة الرمز: الشفافية على الكتلة تسبق العين.
      expect(contrast(ink(LINE), ground)).toBeGreaterThanOrEqual(AA_BODY);
    });

    it(`العنوان يبلغ حدّ النصّ الكبير في الوضع ${mode}`, () => {
      expect(contrast(ink(TITLE), ground)).toBeGreaterThanOrEqual(AA_LARGE);
    });

    it(`العنوان أظهر من السطرين في الوضع ${mode}`, () => {
      // الفرق مقاسٌ وخطّ لا درجةَ حبر: لا درجة في النظام أقوى ممّا يأخذه
      // السطران بعد أن صعدا إليها. فالشرط ألّا يكون العنوان أخفت منهما، وأن
      // يبقى مقاسه أكبر — وعند 28px يكفيه حدّ 3:1 بينما يلزم السطرين 4.5.
      expect(contrast(ink(TITLE), ground)).toBeGreaterThanOrEqual(contrast(ink(LINE), ground));
      expect(px(vars, tokenOf(TITLE, 'font-size'))).toBeGreaterThan(
        px(vars, tokenOf(LINE, 'font-size')),
      );
      expect(px(vars, tokenOf(TITLE, 'font-size'))).toBeGreaterThanOrEqual(24);
    });
  }
});
