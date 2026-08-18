// @vitest-environment jsdom
/**
 * شاشة الفئات — مرسومةً.
 *
 * ## ما تحرسه هذه الاختبارات بالضبط
 *
 * الوعد الذي قامت عليه هذه المرحلة: **لا رقم على الشاشة لم يُحسب من الفهرس**.
 * الأقسام والعمليات هنا مخترعة بالكامل، وأعدادها لا تطابق شيئًا في هذا البناء
 * — فلو عاد أحدٌ يومًا فكتب «ثماني عمليات» في الواجهة، أو أخفى قسمًا بمعرّفه،
 * أو رتّب الشبكة بأسماء مكتوبة بيده، سقط اختبارٌ هنا.
 *
 * ولوحة المفاتيح: الشاشة تُدار كاملةً بها — البؤرة على العنوان عند الوصول،
 * و`/` إلى البحث، وEscape يُفرغه، وكل بطاقةٍ زرٌّ حقيقي يبلغه Tab.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import LibraryScreen from './library-screen';
import CategoryScreen from './category-screen';
import { toCategoryCard, toOperationCard } from './library';
import type { CategoryCard, OperationCard } from './library';
import type { CategoryId, CategorySummary, OperationSummary } from './ipc';
import { AR, t } from './i18n';

afterEach(cleanup);

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

function op(id: string, cat: CategoryId, over: Partial<OperationSummary> = {}): OperationSummary {
  return {
    id,
    title_key: `op.${id}.title`,
    description_key: `op.${id}.desc`,
    category: cat,
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

/**
 * قسمان: أحدهما كل عملياته تعمل، والآخر ينقصه أداة.
 *
 * العددان مختلفان عمدًا في الثاني: هذا هو الحال الذي يجب أن يُقال فيه العددان
 * لا واحد — «ثلاث عمليات، واحدة متاحة هنا».
 */
const CATEGORIES: CategoryCard[] = [
  category('files', { sort_order: 10, operation_count: 2, available_count: 2 }),
  category('git', { sort_order: 20, operation_count: 3, available_count: 1 }),
  category('history', { sort_order: 30, kind: 'journal' }),
].map(toCategoryCard);

const OPERATIONS: OperationCard[] = [
  op('zeta.summon.owl', 'files', { search_terms: ['owlctl', 'استدعاء'] }),
  op('alpha.fold.paper', 'files', { tool: 'origami', search_terms: ['origami', 'طيّ'] }),
  op('mu.chart.stars', 'git', {
    tool: 'astro',
    availability: { state: 'tool_missing', tool: 'astro' },
    search_terms: ['astro'],
  }),
].map((o) => toOperationCard(o, CATEGORIES));

function view(over: Partial<Parameters<typeof LibraryScreen>[0]> = {}) {
  const props = {
    state: { status: 'ready' as const, categories: CATEGORIES, operations: OPERATIONS },
    favourites: [],
    recents: [],
    favouriteIds: [],
    onOpenCategory: vi.fn(),
    onOpenOperation: vi.fn(),
    onToggleFavourite: vi.fn(),
    onRetry: vi.fn(),
    onOpenLog: vi.fn(),
    onOpenSettings: vi.fn(),
    ...over,
  };
  return { ...render(<LibraryScreen {...props} />), props };
}

