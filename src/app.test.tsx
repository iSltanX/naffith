// @vitest-environment jsdom
/**
 * الغلاف: من يركّب الشاشة هو من يعطيها وعاءها.
 *
 * الشاشات الثلاث الثانوية كانت تُعاد من `App` عاريةً بينما شاشة العملية وحدها
 * ملفوفة في وعاءٍ مبطّن، فظهرت قائمة العمليات والإعدادات والسجلّ ملاصقةً
 * لحافّتي النافذة: العنوان يُقصّ عند الحافة، وحلقةُ التركيز حول البطاقة
 * المحاذية لها تُقصّ معها فيغيب نصف دليل التركيز.
 *
 * ولذلك يُفحص هنا **كل** ما يعيده الغلاف لا شاشةٌ بعينها: القاعدة أن لا شاشة
 * بلا وعاء، وشاشةٌ رابعة تُضاف غدًا بلا وعاء تسقط في الاختبار نفسه. أما أن
 * الوعاء يُعطي حشوةً وعرضًا أقصى فعلًا — لا صنفًا فارغًا — فيحرسه
 * `app-shell.source.test.ts`: أنماط الملفّات لا تُحمَّل في jsdom.
 */
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './app';
import { AR } from './i18n';
import type {
  CategorySummary,
  OperationSummary,
  PlanResponse,
  ResultContract,
  RunFinishedEvent,
  RunOutputEvent,
} from './ipc';
import {
  cancel,
  execute,
  listOperations,
  listCategories,
  onRunFinished,
  onRunOutput,
  plan as planOperation,
  recentRuns,
  reveal,
} from './ipc';
import { SETTINGS_SCHEMA_VERSION, SETTINGS_STORAGE_KEY } from './settings';
import { playSuccessChime } from './notification-sound';

// النواة لا تُستدعى: `@tauri-apps/api` نفسه غير موجود في jsdom، والمصنع هنا
// يمنع تحميل الوحدة الحقيقية أصلًا لا يستبدل دالّةً فيها.
vi.mock('./ipc', () => ({
  // ثابتا البناء: الشاشة تقرؤهما عند أول رسم، وغيابهما عن الاستهزاء يرمي.
  APP_VERSION: '0.0.0-test',
  UPDATER_CONFIGURED: false,
  checkForUpdate: vi.fn(),
  downloadAndInstallUpdate: vi.fn(),
  listOperations: vi.fn(),
  listCategories: vi.fn(),
  journalDelete: vi.fn(),
  journalClear: vi.fn(),
  recentRuns: vi.fn(),
  onRunFinished: vi.fn(),
  onRunOutput: vi.fn(),
  plan: vi.fn(),
  execute: vi.fn(),
  cancel: vi.fn(),
  reveal: vi.fn(),
  pickDirectory: vi.fn(),
  asCoreError: (e: unknown) => ({ key: 'err.unknown', input: null, detail: e }),
}));

// النغمة الحقيقية تحتاج `AudioContext` — غائبٌ في jsdom، ولا حاجة إليه هنا
// أصلًا: `notification-sound.test.ts` يحرس التوليف نفسه؛ وهذا يحرس **متى**
// يُستدعى، لا كيف يُصنَع الصوت.
vi.mock('./notification-sound', () => ({ playSuccessChime: vi.fn() }));

/** قسم الضغط كما تعلنه النواة في `categories.rs`. */
const COMPRESS_CATEGORY: CategorySummary = {
  id: 'compress',
  title_key: 'cat.compress.title',
  description_key: 'cat.compress.description',
  icon: '#i-compress',
  sort_order: 20,
  kind: 'operations',
  operation_count: 1,
  available_count: 1,
};

const HISTORY_CATEGORY: CategorySummary = {
  id: 'history',
  title_key: 'cat.history.title',
  description_key: 'cat.history.description',
  icon: '#i-history',
  sort_order: 999,
  kind: 'journal',
  operation_count: 0,
  available_count: 0,
};

const COMPRESS: OperationSummary = {
  id: 'compress.folder.zip',
  title_key: 'op.compress.folder.zip.title',
  description_key: 'op.compress.folder.zip.description',
  category: 'compress',
  danger: 'creates',
  conflict: 'refuse',
  tool: 'ditto',
  availability: { state: 'available' },
  sort_order: 10,
  search_terms: ['ditto', 'zip'],
  inputs: [
    { id: 'source', required: true, kind: 'existing_dir' },
    { id: 'destination', required: true, kind: 'target_dir' },
    { id: 'archive_name', required: true, kind: 'new_name', ext: 'zip' },
  ],
};

/** خطةٌ جاهزة: ما تعيده النواة حين تكتمل الحقول. تكفي لتشغيلٍ يبدأ فعلًا. */
const PLAN: PlanResponse = {
  token: 'tok',
  plan_id: 'p1',
  op_id: COMPRESS.id,
  title_key: COMPRESS.title_key,
  description_key: COMPRESS.description_key,
  category: 'compress',
  danger: 'creates',
  argv_display: ['/usr/bin/ditto', '-c', '-k', '/Users/x/src', '/Users/x/dst/.n.part'],
  explain: [
    { token: '/usr/bin/ditto', key: 'explain.ditto.tool', role: 'tool' },
    { token: '-c', key: 'explain.ditto.create', role: 'flag' },
    { token: '-k', key: 'explain.ditto.pkzip', role: 'flag' },
    { token: '/Users/x/src', key: null, role: 'path' },
    { token: '/Users/x/dst/.n.part', key: null, role: 'path' },
  ],
  warnings: [],
  tool: { id: 'ditto', path: '/usr/bin/ditto' },
  conflict: 'refuse',
  estimate: { approx_source_bytes: 12_400_000, scanned_entries: 42, complete: true },
  produces: '/Users/x/dst/Reports.zip',
  writes_to: '/Users/x/dst/.n.part',
  working_directory: null,
};

const FAILED_RESULT: ResultContract = {
  category: 'diagnostic',
  semantic: 'failed',
  type: 'diagnostic',
  lines: [{ stream: 'stderr', line: 'ditto: /Users/x/src: Permission denied' }],
};

