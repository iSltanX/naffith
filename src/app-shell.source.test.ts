// @vitest-environment node
/**
 * وعاء الصفحة يعطي حشوةً وعرضًا أقصى — لا صنفًا فارغًا.
 *
 * `app.test.tsx` يثبت أن كل شاشة تُلفّ بـ`.page`؛ وهذا يثبت أن اللفّ يعني
 * شيئًا. الاثنان معًا يسدّان الطريقين إلى العطل نفسه: شاشةٌ خارج الوعاء،
 * ووعاءٌ فقد ما يجعله وعاءً. ولا يكفي أحدهما — أنماط الملفّات لا تُحمَّل في
 * jsdom، فاختبار العرض يرى البنية ولا يرى قيمة حشوةٍ واحدة.
 *
 * لذلك يُقرأ الملف نصًّا لا يُستورد. وبيئة الاختبار `node` لأن `jsdom` يعيد
 * كتابة أصل `import.meta.url` إلى عنوان http فيصير المسار من جذر القرص.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const SHELL = readFileSync(new URL('./app-shell.css', import.meta.url).pathname, 'utf8');
const APP = readFileSync(new URL('./app.css', import.meta.url).pathname, 'utf8');

/** جسم قاعدةٍ واحدة بمحدّدها الحرفي. */
function rule(css: string, selector: string): string {
  const at = css.indexOf(`\n${selector} {`);
  expect(at, `القاعدة ${selector} غير موجودة`).toBeGreaterThan(-1);
  const body = css.slice(at + selector.length + 3);
  return body.slice(0, body.indexOf('}'));
}

describe('وعاء الصفحة', () => {
  it('يحمل الحشوة والعرض الأقصى والتوسيط', () => {
    const body = rule(SHELL, '.page');
    // الحشوة هي ما كان غائبًا عن الشاشات الثلاث: بلا هذه القيم يعود العنوان
    // ملاصقًا للحافة وتُقصّ حلقةُ التركيز على البطاقة المحاذية لها.
    expect(body, 'وعاء بلا حشوة ليس وعاءً').toMatch(/padding:\s*var\(--space-/);
    expect(body).toMatch(/max-width:\s*\S/);
    expect(body).toMatch(/margin-inline:\s*auto/);
  });

  it('لا يُكتب مرّتين: الوعاء مِلك الغلاف لا مِلك شاشة «نَفِّذ — سَطْر»', () => {
    // نسخةٌ ثانية في `app.css` هي ما وقع أصلًا: شاشةٌ واحدة تملك الحشوة
    // وأخواتها بلا شيء. الصنف الذي كان يحملها هناك اسمه `.app`.
    expect(APP).not.toMatch(/^\.app\s*\{/m);
    expect(APP).not.toMatch(/margin-inline:\s*auto/);
  });
});
