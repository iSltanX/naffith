/**
 * عقد المفاتيح بين النواة والواجهة.
 *
 * النواة لا تحمل نصًّا عربيًا: ترسل مفاتيح. الثمن أن مفتاحًا يُضاف في Rust
 * وينسى في الواجهة يظهر للمستخدم نصًّا خامًا مثل `err.dest.inside_source`.
 *
 * هذا الاختبار يقرأ مصدر Rust نفسه ويستخرج كل مفتاح يمكن أن يصدر عنه، ثم
 * يتأكّد أن لكلٍّ ترجمة. لا قائمة يدوية تتقادم — المصدر هو المرجع.
 *
 * والاتجاه الثاني مثله: الواجهة تطلب مفاتيح لا تصدر عن النواة أصلًا
 * (`summary.*` و`nav.*` و`action.*`)، فحراسةُ اتجاهٍ واحد تترك نصفَ العقد بلا
 * حارس — وهذا ما أوقع `summary.estimate.entries` معروضًا بحروف لاتينية داخل
 * واجهة عربية. لذا يُمسح مصدر الواجهة أيضًا أدناه.
 */
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { describe, expect, it } from 'vitest';
import { AR, errorText, t } from './i18n';

const CORE_SRC = new URL('../src-tauri/src', import.meta.url).pathname;
const APP_SRC = new URL('.', import.meta.url).pathname;

function rustSources(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) return rustSources(full);
    return full.endsWith('.rs') ? [full] : [];
  });
}

/** كل نصّ يبدأ بواحدة من بادئات المفاتيح داخل مصادر النواة. */
function keysEmittedByCore(): Set<string> {
  const prefixes = ['err.', 'warn.', 'explain.', 'op.', 'result.'];
  const found = new Set<string>();
  for (const file of rustSources(CORE_SRC)) {
    const text = readFileSync(file, 'utf8');
    // ‏`0-9` جزءٌ من صنف المحارف لا زيادة، نظير الشرطة السفلية أدناه في عدّ
    // العمليات: بدونه كان الفحص يُسقط صامتًا كل مفتاحٍ يحمل رقمًا —
    // `explain.shasum.sha256` و`explain.sips.rotate.90` و
    // `op.disk.hash.sha256.*` و`op.text.encoding.utf8.*` وغيرها — فيمرّ
    // مفتاحٌ بلا ترجمة ولا يسقط الحارس الذي وُضع ليمنع ذلك بعينه.
    for (const match of text.matchAll(/"([a-z0-9_]+(?:\.[a-z0-9_]+)+)"/g)) {
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

// ── الاتجاه الثاني: ما تطلبه الواجهة ───────────────────────────────────

/** ملفات الواجهة عدا الاختبارات: الاختبارات تذكر مفاتيح غائبة عمدًا. */
function frontendSources(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) return frontendSources(full);
    if (/\.test\.tsx?$/.test(entry)) return [];
    return /\.tsx?$/.test(entry) ? [full] : [];
  });
}

/** شكل المفتاح: مقطعان لاتينيان فأكثر. يفصل المفاتيح عن بقية النصوص. */
const KEY_SHAPE = /^[a-z][a-z0-9_]*(?:\.[a-z0-9_]+)+$/;

/**
 * نصّ وسائط النداء الذي قوسه عند `open`، بموازنة الأقواس.
 *
 * لا يكفي التقاط `t('key')` بتعبير نمطي واحد: في `naffith.tsx` نداءٌ وسيطُه
 * شرطيّ يمتدّ على أسطر — `t(complete ? 'a' : 'b')` — ومفتاحاه معًا مطلوبان.
 */
function argsAt(text: string, open: number): string {
  let depth = 0;
  for (let i = open; i < text.length; i += 1) {
    if (text[i] === '(') depth += 1;
    else if (text[i] === ')') {
      depth -= 1;
      if (depth === 0) return text.slice(open + 1, i);
    }
  }
  return '';
}