const ARTIFACT_RESULT: ResultContract = {
  category: 'artifact',
  semantic: 'completed',
  type: 'artifact',
  path: '/Users/x/dst/Reports.zip',
  name: 'Reports.zip',
  parent: '/Users/x/dst',
  reveal: 'file',
};

afterEach(cleanup);

beforeEach(() => {
  // بلا هذا يبدأ التطبيق من شاشة الترحيب، فلا يُفحص شيء ممّا بعدها.
  window.localStorage.setItem(
    SETTINGS_STORAGE_KEY,
    JSON.stringify({
      schemaVersion: SETTINGS_SCHEMA_VERSION,
      onboardingCompletedAt: '2026-01-01T00:00:00.000Z',
    }),
  );
  // `restoreMocks: true` يمسح التنفيذ قبل كل اختبار، فبلا هذه الأسطر تعيد
  // الدوالّ `undefined` ويسقط `.then` بخطأ لا علاقة له بما يُختبر.
  vi.mocked(listOperations).mockResolvedValue([COMPRESS]);
  vi.mocked(listCategories).mockResolvedValue([COMPRESS_CATEGORY]);
  vi.mocked(recentRuns).mockResolvedValue([]);
  vi.mocked(onRunFinished).mockResolvedValue(() => {});
  vi.mocked(onRunOutput).mockResolvedValue(() => {});
});

/**
 * يمسك المستمع الذي سجّله الغلاف لحدثٍ ما، ويعيد باثًّا يوقظه.
 *
 * الغلاف يسجّل مستمعيه في أثرٍ واحد عند التركيب، فالمصنع الوهمي هو الطريق
 * الوحيد إلى الدالّة التي سيستدعيها Tauri فعلًا. وبلا هذا لا سبيل لاختبار ما
 * يحدث حين يبثّ النواةُ شيئًا — وهو نصف دورة حياة التشغيل.
 *
 * ## ولماذا **آخر** نداء لا أوّله
 *
 * ‏`restoreMocks: true` يعيد التنفيذ ولا يمسح تاريخ النداءات (انظر `beforeEach`
 * أعلاه: لولا ذلك لما احتاج إلى إعادة التجهيز). فكل تركيبٍ لـ`App` في الملف
 * يضيف نداءً إلى القائمة نفسها، وأوّلُها يخصّ تطبيقًا نُزع من الشجرة — فباثُّه
 * يستدعي `setState` على مكوّنٍ ميت: لا استثناء، ولا أثر، واختبارٌ يسقط وكأن
 * الوصل معطوب وهو سليم. وقع هذا فعلًا وأخذ من التشخيص أكثر ممّا أخذ من الكتابة.
 */
function emitter<T>(listener: { mock: { calls: unknown[][] } }): (event: T) => void {
  const calls = listener.mock.calls;
  const fn = calls[calls.length - 1]?.[0] as ((event: T) => void) | undefined;
  expect(fn, 'الغلاف لم يسجّل مستمعًا').toBeTypeOf('function');
  return (event: T) => act(() => (fn as (e: T) => void)(event));
}

/** جذر الشاشة المعروضة أيًّا كانت: قائمة، أو شاشة ثانوية، أو شاشة عملية. */
function screenRoot(container: HTMLElement): Element {
  const root = container.querySelector('.lib, .screen, .op');
  expect(root, 'لم تُرسم أي شاشة').toBeTruthy();
  return root as Element;
}

describe('وعاء الصفحة في كل شاشة', () => {
  it('قائمة العمليات داخل الوعاء لا ملاصقةً لحافّة النافذة', async () => {
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });
    expect(screenRoot(container).closest('.page')).toBeTruthy();
  });

  it('الإعدادات داخل الوعاء', async () => {
    const user = userEvent.setup();
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(screen.getByRole('button', { name: AR['nav.settings'] }));
    expect(container.querySelector('.screen')).toBeTruthy();
    expect(screenRoot(container).closest('.page')).toBeTruthy();
  });

  it('سجلّ التشغيل داخل الوعاء', async () => {
    const user = userEvent.setup();
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(screen.getByRole('button', { name: AR['nav.log'] }));
    await screen.findByRole('heading', { name: AR['log.title'] });
    expect(container.querySelector('.screen.log')).toBeTruthy();
    expect(screenRoot(container).closest('.page')).toBeTruthy();
  });

  it('بطاقة السجل في المكتبة تفتح سجلّ التشغيل مباشرةً لا قسمًا فارغًا', async () => {
    vi.mocked(listCategories).mockResolvedValue([COMPRESS_CATEGORY, HISTORY_CATEGORY]);
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(screen.getByRole('button', {
      name: new RegExp(`^${AR['cat.history.title']}`),
    }));

    expect(await screen.findByRole('heading', { name: AR['log.title'] })).toBeTruthy();
    expect(screen.queryByText(AR['lib.category.empty'])).toBeNull();
  });

  it('شاشة العملية داخل الوعاء — وهي التي كانت وحدها فيه', async () => {
    const user = userEvent.setup();
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(await opCard());
    // شاشة العملية تملك تخطيطها وتمتدّ إلى حواف النافذة: شريطٌ فوق سطحين،
    // بلا حشوة الوعاء ولا حدّه الأقصى — وحصرُها فيهما كان يترك ثلث النافذة
    // فراغًا ويحشر النموذج في نصفٍ ضيّق.
    expect(container.querySelector('.op__panes')).toBeTruthy();
    expect(container.querySelector('.op > .opbar')).toBeNull();
    expect(container.querySelector('.naffith__scroll > .opintro')).toBeTruthy();
    expect(screenRoot(container).closest('.page--bleed')).toBeTruthy();
  });
});

/**
 * بطاقة العملية، بعد فتح قسمها.
 *
 * المسار صار: الفئات ← القسم ← العملية. الاختبارات التي كانت تفتح العملية من
 * الجذر مباشرةً تمرّ بالقسم الآن — وهو المسار الذي يسلكه المستخدم فعلًا.
 */
