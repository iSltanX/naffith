/**
 * عقد المفاتيح بين النواة والواجهة.
 *
 * النواة لا تحمل نصًّا عربيًا: ترسل مفاتيح. الثمن أن مفتاحًا يُضاف في Rust
 * وينسى في الواجهة يظهر للمستخدم نصًّا خامًا مثل `err.dest.inside_source`.
 *
 * هذا الاختبار يقرأ مصدر Rust نفسه ويستخرج كل مفتاح يمكن أن يصدر عنه، ثم
 * يتأكّد أن لكلٍّ ترجمة. لا قائمة يدوية تتقادم — المصدر هو المرجع.
 */
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { AR, errorText, t } from './i18n';

const CORE_SRC = new URL('../src-tauri/src', import.meta.url).pathname;

function rustSources(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) return rustSources(full);
    return full.endsWith('.rs') ? [full] : [];
  });
}

/** كل نصّ يبدأ بواحدة من بادئات المفاتيح داخل مصادر النواة. */
function keysEmittedByCore(): Set<string> {
  const prefixes = ['err.', 'warn.', 'explain.', 'op.'];
  const found = new Set<string>();
  for (const file of rustSources(CORE_SRC)) {
    const text = readFileSync(file, 'utf8');
    for (const match of text.matchAll(/"([a-z_]+(?:\.[a-z_]+)+)"/g)) {
      const key = match[1];
      if (key && prefixes.some((p) => key.startsWith(p))) found.add(key);
    }
  }
  return found;
}

