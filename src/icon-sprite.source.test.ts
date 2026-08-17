// @vitest-environment node
/**
 * ورقة الرموز — كل ما يُشار إليه موجودٌ فيها.
 *
 * `<use href="#i-x">` على رمزٍ غير معرَّف **لا يفشل**: لا خطأ في وحدة التحكّم،
 * ولا تحذير من `tsc`، ولا اختبار مكوّنٍ يسقط — لأن `@testing-library` تعاين
 * الشجرة لا الرسم. النتيجة فراغٌ في مكان الأيقونة، لا يظهر إلا للعين وفي
 * الشاشة الحقيقية.
 *
 * وقد وقع فعلًا: `run-stream.tsx` كان يشير إلى `#i-terminal` ولا رمز بهذا
 * المعرّف في الورقة، فبقي عنوان مجرى التنفيذ بلا أيقونته وكل الاختبارات خضراء.
 *
 * فالحارس هنا يعبر الحدّين معًا: الواجهة تكتب `#i-*` في مصدرها، والنواة تُخرج
 * أيقونة كل قسم في `categories.rs` — والورقة واحدة، فيجب أن يَرد الطرفان إليها.
 */
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

const SRC = new URL('./', import.meta.url).pathname;
const SPRITE = readFileSync(join(SRC, 'design-system/icons.svg'), 'utf8');
const CATEGORIES = readFileSync(
  new URL('../src-tauri/src/categories.rs', import.meta.url).pathname,
  'utf8',
);

/** معرّفات الرموز المعرَّفة فعلًا في الورقة. */
const defined = new Set(
  [...SPRITE.matchAll(/<symbol\s+id="(i-[a-z0-9-]+)"/g)].map((m) => m[1] as string),
);

/** مصادر الإنتاج وحدها: الاختبارات تحمل معرّفات وهمية عمدًا. */
const productionSources = readdirSync(SRC)
  .filter((f) => /\.tsx?$/.test(f) && !/\.test\.tsx?$/.test(f))
  .map((f) => ({ file: f, text: readFileSync(join(SRC, f), 'utf8') }));

const referencesIn = (text: string) =>
  [...text.matchAll(/#(i-[a-z0-9-]+)/g)].map((m) => m[1] as string);

describe('ورقة الرموز تغطّي كل ما يشير إليها', () => {
  it('تقرأ رموزًا معرَّفة فعلًا، فلا يمرّ الاختبار على ورقةٍ فارغة', () => {
    expect(defined.size).toBeGreaterThan(30);
    expect(defined.has('i-terminal')).toBe(true);
  });

  it('كل رمزٍ تشير إليه الواجهة معرَّفٌ في الورقة', () => {
    const missing = productionSources.flatMap(({ file, text }) =>
      referencesIn(text)
        .filter((id) => !defined.has(id))
        .map((id) => `${file} → #${id}`),
    );
    expect(missing).toEqual([]);
  });

  it('كل أيقونة قسمٍ تُخرجها النواة معرَّفةٌ في الورقة', () => {
    const missing = referencesIn(CATEGORIES).filter((id) => !defined.has(id));
    expect(missing).toEqual([]);
  });

  it('الورقة لا تحمل معرّفًا مكرَّرًا يُسكِت أحد تعريفَيه', () => {
    const ids = [...SPRITE.matchAll(/<symbol\s+id="(i-[a-z0-9-]+)"/g)].map((m) => m[1]);
    expect(ids.length).toBe(defined.size);
  });
});