async function opCard(): Promise<HTMLElement> {
  const inCategory = screen.queryByRole('button', {
    name: new RegExp('^' + AR['op.compress.folder.zip.title']),
  });
  if (inCategory) return inCategory;

  const category = await screen.findByRole('button', {
    name: new RegExp('^' + AR['cat.compress.title']),
  });
  fireEvent.click(category);
  return screen.findByRole('button', {
    name: new RegExp('^' + AR['op.compress.folder.zip.title']),
  });
}

/** زرّ القسم في مسار العملية الداخلي، لا عنصر القسم في تنقّل التطبيق. */
function operationBack(): HTMLElement {
  const breadcrumbs = screen.getByRole('navigation', { name: AR['nav.breadcrumbs'] });
  return within(breadcrumbs).getByRole('button', { name: AR['cat.compress.title'] });
}

type User = ReturnType<typeof userEvent.setup>;

describe('البؤرة عند تبدّل الشاشة', () => {
  // تبديل الشاشة في تطبيق صفحةٍ واحدة لا يحرّك التركيز من تلقائه: الزرّ الذي
  // ضُغط يُنزع من الشجرة فتسقط البؤرة إلى `body`، ويستأنف Tab من رأس المستند.
  // من يتنقّل بلوحة المفاتيح يفقد موضعه في كل انتقال، ومن يقرأ بالصوت لا يُنطق
  // له اسمُ الشاشة التي وصل إليها.

  it('بعد «ابدأ الآن» تقع البؤرة على عنوان القائمة لا على جسم المستند', async () => {
    // بلا إعدادٍ محفوظ يبدأ التطبيق من الترحيب، وهو أول انتقالٍ يراه المستخدم.
    window.localStorage.clear();
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole('button', { name: AR['onboarding.start'] }));

    const heading = await screen.findByRole('heading', { name: AR['lib.heading'] });
    expect(document.activeElement).toBe(heading);
    expect(document.activeElement).not.toBe(document.body);
  });

  it('بعد «رجوع» من شاشة العملية تعود البؤرة إلى عنوان القسم الذي جاءت منه', async () => {
    // العودة إلى **ما جاء منه** لا إلى الجذر: من فتح العملية من قسمٍ يعود
    // إليه، ومن فتحها من بحثٍ يعود إلى البحث. عودةٌ ثابتة إلى الجذر كانت
    // تُخرج من دخل من قسمٍ منه في كل مرّة.
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(await opCard());
    await user.click(operationBack());

    const heading = await screen.findByRole('heading', { name: AR['cat.compress.title'] });
    expect(document.activeElement).toBe(heading);

    // ومن القسم يعود «رجوع» إلى جذر المكتبة.
    await user.click(screen.getByRole('button', { name: AR['nav.back.library'] }));
    expect(document.activeElement).toBe(
      await screen.findByRole('heading', { name: AR['lib.heading'] }),
    );
  });

  it('عند فتح عملية يعلن عنوانها من المستوى الثاني قبل حقولها', async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(await opCard());
    const heading = await screen.findByRole('heading', {
      name: AR['op.compress.folder.zip.title'],
      level: 2,
    });
    expect(document.activeElement).toBe(heading);
  });

  it('وفي الشاشات الثانوية أيضًا: لا انتقال ينتهي بالبؤرة على جسم المستند', async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(screen.getByRole('button', { name: AR['nav.settings'] }));
    // عنوان الشاشة صار عنوانَ اللسان المفتوح (صفحة ٢٠)، وأوّلها «عام».
    expect(document.activeElement).toBe(
      await screen.findByRole('heading', { name: AR['settings.general.title'] }),
    );
  });

  it('النموذج المعدّل يفتح تأكيد المغادرة؛ الإجراء الآمن يحفظه والتأكيد يمسحه', async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(await opCard());
    const source = screen.getByLabelText(AR['field.source.label']) as HTMLInputElement;
    await user.type(source, '/Users/x/src');
    const back = operationBack();
    await user.click(back);

    let dialog = await screen.findByRole('alertdialog');
    expect(within(dialog).getByText(AR['nav.leave.dirty.title'])).toBeTruthy();
    expect(within(dialog).getByText(AR['dialog.safe_dismiss'])).toBeTruthy();
    await user.click(within(dialog).getByRole('button', { name: AR['nav.leave.dirty.stay'] }));
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(source.value).toBe('/Users/x/src');
    expect(document.activeElement).toBe(back);

    await user.click(back);
    dialog = await screen.findByRole('alertdialog');
    const destructiveLeave = within(dialog).getByRole('button', {
      name: AR['nav.leave.dirty.leave'],
    });
    expect(destructiveLeave.classList.contains('btn--danger')).toBe(true);
    expect(destructiveLeave.classList.contains('btn--quiet')).toBe(false);
    await user.click(destructiveLeave);
    expect(document.activeElement).toBe(
      await screen.findByRole('heading', { name: AR['cat.compress.title'] }),
    );
    await user.click(await opCard());
    expect((screen.getByLabelText(AR['field.source.label']) as HTMLInputElement).value).toBe('');
  });

  it('الوعاء هدفٌ للبرنامج لا محطّةٌ لـTab', async () => {
    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });
    expect(document.querySelector('.page')?.getAttribute('tabindex')).toBe('-1');
  });
});

