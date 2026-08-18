// @vitest-environment node
/**
 * التباين العالي يجب ألا يطبّق قيم السمة الداكنة على نظامٍ فاتح لم يختر
 * المستخدم فيه سمةً صراحةً.
 *
 * ## العطل الذي وُلد منه هذا الملف (H-5 في تدقيق الفرع)
 *
 * `theme.ts` يحذف `data-theme` كليًّا حين يكون التفضيل «تلقائي» — وهو
 * الافتراضي، انظر `theme.test.ts`. فمحدِّدٌ صيغته `:not([data-theme="light"])`
 * يطابق نظامًا فاتحًا أيضًا في هذه الحالة، لا الداكن وحده. وحين حملت كتلة
 * `@media (prefers-contrast: more)` هذا المحدِّد بلا قيدٍ على
 * `prefers-color-scheme`، صار نصٌّ ثانوي (‏`--text-muted: #C2C3D0`‏) يظهر على
 * خلفيةٍ شبه بيضاء (‏`--bg-surface`‏) — تباينٌ مقيسٌ ١٫٦:١ تقريبًا، حيث تشترط
 * WCAG AA أربعة ونصف على الأقل. تفعيل «زيادة التباين» كان يجعل الواجهة أقلَّ
 * وضوحًا، لا أكثر، على أكثر الأجهزة شيوعًا (نظامٌ فاتح بلا سمةٍ مثبَّتة).
 *
 * والإصلاح: القيم الداكنة تُقصَر على حالتين منفصلتين — تفضيلٌ صريح
 * (`[data-theme="dark"]`، بلا قيدٍ على نظام التشغيل، فيعمل حتى على نظامٍ
 * فاتح) أو طلب النظام الداكن فعلًا (`:not([data-theme="light"])` داخل
 * `and (prefers-color-scheme: dark)`) — لا الحالة المدمجة السابقة.
 *
 * وبيئته `node`: الأنماط لا تُحمَّل في jsdom، والمصدر هو المرجع الوحيد.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const TOKENS = readFileSync(
  new URL('./design-system/tokens.css', import.meta.url).pathname,
  'utf8',
);

/** جسم أول `@media <query> { ... }` يطابق الاستعلام الحرفي، بعمق أقواسٍ متوازن. */
function mediaBlock(css: string, query: string): string {
  const marker = `@media ${query} {`;
  const at = css.indexOf(marker);
  expect(at, `الاستعلام "${query}" غير موجود حرفيًا`).toBeGreaterThan(-1);
  let depth = 0;
  let i = at + marker.length - 1; // موضع القوس المفتوح
  const start = i;
  do {
    if (css[i] === '{') depth += 1;
    else if (css[i] === '}') depth -= 1;
    i += 1;
  } while (depth > 0 && i < css.length);
  expect(depth, `لم يُغلَق الاستعلام "${query}"`).toBe(0);
  return css.slice(start, i);
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

describe('كتلة التباين العالي غير المشروطة بنظام التشغيل', () => {
  const plainBlock = mediaBlock(TOKENS, '(prefers-contrast: more)');

  it('تحمل التفضيل الصريح للسمة الداكنة', () => {
    expect(plainBlock).toContain(':root[data-theme="dark"]');
  });

  /**
   * الحارس الحقيقي: هذا هو العطل بعينه لو عاد. `:not([data-theme="light"])`
   * لا يجوز أن يظهر في هذه الكتلة غير المشروطة بـ`prefers-color-scheme` —
   * ظهوره هنا يعني أنه يطابق نظامًا فاتحًا بلا سمةٍ مثبَّتة أيضًا.
   */
  it('لا تحمل المحدِّد غير المشروط الذي طابق نظامًا فاتحًا أيضًا', () => {
    expect(plainBlock).not.toContain(':not([data-theme="light"])');
  });
});

describe('نظير `:not([data-theme="light"])` الداكن مقيَّدٌ بطلب النظام الداكن فعلًا', () => {
  const darkOsBlock = mediaBlock(
    TOKENS,
    '(prefers-contrast: more) and (prefers-color-scheme: dark)',
  );

  it('يحمل المحدِّد والقيم الداكنة معًا', () => {
    expect(darkOsBlock).toContain(':root:not([data-theme="light"])');
    expect(darkOsBlock).toContain('--text-muted: #C2C3D0');
  });
});

describe('التباين المقيس فعليًا', () => {
  it('نصّ التباين العالي الفاتح (--text-muted) يجتاز حدّ WCAG AA على خلفيته', () => {
    // القيم الفعلية كما تصل نظامًا فاتحًا بلا سمةٍ مثبَّتة: `--bg-surface`
    // الفاتحة (‏`--ink-000`‏، أبيضٌ خالص) من `:root`، و`--text-muted` من كتلة
    // التباين العالي **الفاتحة** (‏`var(--ink-600)`‏) — لا الكتلة الداكنة
    // المقيَّدة أعلاه، لأنها لا تُطبَّق هنا بعد الإصلاح.
    const inkRoot = TOKENS.slice(0, TOKENS.indexOf('HIGH CONTRAST'));
    const ink000 = /--ink-000:\s*(#[0-9a-fA-F]{6})/.exec(inkRoot)?.[1];
    const ink600 = /--ink-600:\s*(#[0-9a-fA-F]{6})/.exec(inkRoot)?.[1];
    expect(ink000, '--ink-000 غير موجود').toBeTruthy();
    expect(ink600, '--ink-600 غير موجود').toBeTruthy();

    const ratio = contrast(rgb(ink600 as string), rgb(ink000 as string));
    expect(ratio).toBeGreaterThanOrEqual(4.5);
  });
});
