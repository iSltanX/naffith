// @vitest-environment node
/**
 * مجرى التشغيل: مقروءٌ في الوضعين، لا في واحدٍ منهما.
 *
 * ## العطل الذي وُلد منه هذا الملف
 *
 * سطرُ الأداة عنصرُ `code`، و`base.css` يعطي كلَّ `code` رقعةً كاملة: أرضيةٌ
 * وحدٌّ وحشوة بـ`--code-inline-bg` — وهي **فاتحة** في الوضع النهاري، لأنها
 * مصمَّمة لرمزٍ داخل نصٍّ عربي على سطح الصفحة. والمجرى يجلس على أرضية الطرفية
 * الداكنة بحبرٍ فاتح (`--command-text`). فكان المزيج نهارًا: حبرٌ فاتح على رقعةٍ
 * فاتحة — المجرى كلّه غير مقروء تقريبًا.
 *
 * ولم يظهر في الوضع الليلي أبدًا، ولا في jsdom (لا أنماط فيه)، ولا في أي اختبار
 * سلوكي: النصّ موجودٌ في الشجرة ويُقرأ بـ`textContent` سليمًا. ظهر عند فتح
 * الوضع النهاري بالعين وحدها.
 *
 * فما يُحرس هنا هو الشرط الذي كان مفقودًا: **من يأخذ حبر الطرفية يأخذ أرضيتها،
 * ويترك رقعةَ `code` التي فرضها الأساس.**
 *
 * وبيئته `node` لأن الأنماط لا تُحمَّل في jsdom: المصدر هو المرجع الوحيد.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const OPERATION = readFileSync(
  new URL('./operation-layout.css', import.meta.url).pathname,
  'utf8',
);
const BASE = readFileSync(
  new URL('./design-system/base.css', import.meta.url).pathname,
  'utf8',
);
const TOKENS = readFileSync(
  new URL('./design-system/tokens.css', import.meta.url).pathname,
  'utf8',
);

/** حدّ AA لنصّ المتن. سطرُ الأداة نصٌّ يُقرأ حرفًا حرفًا، فلا استثناء له. */
const AA_BODY = 4.5;

/** جسم قاعدةٍ واحدة بمحدّدها الحرفي. */
function rule(css: string, selector: string): string {
  const at = css.indexOf(`\n${selector} {`);
  expect(at, `القاعدة ${selector} غير موجودة`).toBeGreaterThan(-1);
  const body = css.slice(at + selector.length + 3);
  return body.slice(0, body.indexOf('}'));
}

function declarations(body: string): Map<string, string> {
  const out = new Map<string, string>();
  for (const [, name, value] of body.matchAll(/(--[\w-]+):\s*([^;]+);/g)) {
    if (name && value) out.set(name, value.trim());
  }
  return out;
}

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

const LIGHT = declarations(rule(TOKENS, ':root'));
const DARK = new Map([...LIGHT, ...declarations(rule(TOKENS, ':root[data-theme="dark"]'))]);
const MODES: ReadonlyArray<readonly [string, Map<string, string>]> = [
  ['النهاري', LIGHT],
  ['الليلي', DARK],
];

const TEXT = rule(OPERATION, '.stream__text');
const LINES = rule(OPERATION, '.stream__lines');

describe('سطر الأداة ينزع رقعة `code` التي فرضها الأساس', () => {
  it('الأساس فعلًا يفرض رقعةً على كل `code` — حارسٌ على الحارس', () => {
    // لو نُزعت الرقعة من `base.css` يومًا، صار هذا الملف يحرس عطلًا لا وجود له،
    // فيمرّ صامتًا. هذا الفحص يُسقطه فيُراجَع.
    const base = rule(BASE, 'code, .cmd-inline');
    expect(base).toMatch(/background:\s*var\(--code-inline-bg\)/);
    expect(base).toMatch(/padding:/);
  });

  it('ينزع الأرضية والحدّ والحشوة صراحةً', () => {
    expect(TEXT, 'أرضية `code` باقية: حبرٌ فاتح على رقعةٍ فاتحة نهارًا').toMatch(
      /background:\s*(none|transparent)/,
    );
    expect(TEXT).toMatch(/border:\s*0/);
    expect(TEXT).toMatch(/padding:\s*0/);
  });
});

describe('حبر الطرفية على أرضية الطرفية', () => {
  it('كلاهما من لوحة «سَطْر» لا من لوحة الصفحة', () => {
    // الزوج محلولٌ معًا في نظام التصميم. أخذُ أحدهما من لوحةٍ والآخر من أخرى هو
    // بالضبط ما أنتج العطل.
    expect(tokenOf(TEXT, 'color')).toMatch(/^--command-/);
    expect(tokenOf(LINES, 'background')).toMatch(/^--command-/);
  });

  for (const [mode, vars] of MODES) {
    it(`يبلغ حدّ AA في الوضع ${mode}`, () => {
      const ink = rgb(resolve(vars, tokenOf(TEXT, 'color')));
      const ground = rgb(resolve(vars, tokenOf(LINES, 'background')));
      expect(contrast(ink, ground)).toBeGreaterThanOrEqual(AA_BODY);
    });
  }
});