describe('حوار المغادرة أثناء تشغيل', () => {
  /** يقود التطبيق إلى تشغيلٍ جارٍ ثم يطلب الرجوع، فيُفتح الحوار. */
  async function openDialog(user: User) {
    vi.mocked(planOperation).mockResolvedValue(PLAN);
    vi.mocked(execute).mockResolvedValue('run-1');

    const view = render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });
    await user.click(await opCard());

    await user.type(screen.getByLabelText(AR['field.source.label']), '/Users/x/src');
    await user.type(screen.getByLabelText(AR['field.destination.label']), '/Users/x/dst');
    await user.type(screen.getByLabelText(AR['field.archive_name.label']), 'Reports');

    const run = await screen.findByRole('button', { name: AR['action.execute'] });
    expect(run.hasAttribute('disabled')).toBe(false);
    await user.click(run);

    // ‏`PLAN.danger` هنا `creates`، والإعداد الافتراضي `confirmBeforeExecute:
    // true` — فحوار التأكيد يظهر أوّلًا. يُعبَر هنا لا يُختبَر: هذا الوصف
    // يختبر حوار المغادرة، لا حوار التأكيد.
    const confirm = screen.queryByRole('alertdialog');
    if (confirm) {
      await user.click(within(confirm).getByRole('button', { name: AR['action.execute'] }));
    }

    await screen.findByRole('button', { name: AR['satr.action.cancel'] });

    const back = operationBack();
    await user.click(back);
    await screen.findByRole('alertdialog');

    return { ...view, back };
  }

  it('‏Escape يُغلقه بمعنى «البقاء»: التشغيل يبقى والشاشة لا تُغادَر', async () => {
    const user = userEvent.setup();
    const { back } = await openDialog(user);

    await user.keyboard('{Escape}');

    expect(screen.queryByRole('alertdialog')).toBeNull();
    // المخرج الآمن هو غير المكلف: ما زلنا في شاشة العملية والتشغيل جارٍ.
    expect(screen.getByRole('button', { name: AR['satr.action.cancel'] })).toBeTruthy();
    expect(document.activeElement).toBe(back);
  });

  it('يحبس التبويب بين زرّيه فلا يهرب إلى الشاشة المحجوبة تحته', async () => {
    const user = userEvent.setup();
    await openDialog(user);
    const dialog = screen.getByRole('alertdialog');
    const stay = within(dialog).getByRole('button', { name: AR['nav.leave.busy.stay'] });
    const leave = within(dialog).getByRole('button', { name: AR['nav.leave.busy.leave'] });
    expect(leave.classList.contains('btn--quiet')).toBe(true);
    expect(leave.classList.contains('btn--danger')).toBe(false);

    expect(AR['dialog.safe_dismiss']).toBe('Escape والنقر على الخلفية = الإجراء الآمن');
    expect(within(dialog).getByText(AR['dialog.safe_dismiss'])).toBeDefined();

    // «البقاء» يأخذ البؤرة عند الفتح: الخطأ في اتجاهه غير مكلف.
    expect(document.activeElement).toBe(stay);

    await user.tab();
    expect(document.activeElement).toBe(leave);
    // من آخر زرّ يلتفّ إلى أوّله، لا إلى «رجوع» خلف العتمة.
    await user.tab();
    expect(document.activeElement).toBe(stay);
    // وبالعكس كذلك.
    await user.tab({ shift: true });
    expect(document.activeElement).toBe(leave);
  });

  it('النقر على الخلفية يختار البقاء ولا يغادر التشغيل', async () => {
    const user = userEvent.setup();
    const { back } = await openDialog(user);
    const dialog = screen.getByRole('alertdialog');
    const scrim = dialog.parentElement;
    expect(scrim?.classList.contains('scrim')).toBe(true);

    await user.click(scrim as HTMLElement);

    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(screen.getByRole('button', { name: AR['satr.action.cancel'] })).toBeTruthy();
    expect(document.activeElement).toBe(back);
  });

  it('يعيد البؤرة إلى الزرّ الذي فتحه عند الإغلاق', async () => {
    const user = userEvent.setup();
    const { back } = await openDialog(user);

    await user.click(screen.getByRole('button', { name: AR['nav.leave.busy.stay'] }));

    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(document.activeElement).toBe(back);
  });

  it('والمغادرة تنقل البؤرة إلى الشاشة الجديدة لا إلى جسم المستند', async () => {
    const user = userEvent.setup();
    await openDialog(user);

    await user.click(screen.getByRole('button', { name: AR['nav.leave.busy.leave'] }));

    // إلى القسم الذي فُتحت العملية منه، لا إلى الجذر. انظر `origin` في `nav.ts`.
    const heading = await screen.findByRole('heading', { name: AR['cat.compress.title'] });
    expect(document.activeElement).toBe(heading);
  });
});

/**
 * إعداد «تأكيد قبل التنفيذ» — حاجزٌ ثانٍ فوق المعاينة الدائمة، للعمليات التي
 * تعدّل أو تتلف وحدها. انظر توثيقه في `settings.ts` وتوثيق `ConfirmDialog`
 * في `app.tsx` لسبب كون «إلغاء» هو الخيار الآمن هنا لا «نفِّذ».
 */
