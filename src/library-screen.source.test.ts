// @vitest-environment node
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const CSS = readFileSync(new URL('./library-screen.css', import.meta.url).pathname, 'utf8');
const SHELL = readFileSync(new URL('./app-shell.css', import.meta.url).pathname, 'utf8');
const TOKENS = readFileSync(
  new URL('./design-system/tokens.css', import.meta.url).pathname,
  'utf8',
);

function tokenPx(name: string): number {
  const value = TOKENS.match(new RegExp(`${name}:\\s*(\\d+)px`))?.[1];
  expect(value, `الرمز ${name} بلا قيمة px`).toBeTruthy();
  return Number(value);
}

function categoryGrid(): { baseColumns: number; breakpoint: number; wideColumns: number } {
  const base = CSS.match(
    /\.ops__grid--categories\s*\{\s*grid-template-columns:\s*repeat\((\d+),/,
  );
  const wide = CSS.match(
    /@media\s*\(min-width:\s*(\d+)px\)\s*\{\s*\.ops__grid--categories\s*\{\s*grid-template-columns:\s*repeat\((\d+),/,
  );

  expect(base, 'تعريف عمودَي شبكة الفئات مفقود').toBeTruthy();
  expect(wide, 'نقطة انتقال شبكة الفئات مفقودة').toBeTruthy();
  return {
    baseColumns: Number(base?.[1]),
    breakpoint: Number(wide?.[1]),
    wideColumns: Number(wide?.[2]),
  };
}

describe('شبكة فئات المكتبة', () => {
  it.each([
    [680, 2],
    [880, 2],
    [900, 3],
    [920, 3],
    [960, 3],
  ])('تعرض %i عمودًا عند عرض %ipx', (width, expected) => {
    const { baseColumns, breakpoint, wideColumns } = categoryGrid();
    expect(width >= breakpoint ? wideColumns : baseColumns).toBe(expected);
  });

  it('لا تترك عدد الأعمدة لتقدير auto-fit', () => {
    const categoryRules = CSS.match(/\.ops__grid--categories[^}]*}/g)?.join('\n') ?? '';
    expect(categoryRules).not.toContain('auto-fit');
    expect(categoryRules).not.toContain('auto-fill');
  });

  it('يحافظ على دعم WebKit في macOS 12', () => {
    expect(CSS).not.toMatch(/:has\(|color-mix\(|:focus-visible/);
  });

  it('يطابق هندسة المكتبة عند 680×520: 576/44 للبحث و282×80 للبطاقة', () => {
    const minimumStart = CSS.indexOf('@media (max-width: 759px)');
    const minimum = CSS.slice(minimumStart);
    expect(minimum).toMatch(/\.lib__hero \.ops__sub\s*\{\s*display:\s*none/);
    expect(minimum).toMatch(
      /\.lib__search\s*\{[^}]*width:\s*100%[^}]*max-inline-size:\s*none[^}]*height:\s*var\(--control-h-search\)/,
    );
    expect(minimum).toMatch(
      /\.category-card\s*\{\s*min-height:\s*var\(--space-80\)/,
    );

    const shellMinimum = SHELL.slice(SHELL.indexOf('@media (max-width: 759px)'));
    expect(shellMinimum).toMatch(/var\(--space-24\)/);
    const contentWidth =
      680 - tokenPx('--sidebar-w-collapsed') - 2 * tokenPx('--space-24');
    const cardWidth = (contentWidth - tokenPx('--space-12')) / 2;
    expect(contentWidth).toBe(576);
    expect(tokenPx('--control-h-search')).toBe(44);
    expect(cardWidth).toBe(282);
    expect(tokenPx('--space-80')).toBe(80);
  });

  it('يطابق تركيب 1080 الافتراضي: ثلاث بطاقات 240px وفجوتان 24px', () => {
    const defaultStart = CSS.indexOf('@media (min-width: 1080px)');
    const defaultGeometry = CSS.slice(defaultStart);
    expect(defaultGeometry).toMatch(
      /\.ops__grid--categories\s*\{[^}]*grid-template-columns:\s*repeat\(3, 240px\)[^}]*justify-content:\s*start/,
    );
    const wideStart = CSS.indexOf('@media (min-width: 900px)');
    const wideEnd = CSS.indexOf('@media (min-width: 1080px)', wideStart);
    const wide = CSS.slice(wideStart, wideEnd);
    expect(wide).toMatch(/gap:\s*var\(--space-24\)/);
    expect(3 * 240 + 2 * tokenPx('--space-24')).toBe(768);
  });

  it('يثبّت بطاقتي العمليات عند 360px بفجوة 24px في تركيب 1080', () => {
    const defaultStart = CSS.indexOf('@media (min-width: 1080px)');
    const defaultGeometry = CSS.slice(defaultStart);
    expect(defaultGeometry).toMatch(
      /\.ops__grid--operations\s*\{[^}]*grid-template-columns:\s*repeat\(2, 360px\)[^}]*gap:\s*var\(--space-24\)[^}]*justify-content:\s*start/,
    );
    expect(2 * 360 + tokenPx('--space-24')).toBe(744);
  });
});

describe('بطاقات Page 15 الإنتاجية', () => {
  it('تستخدم ارتفاعَي المكوّنين المعتمدين وأيقونة 24px عارية', () => {
    expect(CSS).toMatch(/\.category-card\s*\{[^}]*min-height:\s*var\(--library-card-h\)/);
    expect(CSS).toMatch(/\.op-card--operation\s*\{[^}]*min-height:\s*var\(--operation-card-h\)/);

    const icon = CSS.match(/\.op-card__icon\s*\{([^}]*)}/)?.[1] ?? '';
    expect(icon).toMatch(/width:\s*var\(--icon-lg\)/);
    expect(icon).toMatch(/height:\s*var\(--icon-lg\)/);
    expect(icon).not.toMatch(/background|border-radius/);
  });

  it('لا تعيد سهم البطاقة أو بلاطة القسم القديمة', () => {
    expect(CSS).not.toContain('.op-card__go');
    expect(CSS).not.toContain('.tile__chip');
    expect(CSS).not.toMatch(/\.op-card--off[^}]*border-style:\s*dashed/);
  });

  it('تبقي الأيقونات محايدة ثم تستخدم Ember للتحويم والتركيز والضغط', () => {
    expect(CSS).toMatch(/\.op-card__icon\s*\{[^}]*color:\s*var\(--text-secondary\)/);
    expect(CSS).toContain('.op-card:not(.op-card--off):hover .op-card__icon,');
    expect(CSS).toContain('.op-card:not(.op-card--off):focus .op-card__icon,');
    expect(CSS).toContain('.op-card:not(.op-card--off):active .op-card__icon');
    expect(CSS).toMatch(
      /\.op-card:not\(\.op-card--off\):active \.op-card__icon\s*\{[^}]*color:\s*var\(--text-accent\)/,
    );

    const favourite = CSS.match(/\.tile__star\s*\{([^}]*)}/)?.[1] ?? '';
    expect(favourite).toMatch(/color:\s*var\(--text-muted\)/);
    expect(CSS).toMatch(/\.tile__star--on\s*\{\s*color:\s*var\(--text-accent\)/);
  });

  it('يطابق حالات التحويم والتعذّر المنفصلة في مكوّن Page 15', () => {
    expect(CSS).toMatch(
      /\.op-card\.card--interactive:hover,[\s\S]*?\.op-card\.card--interactive:focus\s*\{\s*background:\s*var\(--bg-hover\)/,
    );
    expect(CSS).toMatch(
      /\.op-card\.card--interactive:hover \.op-card__title,[\s\S]*?color:\s*var\(--text-accent\)/,
    );
    expect(CSS).toMatch(
      /\.op-card--tool-missing\s*\{\s*border-color:\s*var\(--status-warning-border\)/,
    );
    expect(CSS).toMatch(
      /\.op-card--unsupported\s*\{\s*border-color:\s*var\(--status-danger-border\)/,
    );
    expect(CSS).toMatch(
      /\.tile__availability--tool-missing\s*\{\s*color:\s*var\(--status-warning-text\)/,
    );
    expect(CSS).toMatch(
      /\.tile__availability--unsupported\s*\{\s*color:\s*var\(--status-danger-text\)/,
    );
  });
});

/**
 * ارتفاع بطاقة العملية مقيسٌ من محتواها، لا رقمٌ موروث.
 *
 * كان ‎148px‎ حين كان الوصف ثلاثة أسطر. وبعد إعادة صياغة نصوص العمليات صار
 * الوصف جملةً واحدة، فبقي السقف يحجز فراغًا لا يملؤه شيء — نحو ‎46px‎ بين
 * الوصف وصفّ البيانات في بطاقةٍ وصفُها سطر واحد.
 *
 * والحساب هنا من الرموز نفسها لا من رقمٍ مكتوب: تغييرُ مقاس خطٍّ أو حشوة
 * يُعيد حساب الميزانية، فيسقط الاختبار إن عاد السقف يجاوز محتواه.
 */
describe('ميزانية ارتفاع بطاقة العملية', () => {
  const line = (fs: string, lh: string) => {
    const size = tokenPx(fs);
    const ratio = Number(TOKENS.match(new RegExp(`${lh}:\\s*([\\d.]+)`))?.[1]);
    expect(ratio, `الرمز ${lh} بلا قيمة`).toBeGreaterThan(0);
    return size * ratio;
  };

  /** أطول محتوى معقول: عنوان + وصفٌ من سطرين + صفّ البيانات. */
  function contentHeight(descriptionLines: number): number {
    const padding = 2 * tokenPx('--space-14');
    const title = line('--fs-card-title', '--lh-card-title');
    const description = descriptionLines * line('--fs-body-secondary', '--lh-body-secondary');
    const meta = line('--fs-caption', '--lh-caption');
    const gaps = tokenPx('--space-4') + tokenPx('--space-8'); // نصّ ↔ وصف، ووصف ↔ بيانات
    return padding + title + description + meta + gaps;
  }

  it('يقلّ الحشو الرأسي عن الأفقي فلا يُضاف فراغٌ إلى فراغ', () => {
    const rule = CSS.match(/\.op-card--operation\s*\{([^}]*)}/)?.[1] ?? '';
    expect(rule).toMatch(/padding-block:\s*var\(--space-14\)/);
    expect(tokenPx('--space-14')).toBeLessThan(tokenPx('--space-16'));
  });

  it('يسع السقف سطرَ وصفٍ واحدًا بلا فراغٍ ميت', () => {
    const floor = tokenPx('--operation-card-h');
    const one = contentHeight(1);
    expect(floor, `السقف ${floor} أقصر من محتوى سطرٍ واحد ${one}`).toBeGreaterThanOrEqual(one);
    // الفراغ الزائد فوق محتوى السطر الواحد يبقى تنفّسًا لا فجوة.
    expect(
      floor - one,
      `السقف ${floor} يترك ${(floor - one).toFixed(1)}px فارغة فوق محتواه`,
    ).toBeLessThan(tokenPx('--space-16'));
  });

  it('ينمو الصفّ لوصفٍ من سطرين بدل أن يُقصّ', () => {
    // `min-height` أرضيةٌ لا سقف: البطاقة الأطول تكبر، والشبكة تمطّ صفّها.
    expect(CSS).toMatch(/\.op-card--operation\s*\{[^}]*min-height:/);
    expect(CSS).not.toMatch(/\.op-card--operation\s*\{[^}]*max-height:/);
    expect(contentHeight(2)).toBeGreaterThan(tokenPx('--operation-card-h'));
  });

  it('يفصل صفّ البيانات عن الوصف ولو امتلأت البطاقة', () => {
    const meta = CSS.match(/\.tile__meta\s*\{([^}]*)}/)?.[1] ?? '';
    expect(meta).toMatch(/margin-block-start:\s*auto/);
    expect(meta).toMatch(/padding-block-start:\s*var\(--space-8\)/);
  });
});
