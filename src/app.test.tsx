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
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './app';
import { AR } from './i18n';
import type { OperationSummary, PlanResponse } from './ipc';
import { execute, listOperations, onRunFinished, plan as planOperation, recentRuns } from './ipc';
import { SETTINGS_SCHEMA_VERSION, SETTINGS_STORAGE_KEY } from './settings';

// النواة لا تُستدعى: `@tauri-apps/api` نفسه غير موجود في jsdom، والمصنع هنا
// يمنع تحميل الوحدة الحقيقية أصلًا لا يستبدل دالّةً فيها.
vi.mock('./ipc', () => ({
  listOperations: vi.fn(),
  recentRuns: vi.fn(),
  onRunFinished: vi.fn(),
  plan: vi.fn(),
  execute: vi.fn(),
  cancel: vi.fn(),
  reveal: vi.fn(),
  pickDirectory: vi.fn(),
  asCoreError: (e: unknown) => ({ key: 'err.unknown', input: null, detail: e }),
}));

const COMPRESS: OperationSummary = {
  id: 'compress.folder.zip',
  title_key: 'op.compress.folder.zip.title',
  description_key: 'op.compress.folder.zip.description',
  category: 'compress',
  danger: 'creates',
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
  vi.mocked(recentRuns).mockResolvedValue([]);
  vi.mocked(onRunFinished).mockResolvedValue(() => {});
});

/** جذر الشاشة المعروضة أيًّا كانت: قائمة، أو شاشة ثانوية، أو شاشة عملية. */
function screenRoot(container: HTMLElement): Element {
  const root = container.querySelector('.ops, .screen, .op');
  expect(root, 'لم تُرسم أي شاشة').toBeTruthy();
  return root as Element;
}

describe('وعاء الصفحة في كل شاشة', () => {
  it('قائمة العمليات داخل الوعاء لا ملاصقةً لحافّة النافذة', async () => {
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: AR['ops.heading'] });
    expect(screenRoot(container).closest('.page')).toBeTruthy();
  });

  it('الإعدادات داخل الوعاء', async () => {
    const user = userEvent.setup();
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: AR['ops.heading'] });

    await user.click(screen.getByRole('button', { name: AR['nav.settings'] }));
    expect(container.querySelector('.screen')).toBeTruthy();
    expect(screenRoot(container).closest('.page')).toBeTruthy();
  });

  it('سجلّ التشغيل داخل الوعاء', async () => {
    const user = userEvent.setup();
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: AR['ops.heading'] });

    await user.click(screen.getByRole('button', { name: AR['nav.log'] }));
    await screen.findByRole('button', { name: AR['nav.back'] });
    expect(container.querySelector('.screen.log')).toBeTruthy();
    expect(screenRoot(container).closest('.page')).toBeTruthy();
  });

  it('شاشة العملية داخل الوعاء — وهي التي كانت وحدها فيه', async () => {
    const user = userEvent.setup();
    const { container } = render(<App />);
    await screen.findByRole('heading', { name: AR['ops.heading'] });

    await user.click(
      await screen.findByRole('button', { name: new RegExp(AR['op.compress.folder.zip.title']) }),
    );
    // شاشة العملية تملك تخطيطها وتمتدّ إلى حواف النافذة: شريطٌ فوق سطحين،
    // بلا حشوة الوعاء ولا حدّه الأقصى — وحصرُها فيهما كان يترك ثلث النافذة
    // فراغًا ويحشر النموذج في نصفٍ ضيّق.
    expect(container.querySelector('.op__panes')).toBeTruthy();
    expect(screenRoot(container).closest('.page--bleed')).toBeTruthy();
  });
});

/** بطاقة العملية في القائمة. */
function opCard(): Promise<HTMLElement> {
  return screen.findByRole('button', { name: new RegExp(AR['op.compress.folder.zip.title']) });
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

    const heading = await screen.findByRole('heading', { name: AR['ops.heading'] });
    expect(document.activeElement).toBe(heading);
    expect(document.activeElement).not.toBe(document.body);
  });

  it('بعد «رجوع» من شاشة العملية تعود البؤرة إلى عنوان القائمة', async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole('heading', { name: AR['ops.heading'] });

    await user.click(await opCard());
    await user.click(screen.getByRole('button', { name: AR['nav.back'] }));

    const heading = await screen.findByRole('heading', { name: AR['ops.heading'] });
    expect(document.activeElement).toBe(heading);
  });

  it('وفي الشاشات الثانوية أيضًا: لا انتقال ينتهي بالبؤرة على جسم المستند', async () => {
    const user = userEvent.setup();
    render(<App />);
    await screen.findByRole('heading', { name: AR['ops.heading'] });

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
    await screen.findByRole('heading', { name: AR['ops.heading'] });

    await user.click(await opCard());
    await user.type(screen.getByLabelText(AR['field.source.label']), '/Users/x/src');
    await user.click(screen.getByRole('button', { name: AR['nav.back'] }));
    await screen.findByRole('heading', { name: AR['ops.heading'] });
    await user.click(await opCard());

    expect(document.activeElement).toBe(container.querySelector('.page'));
  });

  it('الوعاء هدفٌ للبرنامج لا محطّةٌ لـTab', async () => {
    render(<App />);
    await screen.findByRole('heading', { name: AR['ops.heading'] });
    expect(document.querySelector('.page')?.getAttribute('tabindex')).toBe('-1');
  });
});

describe('حوار المغادرة أثناء تشغيل', () => {
  /** يقود التطبيق إلى تشغيلٍ جارٍ ثم يطلب الرجوع، فيُفتح الحوار. */
  async function openDialog(user: User) {
    vi.mocked(planOperation).mockResolvedValue(PLAN);
    vi.mocked(execute).mockResolvedValue('run-1');

    const view = render(<App />);
    await screen.findByRole('heading', { name: AR['ops.heading'] });
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

    const heading = await screen.findByRole('heading', { name: AR['ops.heading'] });
    expect(document.activeElement).toBe(heading);
  });
});