describe('تأكيد التنفيذ', () => {
  // ‏`execute` مشتركٌ عبر الملفّ كلّه، و`restoreMocks` لا يُنظّف `vi.fn()` من
  // مصنع `vi.mock()` بين الاختبارات — انظر التعليق نظيره في وصف «نغمة
  // الإشعار» أدناه. وصفٌ سابق (حوار المغادرة) يستدعي `execute` فعليًا أيضًا،
  // فتتراكم استدعاءاته هنا لو لم يُصفَّ العدّاد صراحةً.
  beforeEach(() => {
    vi.mocked(execute).mockClear();
  });

  /** يملأ نموذج الضغط ويضغط «نفِّذ»، ويقف عند تلك اللحظة — لا يعبر حوار
   *  التأكيد كما يفعل `startRun` في الوصف التالي؛ هذا الوصف يختبر الحوار نفسه.
   *  ‏`danger` اختياريّ لاختبار درجات الخطر الأخرى فوق نفس مسار الملء. */
  async function clickRun(user: User, danger: PlanResponse['danger'] = PLAN.danger) {
    vi.mocked(planOperation).mockResolvedValue({ ...PLAN, danger });
    vi.mocked(execute).mockResolvedValue('run-1');

    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });
    await user.click(await opCard());

    await user.type(screen.getByLabelText(AR['field.source.label']), '/Users/x/src');
    await user.type(screen.getByLabelText(AR['field.destination.label']), '/Users/x/dst');
    await user.type(screen.getByLabelText(AR['field.archive_name.label']), 'Reports');

    const run = await screen.findByRole('button', { name: AR['action.execute'] });
    await user.click(run);
  }

  it('يظهر حوار التأكيد لعمليةٍ تنشئ ولا يبدأ التشغيل بعد', async () => {
    const user = userEvent.setup();
    await clickRun(user);

    const dialog = await screen.findByRole('alertdialog');
    expect(
      within(dialog).getByRole('heading', { name: AR['run.confirm.creates.title'] }),
    ).toBeTruthy();
    expect(execute).not.toHaveBeenCalled();
  });

  it('التركيز الأولي على «إلغاء» — الخيار الآمن يأخذ الافتراضي هنا لا «نفِّذ»', async () => {
    const user = userEvent.setup();
    await clickRun(user);

    const dialog = await screen.findByRole('alertdialog');
    expect(document.activeElement).toBe(
      within(dialog).getByRole('button', { name: AR['action.cancel'] }),
    );
  });

  it('«إلغاء» يغلق الحوار، ولا ينفَّذ شيء، والنموذج يبقى كما مُلئ', async () => {
    const user = userEvent.setup();
    await clickRun(user);

    const dialog = await screen.findByRole('alertdialog');
    await user.click(within(dialog).getByRole('button', { name: AR['action.cancel'] }));

    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(execute).not.toHaveBeenCalled();
    expect((screen.getByLabelText(AR['field.source.label']) as HTMLInputElement).value).toBe(
      '/Users/x/src',
    );
  });

  it('Escape يوازي «إلغاء»', async () => {
    const user = userEvent.setup();
    await clickRun(user);
    await screen.findByRole('alertdialog');

    await user.keyboard('{Escape}');

    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(execute).not.toHaveBeenCalled();
  });

  it('«نفِّذ» داخل الحوار يبدأ التشغيل فعليًا بنفس رمز الخطّة', async () => {
    const user = userEvent.setup();
    await clickRun(user);

    const dialog = await screen.findByRole('alertdialog');
    await user.click(within(dialog).getByRole('button', { name: AR['action.execute'] }));

    expect(screen.queryByRole('alertdialog')).toBeNull();
    await screen.findByRole('button', { name: AR['satr.action.cancel'] });
    expect(execute).toHaveBeenCalledWith(PLAN.token);
  });

  /**
   * الدرجات الثلاث تُعرض بنصٍّ مختلف، وزرّ «نفِّذ» فيها لا يتساوى صنفه مع
   * زرّ «إلغاء» أبدًا — ولا مع نفسه بين الدرجات: التلف وحده يستحقّ نبرة الخطر.
   * هذا التمييز غاب فعلًا في أول تنفيذٍ (كلاهما `btn--primary` لـ«ينشئ»/
   * «يعدّل») ولم يمسكه أي اختبارٍ سابق لأنه جميعها استعمل `PLAN.danger`
   * الثابتة (`'creates'`) وحدها.
   */
  it.each<['modifies' | 'destructive', 'quiet' | 'danger']>([
    ['modifies', 'quiet'],
    ['destructive', 'danger'],
  ])('درجة «%s»: العنوان الصحيح وزرّ نفِّذ بصنف btn--%s لا btn--primary', async (danger, tone) => {
    const user = userEvent.setup();
    await clickRun(user, danger);

    const dialog = await screen.findByRole('alertdialog');
    expect(
      within(dialog).getByRole('heading', { name: AR[`run.confirm.${danger}.title`] }),
    ).toBeTruthy();

    const cancelBtn = within(dialog).getByRole('button', { name: AR['action.cancel'] });
    const executeBtn = within(dialog).getByRole('button', { name: AR['action.execute'] });
    expect(cancelBtn.classList.contains('btn--primary')).toBe(true);
    expect(executeBtn.classList.contains(`btn--${tone}`)).toBe(true);
    expect(executeBtn.classList.contains('btn--primary')).toBe(false);
  });

  it('يُتخطّى لعمليةٍ آمنة — قراءة فقط لا تصل هذا الحوار مهما كان الإعداد', async () => {
    vi.mocked(planOperation).mockResolvedValue({ ...PLAN, danger: 'safe' });
    vi.mocked(execute).mockResolvedValue('run-1');
    const user = userEvent.setup();

    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });
    await user.click(await opCard());
    await user.type(screen.getByLabelText(AR['field.source.label']), '/Users/x/src');
    await user.type(screen.getByLabelText(AR['field.destination.label']), '/Users/x/dst');
    await user.type(screen.getByLabelText(AR['field.archive_name.label']), 'Reports');
    await user.click(await screen.findByRole('button', { name: AR['action.execute'] }));

    expect(screen.queryByRole('alertdialog')).toBeNull();
    await screen.findByRole('button', { name: AR['satr.action.cancel'] });
    expect(execute).toHaveBeenCalled();
  });

  it('يُتخطّى حين يُعطَّل الإعداد — حتى لعمليةٍ تنشئ', async () => {
    window.localStorage.setItem(
      SETTINGS_STORAGE_KEY,
      JSON.stringify({
        schemaVersion: SETTINGS_SCHEMA_VERSION,
        onboardingCompletedAt: '2026-01-01T00:00:00.000Z',
        confirmBeforeExecute: false,
      }),
    );
    const user = userEvent.setup();
    await clickRun(user);

    expect(screen.queryByRole('alertdialog')).toBeNull();
    await screen.findByRole('button', { name: AR['satr.action.cancel'] });
    expect(execute).toHaveBeenCalled();
  });
});

/**
 * دورة حياة مجرى التشغيل، من طرف الغلاف.
 *
 * ‏`run-stream.test.tsx` يحرس المكوّن؛ وهذا يحرس **الوصل**: أن الغلاف يسجّل
 * المستمع فعلًا، وأن ما يبثّه النواة يظهر، وأن المجرى يُفرَّغ في الموضع الصحيح.
 * والعطل الذي وُلد منه الملفّان واحد: الجسر كان معرَّفًا ومحروسًا بعقد السلك،
 * ولا مستدعي له — فلم يسقط اختبارٌ واحد وهو معطَّل تمامًا.
 */