describe('شبكة الأقسام', () => {
  it('تطابق ترويسة مكتبة Page 15 بلا عنوان قسم زائد', () => {
    view();
    expect(screen.getByRole('heading', { name: 'مكتبة' })).toBeTruthy();
    expect(screen.getByText('اختر فئة لاستعراض العمليات المتاحة فيها.')).toBeTruthy();
    expect(screen.queryByRole('heading', { name: 'كل الفئات' })).toBeNull();
    expect(screen.queryByText('العدد محسوب من فهرس النواة.')).toBeNull();
  });

  it('ترسم كل قسمٍ وصل بعنوانه من مفتاحه', () => {
    view();
    expect(screen.getByRole('button', { name: new RegExp('^' + AR['cat.files.title']) })).toBeTruthy();
    expect(screen.getByRole('button', { name: new RegExp('^' + AR['cat.git.title']) })).toBeTruthy();
  });

  it('البطاقة اسمٌ وعددٌ لا فقرة: الوصف على شاشة القسم لا على بطاقته', () => {
    // ليس تجميلًا: شبكةٌ من عشر بطاقات كلٌّ منها فقرة تعني قراءة عشر فقرات
    // لاختيار بابٍ واحد، والبطاقات تتساوى ارتفاعًا بحكم الشبكة فأطولُ وصفٍ
    // يمطّ الصفّ كلّه. والوصف لم يُقصّ ولم يُخفَ — انتقل إلى ترويسة الشاشة
    // التي يفتحها القسم، ويحرسه اختبار «تعرض اسم القسم ووصفه» أدناه.
    view();
    const files = screen.getByRole('button', { name: new RegExp('^' + AR['cat.files.title']) });
    expect(files.textContent).not.toContain(AR['cat.files.description']);
    expect(screen.queryByText(AR['cat.files.description'])).toBeNull();
  });

  it('تطابق بطاقة القسم بنية Page 15: أيقونة 24 عارية واسم وعدد بلا سهم', () => {
    view();
    const files = screen.getByRole('button', { name: new RegExp('^' + AR['cat.files.title']) });
    const icon = files.querySelector('.op-card__icon');

    expect(files.classList.contains('category-card')).toBe(true);
    expect(icon?.tagName.toLowerCase()).toBe('svg');
    expect(icon?.parentElement).toBe(files);
    expect(files.querySelector('.op-card__go')).toBeNull();
    expect(files.textContent).toContain(`2 ${AR['ops.count.many']}`);
  });

  it('العدد محسوبٌ ممّا وصل، لا رقمًا مكتوبًا في الواجهة', () => {
    // الأرقام هنا (٢ و٣) لا تطابق أي قسمٍ في هذا البناء عمدًا: لو كان في
    // الواجهة رقمٌ مكتوب لظهر هو بدلها.
    const { container } = view();
    const numbers = [...container.querySelectorAll('.num')].map((n) => n.textContent);
    expect(numbers).toContain('2');
    expect(numbers).toContain('3');
  });

  it('القسم الجزئي يقول المتاح من الإجمالي في سطر العدد نفسه', () => {
    view();
    const git = screen.getByRole('button', { name: new RegExp('^' + AR['cat.git.title']) });
    expect(git.textContent).toContain('1 من 3 عمليات متاحة');
    expect(git.classList.contains('category-card--partial')).toBe(true);
  });

  it('والقسم الذي كل عملياته تعمل يعرض العدد مرةً واحدة', () => {
    view();
    const files = screen.getByRole('button', { name: new RegExp('^' + AR['cat.files.title']) });
    expect(files.textContent).toContain('2 عمليات متاحة');
    expect(files.textContent).not.toContain('من 2');
    expect(files.classList.contains('category-card--available')).toBe(true);
  });

  it('القسم الخالي يحمل حالة فراغ صريحة داخل بنية العدد', () => {
    const empty = toCategoryCard(
      category('images', { operation_count: 0, available_count: 0 }),
    );
    view({
      state: { status: 'ready', categories: [empty], operations: [] },
    });

    const card = screen.getByRole('button', { name: new RegExp('^' + AR['cat.images.title']) });
    expect(card.textContent).toContain(AR['lib.category.empty.count']);
    expect(card.classList.contains('category-card--empty')).toBe(true);
  });

  it('قسم السجلّ يُعرض بوصفه لا بعددٍ صفر', () => {
    view();
    const history = screen.getByRole('button', { name: new RegExp('^' + AR['cat.history.title']) });
    expect(history.textContent).toContain(AR['lib.category.journal']);
    expect(history.classList.contains('category-card--history')).toBe(true);
  });

  it('كل بطاقةٍ زرٌّ حقيقي يفتح قسمها بمعرّفه', async () => {
    const user = userEvent.setup();
    const { props } = view();
    await user.click(screen.getByRole('button', { name: new RegExp('^' + AR['cat.git.title']) }));
    expect(props.onOpenCategory).toHaveBeenCalledWith('git');
  });
});

