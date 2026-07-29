import { describe, it, expect } from 'vitest';
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

import { CRITICAL_FONT_FILES, CRITICAL_FONT_FACES } from './critical-fonts';

// البيئة `node` هي الافتراضية هنا عن قصد: هذا الاختبار يقرأ المصدر من القرص،
// وjsdom يعيد كتابة `import.meta.url` إلى عنوان http فيضيع المسار.
const at = (rel: string): string => fileURLToPath(new URL(rel, import.meta.url));

const fontsCss = readFileSync(at('./design-system/fonts.css'), 'utf8');
const indexHtml = readFileSync(at('../index.html'), 'utf8');

/**
 * ما يحرسه هذا الملف ليس سلوكًا بل اتّساق ثلاثة مواضع لا يجمعها مترجم:
 * قائمة الملفّات هنا، وقواعد `@font-face` في نظام التصميم المنسوخ، والعلامة في
 * ‏`index.html`. نظام التصميم يُنسخ من مشروع الهوية يدويًا، وتغيّرُ ترقيم
 * المقاطع فيه يجعل التحميل المسبق يشير إلى العدم — وهو عطلٌ صامت: لا خطأ في
 * الطرفية، ولا شيء ظاهر إلا عودة القفز الذي أُصلح.
 */
describe('الخطوط الحرجة', () => {
  it('كل ملفّ في القائمة موجود فعلًا في نظام التصميم', () => {
    for (const file of CRITICAL_FONT_FILES) {
      expect(existsSync(at(`./design-system/fonts/${file}`)), file).toBe(true);
    }
  });

  it('كل ملفّ في القائمة تشير إليه قاعدة `@font-face`', () => {
    for (const file of CRITICAL_FONT_FILES) {
      expect(fontsCss.includes(`url('fonts/${file}')`), file).toBe(true);
    }
  });

  it('كل وجهٍ منتظَر معلَنٌ بعائلته ووزنه في نظام التصميم', () => {
    for (const { spec } of CRITICAL_FONT_FACES) {
      const match = /^(\d{3}) 1rem (?:"([^"]+)"|(\S+))$/.exec(spec);
      expect(match, spec).not.toBeNull();
      const weight = match?.[1];
      const family = match?.[2] ?? match?.[3];
      // العائلة والوزن يجب أن يجتمعا في قاعدةٍ واحدة، لا أن يوجد كلٌّ وحده.
      const rule = new RegExp(
        `font-family: '${family}';[^}]*?font-weight: ${weight};`,
        's',
      );
      expect(rule.test(fontsCss), spec).toBe(true);
    }
  });

  it('علامة الحقن ما تزال في `index.html`', () => {
    // الإضافة تُسقط البناء عند غيابها، لكن الاختبار يقولها قبل البناء وبلغةٍ
    // تشرح: العلامة ليست تعليقًا يُنظَّف.
    expect(indexHtml).toContain('<!--@preload-critical-fonts-->');
  });

  it('لا وجهَ منتظَرٌ بلا تحميلٍ مسبق: الطرفان مشتقّان من قائمةٍ واحدة', () => {
    const families = new Set(
      CRITICAL_FONT_FACES.map(({ spec }) => /(?:"([^"]+)"|(\S+))$/.exec(spec)?.[0] ?? ''),
    );
    // كل عائلةٍ ننتظرها لا بدّ أن يكون لها ملفٌّ واحد على الأقل في التحميل
    // المسبق، وإلّا انتظرنا طلبًا لم يبدأ.
    const stems = CRITICAL_FONT_FILES.map((f) => f.split('-')[0]);
    for (const family of families) {
      const stem = family.toLowerCase().replace(/["']/g, '').split(' ')[0];
      expect(stems.some((s) => s === stem), family).toBe(true);
    }
  });
});