describe('مجرى التشغيل في الغلاف', () => {
  /** يقود التطبيق إلى تشغيلٍ جارٍ، ويعيد باثَّي الخرج والنهاية. */
  async function startRun(user: User) {
    vi.mocked(planOperation).mockResolvedValue(PLAN);
    vi.mocked(execute).mockResolvedValue('run-1');

    const view = render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });
    await user.click(await opCard());

    await user.type(screen.getByLabelText(AR['field.source.label']), '/Users/x/src');
    await user.type(screen.getByLabelText(AR['field.destination.label']), '/Users/x/dst');
    await user.type(screen.getByLabelText(AR['field.archive_name.label']), 'Reports');

    const run = await screen.findByRole('button', { name: AR['action.execute'] });
    expect(run.hasAttribute('disabled')).toBe(false);
    await user.click(run);

    // ‏`PLAN.danger` هنا `creates`، والإعداد الافتراضي `confirmBeforeExecute:
    // true` — فحوار التأكيد يظهر أوّلًا. هذا المساعد يعبره بضغطة تنفيذٍ ثانية
    // كي يصل مستدعوه إلى تشغيلٍ جارٍ فعليًا دون أن يعرف كلٌّ منهم بوجود الحوار.
    // ‏`describe('تأكيد التنفيذ')` أدناه يختبر الحوار نفسه.
    const confirm = screen.queryByRole('alertdialog');
    if (confirm) {
      await user.click(within(confirm).getByRole('button', { name: AR['action.execute'] }));
    }

    await screen.findByRole('button', { name: AR['satr.action.cancel'] });

    return {
      ...view,
      out: emitter<RunOutputEvent>(vi.mocked(onRunOutput)),
      done: emitter<RunFinishedEvent>(vi.mocked(onRunFinished)),
    };
  }

  const line = (text: string): RunOutputEvent => ({
    run_id: 'run-1',
    stream: 'stdout',
    line: text,
  });

  it('يسجّل مستمعًا للخرج عند التركيب', () => {
    // العطل الأصلي حرفيًّا: `onRunOutput` معرَّفة ولا تُستدعى.
    render(<App />);
    expect(vi.mocked(onRunOutput)).toHaveBeenCalled();
  });

  it('يعرض ما تبثّه الأداة في «سَطْر» أثناء التشغيل', async () => {
    const user = userEvent.setup();
    const { container, out } = await startRun(user);

    out(line('copying /Users/x/src'));
    out({ run_id: 'run-1', stream: 'stderr', line: 'ditto: warning' });

    const texts = [...container.querySelectorAll('.stream__text')].map((n) => n.textContent);
    expect(texts).toEqual(['copying /Users/x/src', 'ditto: warning']);
    // ومكانه اللوحة التقنية لا سطح النموذج.
    expect(container.querySelector('.satr .stream')).toBeTruthy();
  });

  it('يعيد إتاحة الإلغاء إذا رفضت النواة الطلب ولا يحبس التشغيل', async () => {
    vi.mocked(cancel).mockRejectedValueOnce(new Error('cancel unavailable'));
    const user = userEvent.setup();
    const { done } = await startRun(user);

    await user.click(screen.getByRole('button', { name: AR['satr.action.cancel'] }));
    expect(await screen.findByText(AR['err.unknown'])).toBeTruthy();
    expect((screen.getByRole('button', { name: AR['satr.action.cancel'] }) as HTMLButtonElement).disabled)
      .toBe(false);

    done({
      run_id: 'run-1',
      status: 'success',
      produced: '/Users/x/dst/Reports.zip',
      result: ARTIFACT_RESULT,
    });
    await screen.findByRole('heading', { name: AR['result.semantic.completed'] });
    expect(screen.queryByText(AR['err.unknown'])).toBeNull();
  });

  it('ينقل التشخيص الخام إلى شاشة النتيجة المستقلة بعد انتهاء التشغيل', async () => {
    const user = userEvent.setup();
    const { container, out, done } = await startRun(user);
    out(line('ditto: /Users/x/src: Permission denied'));
    done({ run_id: 'run-1', status: 'failed', code: 1, result: FAILED_RESULT });

    await screen.findByRole('heading', { name: AR['result.semantic.failed'] });
    expect(container.querySelector('.result-output code')?.textContent).toBe(
      'ditto: /Users/x/src: Permission denied',
    );
    expect(container.querySelector('.satr')).toBeNull();
  });

  it('يعرض رمز الخروج الذي تعلنه النواة', async () => {
    const user = userEvent.setup();
    const { container, done } = await startRun(user);
    done({ run_id: 'run-1', status: 'failed', code: 2, result: FAILED_RESULT });

    await screen.findByRole('heading', { name: AR['result.semantic.failed'] });
    const detail = container.querySelector('.result-diagnostic');
    expect(detail?.textContent).toContain(AR['state.failed.code']);
    expect(detail?.textContent).toContain('2');
  });

  it('يعرض رقم الإشارة حين تُنهي الأداةَ إشارة', async () => {
    const user = userEvent.setup();
    const { container, done } = await startRun(user);
    done({ run_id: 'run-1', status: 'signalled', signal: 9, result: FAILED_RESULT });

    await screen.findByRole('heading', { name: AR['result.execution.signalled'] });
    const detail = container.querySelector('.result-diagnostic');
    expect(detail?.textContent).toContain(AR['state.failed.signal']);
    expect(detail?.textContent).toContain('9');
  });

  it('يمرّر حقائق الخطة المستهلكة إلى تفاصيل النتيجة بلا argv', async () => {
    const user = userEvent.setup();
    const { container, done } = await startRun(user);
    done({
      run_id: 'run-1',
      status: 'success',
      produced: '/Users/x/dst/Reports.zip',
      result: ARTIFACT_RESULT,
    });

    await screen.findByRole('heading', { name: AR['result.semantic.completed'] });
    await user.click(screen.getByText(AR['result.technical']));
    const details = container.querySelector('.result-technical__body');
    expect(details?.textContent).toContain('run-1');
    expect(details?.textContent).toContain('/usr/bin/ditto');
    expect(details?.textContent).not.toContain('/Users/x/src');
  });

  it('يُفرّغ المجرى عند «مرّة أخرى» فلا يُقرأ خرجُ تشغيلٍ مضى', async () => {
    const user = userEvent.setup();
    const { container, out, done } = await startRun(user);
    out(line('من التشغيل الأول'));
    done({
      run_id: 'run-1',
      status: 'success',
      produced: '/Users/x/dst/Reports.zip',
      result: ARTIFACT_RESULT,
    });

    await user.click((await screen.findAllByRole('button', { name: AR['action.again'] }))[0] as HTMLElement);
    expect(container.querySelector('.stream')).toBeNull();
    expect(document.activeElement).toBe(screen.getByLabelText(AR['field.source.label']));
  });

  it('يبقي النتيجة ظاهرة ويشرح تعذّر Reveal كفعل لاحق', async () => {
    const user = userEvent.setup();
    const { done } = await startRun(user);
    done({
      run_id: 'run-1',
      status: 'success',
      produced: '/Users/x/dst/Reports.zip',
      result: ARTIFACT_RESULT,
    });
    vi.mocked(reveal).mockRejectedValueOnce(new Error('moved'));

    const actions = await waitFor(() => {
      const node = document.querySelector('.result-actions');
      expect(node).toBeTruthy();
      return node as HTMLElement;
    });
    await user.click(within(actions).getByRole('button', { name: AR['action.reveal'] }));
    expect(await screen.findByText(AR['err.reveal.failed'])).toBeTruthy();
    expect(screen.getByRole('heading', { name: AR['result.semantic.completed'] })).toBeTruthy();
  });

  /**
   * إعداد «صوت الإشعارات» — نغمةٌ خفيفة عند اكتمال التشغيل بنجاح، ولا شيء
   * غيره. متداخلٌ هنا (لا وصفٌ شقيق) كي يصل `startRun` أعلاه: الاختبارات
   * تحرس **متى** يُستدعى `playSuccessChime`، لا كيف تُصنَع النغمة نفسها —
   * انظر `notification-sound.test.ts` لتلك.
   */
  describe('نغمة الإشعار', () => {
    // ‏`restoreMocks` لا يُنظّف `vi.fn()` من مصنع `vi.mock()` — يعمل على
    // جواسيس `vi.spyOn` وحدها. واختباراتٌ شقيقة أعلاه تبثّ `status: 'success'`
    // أيضًا لأغراضها الخاصة، فتتراكم استدعاءاتٌ حقيقية من قبل هذا الوصف لو لم
    // يُصفَّ العدّاد هنا صراحةً.
    beforeEach(() => {
      vi.mocked(playSuccessChime).mockClear();
    });

    it('تُشغَّل عند نجاح التشغيل — الإعداد الافتراضي مفعَّل', async () => {
      const user = userEvent.setup();
      const { done } = await startRun(user);

      done({
        run_id: 'run-1',
        status: 'success',
        produced: '/Users/x/dst/Reports.zip',
        result: ARTIFACT_RESULT,
      });

      await screen.findByRole('heading', { name: AR['result.semantic.completed'] });
      expect(playSuccessChime).toHaveBeenCalledTimes(1);
    });

    it.each<RunFinishedEvent['status']>(['failed', 'signalled', 'cancelled', 'error'])(
      'لا تُشغَّل عند نهايةٍ ليست نجاحًا (%s)',
      async (status) => {
        const user = userEvent.setup();
        const { done } = await startRun(user);

        done({ run_id: 'run-1', status, result: FAILED_RESULT } as RunFinishedEvent);

        // زرّ الإلغاء يختفي فور «انتهى»، أيًّا كان نصّ النتيجة المعروض —
        // علامةٌ مستقرّة على أن حدث النهاية وصل ومُعولج بالكامل.
        await waitFor(() =>
          expect(screen.queryByRole('button', { name: AR['satr.action.cancel'] })).toBeNull(),
        );
        expect(playSuccessChime).not.toHaveBeenCalled();
      },
    );

    it('لا تُشغَّل حين يُعطَّل الإعداد رغم نجاح التشغيل', async () => {
      window.localStorage.setItem(
        SETTINGS_STORAGE_KEY,
        JSON.stringify({
          schemaVersion: SETTINGS_SCHEMA_VERSION,
          onboardingCompletedAt: '2026-01-01T00:00:00.000Z',
          notificationSound: false,
        }),
      );
      const user = userEvent.setup();
      const { done } = await startRun(user);

      done({
        run_id: 'run-1',
        status: 'success',
        produced: '/Users/x/dst/Reports.zip',
        result: ARTIFACT_RESULT,
      });

      await screen.findByRole('heading', { name: AR['result.semantic.completed'] });
      expect(playSuccessChime).not.toHaveBeenCalled();
    });
  });
});