describe('البحث', () => {
  it('يستبدل الشاشة بالنتائج ولا يضيف إليها', async () => {
    const user = userEvent.setup();
    const { container } = view();
    await user.type(screen.getByLabelText(AR['lib.search.label']), 'origami');

    expect(screen.getByRole('heading', { name: 'نتائج البحث' })).toBeTruthy();
    expect(container.querySelector('.ops__sub')?.textContent).toBe('1 نتائج لـ ‹origami›');
    expect(screen.getByText(AR['lib.search.operations'])).toBeTruthy();
    expect(screen.queryByRole('heading', { name: 'مكتبة' })).toBeNull();
    // شبكة الأقسام كاملةً لم تعد معروضة، ولا يعود عنوانها القديم فوقها.
    expect(screen.queryByRole('heading', { name: 'كل الفئات' })).toBeNull();
  });

  it('يعيد تركيب نتيجة البحث من الاستعلام المحفوظ عند العودة', () => {
    const { container } = view({ initialQuery: 'origami' });
    const box = screen.getByLabelText(AR['lib.search.label']) as HTMLInputElement;

    expect(box.value).toBe('origami');
    expect(screen.getByRole('heading', { name: 'نتائج البحث' })).toBeTruthy();
    expect(container.querySelector('.ops__sub')?.textContent).toBe('1 نتائج لـ ‹origami›');
  });

  it('يجد العملية باسم أداتها', async () => {
    const user = userEvent.setup();
    view();
    await user.type(screen.getByLabelText(AR['lib.search.label']), 'origami');
    expect(
      screen.getByRole('button', { name: new RegExp('^' + t('op.alpha.fold.paper.title')) }),
    ).toBeTruthy();
  });

  it('بطاقةُ العملية تعرض الوصف والأداة وفق هوية المكتبة', async () => {
    const user = userEvent.setup();
    view();
    await user.type(screen.getByLabelText(AR['lib.search.label']), 'origami');

    const card = screen.getByRole('button', {
      name: new RegExp('^' + t('op.alpha.fold.paper.title')),
    });
    expect(card.textContent).toContain('origami');
    expect(card.textContent).toContain(t('op.alpha.fold.paper.desc'));
  });

  it('تطابق بطاقة العملية بنية Page 15 بلا بلاطة أيقونة أو سهم', async () => {
    const user = userEvent.setup();
    view();
    await user.type(screen.getByLabelText(AR['lib.search.label']), 'origami');

    const card = screen.getByRole('button', {
      name: new RegExp('^' + t('op.alpha.fold.paper.title')),
    });
    const icon = card.querySelector('.op-card__icon');
    const origin = card.querySelector('.tile__origin');

    expect(card.classList.contains('op-card--operation')).toBe(true);
    expect(icon?.tagName.toLowerCase()).toBe('svg');
    expect(icon?.parentElement).toBe(card);
    expect(card.querySelector('.op-card__go')).toBeNull();
    expect(card.querySelector('.tile__chip')).toBeNull();
    expect(origin?.textContent).toContain(AR['cat.files.title']);
    expect(origin?.textContent).toContain('·');
    expect(origin?.textContent).toContain('origami');
    expect(card.querySelector('.tile__availability')?.textContent).toBe(AR['ops.available']);
  });

  it('نتيجةُ البحث تحمل اسم قسمها', async () => {
    // بلا اسم القسم يبقى المستخدم يخمّن أين وجدها، فلا يستطيع العودة إليها
    // إلا بالبحث مرّة أخرى.
    const user = userEvent.setup();
    const { container } = view();
    await user.type(screen.getByLabelText(AR['lib.search.label']), 'origami');
    const categories = [...container.querySelectorAll('.tile__category')].map((n) => n.textContent);
    expect(categories).toContain(AR['cat.files.title']);
  });

  it('يعرض غير المتاح بسببه بدل أن يخفيه', async () => {
    const user = userEvent.setup();
    view();
    await user.type(screen.getByLabelText(AR['lib.search.label']), 'astro');
    await user.click(
      screen.getByRole('button', { name: AR['lib.search.unavailable.action'] }),
    );

    const card = screen.getByRole('button', { name: new RegExp('^' + t('op.mu.chart.stars.title')) });
    expect(card).toHaveProperty('disabled', true);
    expect(card.classList.contains('op-card--tool-missing')).toBe(true);
    expect(card.textContent).toContain(AR['ops.unavailable.tool.label']);
    // واسم الأداة الناقصة مكتوبٌ ظاهرًا: هو الشيء الوحيد الذي يستطيع
    // المستخدم أن يفعل حياله شيئًا. وخارج الزرّ لأن المعطَّل لا يبلغه القارئ.
    // اسم الأداة يظهر مرّتين عمدًا: وسمًا على البطاقة يجده البحث، وداخل سبب
    // التعطيل. المطلوب هنا الثاني — الجملة التي تقول ما ينقص.
    const reason = screen.getByText(new RegExp(AR['ops.unavailable.tool']));
    expect(reason.textContent).toContain('astro');
  });

  it('يعلن عدد النتائج والاستعلام في ملخّص التركيب، حتى عند الصفر', async () => {
    const user = userEvent.setup();
    const { container } = view();
    const box = screen.getByLabelText(AR['lib.search.label']);

    await user.type(box, 'origami');
    expect(container.querySelector('.ops__sub')?.textContent).toBe('1 نتائج لـ ‹origami›');

    await user.clear(box);
    await user.type(box, 'لا-شيء-يطابق');
    expect(container.querySelector('.ops__sub')?.textContent).toBe(
      '0 نتائج لـ ‹لا-شيء-يطابق›',
    );
    expect(screen.getByText(AR['lib.search.empty.title'])).toBeTruthy();
  });

  // 14.3 `Search/Field · State=NoResults`: الحقل نفسه يحمل الحالة، لا اللوحة
  // تحته وحدها. والحالة تزول بأول استعلامٍ يطابق — فلا تعلق نبرة التحذير.
  it('يحمل مربّع البحث حالة «لا نتيجة» ويتخلّى عنها عند أول تطابق', async () => {
    const user = userEvent.setup();
    const { container } = view();
    const box = screen.getByLabelText(AR['lib.search.label']);
    const field = () => container.querySelector('.lib__search');

    expect(field()?.className).not.toContain('field--no-results');

    await user.type(box, 'لا-شيء-يطابق');
    expect(field()?.className).toContain('field--no-results');

    await user.clear(box);
    await user.type(box, 'origami');
    expect(field()?.className).not.toContain('field--no-results');

    // ومربّعٌ فارغ ليس «لا نتيجة»: لا استعلام أصلًا.
    await user.clear(box);
    expect(field()?.className).not.toContain('field--no-results');
  });

  it('‏Escape يُفرغ البحث ويُبقي البؤرة في المربّع', async () => {
    const user = userEvent.setup();
    view();
    const box = screen.getByLabelText(AR['lib.search.label']) as HTMLInputElement;
    await user.type(box, 'origami');
    await user.keyboard('{Escape}');

    expect(box.value).toBe('');
    expect(document.activeElement).toBe(box);
    // والمكتبة عادت كاملةً.
    expect(screen.getByRole('heading', { name: 'مكتبة' })).toBeTruthy();
    expect(screen.queryByRole('heading', { name: 'كل الفئات' })).toBeNull();
  });

  it('‏/ ينقل البؤرة إلى البحث من أي موضع في الشاشة', async () => {
    const user = userEvent.setup();
    view();
    const heading = screen.getByRole('heading', { name: AR['lib.heading'] });
    (heading as HTMLElement).focus();

    await user.keyboard('/');
    expect(document.activeElement).toBe(screen.getByLabelText(AR['lib.search.label']));
  });

  it('و/ داخل حقلٍ يكتب شرطةً مائلة ولا يقفز', async () => {
    // من يكتب `/` في مسارٍ يريد شرطةً لا اختصارًا.
    const user = userEvent.setup();
    view();
    const box = screen.getByLabelText(AR['lib.search.label']) as HTMLInputElement;
    await user.click(box);
    await user.keyboard('a/b');
    expect(box.value).toBe('a/b');
  });
});

