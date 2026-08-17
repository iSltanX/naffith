/**
 * مكتبة العمليات — الاشتقاق والبحث.
 *
 * هذا هو الاختبار الذي وعد به رأس `library.ts`: كل الأقسام والعمليات هنا
 * **مخترعة**، ولا واحدة منها موجودة في هذا البناء. لو عاد أحدٌ يومًا فكتب
 * `if (op.id === '…')`، أو رتّب القائمة بأسماء مكتوبة بيده، أو كتب عددًا بدل
 * أن يحسبه — سقط أول اختبار في الملف.
 *
 * والنصف الثاني عن البحث العربي: أن يجد من كتب «الارشيف» ما كُتب «الأرشيف»،
 * وأن تُضيّق كلمةٌ ثانيةٌ النتيجة لا تُوسّعها.
 */
import { describe, expect, it } from 'vitest';
import {
  availabilityOf,
  fold,
  favouriteOperations,
  findCategory,
  findOperation,
  iconForCategory,
  isAvailable,
  operationsIn,
  recentOperations,
  search,
  toCategoryCard,
  toOperationCard,
  toggleFavourite,
} from './library';
import type { CategoryCard, OperationCard } from './library';
import type {
  CategoryId,
  CategorySummary,
  InputSummary,
  JournalEntry,
  OperationSummary,
} from './ipc';

// ── فهرسٌ مخترع بالكامل ────────────────────────────────────────────────

function input(id: string, kind: string, extra: Record<string, unknown> = {}): InputSummary {
  return { ...extra, id, required: true, kind } as InputSummary;
}

function op(
  id: string,
  category: CategoryId,
  over: Partial<OperationSummary> = {},
): OperationSummary {
  return {
    id,
    title_key: `op.${id}.title`,
    description_key: `op.${id}.desc`,
    category,
    danger: 'safe',
    conflict: 'no_artifact',
    tool: 'owlctl',
    availability: { state: 'available' },
    sort_order: 10,
    search_terms: ['owlctl'],
    inputs: [],
    ...over,
  };
}

function category(id: CategoryId, over: Partial<CategorySummary> = {}): CategorySummary {
  return {
    id,
    title_key: `cat.${id}.title`,
    description_key: `cat.${id}.description`,
    icon: `#i-${id}`,
    sort_order: 10,
    kind: 'operations',
    operation_count: 0,
    available_count: 0,
    ...over,
  };
}

const CATEGORIES: CategoryCard[] = [
  category('files', { sort_order: 10, operation_count: 2, available_count: 2 }),
  category('git', { sort_order: 20, operation_count: 1, available_count: 0 }),
  category('history', { sort_order: 30, kind: 'journal' }),
].map(toCategoryCard);

const OPERATIONS: OperationSummary[] = [
  op('zeta.summon.owl', 'files', { search_terms: ['owlctl', 'استدعاء', 'summon'] }),
  op('alpha.fold.paper', 'files', {
    danger: 'creates',
    conflict: 'refuse',
    tool: 'origami',
    search_terms: ['origami', 'طيّ', 'fold'],
    sort_order: 20,
  }),
  op('mu.chart.stars', 'git', {
    tool: 'astro',
    availability: { state: 'tool_missing', tool: 'astro' },
    search_terms: ['astro', 'نجوم'],
  }),
];

const CARDS: OperationCard[] = OPERATIONS.map((o) => toOperationCard(o, CATEGORIES));

// ── البطاقات ───────────────────────────────────────────────────────────