/**
 * أدوات المطوّرين — مسار Node.js يُملأ من الإعداد لا فارغًا في كل مرّة.
 *
 * الحارس الحقيقي هنا ضدّ عطلٍ وقع فعلًا أثناء الكتابة: حقلٌ نُسي له مفتاح
 * `.label` (وله `.help` وحدها) يظهر بنصّ مفتاحه الحرفي — `field.node_path.label`
 * — بدل تسميته العربية، ولا اختبار مطابقةٍ ثابت آخر في المشروع كان سيمسكه؛
 * إذ يفحص كل اختبارٍ سابق حقولًا موجودة أصلًا، لا غياب حقلٍ جديد. فهذا
 * الاختبار يرسم عمليةً حقيقية من هذا القسم ويقرأ **التسمية المعروضة فعلًا**،
 * لا افتراض أنها صحيحة.
 */
const DEVELOPER_CATEGORY: CategorySummary = {
  id: 'developer',
  title_key: 'cat.developer.title',
  description_key: 'cat.developer.description',
  icon: '#i-terminal',
  sort_order: 95,
  kind: 'operations',
  operation_count: 1,
  available_count: 1,
};

const DEV_TYPECHECK: OperationSummary = {
  id: 'dev.npm.typecheck',
  title_key: 'op.dev.npm.typecheck.title',
  description_key: 'op.dev.npm.typecheck.description',
  category: 'developer',
  danger: 'safe',
  conflict: 'no_artifact',
  tool: 'npm',
  availability: { state: 'available' },
  sort_order: 10,
  search_terms: ['typecheck', 'npm'],
  inputs: [
    { id: 'project', required: true, kind: 'existing_dir' },
    { id: 'node_path', required: true, kind: 'existing_file' },
  ],
};

