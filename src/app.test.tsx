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
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './app';
import { AR } from './i18n';
import type {
  CategorySummary,
  OperationSummary,
  PlanResponse,
  RunFinishedEvent,
  RunOutputEvent,
} from './ipc';
import {
  execute,
  listOperations,
  listCategories,
  onRunFinished,
  onRunOutput,
  plan as planOperation,
  recentRuns,
} from './ipc';
import { SETTINGS_SCHEMA_VERSION, SETTINGS_STORAGE_KEY } from './settings';

// النواة لا تُستدعى: `@tauri-apps/api` نفسه غير موجود في jsdom، والمصنع هنا
// يمنع تحميل الوحدة الحقيقية أصلًا لا يستبدل دالّةً فيها.
vi.mock('./ipc', () => ({
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
    await screen.findByRole('button', { name: AR['nav.back'] });
    expect(container.querySelector('.screen.log')).toBeTruthy();
    expect(screenRoot(container).closest('.page')).toBeTruthy();
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
    await user.click(screen.getByRole('button', { name: AR['nav.back'] }));

    const heading = await screen.findByRole('heading', { name: AR['cat.compress.title'] });
    expect(document.activeElement).toBe(heading);

    // ومن القسم يعود «رجوع» إلى جذر المكتبة.
    await user.click(screen.getByRole('button', { name: AR['nav.back.library'] }));
    expect(document.activeElement).toBe(
      await screen.findByRole('heading', { name: AR['lib.heading'] }),
    );
  });

  it('وفي الشاشات الثانوية أيضًا: لا انتقال ينتهي بالبؤرة على جسم المستند', async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(screen.getByRole('button', { name: AR['nav.settings'] }));
    expect(document.activeElement).toBe(
      await screen.findByRole('heading', { name: AR['settings.title'] }),
    );
  });

  it('العودة إلى نموذجٍ مملوء تلتقطها الصفحة: النموذج لا يسحب المؤشّر عمدًا', async () => {
    // نموذجٌ فيه كتابة لا يأخذ المؤشّر إلى أول حقل (انظر `naffith.tsx`) — وهو
    // قرارٌ سليم، لكنه يترك البؤرة بلا حامل. هنا تعمل شبكة الأمان في `Page`.
    const user = userEvent.setup();
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: AR['lib.heading'] });

    await user.click(await opCard());
    await user.type(screen.getByLabelText(AR['field.source.label']), '/Users/x/src');
    await user.click(screen.getByRole('button', { name: AR['nav.back'] }));
    await screen.findByRole('heading', { name: AR['cat.compress.title'] });
    await user.click(await opCard());

    expect(document.activeElement).toBe(container.querySelector('.page'));
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

    const run = screen.getByRole('button', { name: AR['action.execute'] });
    await waitFor(() => expect(run.hasAttribute('disabled')).toBe(false));
    await user.click(run);
    await screen.findByRole('button', { name: AR['action.cancel'] });

    const back = screen.getByRole('button', { name: AR['nav.back'] });
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
    expect(screen.getByRole('button', { name: AR['action.cancel'] })).toBeTruthy();
    expect(document.activeElement).toBe(back);
  });

  it('يحبس التبويب بين زرّيه فلا يهرب إلى الشاشة المحجوبة تحته', async () => {
    const user = userEvent.setup();
    await openDialog(user);
    const stay = screen.getByRole('button', { name: AR['nav.leave.busy.stay'] });
    const leave = screen.getByRole('button', { name: AR['nav.leave.busy.leave'] });

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

    const run = screen.getByRole('button', { name: AR['action.execute'] });
    await waitFor(() => expect(run.hasAttribute('disabled')).toBe(false));
    await user.click(run);
    await screen.findByRole('button', { name: AR['action.cancel'] });

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

  it('يُبقي المجرى مقروءًا بعد انتهاء التشغيل', async () => {
    // بعد النهاية يُسأل «ماذا قالت الأداة؟». محوُه عند `run://finished` يمحوه
    // في اللحظة التي يُحتاج فيها.
    const user = userEvent.setup();
    const { container, out, done } = await startRun(user);
    out(line('ditto: /Users/x/src: Permission denied'));
    done({ run_id: 'run-1', status: 'failed', code: 1 });

    await screen.findByText(AR['state.failed']);
    expect(container.querySelector('.stream__text')?.textContent).toBe(
      'ditto: /Users/x/src: Permission denied',
    );
  });

  it('يعرض رمز الخروج الذي تعلنه النواة', async () => {
    const user = userEvent.setup();
    const { container, done } = await startRun(user);
    done({ run_id: 'run-1', status: 'failed', code: 2 });

    await screen.findByText(AR['state.failed']);
    const detail = container.querySelector('.runstate__detail');
    expect(detail?.textContent).toContain(AR['state.failed.code']);
    expect(detail?.textContent).toContain('2');
  });

  it('يعرض رقم الإشارة حين تُنهي الأداةَ إشارة', async () => {
    const user = userEvent.setup();
    const { container, done } = await startRun(user);
    done({ run_id: 'run-1', status: 'signalled', signal: 9 });

    await screen.findByText(AR['state.failed']);
    const detail = container.querySelector('.runstate__detail');
    expect(detail?.textContent).toContain(AR['state.failed.signal']);
    expect(detail?.textContent).toContain('9');
  });

  it('يُفرّغ المجرى عند «مرّة أخرى» فلا يُقرأ خرجُ تشغيلٍ مضى', async () => {
    const user = userEvent.setup();
    const { container, out, done } = await startRun(user);
    out(line('من التشغيل الأول'));
    done({ run_id: 'run-1', status: 'success', produced: '/Users/x/dst/Reports.zip' });

    await user.click(await screen.findByRole('button', { name: AR['action.again'] }));
    expect(container.querySelector('.stream')).toBeNull();
  });
});
