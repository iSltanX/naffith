// @vitest-environment node
/**
 * قانون التخطيط، مفروضًا على كل شاشة — الحالية والقادمة.
 *
 * هذا الملف ليس اختبارًا لشاشةٍ بعينها، بل هو القاعدة نفسها مكتوبةً حيث تُنفَّذ.
 * شاشةٌ تُضاف غدًا تدخل هذا الفحص من تلقائها: المسح على كل ملفّ أنماطٍ في
 * `src/`، لا على قائمةٍ تُحدَّث باليد وتُنسى.
 *
 * ── ما يفرضه، ولماذا
 *
 * العطل الذي وُلد منه: ملفات `mockups-v3` صفحاتُ HTML تُعرض في متصفّح، فبطاقتها
 * الخارجية أداةُ عرضٍ تحصر التصميم وسط صفحةٍ لا حدّ لعرضها. نُسخت تلك البطاقة
 * إلى شاشة الترحيب بوصفها عنصرًا من المنتج، فصار المستخدم يرى نافذةً داخل
 * نافذة: فراغٌ يتّسع حول بطاقةٍ ثابتة كلّما كبرت النافذة.
 *
 * ثم تبيّن أن للعطل وجهًا ثانيًا أعمق: المقادير كانت تقرأ المرأى (`vh` ثم
 * `vw`)، فكان *الشيء* يتغيّر مع النافذة بدل *الفراغ*. الشعار كان ينكمش من
 * ‎72px‎ إلى ‎26.8px‎ مع السحب — وذلك تصغيرٌ للتصميم لا إعادةُ ترتيب.
 *
 * فالقانون ثلاث جمل:
 *
 * 1. **نافذة macOS هي الإطار الوحيد.** لا بطاقةٌ كبرى تحيط بشاشةٍ كاملة إلا أن
 *    تكون عنصرًا وظيفيًا مقصودًا — والحوار المشروط وحده كذلك.
 * 2. **مقاس عنصرٍ لا يقرأ النافذة.** الشعار والخطوط والأزرار والأيقونات قيمٌ من
 *    سلّم الهوية. ما يقرأ المرأى هو الفراغ وحده.
 * 3. **لا يُخفى محتوًى ولا يُقصّ.** `overflow: hidden` ليس علاجًا لتخطيطٍ لا
 *    يتّسع؛ العلاج تمريرٌ ظاهر أو إعادة ترتيب.
 */
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

const SRC = new URL('.', import.meta.url).pathname;

/** ملفات أنماط المشروع. `design-system/` منسوخ من الهوية فلا يُحاكَم بقانوننا. */
function screenStyleSheets(): { name: string; css: string }[] {
  return readdirSync(SRC)
    .filter((f) => f.endsWith('.css'))
    .map((name) => ({ name, css: stripComments(readFileSync(join(SRC, name), 'utf8')) }));
}

