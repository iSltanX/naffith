// @vitest-environment jsdom
/**
 * تنفيذ السمة — العقد بين `settings.ts` و`tokens.css`.
 *
 * الثلاث حالات هنا ليست تفضيلًا في الشكل: `tokens.css` يقرأ غياب السمة على
 * أنه «اتبع النظام»، فكتابة `data-theme="system"` كانت ستُسقط الوضع التلقائي
 * صامتةً — تبقى الصفحة فاتحةً على نظامٍ داكن ولا شيء يشتكي.
 */
import { describe, expect, it } from 'vitest';
import { applyTheme, THEME_ATTRIBUTE } from './theme';

function root(): HTMLElement {
  return document.createElement('html');
}

describe('تنفيذ السمة', () => {
  it('السمة المثبّتة تُكتب على الجذر', () => {
    const el = root();
    applyTheme('dark', el);
    expect(el.getAttribute(THEME_ATTRIBUTE)).toBe('dark');

    applyTheme('light', el);
    expect(el.getAttribute(THEME_ATTRIBUTE)).toBe('light');
  });

  it('«تلقائي» تحذف السمة ولا تكتب قيمةً باسمها', () => {
    const el = root();
    applyTheme('dark', el);
    applyTheme('system', el);

    expect(el.hasAttribute(THEME_ATTRIBUTE)).toBe(false);
    // الحارس الحقيقي: لا قيمة نصّية اسمها `system` تصل إلى الجذر، لأن
    // `tokens.css` لا يعرف محدّدًا لها ولن يطابق شيئًا.
    expect(el.getAttribute(THEME_ATTRIBUTE)).toBeNull();
  });

  it('الاستدعاء المتكرّر بنفس القيمة لا يغيّر النتيجة', () => {
    const el = root();
    applyTheme('dark', el);
    applyTheme('dark', el);
    expect(el.getAttribute(THEME_ATTRIBUTE)).toBe('dark');
  });
});