describe('البطاقات تُشتقّ ولا تُكتب', () => {
  it('يعرض عملياتٍ وأقسامًا لا وجود لها في هذا البناء', () => {
    expect(CARDS.map((c) => c.id)).toEqual([
      'zeta.summon.owl',
      'alpha.fold.paper',
      'mu.chart.stars',
    ]);
    expect(CATEGORIES.map((c) => c.id)).toEqual(['files', 'git', 'history']);
  });

  it('يحفظ ترتيب النواة ولا يفرز', () => {
    // النواة رتّبت بـ(ترتيب القسم، ترتيب العملية، المعرّف). فرزٌ ثانٍ هنا
    // يجعل الواجهة تقرّر أولويةَ عرضٍ ليست لها.
    expect(CARDS.map((c) => c.id)).not.toEqual([...CARDS.map((c) => c.id)].sort());
  });

  it('البطاقة لا تحمل نصًّا بل مفاتيح نصوص', () => {
    const [first] = CARDS;
    expect(first?.titleKey).toBe('op.zeta.summon.owl.title');
    expect(first?.descriptionKey).toBe('op.zeta.summon.owl.desc');
    // ولا نصَّ عرضٍ واحدًا في البطاقة: كل ما يُقرأ على الشاشة يأتي من
    // `i18n.ts` عبر مفتاح. ويُستثنى `searchTerms` — وهي عربيةٌ عمدًا لأنها
    // مرادفاتٌ تعلنها النواة كي يجدها البحث، ولا تُعرض على أحد.
    const { searchTerms, ...displayed } = first ?? ({} as OperationCard);
    expect(searchTerms.some((term) => /[؀-ۿ]/.test(term))).toBe(true);
    expect(JSON.stringify(displayed)).not.toMatch(/[؀-ۿ]/);
  });

  it('أيقونة العملية أيقونة قسمها، لا خريطةٌ ثانية في الواجهة', () => {
    expect(iconForCategory('files', CATEGORIES)).toBe('#i-files');
    expect(CARDS[0]?.icon).toBe('#i-files');
    expect(CARDS[2]?.icon).toBe('#i-git');
  });

  it('قسمٌ لا تعرفه هذه النسخة يأخذ أيقونةً محايدة لا يسقط', () => {
    // نواةٌ أحدث قد تعلن قسمًا لا يعرفه هذا البناء. بطاقةٌ بلا أيقونة أهون من
    // شاشةٍ لا تُرسم.
    expect(iconForCategory('images', CATEGORIES)).toBe('#i-file');
  });

  it('يجد البطاقة بمعرّفها، ولا يجد ما لم يعد في الفهرس', () => {
    expect(findOperation(CARDS, 'alpha.fold.paper')?.id).toBe('alpha.fold.paper');
    expect(findOperation(CARDS, 'gone.op')).toBeUndefined();
    expect(findCategory(CATEGORIES, 'files')?.id).toBe('files');
    expect(findCategory(CATEGORIES, 'nowhere')).toBeUndefined();
  });

  it('يصفّي عمليات القسم بلا إعادة ترتيب', () => {
    expect(operationsIn(CARDS, 'files').map((c) => c.id)).toEqual([
      'zeta.summon.owl',
      'alpha.fold.paper',
    ]);
    expect(operationsIn(CARDS, 'git').map((c) => c.id)).toEqual(['mu.chart.stars']);
    expect(operationsIn(CARDS, 'text')).toEqual([]);
  });
});

// ── التوفّر ────────────────────────────────────────────────────────────

describe('التوفّر: سؤالان لا سؤال', () => {
  it('أداةٌ غائبة تُعلن باسمها كي يعرف المستخدم ما ينقصه', () => {
    const a = availabilityOf(op('x.y', 'git', { availability: { state: 'tool_missing', tool: 'git' } }));
    expect(a).toEqual({ state: 'tool_missing', tool: 'git' });
    expect(isAvailable(a)).toBe(false);
  });

  it('مدخلٌ لا تعرف هذه الواجهة رسمه يعطّل العملية ويسمّي نوعه', () => {
    // تطبيقٌ يشحن نواةً أحدث من واجهته: العملية تُعرض معطّلةً بسببٍ مفهوم بدل
    // أن تفتح شاشةً فارغة أو تُسقط الرسم.
    const a = availabilityOf(op('x.y', 'files', { inputs: [input('mystery', 'hologram')] }));
    expect(a).toEqual({ state: 'unsupported', unknownKinds: ['hologram'] });
  });

  it('غياب الأداة يسبق نوع المدخل: هو السبب الذي يمكن فعل شيء حياله', () => {
    const a = availabilityOf(
      op('x.y', 'git', {
        availability: { state: 'tool_missing', tool: 'git' },
        inputs: [input('mystery', 'hologram')],
      }),
    );
    expect(a.state).toBe('tool_missing');
  });

  it('كل نوعٍ تعلنه النواة اليوم مرسومٌ في هذه الواجهة', () => {
    // القائمة تُقرأ من `ipc.ts` نفسه في `wire-contract.test.ts`؛ هنا يُثبَّت
    // أن ما تعرفه النواة يُرسم فعلًا لا يُعطَّل.
    const kinds = [
      'existing_dir',
      'existing_file',
      'existing_path',
      'target_dir',
      'new_name',
      'new_dir_name',
      'text',
      'choice',
      'number',
      'url',
      'flag',
    ];
    const a = availabilityOf(op('x.y', 'files', { inputs: kinds.map((k) => input(k, k)) }));
    expect(a).toEqual({ state: 'available' });
  });

  it('الأنواع المجهولة تُجمع بلا تكرار ومرتّبة', () => {
    const a = availabilityOf(
      op('x.y', 'files', {
        inputs: [input('a', 'zeta'), input('b', 'alpha'), input('c', 'zeta')],
      }),
    );
    expect(a).toEqual({ state: 'unsupported', unknownKinds: ['alpha', 'zeta'] });
  });
});