/**
 * طلبُ ترجمة واحد.
 *
 * `all`: نداء `t` — كل مفتاح في وسيطه معروضٌ في حالةٍ ما، فكلٌّ مطلوب.
 * `any`: نداء `tFirst` — عقدُه «أوّل موجودٍ يفوز، وآخرُ السلسلة هو المعروض
 * حين تغيب كلها»، فيكفي وجود واحد. اشتراط الكل كان سيرفض السلسلة
 * `[field.<op>.<input>.label, field.<input>.label]` التي تبنيها `fieldKeys`
 * في `operations.ts`: أوّلها تخصيصٌ اختياري لا يُكتب إلا عند الحاجة.
 */
type Request = { keys: string[]; any: boolean; where: string };

function requestsFromUi(): Request[] {
  const out: Request[] = [];
  for (const file of frontendSources(APP_SRC)) {
    const text = readFileSync(file, 'utf8');
    const where = relative(APP_SRC, file);
    // ‏`(?<![\w$.])` يمنع التقاط `errorText(` و`.t(` ونحوهما.
    // و`tFormat` مشمولة: مفتاحها نصٌّ حرفيّ كغيره، وإغفالها كان يعني أن نصوص
    // «حول» ذات المواضع المسمّاة تعيش خارج الحارس.
    for (const call of text.matchAll(/(?<![\w$.])(t|tFirst|tFormat)\(/g)) {
      const open = (call.index ?? 0) + call[0].length - 1;
      const args = argsAt(text, open);
      // مفتاحٌ يُبنى وقت التشغيل (قالبٌ نصّي أو متغيّر) لا يُقرأ من المصدر.
      // تخطّيه هنا مقصود، وعائلاته المغلقة تُفتح في الاختبار الذي يليه —
      // فلا يمرّ هذا الفحص فارغًا من ناحيتها.
      if (args.includes('`')) continue;
      const keys = [...args.matchAll(/['"]([^'"]+)['"]/g)]
        .map((m) => m[1] as string)
        .filter((k) => KEY_SHAPE.test(k));
      if (keys.length > 0) out.push({ keys, any: call[1] === 'tFirst', where });
    }
  }
  return out;
}

describe('مفاتيح تطلبها الواجهة', () => {
  it('يستخرج مفاتيح فعلية من مصدر الواجهة', () => {
    // حارس على الحارس: تعبيرٌ نمطيّ معطوب يجعل الفحص التالي يمرّ بلا أن يفحص.
    const keys = new Set(requestsFromUi().flatMap((r) => r.keys));
    expect(keys.size).toBeGreaterThan(20);
    expect(keys.has('summary.estimate.entries')).toBe(true);
    expect(keys.has('summary.estimate.partial')).toBe(true);
    expect(keys.has('action.execute')).toBe(true);
  });

  it('يترجم كل مفتاح ثابت تطلبه الواجهة', () => {
    const missing = new Set<string>();
    for (const req of requestsFromUi()) {
      if (req.any) {
        if (!req.keys.some((k) => k in AR)) missing.add(`${req.where}: ${req.keys.join(' ← ')}`);
      } else {
        for (const key of req.keys) if (!(key in AR)) missing.add(`${req.where}: ${key}`);
      }
    }
    const list = [...missing].sort();
    expect(list, `مفاتيح تطلبها الواجهة بلا ترجمة: ${list.join('، ')}`).toEqual([]);
  });

  it('يترجم كل عائلة مفاتيح تبنيها الواجهة وقت التشغيل', () => {
    // القوالب النصّية يتخطّاها المسح أعلاه، فتُفتح هنا بمتغيّراتها مقروءةً من
    // المصدر لا مكتوبةً يدويًا — قائمةٌ يدوية تتقادم بصمت.
    //
    // ‏`summary.conflict.*` مفتوحةٌ في اختبار سياسة التضارب أعلاه.
    // ‏`log.state.*` و`op.*.title` في `run-log.tsx` تأتيان من سجلّ النواة،
    // والأولى مغلقة فتُفتح هنا؛ والثانية مفتوحة عمدًا لأن السجل قد يحمل عملية
    // لا تعرفها هذه النسخة، ولهذا يقارن `run-log.tsx` النصّ بالمفتاح نفسه.
    const families: Array<{ variants: string[]; keys: (v: string) => string[] }> = [];

    // أسباب المغادرة: `ExitCost` بلا `free` — «بلا كلفة» لا حوار له.
    const nav = readFileSync(join(APP_SRC, 'nav.ts'), 'utf8');
    const costBody = nav.slice(nav.indexOf('export type ExitCost'));
    families.push({
      variants: [...costBody.slice(0, costBody.indexOf(';')).matchAll(/\|\s*'([a-z_]+)'/g)]
        .map((m) => m[1] as string)
        .filter((c) => c !== 'free'),
      keys: (v) => ['title', 'body', 'stay', 'leave'].map((p) => `nav.leave.${v}.${p}`),
    });

    // خطوات الترحيب: `STEPS` في `onboarding.tsx`.
    const onboarding = readFileSync(join(APP_SRC, 'onboarding.tsx'), 'utf8');
    const stepsBody = onboarding.slice(onboarding.indexOf('const STEPS'));
    families.push({
      variants: [...stepsBody.slice(0, stepsBody.indexOf(']')).matchAll(/'([a-z0-9_]+)'/g)].map(
        (m) => m[1] as string,
      ),
      keys: (v) => ['title', 'body'].map((p) => `onboarding.${v}.${p}`),
    });

    // حالات السجل: `State` في `journal.rs`، وserde يكتبها snake_case.
    const journal = readFileSync(join(CORE_SRC, 'journal.rs'), 'utf8');
    const stateBody = journal.slice(journal.indexOf('pub enum State {'));
    families.push({
      variants: [...stateBody.slice(0, stateBody.indexOf('\n}')).matchAll(/^ {4}(\w+)\s*[,{]/gm)]
        .map((m) => (m[1] as string).replace(/(?<!^)([A-Z])/g, '_$1').toLowerCase()),
      keys: (v) => [`log.state.${v}`],
    });

    for (const family of families) {
      expect(family.variants.length, 'لم تُقرأ متغيّرات العائلة من المصدر').toBeGreaterThan(1);
      for (const variant of family.variants) {
        for (const key of family.keys(variant)) {
          expect(key in AR, `${key} غير مترجم`).toBe(true);
        }
      }
    }
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

/**
 * سلّم النصّ الثلاثي: البطاقة «ماذا تفعل؟»، وصفحة التنفيذ «ماذا سيحدث؟»،
 * والحقل «ماذا أختار هنا؟».
 *
 * الطبقتان الأوليان مفتاحان لكل عملية. وغيابُ `execution` لا يُسقط شيئًا في
 * التشغيل — `tOptional` تطويه بصمت — فالعمليةُ الجديدة تصل إلى الشاشة بلا
 * جوابٍ عن «ماذا سيحدث؟» ولا أحد يعلم. هذا الحارس هو ما يجعل الغياب مسموعًا.
 */
/**
 * عائلات مفاتيح الإعدادات المبنيّة وقت التشغيل.
 *
 * `settings-screen.tsx` يبني مفاتيحه بقوالب نصّية — `t(`${keyPrefix}.change`)`
 * و`t(`settings.${name}.title`)` — والماسح أعلاه يتخطّى القوالب عمدًا لأنه لا
 * يستطيع تقييمها. فمرّ مفتاحٌ ناقص (`settings.workpath.choose`) إلى الشاشة
 * وظهر حرفيًّا للمستخدم. هذا الحارس يفتح تلك العائلات المغلقة صراحةً.
 */
describe('عائلات مفاتيح الإعدادات', () => {
  it('كل بطاقة مسار تملك لواحقها الخمس', () => {
    for (const prefix of ['settings.workpath', 'settings.node', 'settings.cargo']) {
      for (const suffix of ['title', 'unset', 'choose', 'change', 'clear']) {
        const key = `${prefix}.${suffix}`;
        expect(AR, key).toHaveProperty(key);
      }
    }
  });

  it('كل صفّ تبديل يملك عنوانه ومتنه', () => {
    for (const name of ['welcome', 'sound', 'confirm']) {
      for (const suffix of ['title', 'body']) {
        const key = `settings.${name}.${suffix}`;
        expect(AR, key).toHaveProperty(key);
      }
    }
  });

  it('كل لسان يملك اسمه وعنوان قسمه', () => {
    for (const tab of ['general', 'appearance', 'developer', 'about']) {
      expect(AR, `settings.tab.${tab}`).toHaveProperty(`settings.tab.${tab}`);
      expect(AR, `settings.${tab}.title`).toHaveProperty(`settings.${tab}.title`);
    }
  });

  it('كل خيار سمة وحجم أيقونات يملك نصّه', () => {
    for (const theme of ['system', 'light', 'dark']) {
      expect(AR, `settings.theme.${theme}`).toHaveProperty(`settings.theme.${theme}`);
    }
    for (const size of ['small', 'medium', 'large']) {
      expect(AR, `settings.iconsize.${size}`).toHaveProperty(`settings.iconsize.${size}`);
    }
  });
});

/**
 * حوار تأكيد التنفيذ (إعداد «تأكيد قبل التنفيذ») يبني مفتاحه بقالبٍ نصّي —
 * `t(`run.confirm.${danger}.title`)` في `app.tsx` — فيتخطّاه الماسح أعلاه
 * كما يتخطّى عائلات الإعدادات. ثلاث درجات لا أربع: «قراءة فقط» لا تصل هذا
 * الحوار أصلًا، فلا مفتاح لها هنا عمدًا.
 */
describe('عائلة مفاتيح تأكيد التنفيذ', () => {
  it('كل درجة خطرٍ تُعدّل أو تتلف تملك عنوانها ومتنها', () => {
    // القائمة تُقرأ من نوع `Danger` نفسه في ipc.ts لا تُكتب هنا يدويًا —
    // كأخواتها في هذا الملف (`nav.leave.*` من nav.ts، `log.state.*` من
    // journal.rs): درجة خطرٍ رابعة تُضاف يومًا هناك يجب أن تُفقد ترجمتها هنا
    // لا أن تمرّ صامتة لأن أحدًا نسي تحديث قائمةٍ ثانية.
    const ipc = readFileSync(join(APP_SRC, 'ipc.ts'), 'utf8');
    const match = /export type Danger =([^;]+);/.exec(ipc);
    expect(match, 'لم يُعثر على تعريف Danger في ipc.ts').toBeTruthy();
    const allDangers = [...(match?.[1] ?? '').matchAll(/'([a-z]+)'/g)].map((m) => m[1] as string);
    expect(allDangers.length, 'لم تُستخرج أي درجة من Danger').toBeGreaterThan(1);

    // «قراءة فقط» لا تصل حوار التأكيد أصلًا — انظر توثيق `confirmBeforeExecute`
    // في settings.ts — فلا مفتاح لها هنا عمدًا، لا سهوًا.
    const gated = allDangers.filter((d) => d !== 'safe');
    expect(gated).toEqual(['creates', 'modifies', 'destructive']);

    for (const danger of gated) {
      for (const suffix of ['title', 'body']) {
        const key = `run.confirm.${danger}.${suffix}`;
        expect(AR, key).toHaveProperty(key);
      }
    }
  });
});

describe('سلّم نصّ العمليات', () => {
  // الشرطة السفلية جزءٌ من صنف المحارف لا زيادة: معرّفات مثل
  // `system.process.open_files` و`disk.directory.open_handles` تحملها، وكان
  // النمط بدونها **يُسقطها من العدّ صامتًا** — فتصل عمليةٌ إلى الشاشة بلا
  // جوابٍ عن «ماذا سيحدث؟» ولا يسقط الحارس أدناه، وهو بالضبط ما وُجد ليمنعه.
  const ids = Object.keys(AR)
    .filter((k) => /^op\.[a-z0-9._]+\.title$/.test(k))
    .map((k) => k.slice(3, -'.title'.length))
    // العملية الداخلية لا تظهر في أي إصدار، فلا شاشة تنفيذ لها.
    .filter((id) => id !== 'internal.echo');

  it('يقرأ التسع والسبعين عملية من القاموس نفسه', () => {
    expect(ids).toHaveLength(79);
  });

  it('لكل عملية جوابٌ عن «ماذا تفعل؟» وآخر عن «ماذا سيحدث؟»', () => {
    const missing = ids.flatMap((id) => [
      ...(`op.${id}.description` in AR ? [] : [`op.${id}.description`]),
      ...(`op.${id}.execution` in AR ? [] : [`op.${id}.execution`]),
    ]);
    expect(missing, `نصٌّ ناقص: ${missing.join(', ')}`).toEqual([]);
  });

  it('جواب البطاقة جملةٌ واحدة، وجواب التنفيذ ليس تكرارًا لها', () => {
    for (const id of ids) {
      const card = t(`op.${id}.description`);
      const exec = t(`op.${id}.execution`);
      // جملةٌ واحدة: تنتهي بنقطة ولا تحمل فاصلَ جملةٍ في وسطها. والمقياس
      // «نقطةٌ يتلوها فراغ» لا «نقطة»، وإلا عُدّت ⁦TAR.GZ⁩ جملتين.
      expect(card, `بطاقة ${id} أكثر من جملة`).not.toMatch(/\.\s/u);
      expect(card.trimEnd().endsWith('.'), `بطاقة ${id} لا تنتهي بنقطة`).toBe(true);
      expect(card.length, `بطاقة ${id} طويلة`).toBeLessThan(90);
      expect(exec, `تنفيذ ${id} يكرّر البطاقة`).not.toBe(card);
      expect(exec.length, `تنفيذ ${id} أطول من ثلاث جمل قصيرة`).toBeLessThan(220);
    }
  });
});

/**
 * الطبقة الثالثة: «ماذا أختار هنا؟».
 *
 * نصّ الحقل يُقرأ في لحظة الاختيار، بين تسميةٍ فوقه وحقلٍ تحته — فالسرد فيه
 * يُتخطّى لا يُقرأ. والطمأنةُ العامّة («لا يُمسّ الأصل») صارت جواب صفحة
 * التنفيذ، فتكرارُها هنا يُطيل النصّ ويزحم المعلومة الخاصّة بالحقل.
 */
describe('نصّ الحقول', () => {
  const help = Object.entries(AR).filter(([k]) => /^field\.[a-z0-9._]+\.help$/.test(k));

  it('يغطّي الحقول المعلَنة كلّها', () => {
    expect(help.length).toBe(92);
  });

  it('يبقى قصيرًا بما يُقرأ في لحظة الاختيار', () => {
    const long = help.filter(([, v]) => v.length > 110).map(([k, v]) => `${k} (${v.length})`);
    expect(long, `نصّ حقلٍ أطول ممّا يُقرأ: ${long.join(', ')}`).toEqual([]);
  });

  it('لا يعيد جملة «ماذا سيحدث؟» في كل حقل مسار', () => {
    // العبارتان كانتا مكرّرتين في عشرة حقول؛ موضعهما صفحة التنفيذ.
    const echoed = help
      .filter(([, v]) => /لا يُمسّ ولا يتغيّر|يُقرأ ولا يُعدَّل\.$/.test(v))
      .map(([k]) => k);
    expect(echoed, `طمأنةٌ مكرّرة عن صفحة التنفيذ: ${echoed.join(', ')}`).toEqual([]);
  });
});
