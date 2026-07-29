// @vitest-environment node
/**
 * حرّاس ورقة أنماط الترحيب، مقروءةً نصًّا.
 *
 * اختبارات العرض تثبت ما يظهر في `jsdom`؛ وهي عمياء تمامًا عن CSS — لا يُحمّل
 * محرّك أنماط، ولا يُحسب استعلامُ وسائط، ولا يُقاس ارتفاع. وكل عطلٍ رُفضت به
 * هذه الشاشة حتى اليوم عاش هناك بالضبط:
 *
 * - نقطةُ تحوّلٍ عند `max-width: 480` والنافذة لا تنزل تحت ‎٦٨٠. أي: التخطيط لا
 *   يُعاد تشكيله أبدًا، فتُقرأ الشاشة صفحةً مصغّرة.
 * - `animation-delay` على المقتطف مع `fill-mode: both`، فيُخفى ٢٠٠ms.
 * - **استعلامٌ لاحق يُرخي ما شدّه سابق**: `max-width: 880` يضبط
 *   `padding-block: var(--space-16)`، ثم `max-height: 580` يضبط
 *   `var(--space-20)` بالتخصيص نفسه وبعده في الملف. فالنافذة الضيّقة القصيرة
 *   تأخذ الأوسع: تقصيرها بكسلٍ واحد (‎581 ← ‎580) كان يزيد المحتوى ‎16px. لم
 *   يكن في هذا الملف ما يمنعه، وهذا هو الحارس الذي أُضيف — `يبقى ترتيب
 *   الاستعلامات` أدناه.
 *
 * لذلك تُقرأ الورقة نصًّا لا تُستورد. وبيئة الاختبار `node` لأن `jsdom` يعيد
 * كتابة أصل `import.meta.url` إلى عنوان http فيصير المسار من جذر القرص.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const CSS = readFileSync(new URL('./onboarding.css', import.meta.url).pathname, 'utf8');
const TOKENS = readFileSync(new URL('./app-tokens.css', import.meta.url).pathname, 'utf8');
const DS_TOKENS = readFileSync(
  new URL('./design-system/tokens.css', import.meta.url).pathname,
  'utf8',
);

/** الورقة بلا تعليقات: التعليقات هنا تشرح ما أُزيل، فتُطابق أنماطَ ما أُزيل. */
const BARE = CSS.replace(/\/\*[\s\S]*?\*\//g, '');

/** حدّا النافذة كما يضبطهما `src-tauri/tauri.conf.json`. */
const MIN_WINDOW_W = 680;
const MIN_WINDOW_H = 520;

/**
 * المقاسات التي يسمّيها المالك ويختبر عندها — من `docs/brand/UX-SPEC.md §12`
 * ومن شبكة المراجعة. لا يجوز أن تقع نقطة تحوّلٍ عند واحدٍ منها ولا بجواره:
 * نقطةٌ تنطبق عند المقاس المسمّى تعني أن نتيجة الاختبار عنده تخصّ حالةً
 * واحدة من اثنتين، وأن بكسلًا واحدًا يقلبها.
 */
const NAMED_WIDTHS = [680, 700, 760, 840, 860, 880, 881, 900, 1000, 1080, 1180];
const NAMED_HEIGHTS = [520, 560, 600, 640, 680, 720, 760, 900];

// ─────────────────────────────────────────────────────────────────────────
//  محلّل صغير: يكفي لهذه الورقة ولا يدّعي أكثر
// ─────────────────────────────────────────────────────────────────────────

interface Decl {
  readonly selector: string;
  readonly prop: string;
  readonly value: string;
}

/** المدى الذي ينطبق فيه استعلامٌ ما، في فضاء (عرض، ارتفاع). */
interface Region {
  readonly minW: number;
  readonly maxW: number;
  readonly minH: number;
  readonly maxH: number;
}

interface MediaBlock {
  readonly query: string;
  readonly region: Region;
  readonly decls: readonly Decl[];
  /** ترتيب الظهور في الملف: هو ما يقرّر الغالب عند تساوي التخصيص. */
  readonly order: number;
}

/** كل كتلة `@media` بجسمها، مع مطابقة أقواسٍ حقيقية لا تعبيرٍ نمطيّ جشِع. */
function mediaBodies(css: string): { query: string; body: string; order: number }[] {
  const out: { query: string; body: string; order: number }[] = [];
  const head = /@media([^{]+)\{/g;
  let hit: RegExpExecArray | null;
  let order = 0;
  while ((hit = head.exec(css)) !== null) {
    const start = hit.index + hit[0].length;
    let depth = 1;
    let i = start;
    while (i < css.length && depth > 0) {
      if (css[i] === '{') depth += 1;
      else if (css[i] === '}') depth -= 1;
      i += 1;
    }
    out.push({ query: (hit[1] ?? '').trim(), body: css.slice(start, i - 1), order });
    order += 1;
    head.lastIndex = i;
  }
  return out;
}

/** تصريحات جسمٍ ما، مسطّحةً إلى (محدّد، خاصّية، قيمة). */
function declarations(body: string): Decl[] {
  const out: Decl[] = [];
  const rule = /([^{}]+)\{([^{}]*)\}/g;
  let hit: RegExpExecArray | null;
  while ((hit = rule.exec(body)) !== null) {
    const heads = (hit[1] ?? '').trim();
    if (heads === '' || heads.startsWith('@')) continue;
    for (const part of (hit[2] ?? '').split(';')) {
      const at = part.indexOf(':');
      if (at < 0) continue;
      const prop = part.slice(0, at).trim();
      const value = part.slice(at + 1).trim().replace(/\s+/g, ' ');
      if (prop === '' || value === '') continue;
      for (const selector of heads.split(',')) out.push({ selector: selector.trim(), prop, value });
    }
  }
  return out;
}

/** يقرأ حدود الاستعلام. ما لا يُذكر لا يَحدّ. */
function toRegion(condition: string): Region {
  const read = (feature: string): number | null => {
    const hit = new RegExp(`\\b${feature}:\\s*(\\d+(?:\\.\\d+)?)px`).exec(condition);
    return hit ? Number(hit[1]) : null;
  };
  return {
    minW: read('min-width') ?? 0,
    maxW: read('max-width') ?? Infinity,
    minH: read('min-height') ?? 0,
    maxH: read('max-height') ?? Infinity,
  };
}

/** الكتل، مفكوكةً عند فواصل قائمة الاستعلامات: كل شرطٍ مدًى مستقلّ. */
function mediaBlocks(css: string): MediaBlock[] {
  const out: MediaBlock[] = [];
  for (const { query, body, order } of mediaBodies(css)) {
    const decls = declarations(body);
    for (const condition of query.split(',')) {
      out.push({ query: condition.trim(), region: toRegion(condition), decls, order });
    }
  }
  return out;
}

/** هل يوجد مقاسٌ *تبلغه النافذة فعلًا* ينطبق عنده المدَيان معًا؟ */
function overlaps(a: Region, b: Region): boolean {
  const lowW = Math.max(a.minW, b.minW, MIN_WINDOW_W);
  const highW = Math.min(a.maxW, b.maxW);
  const lowH = Math.max(a.minH, b.minH, MIN_WINDOW_H);
  const highH = Math.min(a.maxH, b.maxH);
  return lowW <= highW && lowH <= highH;
}

/** هل `inner` محصورٌ تمامًا داخل `outer`؟ */
function nestedIn(inner: Region, outer: Region): boolean {
  return (
    inner.minW >= outer.minW &&
    inner.maxW <= outer.maxW &&
    inner.minH >= outer.minH &&
    inner.maxH <= outer.maxH
  );
}

/** جدول الرموز الطولية: اسمٌ ← بكسل. من نظام التصميم وطبقة التطبيق معًا. */
const PX = new Map<string, number>();
for (const source of [DS_TOKENS, TOKENS]) {
  for (const hit of source.matchAll(/(--[a-z0-9-]+)\s*:\s*(-?\d+(?:\.\d+)?)px\s*;/g)) {
    PX.set(hit[1] ?? '', Number(hit[2]));
  }
}

/**
 * يحلّ قيمةً إلى أرقام بكسل، أو `null` إن لم تكن قابلةً للمقارنة عدديًا
 * (‏`clamp` و`min` و`calc` وما شابه). عدمُ القابلية ليس نجاحًا ولا فشلًا: هو
 * إعفاءٌ من *المقارنة العددية* وحدها، ولا يعفي من قاعدة التداخل أدناه.
 */
function toPx(value: string): number[] | null {
  const parts = value.split(' ').filter((p) => p !== '');
  const out: number[] = [];
  for (const part of parts) {
    const named = /^var\((--[a-z0-9-]+)\)$/.exec(part);
    if (named) {
      const px = PX.get(named[1] ?? '');
      if (px === undefined) return null;
      out.push(px);
      continue;
    }
    if (/^-?\d+(\.\d+)?px$/.test(part)) {
      out.push(Number.parseFloat(part));
      continue;
    }
    if (part === '0') {
      out.push(0);
      continue;
    }
    return null;
  }
  return out.length > 0 ? out : null;
}

/**
 * الحارس نفسه. يعيد قائمة المخالفات نصًّا — فارغةً حين تكون الورقة سليمة.
 *
 * قاعدتان، والأولى أهمّهما:
 *
 * 1. **لا استعلامان متقاطعان يكتبان الخاصّية نفسها على المحدّد نفسه.** حين
 *    يتقاطع المدَيان ولا يحتوي أحدهما الآخر (ضيّقٌ ×قصير مثلًا) فليس في
 *    الورقة ما يقول أيّهما المقصود: يقرّرها ترتيبُ الملف وحده. وهذا هو العطل
 *    الذي رُفضت به النسخة السابقة حرفيًا.
 * 2. **الاستعلام الأضيق لا يكبّر ما صغّره الأوسع.** حين يكون أحد المدَيين
 *    محصورًا داخل الآخر فالنيّة مفهومة — تضييقٌ عند مقاسٍ أصغر — ويبقى أن
 *    يُتحقّق منها عدديًا.
 */
function cascadeViolations(css: string): string[] {
  const blocks = mediaBlocks(css);
  const bad: string[] = [];
  for (let i = 0; i < blocks.length; i += 1) {
    for (let j = i + 1; j < blocks.length; j += 1) {
      const a = blocks[i] as MediaBlock;
      const b = blocks[j] as MediaBlock;
      if (a.order === b.order) continue; // شرطان في قائمةٍ واحدة: جسمٌ واحد
      if (!overlaps(a.region, b.region)) continue;

      for (const da of a.decls) {
        for (const db of b.decls) {
          if (da.selector !== db.selector || da.prop !== db.prop) continue;
          if (da.value === db.value) continue;

          const aInB = nestedIn(a.region, b.region);
          const bInA = nestedIn(b.region, a.region);
          if (!aInB && !bInA) {
            bad.push(
              `استعلامان متقاطعان يتنازعان \`${da.prop}\` على \`${da.selector}\`: ` +
                `«${a.query}» و«${b.query}» — الغالب يقرّره ترتيب الملف لا التصميم`,
            );
            continue;
          }

          const narrow = aInB ? a : b;
          const wide = aInB ? b : a;
          const nv = toPx(aInB ? da.value : db.value);
          const wv = toPx(aInB ? db.value : da.value);
          if (nv === null || wv === null || nv.length !== wv.length) continue;
          for (let k = 0; k < nv.length; k += 1) {
            if ((nv[k] as number) > (wv[k] as number)) {
              bad.push(
                `الاستعلام الأضيق «${narrow.query}» يكبّر \`${da.prop}\` على ` +
                  `\`${da.selector}\` (${nv[k]}px) فوق ما ضبطه الأوسع «${wide.query}» ` +
                  `(${wv[k]}px)`,
              );
            }
          }
        }
      }
    }
  }
  return bad;
}

/** كل محدّدٍ في الورقة، بما فيه ما داخل استعلامات الوسائط. */
function selectors(css: string): string[] {
  const out: string[] = [];
  for (const match of css.matchAll(/([^{}]+)\{/g)) {
    const head = (match[1] ?? '').trim();
    if (head === '' || head.startsWith('@')) continue;
    for (const part of head.split(',')) out.push(part.trim());
  }
  return out;
}

// ─────────────────────────────────────────────────────────────────────────

describe('الحارس نفسه يُختبر قبل أن يَحرس', () => {
  // حارسٌ لا يسقط أمام العطل الذي كُتب له ليس حارسًا بل تعليقٌ مكلف. هذه
  // الأوراق الثلاث مصطنعة، وأوّلها منقولٌ حرفيًا عن الورقة المرفوضة.
  it('يمسك العطل المرفوض: ضيّقٌ يشدّ ثم قصيرٌ يُرخي', () => {
    const rejected = `
      @media (max-width: 880px) {
        .onboarding__visual, .onboarding__copy { padding-block: var(--space-16); }
      }
      @media (max-height: 580px) {
        .onboarding__visual, .onboarding__copy { padding-block: var(--space-20); }
      }`;
    const found = cascadeViolations(rejected);
    expect(found.length, 'الحارس لم يرَ العطل الذي كُتب له').toBeGreaterThan(0);
    expect(found.join(' ')).toContain('padding-block');
  });

  it('يمسك الانقلاب المتداخل: الأضيق أكبر من الأوسع', () => {
    const inverted = `
      @media (max-width: 900px) { .onboarding__steps { gap: var(--space-8); } }
      @media (max-width: 800px) { .onboarding__steps { gap: var(--space-12); } }`;
    expect(cascadeViolations(inverted).length).toBeGreaterThan(0);
  });

  it('لا يشكو من تضييقٍ سليم ولا من مدَيين منفصلين', () => {
    const fine = `
      @media (max-width: 900px) { .onboarding__steps { gap: var(--space-12); } }
      @media (max-width: 800px) { .onboarding__steps { gap: var(--space-8); } }
      @media (min-width: 901px) { .onboarding__steps { gap: var(--space-24); } }`;
    expect(cascadeViolations(fine)).toEqual([]);
  });
});

describe('ترتيب الاستعلامات لا يقرّر شيئًا', () => {
  it('لا استعلام يُرخي ما شدّه غيره', () => {
    expect(cascadeViolations(BARE)).toEqual([]);
  });

  it('لا مقدار رأسيّ يُكتب داخل استعلام وسائط أصلًا', () => {
    // هذا هو العلاج البنيويّ لا العدديّ: ما دام الإيقاع الرأسي كلّه منحنيات
    // `clamp()` تقرأ `vh` في `.onboarding` وحدها، فلا يوجد موضعٌ يمكن أن
    // يتنازعه استعلامان أصلًا. الحارس أعلاه يمسك الانقلاب؛ وهذا يمنع وجوده.
    const vertical =
      /^(padding|margin)(-block(-start|-end)?)?$|^(row-)?gap$|^(min-|max-)?height$|^(top|bottom|inset-block(-start|-end)?)$/;
    const offenders = mediaBlocks(BARE).flatMap((block) =>
      block.decls
        .filter((decl) => vertical.test(decl.prop))
        .map((decl) => `«${block.query}» → ${decl.selector} { ${decl.prop} }`),
    );
    expect(offenders, `مقدارٌ رأسيّ داخل استعلام: ${offenders.join(' | ')}`).toEqual([]);
  });

  it('الإيقاع الرأسي منحنياتٌ تقرأ المرأى، لا درجاتٍ في استعلامات', () => {
    // الأربعة معلَنةٌ مرّةً واحدة، في `.onboarding` وحدها، وكلٌّ منها
    // `clamp()` يقرأ `vh`. `clamp(a, k·vh − c, b)` غير متناقصة في الارتفاع
    // بحكم تعريفها، فلا يمكن لنافذةٍ أقصر أن تُنتج مقدارًا أكبر.
    for (const name of ['--v-pane', '--v-block', '--v-tight', '--v-gap']) {
      const decls = [...BARE.matchAll(new RegExp(`${name}\\s*:\\s*([^;]+);`, 'g'))];
      expect(decls.length, `${name} معلَنٌ ${decls.length} مرّة لا مرّةً واحدة`).toBe(1);
      const value = decls[0]?.[1] ?? '';
      expect(value, `${name} لا يقرأ الارتفاع`).toMatch(/vh/);
      expect(value, `${name} بلا حدَّين`).toMatch(/clamp\(/);
    }
  });
});

describe('نقاط تحوّل شاشة الترحيب', () => {
  it('لا نقطة عرضٍ تحت أدنى عرضٍ تبلغه النافذة', () => {
    // جذر العطل الأول حرفيًا: نقطةٌ عند ‎٤٨٠ لا يمكن للمستخدم بلوغها، فلا شيء
    // يتغيّر بين ‎١٠٨٠ و‎٦٨٠. أي رقمٍ هنا دون ‎٦٨٠ كودٌ ميّت يوهم قارئه بأن
    // الشاشة تستجيب.
    for (const query of BARE.match(/@media[^{]+/g) ?? []) {
      for (const hit of query.matchAll(/\b(?:max|min)-width:\s*(\d+(?:\.\d+)?)px/g)) {
        expect(Number(hit[1]), `نقطة عرضٍ لا تُبلَغ: ${query.trim()}`).toBeGreaterThanOrEqual(
          MIN_WINDOW_W,
        );
      }
    }
  });

  it('لا نقطة ارتفاعٍ تحت أدنى ارتفاعٍ تبلغه النافذة', () => {
    for (const query of BARE.match(/@media[^{]+/g) ?? []) {
      for (const hit of query.matchAll(/\b(?:max|min)-height:\s*(\d+(?:\.\d+)?)px/g)) {
        expect(Number(hit[1]), `نقطة ارتفاعٍ لا تُبلَغ: ${query.trim()}`).toBeGreaterThanOrEqual(
          MIN_WINDOW_H,
        );
      }
    }
  });

  it('لا نقطة تحوّلٍ تنطبق عند مقاسٍ يسمّيه المالك ولا بجواره', () => {
    // `max-width: 880px` كانت تنطبق عند ‎٨٨٠ بالضبط — وهو مقاس «متوسط» الذي
    // يختبر عنده المالك. فكان اختبارُ المقاس المسمّى يقيس إحدى حالتين، وبكسلٌ
    // واحد يقلبها. الحدّان المفحوصان هنا هما ما ينطبق عنده الاستعلام وما
    // يليه مباشرةً: كلاهما يجب أن يقع بعيدًا عن الأسماء.
    const complaints: string[] = [];
    for (const query of BARE.match(/@media[^{]+/g) ?? []) {
      for (const hit of query.matchAll(/\b(max|min)-(width|height):\s*(\d+(?:\.\d+)?)px/g)) {
        const at = Number(hit[3]);
        const other = hit[1] === 'max' ? at + 1 : at - 1;
        const named = hit[2] === 'width' ? NAMED_WIDTHS : NAMED_HEIGHTS;
        for (const edge of [at, other]) {
          if (named.includes(edge)) {
            complaints.push(`${hit[0]} في «${query.trim()}» يقع حدّها عند ${edge}`);
          }
        }
      }
    }
    expect(complaints, complaints.join(' | ')).toEqual([]);
  });

  it('تستجيب الشاشة بتغيير ما تعرضه، لا بتصغير ما تعرضه', () => {
    // إعادة تشكيلٍ حقيقية تعني أن ما يُعرض أو كيف يُوزَّع هو ما يتغيّر.
    // `transform: scale` و`zoom` يعطيان الشكل نفسه مصغّرًا: يبهت النص وتضيق
    // مساحات النقر ولا يُكسب سطرٌ واحد. هما البديل المرفوض صراحةً.
    expect(BARE).not.toMatch(/transform:[^;}]*\bscale\s*\(/);
    expect(BARE).not.toMatch(/(^|[\s;{])zoom\s*:/);

    // ولا بدّ من نقطةٍ واحدة على الأقل داخل المدى تغيّر *ما يُعرض أو كيف
    // يُوزَّع* — لا الحشو وحده، فذلك ما كان: ورقةٌ «تستجيب» بالفراغ.
    const structural = /^(display|grid-template-columns|grid-template-areas|flex-direction)$/;
    const responded = mediaBlocks(BARE).some(
      (block) =>
        block.region.maxW < Infinity &&
        block.region.maxW >= MIN_WINDOW_W &&
        block.decls.some((decl) => structural.test(decl.prop)),
    );
    expect(responded, 'لا نقطة عرضٍ تُبلَغ تغيّر ما يُعرض').toBe(true);
  });

  it('الفعل الأساسي لا يُثبَّت فوق النص ولا يُقصّ المثال', () => {
    // ‏UX-SPEC §4 صريحة: «لا يثبت زر فوق النص ولا يُقص المثال». تثبيت الزر
    // كان أسهل طريقٍ إلى إبقائه داخل المرأى، وهو الطريق المحرّم: يحجب سطرًا
    // من المحتوى دائمًا مقابل إنقاذ حالةٍ نادرة.
    expect(BARE).not.toMatch(/position:\s*(sticky|fixed)/);
  });
});

describe('حضور المقتطف في أول إطار', () => {
  it('لا تأخير حركةٍ في الورقة كلّها، ولا في صيغةٍ مختصرة', () => {
    // `animation-delay: var(--duration-base)` مع `fill-mode: both` هو نصّ
    // العطل الثاني: يُثبَّت `opacity: 0` طوال التأخير.
    expect(BARE).not.toMatch(/animation-delay\s*:/);
    expect(BARE).not.toMatch(/transition-delay\s*:/);

    // والصيغة المختصرة تخفي التأخير في وسيطها *الثاني* الزمنيّ. الحارس السابق
    // كان يمنع `animation:` داخل قواعد `.onboarding__peek` وحدها — وهو ثقبٌ
    // واسع: تأخيرٌ على سلفٍ مثل `.onboarding[data-motion='enter']
    // .onboarding__panes` يؤخّر المقتطف مع اللوحة كلّها ويمرّ من كل حارس.
    // فالفحص هنا على *كل* صيغةٍ مختصرة في الورقة أيًّا كان محدّدها: لا يجوز
    // أن تحمل أكثر من مقدارٍ زمنيّ واحد، وهو المدّة.
    const timeish = /var\(\s*--duration-[a-z-]+\s*\)|(?:^|[\s,(])-?\d*\.?\d+m?s(?![a-z0-9-])/g;
    for (const hit of BARE.matchAll(/(?:^|[\s;{])(animation|transition)\s*:\s*([^;}]+)/g)) {
      const times = [...(hit[2] ?? '').matchAll(timeish)];
      expect(
        times.length,
        `\`${hit[1]}\` بمقدارين زمنيّين — الثاني تأخير: ${(hit[2] ?? '').trim()}`,
      ).toBeLessThanOrEqual(1);
    }

    // والطريق الأخير: رمزٌ محلّي بقيمةٍ زمنية يُدسّ في الصيغة المختصرة فلا
    // يطابق `--duration-*`. لا داعي لواحدٍ في هذه الورقة، فلا يُسمح به.
    expect(BARE).not.toMatch(/--[a-z0-9-]+\s*:\s*-?\d*\.?\d+m?s\s*[;)]/);
  });

  it('ارتفاع الصندوق محجوز بحسابٍ لا يعرف الخطّ', () => {
    // بلا هذا الحجز يقفز ما تحت المقتطف حين يصل خطّ `mono` بمقاييسه المختلفة
    // (‏`font-display: swap`). والحساب من رموزٍ لا من px مقدّرة، فيتبع أي
    // تغييرٍ في مقاس الطرفية بدل أن ينفصل عنه.
    const body = BARE.match(/\.onboarding__peek\s+\.command__body\s*\{([^}]*)\}/);
    expect(body, 'قاعدة جسم الأمر غير موجودة').not.toBeNull();
    const decls = body?.[1] ?? '';
    expect(decls).toMatch(/min-height:\s*calc\(/);
    expect(decls).toMatch(/--peek-lines:\s*4\s*;/);
    expect(decls).toMatch(/var\(--fs-terminal\)/);
    expect(decls).toMatch(/var\(--lh-terminal\)/);
    expect(decls).not.toMatch(/min-height:\s*\d/);
  });

  it('لا يُخفى المقتطف إلا بحدّ عرضٍ واحد، ولا يُصغَّر خطّه', () => {
    // المقتطف `white-space: pre` فلا يضيق: إمّا أن يسعه موضعه أو يمرّر أفقيًا
    // فيأكل الشريطُ من ارتفاعه. الإخفاء تحت ‎856px هو الجواب المقيس، والبديلان
    // المرفوضان أن يُصغَّر خطّه (فيخرج عن سلّم الرموز) أو يُلَفّ (فيُكسر الأمر
    // في موضعٍ لا تكسره فيه الصدفة، فيصير أمرًا لا يُنسخ).
    const hiding = mediaBlocks(BARE).filter((block) =>
      block.decls.some((d) => d.selector.includes('__peek') && d.prop === 'display'),
    );
    expect(hiding.length, 'المقتطف يُخفى في أكثر من موضع').toBe(1);
    expect(hiding[0]?.region.maxH, 'إخفاء المقتطف مربوطٌ بالارتفاع بلا داعٍ').toBe(Infinity);
    expect(BARE).not.toMatch(/\.onboarding__peek[\s\S]{0,400}?white-space:\s*pre-wrap/);
    expect(BARE).not.toMatch(/\.onboarding__peek[^{}]*\{[^{}]*font-size:\s*\d/);
  });
});

describe('نطاق أنماط الترحيب', () => {
  it('كل محدّدٍ محبوسٌ داخل `.onboarding`', () => {
    // لغة mockups-v3 نُقلت إلى هنا بدل استيراد الورقة كلّها، والثمن الوحيد
    // المقبول لذلك أن لا يتسرّب منها شيء: أربع شاشاتٍ أخرى لم تُحوَّل بعد،
    // ومحدّدٌ عارٍ واحد يعيد تلوينها في مرحلةٍ لم تُطلب.
    const loose = selectors(BARE).filter(
      (sel) => !sel.includes('.onboarding') && !/^(from|to|\d+(\.\d+)?%)$/.test(sel),
    );
    expect(loose, `محدّدات خارج الشاشة: ${loose.join(' | ')}`).toEqual([]);
  });

  it('لا تُستورد ورقة النماذج كاملةً', () => {
    // محدّداتها العارية `.screen` و`.dialog` تصطدم بأصناف التطبيق، فيقرّر
    // ترتيبُ التحميل وحده الغالب.
    expect(BARE).not.toMatch(/@import[^;]*mockups-v3/);
  });
});

describe('طبقة رموز التطبيق', () => {
  it('تُعرَّف الثلاثة المفقودة، ولا يُعاد تعريف رمزٍ من نظام التصميم', () => {
    expect(TOKENS).toMatch(/--space-10:\s*10px/);
    expect(TOKENS).toMatch(/--space-14:\s*14px/);
    expect(TOKENS).toMatch(/--space-28:\s*28px/);

    // الملف طبقةٌ فوق `tokens.css` لا نسخةٌ ثانية منه. أي رمزٍ رابع يُضاف هنا
    // هو أول خطوةٍ نحو سلّمين لأحدهما ٢٤٤ درجة وللآخر ما تيسّر.
    const declared = [...TOKENS.matchAll(/(--[a-z0-9-]+)\s*:/g)].map((hit) => hit[1]);
    expect(declared.sort()).toEqual(['--space-10', '--space-14', '--space-28']);
  });

  it('تُستورد من الشاشة وحدها لا من الجذر', () => {
    // في هذه المرحلة شاشةٌ واحدة تتكلّم لغة mockups-v3. تحميل رموزها عالميًا
    // يوسّع أثر التحويل إلى ما لم يُطلب تحويله.
    expect(BARE).toMatch(/@import\s+['"]\.\/app-tokens\.css['"]/);
  });
});