// ── طيّ النصّ العربي ───────────────────────────────────────────────────

describe('طيّ النصّ قبل المقارنة', () => {
  it('يوحّد الألفات فيجد «الأرشيف» من كتب «الارشيف»', () => {
    expect(fold('الأرشيف')).toBe(fold('الارشيف'));
    expect(fold('إنشاء')).toBe(fold('انشاء'));
    expect(fold('آخر')).toBe(fold('اخر'));
  });

  it('يُسقط التشكيل والتطويل', () => {
    expect(fold('نَفِّذ')).toBe(fold('نفذ'));
    expect(fold('ســطر')).toBe(fold('سطر'));
  });

  it('يوحّد التاء المربوطة والياء والهمزات المتوسّطة', () => {
    expect(fold('عملية')).toBe(fold('عمليه'));
    expect(fold('على')).toBe(fold('علي'));
    expect(fold('مسؤول')).toBe(fold('مسوول'));
    expect(fold('قائمة')).toBe(fold('قايمه'));
  });

  it('يوحّد الأرقام العربية-الهندية مع اللاتينية', () => {
    expect(fold('٢٥٦')).toBe('256');
    expect(fold('۲۵۶')).toBe('256');
  });

  it('يخفض حالة الأحرف اللاتينية ويسوّي الفراغ', () => {
    expect(fold('  DiTTo   Zip ')).toBe('ditto zip');
  });
});

// ── البحث ─────────────────────────────────────────────────────────────

describe('البحث في المكتبة', () => {
  it('بلا استعلام يعرض المكتبة كاملةً ويعلن أنه غير نشط', () => {
    const r = search('', CARDS, CATEGORIES);
    expect(r.active).toBe(false);
    expect(r.operations).toHaveLength(CARDS.length);
    expect(r.categories).toHaveLength(CATEGORIES.length);
  });

  it('الفراغ وحده ليس استعلامًا', () => {
    expect(search('   ', CARDS, CATEGORIES).active).toBe(false);
  });

  it('يجد العملية باسم أداتها', () => {
    // من يعرف `origami` يكتبها، ولا يعرف عنوانها العربي.
    const r = search('origami', CARDS, CATEGORIES);
    expect(r.operations.map((c) => c.id)).toEqual(['alpha.fold.paper']);
  });

  it('يجد العملية بمعرّفها وبمرادفٍ في كلمات بحثها', () => {
    expect(search('zeta.summon', CARDS, CATEGORIES).operations.map((c) => c.id)).toEqual([
      'zeta.summon.owl',
    ]);
    expect(search('summon', CARDS, CATEGORIES).operations.map((c) => c.id)).toEqual([
      'zeta.summon.owl',
    ]);
    expect(search('استدعاء', CARDS, CATEGORIES).operations.map((c) => c.id)).toEqual([
      'zeta.summon.owl',
    ]);
  });

  it('كل كلمة يجب أن تُوجد لا إحداها: الكتابة تُضيّق ولا تُوسّع', () => {
    // «origami طيّ» كلمتان في العملية نفسها، فتُوجد. وإضافة كلمةٍ من عمليةٍ
    // أخرى لا تُوسّع النتيجة بل تُفرغها.
    expect(search('origami طي', CARDS, CATEGORIES).operations).toHaveLength(1);
    expect(search('origami astro', CARDS, CATEGORIES).operations).toHaveLength(0);
  });

  it('يجد القسم كما يجد العملية', () => {
    const r = search('git', CARDS, CATEGORIES);
    expect(r.categories.map((c) => c.id)).toContain('git');
  });

  it('لا يخفي غير المتاح بل يؤخّره', () => {
    // من يبحث عن أداةٍ غير مثبّتة يجب أن يجد عملياتها معطّلةً بسببها، لا أن
    // يجد لا شيء ويظنّ أن المنتج لا يعرفها أصلًا.
    const found = search('astro', CARDS, CATEGORIES).operations;
    expect(found.map((c) => c.id)).toEqual(['mu.chart.stars']);
    expect(found[0]?.availability.state).toBe('tool_missing');
  });

  it('يقدّم المتاح على غير المتاح في النتائج', () => {
    const both = search('owlctl astro', CARDS, CATEGORIES);
    // لا تقاطع بينهما، فالنتيجة فارغة — والترتيب يُختبر على استعلامٍ يجمعهما.
    expect(both.operations).toHaveLength(0);

    const shared = search('.', CARDS, CATEGORIES).operations;
    const states = shared.map((c) => c.availability.state);
    expect(states.indexOf('tool_missing')).toBe(states.length - 1);
  });

  it('استعلامٌ لا يطابق شيئًا يعود فارغًا ونشطًا', () => {
    const r = search('لا-شيء-يطابق-هذا', CARDS, CATEGORIES);
    expect(r.active).toBe(true);
    expect(r.operations).toEqual([]);
    expect(r.categories).toEqual([]);
  });
});