/** التعليقات تشرح القانون وتذكر ما خالفه، فتركُها يجعل كل فحصٍ يمسك نفسه. */
const stripComments = (css: string) => css.replace(/\/\*[\s\S]*?\*\//g, '');

/** كل تصريح `خاصّية: قيمة` مع المحدِّد الذي يملكه. */
function declarations(css: string): { selector: string; prop: string; value: string }[] {
  const out: { selector: string; prop: string; value: string }[] = [];
  for (const [, selector, body] of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    for (const [, prop, value] of (body ?? '').matchAll(/([a-z-]+)\s*:\s*([^;]+)/g)) {
      out.push({ selector: (selector ?? '').trim(), prop: prop ?? '', value: value ?? '' });
    }
  }
  return out;
}

describe('قانون التخطيط على كل الشاشات', () => {
  it('يجد ملفات الأنماط فعلًا', () => {
    // حارسٌ على الحارس: مسحٌ فارغ يجعل كل ما تحته ينجح بلا أن يفحص شيئًا.
    const sheets = screenStyleSheets();
    expect(sheets.length, 'لم يُعثر على ملفات أنماط').toBeGreaterThan(3);
    expect(sheets.map((s) => s.name)).toContain('onboarding.css');
  });

  it('لا مقاس عنصرٍ يقرأ المرأى', () => {
    // الاستثناء الوحيد `min-height: 100vh|100dvh`: هو بسطُ سطح الشاشة على
    // النافذة — أي الخلفية تصل الحواف، وهو نقيض البطاقة لا مثالها.
    // و`max-height`/`max-block-size` مسموحة: سقفُ منطقةٍ تُمرَّر، وهو آليةُ
    // «تمريرٌ ظاهر حين تضيق المساحة» التي يوجبها القانون لا مخالفةٌ له.
    const sized = /^(width|height|min-width|min-height|font-size|inline-size|block-size|min-inline-size|min-block-size)$/;
    const offenders: string[] = [];
    for (const { name, css } of screenStyleSheets()) {
      for (const { selector, prop, value } of declarations(css)) {
        if (!/\d\s*(vw|vh|dvw|dvh|svh|lvh)\b/.test(value)) continue;
        const fills = prop === 'min-height' && /^(100vh|100dvh)$/.test(value.trim());
        if (sized.test(prop) && !fills) offenders.push(`${name}: ${selector} { ${prop}: ${value} }`);
      }
    }
    expect(offenders, `مقاسٌ يتبع النافذة: ${offenders.join(' | ')}`).toEqual([]);
  });

  it('لا رمزًا مخصَّصًا يحمل مقاس عنصرٍ يقرأ المرأى', () => {
    // الالتفاف الذي وقع فعلًا: المقاس لم يُكتب على الخاصّية بل في رمزٍ
    // (`--mark-size: clamp(…, 6vw, …)`) ثم قُرئ في `width`. الفحص أعلاه لا
    // يراه، فيُفحص الرمز نفسه: ما اسمه يدلّ على مقاسٍ لا يجوز أن يقرأ المرأى.
    const looksLikeSize = /-(size|width|height|scale)$/;
    const offenders: string[] = [];
    for (const { name, css } of screenStyleSheets()) {
      for (const { selector, prop, value } of declarations(css)) {
        if (!prop.startsWith('--')) continue;
        if (looksLikeSize.test(prop) && /\d\s*(vw|vh|dvw|dvh)\b/.test(value)) {
          offenders.push(`${name}: ${selector} { ${prop}: ${value} }`);
        }
      }
    }
    expect(offenders, `رمزُ مقاسٍ يتبع النافذة: ${offenders.join(' | ')}`).toEqual([]);
  });

  it('لا تصغير للشاشة كوحدة واحدة', () => {
    // `transform: scale` و`zoom` يعطيان الشكل نفسه مصغّرًا: يبهت النصّ وتضيق
    // مساحات النقر ولا يُكسب سطرٌ واحد. وهما أسهل طريقٍ إلى «إبقاء كل شيء
    // داخل النافذة»، ولذلك يُمنعان بالاسم.
    for (const { name, css } of screenStyleSheets()) {
      expect(css, `${name}: تصغيرٌ بـ scale`).not.toMatch(/transform:[^;}]*\bscale\s*\(/);
      expect(css, `${name}: تصغيرٌ بـ zoom`).not.toMatch(/(^|[\s;{])zoom\s*:/);
    }
  });

  it('لا يُخفى فيضٌ بـ overflow: hidden', () => {
    // إخفاء ما لا يتّسع يجعل العطل غير مرئيّ لا غير موجود: زرٌّ أساسي خلف
    // الحافّة يبقى غير قابلٍ للوصول، والفرق أن أحدًا لن يراه.
    const offenders: string[] = [];
    for (const { name, css } of screenStyleSheets()) {
      for (const { selector, prop, value } of declarations(css)) {
        if (/^overflow(-x|-y|-inline|-block)?$/.test(prop) && /\bhidden\b/.test(value)) {
          offenders.push(`${name}: ${selector} { ${prop}: ${value} }`);
        }
      }
    }
    expect(offenders, `فيضٌ مخفيّ: ${offenders.join(' | ')}`).toEqual([]);
  });

  it('لا بطاقةٌ كبرى تحيط بشاشةٍ كاملة', () => {
    // البطاقة = عرضٌ أقصى + زوايا + ظلّ في محدِّدٍ واحد. هذا ما نُسخ من
    // الـ`mockup` فأنتج «نافذةً داخل نافذة».
    //
    // والمستثنى `.dialog` وحده: حوارٌ مشروط يطفو فوق الشاشة عن قصد، فبطاقته
    // *هي* وظيفته — بها يُعرف أنه طبقةٌ فوق ما تحتها لا جزءٌ منه.
    const offenders: string[] = [];
    for (const { name, css } of screenStyleSheets()) {
      for (const [, selector, body] of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
        const sel = (selector ?? '').trim();
        if (/\bdialog\b/.test(sel)) continue;
        const b = body ?? '';
        const card =
          /max-width\s*:/.test(b) && /border-radius\s*:/.test(b) && /box-shadow\s*:/.test(b);
        if (card) offenders.push(`${name}: ${sel}`);
      }
    }
    expect(offenders, `بطاقةٌ تحيط بشاشة: ${offenders.join(' | ')}`).toEqual([]);
  });

  it('لا استعلام وسائط يغيّر مقاسًا، إنما ترتيبًا أو فراغًا', () => {
    // الاستجابة في المسافة والترتيب لا في المقاس: استعلامٌ يكتب `font-size` أو
    // `width` يعيد بالضبط ما مُنع أعلاه، من بابٍ آخر.
    const sized = /^(width|height|min-width|min-height|font-size|inline-size|block-size)$/;
    const offenders: string[] = [];
    for (const { name, css } of screenStyleSheets()) {
      for (const [, query, block] of css.matchAll(/@media([^{]+)\{((?:[^{}]*\{[^{}]*\})*)/g)) {
        for (const { selector, prop } of declarations(block ?? '')) {
          if (sized.test(prop)) {
            offenders.push(`${name}: @media${(query ?? '').trim()} → ${selector} { ${prop} }`);
          }
        }
      }
    }
    expect(offenders, `استعلامٌ يغيّر مقاسًا: ${offenders.join(' | ')}`).toEqual([]);
  });
});
