/**
 * تنفيذ السمة على عنصر الجذر.
 *
 * ## لماذا ملفٌّ مستقل بدالّةٍ واحدة
 *
 * `tokens.css` يعرف ثلاث حالات: لا سمة مثبّتة (فيتبع `prefers-color-scheme`)،
 * أو `data-theme="light"`، أو `data-theme="dark"`. و`settings.ts` يعرف نفس
 * الثلاث بأسماء المنتج. الترجمة بينهما سطران — لكن موضعها يقرّر شيئًا: لو
 * عاشت داخل شاشة الإعدادات لَما طُبِّقت السمة إلا بعد أن يفتحها المستخدم،
 * فيبدأ التطبيق فاتحًا ثم يقفز إلى الداكن. فهي هنا، ويستدعيها الجذر عند
 * الإقلاع قبل أول رسم، وتستدعيها الإعدادات عند كل تغيير.
 *
 * ## لماذا `system` تحذف السمة ولا تكتب قيمة
 *
 * لأن «اتبع النظام» ليست لونًا ثالثًا بل غيابُ قرار. كتابة `data-theme="system"`
 * كانت ستجعل كل محدِّد في `tokens.css` يحتاج استثناءً لها؛ وحذفُ السمة يعيد
 * الأمر إلى `@media (prefers-color-scheme: dark)` وحده — وهو المكان الذي يعرف
 * الجواب أصلًا، ويتابع تغيّره في نفس الجلسة بلا أن يسأله أحد.
 */
import type { ThemePreference } from './settings';

/** السمة على عنصر الجذر. تُقرأ في `tokens.css` وفي حارس المصدر معًا. */
export const THEME_ATTRIBUTE = 'data-theme';

/**
 * يطبّق السمة على المستند. آمنة الاستدعاء قبل الرسم وبعده وأكثر من مرّة.
 *
 * `root` مُمرَّرة كي يكون الاختبار حتميًا ولا يعتمد على `document` عالمي.
 */
export function applyTheme(theme: ThemePreference, root: HTMLElement): void {
  if (theme === 'system') {
    root.removeAttribute(THEME_ATTRIBUTE);
    return;
  }
  root.setAttribute(THEME_ATTRIBUTE, theme);
}