// ── المستخدَمة حديثًا ──────────────────────────────────────────────────

function entry(over: Partial<JournalEntry>): JournalEntry {
  return {
    id: 'r1',
    op_id: 'zeta.summon.owl',
    at: 1_700_000_000,
    program: '/usr/bin/owlctl',
    args: [],
    state: 'succeeded',
    ...over,
  };
}

describe('المستخدَمة حديثًا تأتي من سجلٍّ حقيقي', () => {
  it('الأحدث أولًا', () => {
    const journal = [
      entry({ id: 'a', op_id: 'zeta.summon.owl' }),
      entry({ id: 'b', op_id: 'alpha.fold.paper' }),
    ];
    expect(recentOperations(journal, CARDS).map((c) => c.id)).toEqual([
      'alpha.fold.paper',
      'zeta.summon.owl',
    ]);
  });

  it('تُهمل قيود التخطيط: هي معاينات لا استخدام', () => {
    // الواجهة تعيد التخطيط بعد كل سكونٍ في الكتابة، فتعديل اسمٍ واحد يولّد
    // عشرات القيود. عدّها استخدامًا يجعل الصفّ يعني «آخر ما لمسته لوحة المفاتيح».
    const journal = [
      entry({ id: 'a', op_id: 'zeta.summon.owl', state: 'succeeded' }),
      entry({ id: 'b', op_id: 'alpha.fold.paper', state: 'planned' }),
    ];
    expect(recentOperations(journal, CARDS).map((c) => c.id)).toEqual(['zeta.summon.owl']);
  });

  it('بلا تكرار مهما تكرّر التشغيل', () => {
    const journal = Array.from({ length: 9 }, (_, i) =>
      entry({ id: `r${i}`, op_id: 'zeta.summon.owl' }),
    );
    expect(recentOperations(journal, CARDS)).toHaveLength(1);
  });

  it('تُسقط ما لم يعد في الفهرس بدل أن تعرض بطاقةً لا تُفتح', () => {
    const journal = [entry({ op_id: 'operation.that.was.removed' })];
    expect(recentOperations(journal, CARDS)).toEqual([]);
  });

  it('تحترم السقف المطلوب', () => {
    const journal = CARDS.map((c, i) => entry({ id: `r${i}`, op_id: c.id }));
    expect(recentOperations(journal, CARDS, 2)).toHaveLength(2);
  });

  it('السجلّ الفارغ صفٌّ فارغ لا حالةٌ خاصة', () => {
    expect(recentOperations([], CARDS)).toEqual([]);
  });

  it('الفشل والإلغاء استخدامٌ كذلك', () => {
    // «آخر ما استخدمت» لا «آخر ما نجح»: من فشلت عمليته يعود إليها أوّلًا.
    const journal = [
      entry({ id: 'a', op_id: 'zeta.summon.owl', state: 'failed' }),
      entry({ id: 'b', op_id: 'alpha.fold.paper', state: 'cancelled' }),
    ];
    expect(recentOperations(journal, CARDS)).toHaveLength(2);
  });
});

// ── المفضّلة ───────────────────────────────────────────────────────────

describe('المفضّلة', () => {
  it('تُعرض بترتيب المكتبة لا بترتيب الإضافة', () => {
    // قائمةٌ تتبع ترتيب الإضافة تتحرّك تحت اليد كلما أُضيف شيء.
    const ids = ['alpha.fold.paper', 'zeta.summon.owl'];
    expect(favouriteOperations(ids, CARDS).map((c) => c.id)).toEqual([
      'zeta.summon.owl',
      'alpha.fold.paper',
    ]);
  });

  it('تُسقط ما لم يعد في الفهرس صامتةً ولا تحذفه من الإعداد', () => {
    // نواةٌ أقدم قد لا تحمل عمليةً يحملها الإعداد؛ حذفُها كان سيُفقدها حين
    // يعود المستخدم إلى إصدارٍ أحدث.
    expect(favouriteOperations(['gone.op', 'zeta.summon.owl'], CARDS).map((c) => c.id)).toEqual([
      'zeta.summon.owl',
    ]);
  });

  it('القلب يضيف ثم يزيل', () => {
    expect(toggleFavourite([], 'a')).toEqual(['a']);
    expect(toggleFavourite(['a'], 'a')).toEqual([]);
    expect(toggleFavourite(['a', 'b'], 'b')).toEqual(['a']);
  });
});