describe('أدوات المطوّرين — مسار Node.js', () => {
  async function openDevTypecheck(): Promise<void> {
    vi.mocked(listOperations).mockResolvedValue([DEV_TYPECHECK]);
    vi.mocked(listCategories).mockResolvedValue([DEVELOPER_CATEGORY]);
    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    const category = await screen.findByRole('button', {
      name: new RegExp('^' + AR['cat.developer.title']),
    });
    fireEvent.click(category);
    const card = await screen.findByRole('button', {
      name: new RegExp('^' + AR['op.dev.npm.typecheck.title']),
    });
    fireEvent.click(card);
    await screen.findByRole('heading', { name: AR['op.dev.npm.typecheck.title'] });
  }

  it('حقلا المشروع ومسار Node.js يحملان تسميتهما العربية الحقيقية', async () => {
    await openDevTypecheck();

    // `getByLabelText` يفشل إن كانت التسمية غائبة أو معطوبة — وهذا بالضبط ما
    // يُثبت هنا: لا `field.project.label` ولا `field.node_path.label` حرفيًّا
    // على الشاشة.
    expect(screen.getByLabelText(AR['field.project.label'])).toBeTruthy();
    expect(screen.getByLabelText(AR['field.node_path.label'])).toBeTruthy();
    expect(screen.queryByText('field.node_path.label')).toBeNull();
    expect(screen.queryByText('field.project.label')).toBeNull();
  });

  it('مسار Node.js فارغٌ حين لا إعداد محفوظ، والمشروع لا يُملأ من شيء', async () => {
    await openDevTypecheck();

    const nodeField = screen.getByLabelText(AR['field.node_path.label']) as HTMLInputElement;
    expect(nodeField.value).toBe('');
  });

  it('مسار Node.js يُملأ تلقائيًا من الإعداد المحفوظ عند فتح العملية', async () => {
    window.localStorage.setItem(
      SETTINGS_STORAGE_KEY,
      JSON.stringify({
        schemaVersion: SETTINGS_SCHEMA_VERSION,
        onboardingCompletedAt: '2026-01-01T00:00:00.000Z',
        nodePath: '/usr/local/bin/node',
      }),
    );
    await openDevTypecheck();

    const nodeField = screen.getByLabelText(AR['field.node_path.label']) as HTMLInputElement;
    expect(nodeField.value).toBe('/usr/local/bin/node');
    // والتعبئة التلقائية لا تمنع التعديل: الحقل يبقى نصًّا عاديًا قابلًا للكتابة.
    expect(nodeField.readOnly).toBe(false);
  });
});

/** نظير القسم أعلاه تمامًا، لكن لعملية Cargo — مسارٌ مستقل عن Node.js تمامًا. */
const DEV_CARGO_TEST: OperationSummary = {
  id: 'dev.cargo.test',
  title_key: 'op.dev.cargo.test.title',
  description_key: 'op.dev.cargo.test.description',
  category: 'developer',
  danger: 'safe',
  conflict: 'no_artifact',
  tool: 'cargo',
  availability: { state: 'available' },
  sort_order: 80,
  search_terms: ['cargo', 'test', 'rust'],
  inputs: [
    { id: 'project', required: true, kind: 'existing_dir' },
    { id: 'cargo_path', required: true, kind: 'existing_file' },
  ],
};

describe('أدوات المطوّرين — مسار Cargo', () => {
  async function openDevCargoTest(): Promise<void> {
    vi.mocked(listOperations).mockResolvedValue([DEV_CARGO_TEST]);
    vi.mocked(listCategories).mockResolvedValue([{ ...DEVELOPER_CATEGORY, operation_count: 1, available_count: 1 }]);
    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    const category = await screen.findByRole('button', {
      name: new RegExp('^' + AR['cat.developer.title']),
    });
    fireEvent.click(category);
    const card = await screen.findByRole('button', {
      name: new RegExp('^' + AR['op.dev.cargo.test.title']),
    });
    fireEvent.click(card);
    await screen.findByRole('heading', { name: AR['op.dev.cargo.test.title'] });
  }

  it('حقلا المشروع ومسار Cargo يحملان تسميتهما العربية الحقيقية', async () => {
    await openDevCargoTest();

    expect(screen.getByLabelText(AR['field.project.label'])).toBeTruthy();
    expect(screen.getByLabelText(AR['field.cargo_path.label'])).toBeTruthy();
    expect(screen.queryByText('field.cargo_path.label')).toBeNull();
    expect(screen.queryByText('field.project.label')).toBeNull();
  });

  it('مسار Cargo فارغٌ حين لا إعداد محفوظ', async () => {
    await openDevCargoTest();

    const cargoField = screen.getByLabelText(AR['field.cargo_path.label']) as HTMLInputElement;
    expect(cargoField.value).toBe('');
  });

  it('مسار Cargo يُملأ تلقائيًا من الإعداد المحفوظ عند فتح العملية، ومسار Node.js لا يتأثر', async () => {
    window.localStorage.setItem(
      SETTINGS_STORAGE_KEY,
      JSON.stringify({
        schemaVersion: SETTINGS_SCHEMA_VERSION,
        onboardingCompletedAt: '2026-01-01T00:00:00.000Z',
        nodePath: '/usr/local/bin/node',
        cargoPath: '/Users/dev/.cargo/bin/cargo',
      }),
    );
    await openDevCargoTest();

    const cargoField = screen.getByLabelText(AR['field.cargo_path.label']) as HTMLInputElement;
    expect(cargoField.value).toBe('/Users/dev/.cargo/bin/cargo');
    expect(cargoField.readOnly).toBe(false);
  });
});