describe('المفضّلة والمستخدَمة حديثًا', () => {
  it('الصفّ الفارغ لا يُعرض بعنوانٍ بلا محتوى', () => {
    view();
    expect(screen.queryByText(AR['lib.favourites.title'])).toBeNull();
    expect(screen.queryByText(AR['lib.recents.title'])).toBeNull();
  });

  it('وحين يمتلئ يُعرض بعنوانه', () => {
    view({ favourites: [OPERATIONS[0]!], recents: [OPERATIONS[1]!] });
    expect(screen.getByText(AR['lib.favourites.title'])).toBeTruthy();
    expect(screen.getByText(AR['lib.recents.title'])).toBeTruthy();
  });

  it('نجمة التفضيل زرٌّ منفصل عن البطاقة، ووسمُه يذكر العملية', async () => {
    // زرٌّ داخل زرّ ليس HTML صالحًا، ويجعل النقر على النجمة يفتح العملية
    // أحيانًا حسب أين وقعت الضغطة.
    const user = userEvent.setup();
    const { props } = view({ favourites: [OPERATIONS[0]!], favouriteIds: ['zeta.summon.owl'] });
    const star = screen.getByRole('button', {
      name: new RegExp(`^${AR['lib.favourite.remove']}`),
    });
    await user.click(star);

    expect(props.onToggleFavourite).toHaveBeenCalledWith('zeta.summon.owl');
    expect(props.onOpenOperation).not.toHaveBeenCalled();
    expect(star.getAttribute('aria-pressed')).toBe('true');
  });
});