describe('قاموس الواجهة', () => {
  it('يستخرج مفاتيح فعلية من مصدر النواة', () => {
    const keys = keysEmittedByCore();
    // حارس على الاختبار نفسه: لو تعطّلت القراءة لصار يمرّ بلا أن يفحص شيئًا.
    expect(keys.size).toBeGreaterThan(20);
    expect(keys.has('err.dest.inside_source')).toBe(true);
    expect(keys.has('explain.ditto.keep_parent')).toBe(true);
  });

  it('يترجم كل مفتاح تصدره النواة', () => {
    const missing = [...keysEmittedByCore()].filter((k) => !(k in AR)).sort();
    expect(missing, `مفاتيح بلا ترجمة: ${missing.join(', ')}`).toEqual([]);
  });

  it('يترجم كل حالة رفضٍ لاسم ملف', () => {
    // مفاتيح فرعية تُبنى وقت التشغيل من `detail`، فلا يلتقطها المسح أعلاه.
    for (const reason of [
      'empty',
      'too_long',
      'contains_separator',
      'contains_nul',
      'contains_control',
      'dot_or_dot_dot',
      'leading_dot',
      'trailing_space_or_dot',
    ]) {
      const key = `err.name.invalid.${reason}`;
      expect(key in AR, `${key} غير مترجم`).toBe(true);
    }
  });

  it('يترجم كل سبب لبطلان الخطة', () => {
    for (const reason of [
      'source_gone',
      'source_replaced',
      'destination_gone',
      'destination_not_writable',
      'final_path_appeared',
      'tool_gone',
    ]) {
      expect(`err.plan.stale.${reason}` in AR, `${reason} غير مترجم`).toBe(true);
    }
  });

  it('يترجم كل سياسة تضارب يمكن أن تعلنها عملية', () => {
    // المفتاح يُبنى وقت التشغيل من `plan.conflict`، والمتغيّرات تُقرأ من
    // `spec.rs` نفسه: إضافة متغيّر ثالث في Rust دون ترجمة تسقط هنا.
    const spec = readFileSync(join(CORE_SRC, 'spec.rs'), 'utf8');
    const body = spec.slice(spec.indexOf('pub enum Conflict'));
    const variants = [...body.slice(0, body.indexOf('}')).matchAll(/^\s{4}(\w+),/gm)].map(
      (m) => m[1] as string,
    );
    expect(variants.length, 'لم تُقرأ متغيّرات Conflict من المصدر').toBeGreaterThan(1);
    for (const v of variants) {
      // serde يكتبها snake_case، وهو ما يصل الواجهة.
      const key = `summary.conflict.${v.replace(/(?<!^)([A-Z])/g, '_$1').toLowerCase()}`;
      expect(key in AR, `${key} غير مترجم`).toBe(true);
    }
  });

  it('لا يترك نصًّا فارغًا في القاموس', () => {
    const empty = Object.entries(AR).filter(([, v]) => v.trim() === '');
    expect(empty).toEqual([]);
  });

  it('يعزل الرموز اللاتينية بعزلٍ حقيقي لا بعلامة اتجاه', () => {
    // بلا أي علامة، ينقلب `__MACOSX` إلى `MACOSX__` داخل جملة عربية: الشرطتان
    // محايدتان اتجاهيًا فتنضمّان إلى الجملة. قيس هذا في المتصفّح فعلًا.
    //
    // ‏LRM ‎(U+200E)‎ يعالج الحالة الراهنة، لكنه محرفٌ قويّ لا عازل: أثره يعتمد
    // على ما حوله، فيكفي أن تتغيّر صياغة الجملة ليعود الانقلاب. LRI…PDI عزلٌ
    // للمقطع نفسه مهما تغيّر جواره — وقاموسٌ يُحرَّر بمرور الوقت يحتاج الأمتن.
    const withLrm = Object.entries(AR).filter(([, v]) => v.includes('‎'));
    expect(withLrm.map(([k]) => k), 'استعمل \\u2066…\\u2069 بدل \\u200E').toEqual([]);
  });

  it('يوازن كل عزلٍ بإغلاقه', () => {
    // عزلٌ غير مغلق يسرّب اتجاهه إلى بقية الفقرة.
    const unbalanced = Object.entries(AR).filter(([, v]) => {
      const open = (v.match(/⁦/g) ?? []).length;
      const close = (v.match(/⁩/g) ?? []).length;
      return open !== close;
    });
    expect(unbalanced.map(([k]) => k)).toEqual([]);
  });

  it('يعزل كل رمز تقني يبدأ أو ينتهي بمحرف محايد', () => {
    // النقطة والشرطة السفلية والشرطة محايدة اتجاهيًا: تركها بلا عزل داخل جملة
    // عربية يضعها في الطرف الخطأ.
    const risky = ['__MACOSX', '.zip'];
    for (const [key, value] of Object.entries(AR)) {
      for (const token of risky) {
        if (!value.includes(token)) continue;
        const at = value.indexOf(token);
        expect(
          value[at - 1] === '⁦' && value[at + token.length] === '⁩',
          `${key}: ${token} غير معزول`,
        ).toBe(true);
      }
    }
  });

  it('لا يكتب مفتاحًا مكان نصّ', () => {
    // نصٌّ يبدأ بـ `err.` أو `explain.` يعني أن أحدهم نسخ المفتاح في مكان النص.
    const suspicious = Object.entries(AR).filter(([, v]) => /^(err|warn|explain|op)\./.test(v));
    expect(suspicious).toEqual([]);
  });
});

describe('t و errorText', () => {
  it('يعيد النص العربي للمفتاح المعروف', () => {
    expect(t('action.execute')).toBe('نفِّذ');
  });

  it('يعيد المفتاح كما هو إن لم يُعرف، فلا يختفي النص', () => {
    expect(t('err.does.not.exist')).toBe('err.does.not.exist');
  });

  it('يفضّل الصياغة الأدقّ حين تصف النواة السبب', () => {
    expect(errorText('err.name.invalid', 'leading_dot')).toBe(AR['err.name.invalid.leading_dot']);
    expect(errorText('err.name.invalid')).toBe(AR['err.name.invalid']);
  });

  it('يرجع إلى الصياغة العامة إن كان التفصيل غير معروف', () => {
    expect(errorText('err.name.invalid', 'something_new')).toBe(AR['err.name.invalid']);
  });

  it('يتحمّل تفصيلًا ليس نصًّا', () => {
    expect(errorText('err.path.missing', { tool: 'ditto' })).toBe(AR['err.path.missing']);
    expect(errorText('err.path.missing', null)).toBe(AR['err.path.missing']);
  });
});