describe('الحالات', () => {
  it('يعرض التحميل', () => {
    view({ state: { status: 'loading' } });
    expect(screen.getByText(AR['lib.loading'])).toBeTruthy();
  });

  it('يعرض العطل بسببه من النواة، ويُبقي إعادة المحاولة ممكنة', async () => {
    const user = userEvent.setup();
    const { props } = view({
      state: { status: 'failed', error: { key: 'err.io', input: null, detail: null } },
    });
    expect(screen.getByText(AR['lib.failed.title'])).toBeTruthy();
    // السبب من النواة لا رسالةً عامّة: بدونه تتشابه كل أعطال البدء.
    expect(screen.getByText(new RegExp(AR['err.io']))).toBeTruthy();

    await user.click(screen.getByRole('button', { name: AR['ops.retry'] }));
    expect(props.onRetry).toHaveBeenCalled();
  });

  it('البؤرة تقع على عنوان الشاشة عند الوصول، لا على جسم المستند', () => {
    view();
    expect(document.activeElement).toBe(
      screen.getByRole('heading', { name: AR['lib.heading'] }),
    );
  });
});

// ── شاشة القسم ─────────────────────────────────────────────────────────

describe('شاشة القسم', () => {
  function categoryView(over: Partial<Parameters<typeof CategoryScreen>[0]> = {}) {
    const props = {
      category: CATEGORIES[1]!,
      operations: OPERATIONS.filter((o) => o.category === 'git'),
      favouriteIds: [],
      onOpenOperation: vi.fn(),
      onToggleFavourite: vi.fn(),
      onBack: vi.fn(),
      ...over,
    };
    return { ...render(<CategoryScreen {...props} />), props };
  }

  it('تعرض اسم القسم وتستبدل الوصف بملخّص الإتاحة، والبؤرة على العنوان', () => {
    const { container } = categoryView();
    const heading = screen.getByRole('heading', { name: AR['cat.git.title'] });
    expect(document.activeElement).toBe(heading);
    expect(container.querySelector('.cat__availability')?.textContent).toBe(
      '0 من 1 عمليات متاحة',
    );
    expect(screen.queryByText(AR['cat.git.description'])).toBeNull();
  });

  it('ملخّص الإتاحة يُحسب من البطاقات ويظهر حتى حين تكون كلها متاحة', () => {
    const unavailable = categoryView();
    expect(unavailable.container.querySelector('.cat__availability')?.textContent).toBe(
      '0 من 1 عمليات متاحة',
    );

    cleanup();
    const all = categoryView({
      category: CATEGORIES[0]!,
      operations: OPERATIONS.filter((o) => o.category === 'files'),
    });
    expect(all.container.querySelector('.cat__availability')?.textContent).toBe(
      '2 من 2 عمليات متاحة',
    );

    cleanup();
    const partial = categoryView({
      operations: [OPERATIONS[0]!, OPERATIONS[2]!],
    });
    expect(partial.container.querySelector('.cat__availability')?.textContent).toBe(
      '1 من 2 عمليات متاحة',
    );
  });

  it('تعرض غير المتاح معطّلًا بسببه لا تخفيه', () => {
    categoryView();
    const card = screen.getByRole('button', {
      name: new RegExp('^' + t('op.mu.chart.stars.title')),
    });
    expect(card).toHaveProperty('disabled', true);
    expect(screen.getByText(new RegExp(AR['ops.unavailable.tool']))).toBeTruthy();
  });

  it('زرّ الرجوع أوّل زرّ في ترتيب التبويب، ويُفعَّل بلوحة المفاتيح', async () => {
    // المخرج يجب أن يكون أقرب ما يُبلغ لا أبعده. والبؤرة تبدأ على العنوان —
    // وهو بعده في المستند — فيُقاس الترتيب على المستند لا على أول ضغطة Tab.
    const user = userEvent.setup();
    const { container, props } = categoryView();
    const back = screen.getByRole('button', { name: AR['nav.back.library'] });
    expect(container.querySelector('button')).toBe(back);

    back.focus();
    await user.keyboard('{Enter}');
    expect(props.onBack).toHaveBeenCalled();
  });

  it('القسم الفارغ يقول ذلك بدل أن يعرض شبكةً بلا خلايا', () => {
    categoryView({ operations: [] });
    expect(screen.getByText(AR['lib.category.empty'])).toBeTruthy();
  });

  it('اختيار عملية يمرّر معرّفها', async () => {
    const user = userEvent.setup();
    const { props } = categoryView({
      operations: OPERATIONS.filter((o) => o.category === 'files'),
    });
    await user.click(
      screen.getByRole('button', { name: new RegExp('^' + t('op.zeta.summon.owl.title')) }),
    );
    expect(props.onOpenOperation).toHaveBeenCalledWith('zeta.summon.owl');
  });

  it('كل بطاقةٍ متاحة يبلغها Tab وتُفعَّل بلوحة المفاتيح', async () => {
    const user = userEvent.setup();
    const { props } = categoryView({
      operations: OPERATIONS.filter((o) => o.category === 'files'),
    });
    const list = screen.getByRole('list');
    const cards = within(list).getAllByRole('button');
    // بطاقتان ونجمتاهما: أربعة أزرار، كلها في ترتيب التبويب.
    expect(cards).toHaveLength(4);

    for (let i = 0; i < cards.length; i += 1) await user.tab();
    expect(props.onOpenOperation).not.toHaveBeenCalled();
  });
});
