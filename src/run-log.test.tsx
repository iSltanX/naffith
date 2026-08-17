// @vitest-environment jsdom
/**
 * سجلّ التشغيل.
 *
 * النواة مستهزأ بها كاملةً: هذه الشاشة تُختبر على ما يصل لا على ما يجري في
 * Rust، وحالاتٌ خمس لا تجتمع في تشغيلٍ حقيقيّ واحد تجتمع هنا في مصفوفةٍ واحدة.
 *
 * أهمّ ما يحرسه هذا الملف شيئان لا يظهران في الشاشة كخطأ:
 *
 * 1. **الوقت ثوانٍ لا أجزاء ألف.** `at` القادم من `journal.rs` بالثواني، وتمريرُه
 *    إلى `Date` كما هو يعطي تاريخًا معقول الشكل في ١٩٧٠. لذلك يُفحص السَنَة.
 * 2. **المسارات معزولة.** مسارٌ لاتينيّ بلا `bdi` داخل صفٍّ عربيّ ينقلب ترتيب
 *    مقاطعه، فيقرأ المستخدم أمرًا غير الذي شُغّل.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { JournalEntry, ResultContract } from './ipc';
import { cancel, journalClear, journalDelete, onRunFinished, recentRuns, reveal } from './ipc';
import { t } from './i18n';
import RunLog, { coalesceRuns, matchesQuery } from './run-log';

// النواة لا تُستدعى: `@tauri-apps/api` نفسه غير موجود في jsdom، والمصنع هنا
// يمنع تحميل الوحدة الحقيقية أصلًا لا يستبدل دالّةً فيها.
vi.mock('./ipc', () => ({
  recentRuns: vi.fn(),
  journalClear: vi.fn(),
  journalDelete: vi.fn(),
  cancel: vi.fn(),
  onRunFinished: vi.fn(),
  reveal: vi.fn(),
}));

afterEach(cleanup);

/** ثوانٍ منذ الحقبة — كما تكتبها النواة. تقع في ٢٠٢٦. */
const AT = 1_770_000_000;

const SOURCE = '/Users/sara/مستندات/تقارير';
const TEMP = '/Users/sara/Desktop/.naffith-1770000000.zip';

function entry(over: Partial<JournalEntry>): JournalEntry {
  return {
    id: 'run-1',
    op_id: 'compress.folder.zip',
    at: AT,
    duration_ms: null,
    program: '/usr/bin/ditto',
    args: ['-c', '-k', '--sequesterRsrc', SOURCE, TEMP],
    cwd: null,
    state: 'succeeded',
    ...over,
  };
}

/** الحالات الخمس، بترتيب كتابة النواة لها: الأقدم أوّلًا. */
const ALL_STATES: JournalEntry[] = [
  entry({ id: 'a', state: 'planned' }),
  entry({ id: 'b', state: 'running' }),
  entry({ id: 'c', state: 'succeeded', duration_ms: 4200 }),
  entry({ id: 'd', state: 'failed', code: 1, reason: 'exit' }),
  entry({ id: 'e', state: 'cancelled' }),
];

beforeEach(() => {
  vi.clearAllMocks();
  // `restoreMocks: true` يمسح التنفيذ قبل كل اختبار، فبلا هذا السطر تعيد
  // الدالّة `undefined` ويسقط `.then` بخطأ لا علاقة له بما يُختبر.
  vi.mocked(recentRuns).mockResolvedValue([]);
  vi.mocked(journalClear).mockResolvedValue();
  vi.mocked(journalDelete).mockResolvedValue();
  vi.mocked(cancel).mockResolvedValue();
  vi.mocked(onRunFinished).mockResolvedValue(() => {});
  vi.mocked(reveal).mockResolvedValue();
});

describe('سجلّ التشغيل', () => {
  it('يعرض التحميل في State Panel بعنوانٍ ومتن مستقلّين', async () => {
    vi.mocked(recentRuns).mockImplementation(() => new Promise<JournalEntry[]>(() => {}));
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const title = screen.getByRole('heading', { name: t('log.title'), level: 2 });
    expect(document.activeElement).toBe(title);
    const panel = await screen.findByRole('status');
    expect(panel.classList.contains('state-panel')).toBe(true);
    expect(within(panel).getByRole('heading', { name: 'يجري تحميل…' })).toBeDefined();
    expect(within(panel).getByText('انتظار هادئ.')).toBeDefined();
    expect(panel.getAttribute('aria-busy')).toBe('true');
  });

  it('يعرض حالات دورة الحياة بعناوين Page 18 وحدودها الكاملة', async () => {
    vi.mocked(recentRuns).mockResolvedValue(ALL_STATES);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const list = await screen.findByRole('list');
    const status = (state: string) =>
      list.querySelector(`.entry--${state} .entry__status`)?.textContent ?? '';
    expect(status('planned')).toBe(t('log.entry.status.planned'));
    expect(status('running').startsWith(t('log.entry.status.running'))).toBe(true);
    expect(status('succeeded').startsWith(t('log.entry.status.succeeded'))).toBe(true);
    expect(status('failed')).toBe(
      `${t('log.entry.status.failed')} · ${t('state.failed.code')} 1`,
    );
    expect(status('cancelled')).toBe(t('log.entry.status.cancelled'));

    for (const item of within(list).getAllByRole('listitem')) {
      expect(item.querySelectorAll('.entry__divider')).toHaveLength(1);
      expect(item.querySelector('.entry__summary')?.getAttribute('dir')).toBeNull();
      expect(item.querySelector('.entry__technical')?.getAttribute('dir')).toBe('ltr');
      expect(item.querySelector('.entry__identity')?.getAttribute('dir')).toBe('rtl');
      expect(item.querySelector('.entry__state-mark')).not.toBeNull();
      expect(item.querySelector('.chip')).toBeNull();
      expect(item.querySelector('.entry__delete')).toBeNull();
      expect(item.querySelector('.entry__disclosure svg')).toBeNull();
    }
  });

  it('يحجب القيد غير المعروف خلف لوحة تحذير ثم يحتفظ بأمره ومعرّفه', async () => {
    const unknown = entry({
      id: 'future-run',
      op_id: 'future.operation',
      state: 'paused' as JournalEntry['state'],
      program: '/usr/bin/future-tool',
      args: ['--inspect'],
    });
    vi.mocked(recentRuns).mockResolvedValue([unknown]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const panel = await screen.findByRole('status', { name: t('log.unknown.title') });
    expect(panel.classList.contains('log-unknown')).toBe(true);
    expect(within(panel).getByText(t('log.unknown.body'))).toBeDefined();
    expect(panel.querySelector('svg')).toBeNull();
    expect(screen.queryByRole('list')).toBeNull();
    expect(screen.queryByText('future.operation')).toBeNull();

    await userEvent.click(within(panel).getByRole('button', { name: t('log.unknown.continue') }));
    const list = await screen.findByRole('list');
    expect(within(list).getByText(t('log.entry.status.unknown'))).toBeDefined();
    expect(within(list).getByText('future.operation')).toBeDefined();
    expect(within(list).queryByText('log.state.paused')).toBeNull();
    expect(within(list).queryByRole('button', { name: t('log.rerun') })).toBeNull();
    expect(within(list).getByRole('button', { name: t('log.delete') }).classList.contains('entry__action--danger')).toBe(true);

    await userEvent.click(within(list).getByRole('button', { name: t('log.action.details') }));
    const item = within(list).getByRole('listitem');
    expect(item.classList.contains('is-expanded')).toBe(true);
    expect(item.querySelector('.entry__target')?.textContent).toBe(
      '/usr/bin/future-tool --inspect',
    );
    expect(item.querySelector('.entry__detail-note')?.textContent).toBe('future-run');
    expect(item.querySelector('.entry__detail-note')?.classList.contains('entry__detail-note--technical')).toBe(true);
    expect(item.querySelector('.entry__run-id')?.textContent).toBe('future-run');
  });

  it('يعلن سقف أحدث مئتي قيد ولا يرسم أكثر منه', async () => {
    vi.mocked(recentRuns).mockResolvedValue(
      Array.from({ length: 200 }, (_, index) => entry({ id: `run-${index}` })),
    );
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const list = await screen.findByRole('list');
    expect(within(list).getAllByRole('listitem')).toHaveLength(200);
    const cap = screen.getByRole('status', { name: t('log.cap.title') });
    expect(cap.classList.contains('log-cap')).toBe(true);
    expect(within(cap).getByText(t('log.cap.body'))).toBeDefined();
    expect(cap.querySelector('svg')).toBeNull();
  });

  it('يعرض القيد العالق في «جارية» كما هو لا ناجحًا ولا فاشلًا', async () => {
    // تطبيقٌ قُتل في منتصف تشغيل يترك هذا القيد إلى الأبد. استنتاجُ نهايةٍ له
    // كذبٌ في الاتجاهين.
    vi.mocked(recentRuns).mockResolvedValue([entry({ state: 'running' })]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const list = await screen.findByRole('list');
    const item = within(list).getByRole('listitem');
    expect(item.classList.contains('entry--running')).toBe(true);
    expect(item.querySelector('.entry__status')?.textContent?.startsWith(t('log.entry.status.running'))).toBe(true);
    expect(item.classList.contains('entry--succeeded')).toBe(false);
    expect(item.classList.contains('entry--failed')).toBe(false);
  });

  it('يعرض الفراغ حين لا تشغيلات', async () => {
    vi.mocked(recentRuns).mockResolvedValue([]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const empty = (await screen.findByText(t('log.empty'))).closest('.log-empty');
    expect(empty).not.toBeNull();
    expect(within(empty as HTMLElement).getByText(t('log.empty.body'))).toBeDefined();
    expect(empty?.querySelector('svg')).toBeNull();
  });

  it('يعرض تعذّر القراءة حين ترفض النواة، ويُبقي إعادة المحاولة ممكنة', async () => {
    vi.mocked(recentRuns).mockRejectedValue({ key: 'err.io', input: null, detail: null });
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const panel = await screen.findByRole('alert');
    expect(panel.classList.contains('state-panel')).toBe(true);
    expect(within(panel).getByRole('heading', { name: 'تعذّر تحميل سجلّ التشغيل.' })).toBeDefined();
    expect(
      within(panel).getByText('تعذّر الوصول إلى سجلّ التشغيل الآن. حاول مرة أخرى.'),
    ).toBeDefined();

    vi.mocked(recentRuns).mockResolvedValue([entry({ state: 'succeeded' })]);
    await userEvent.click(within(panel).getByRole('button', { name: 'إعادة المحاولة' }));

    const list = await screen.findByRole('list');
    expect(list.querySelector('.entry--succeeded .entry__status')?.textContent).toBe(
      t('log.entry.status.succeeded'),
    );
    await waitFor(() => expect(screen.queryByText(t('log.failed'))).toBeNull());
  });

  it('يكشف تفاصيل Page 18 في البطاقة نفسها بتسمية متبدّلة', async () => {
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ tail: ['first output line', 'second output line'] }),
    ]);
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const item = within(await screen.findByRole('list')).getByRole('listitem');
    expect(item.querySelectorAll('details')).toHaveLength(0);

    const show = within(item).getByRole('button', { name: 'إظهار التفاصيل' });
    const controlledId = show.getAttribute('aria-controls');
    const details = controlledId ? document.getElementById(controlledId) : null;
    expect(show.getAttribute('aria-expanded')).toBe('false');
    expect(details?.classList.contains('entry__details')).toBe(true);
    expect(details?.hasAttribute('hidden')).toBe(true);
    expect(details?.querySelector('.entry__target')).not.toBeNull();
    expect(details?.querySelector('.entry__detail-note')).not.toBeNull();
    expect(details?.querySelector('.entry__technical-output')).not.toBeNull();
    expect(details?.querySelector('.entry__command')).toBeNull();
    expect(details?.querySelector('.entry__tail')).toBeNull();
    expect(show.querySelector('svg')).toBeNull();

    await user.click(show);
    const hide = within(item).getByRole('button', { name: 'إخفاء التفاصيل' });
    expect(hide.getAttribute('aria-expanded')).toBe('true');
    expect(details?.hasAttribute('hidden')).toBe(false);
    expect(item.classList.contains('is-expanded')).toBe(true);
    expect(details?.querySelector('.entry__detail-note')?.textContent).toContain('second output line');
    expect(details?.querySelector('.entry__technical-output')?.textContent).toContain(
      'semantic completed',
    );

    await user.click(hide);
    expect(within(item).getByRole('button', { name: 'إظهار التفاصيل' })).toBeDefined();
    expect(details?.hasAttribute('hidden')).toBe(true);
  });

  it('يعزل المسارات فلا ينقلب ترتيبها داخل الصفّ العربي', async () => {
    const artifact: ResultContract = {
      category: 'artifact',
      semantic: 'completed',
      type: 'artifact',
      path: TEMP,
      name: '.naffith-1770000000.zip',
      parent: '/Users/sara/Desktop',
      reveal: 'file',
    };
    vi.mocked(recentRuns).mockResolvedValue([entry({ id: 'run-path', result: artifact })]);
    const { container } = render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    await screen.findByRole('list');

    const isolated = [...container.querySelectorAll('bdi[dir="ltr"]')].map((n) => n.textContent);
    expect(isolated).toContain('run-path');
    expect(isolated).toContain('COMPRESS · ZIP');
    expect(isolated).toContain(TEMP);
    expect(isolated).not.toContain(SOURCE);
  });

  it('يقرأ الوقت ثوانيَ لا أجزاءَ ألفٍ من الثانية', async () => {
    // ‏`at` بالثواني في `journal.rs`. لو مُرّر إلى `Date` كما هو لَظهر ١٩٧٠،
    // وهو تاريخ لا يبدو خطأً في الشاشة.
    vi.mocked(recentRuns).mockResolvedValue([entry({})]);
    const { container } = render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    await screen.findByRole('list');

    const time = container.querySelector('time');
    expect(time?.getAttribute('datetime')?.startsWith('2026')).toBe(true);
  });

  it('يعرض الأحدث أوّلًا', async () => {
    // النواة تُلحق القيود، وأوّل ما يُسأل عنه آخرُ ما وقع.
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ id: 'old', state: 'cancelled' }),
      entry({ id: 'new', state: 'succeeded' }),
    ]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const items = within(await screen.findByRole('list')).getAllByRole('listitem');
    const [newest, oldest] = items;
    if (!newest || !oldest) throw new Error('لم يُرسم قيدان');
    expect(newest.classList.contains('entry--succeeded')).toBe(true);
    expect(newest.querySelector('.entry__run-id')?.textContent).toBe('new');
    expect(oldest.classList.contains('entry--cancelled')).toBe(true);
    expect(oldest.querySelector('.entry__run-id')?.textContent).toBe('old');
    expect(document.querySelector('.chip')).toBeNull();
  });

  it('تستعمل دعوة الفراغ طريق المكتبة بدل زرّ رجوع مكرر مع التنقّل الثابت', async () => {
    const onBack = vi.fn();
    render(<RunLog onBack={onBack} onRerun={() => {}} onChanged={() => {}} />);

    await userEvent.click(await screen.findByRole('button', { name: t('log.start') }));
    expect(onBack).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole('button', { name: t('nav.back') })).toBeNull();
  });

  it('يصفّي بشرائح Page 18 غير التدميرية ويتيح إلغاءها من حالة الفراغ', async () => {
    const user = userEvent.setup();
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ id: 'one', state: 'succeeded' }),
      entry({ id: 'two', state: 'failed' }),
    ]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);
    await screen.findByRole('list');

    const failed = screen.getByRole('button', { name: t('log.filter.failed') });
    await user.click(failed);
    expect(failed.getAttribute('aria-pressed')).toBe('true');
    expect(screen.getByText(t('log.entry.status.failed'))).toBeTruthy();
    expect(screen.queryByText(t('log.entry.status.succeeded'))).toBeNull();

    await user.click(screen.getByRole('button', { name: t('log.filter.cancelled') }));
    const filteredEmpty = screen.getByText(t('log.empty.filtered')).closest('.log-empty');
    expect(filteredEmpty).not.toBeNull();
    await user.click(
      within(filteredEmpty as HTMLElement).getByRole('button', {
        name: t('log.filter.reset'),
      }),
    );
    expect(await screen.findByRole('list')).toBeTruthy();
  });

  it('لا يعرض Reveal لمجرد وجود produced؛ يتبع عقد النواة', async () => {
    const artifact: ResultContract = {
      category: 'artifact',
      semantic: 'completed',
      type: 'artifact',
      path: '/Users/sara/out.zip',
      name: 'out.zip',
      parent: '/Users/sara',
      output: [],
      reveal: 'file',
    };
    const withResult = entry({ produced: '/Users/sara/out.zip' }) as JournalEntry & {
      result: ResultContract;
    };
    withResult.result = artifact;
    vi.mocked(recentRuns).mockResolvedValue([withResult]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);
    expect(await screen.findByRole('button', { name: t('action.reveal') })).toBeTruthy();

    cleanup();
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ id: 'legacy', produced: '/Users/sara/old.zip' }),
    ]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);
    await screen.findByRole('list');
    expect(screen.queryByRole('button', { name: t('action.reveal') })).toBeNull();

    cleanup();
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ id: 'failed-artifact', state: 'failed', result: artifact }),
    ]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);
    await screen.findByRole('list');
    expect(screen.queryByRole('button', { name: t('action.reveal') })).toBeNull();
  });

  it('يعرض تعذّر Reveal داخل القيد بلا تغيير نتيجة التشغيل', async () => {
    const artifact: ResultContract = {
      category: 'artifact',
      semantic: 'completed',
      type: 'artifact',
      path: '/Users/sara/out.zip',
      name: 'out.zip',
      parent: '/Users/sara',
      reveal: 'file',
    };
    vi.mocked(recentRuns).mockResolvedValue([entry({ result: artifact })]);
    vi.mocked(reveal).mockRejectedValueOnce({ key: 'err.reveal.nothing' });
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    await user.click(await screen.findByRole('button', { name: t('action.reveal') }));
    expect(await screen.findByText(t('err.reveal.failed'))).toBeTruthy();
    expect(screen.getByText(t('log.entry.status.succeeded'))).toBeTruthy();
  });

  it('يلغي قيد المعاينة المتروك فورًا بمسحه المحلي الآمن', async () => {
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ id: 'preview-to-cancel', state: 'planned' }),
    ]);
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);
    await screen.findByRole('list');

    await user.click(screen.getByRole('button', { name: t('log.action.cancel') }));
    expect(journalDelete).toHaveBeenCalledWith('preview-to-cancel');
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(screen.queryByRole('button', { name: t('log.rerun') })).toBeNull();
  });

  it('يبقي المعاينة ويظهر فشل إلغائها', async () => {
    vi.mocked(recentRuns).mockResolvedValue([entry({ id: 'uncancelled', state: 'planned' })]);
    vi.mocked(journalDelete).mockRejectedValueOnce(new Error('write failed'));
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);
    await screen.findByRole('list');

    await user.click(screen.getByRole('button', { name: t('log.action.cancel') }));

    expect(await screen.findByText(t('log.preview.cancel.failed'))).toBeTruthy();
    expect(screen.getByText('uncancelled')).toBeTruthy();
  });

  it('يحذف القيد النهائي فورًا بفعل خطر بلا حوار', async () => {
    vi.mocked(recentRuns).mockResolvedValue([entry({ id: 'terminal-to-delete' })]);
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const remove = await screen.findByRole('button', { name: t('log.delete') });
    expect(remove.classList.contains('entry__action--danger')).toBe(true);
    await user.click(remove);

    expect(journalDelete).toHaveBeenCalledWith('terminal-to-delete');
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('يبقي القيد النهائي ويظهر فشل حذفه', async () => {
    vi.mocked(recentRuns).mockResolvedValue([entry({ id: 'terminal-retained', state: 'failed' })]);
    vi.mocked(journalDelete).mockRejectedValueOnce(new Error('write failed'));
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    await user.click(await screen.findByRole('button', { name: t('log.delete') }));

    expect(await screen.findByText(t('log.delete.failed'))).toBeTruthy();
    expect(screen.getByText('terminal-retained')).toBeTruthy();
    expect(screen.queryByRole('alertdialog')).toBeNull();
  });

  it('يمرّر إيقاف القيد الجاري بمعرّف التشغيل ويمنع التكرار', async () => {
    vi.mocked(recentRuns).mockResolvedValue([entry({ id: 'running-id', state: 'running' })]);
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const stop = await screen.findByRole('button', { name: t('log.action.stop') });
    await user.click(stop);
    expect(cancel).toHaveBeenCalledWith('running-id');
    expect(stop.hasAttribute('disabled')).toBe(true);
    expect(stop.getAttribute('aria-busy')).toBe('true');
    expect(journalDelete).not.toHaveBeenCalled();
    expect(screen.queryByRole('button', { name: t('log.rerun') })).toBeNull();
  });

  it('يعرض فشل الإيقاف ويعيد الفعل للمحاولة', async () => {
    vi.mocked(recentRuns).mockResolvedValue([entry({ id: 'unstopped', state: 'running' })]);
    vi.mocked(cancel).mockRejectedValueOnce(new Error('not active'));
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const stop = await screen.findByRole('button', { name: t('log.action.stop') });
    await user.click(stop);
    expect(await screen.findByText(t('log.stop.failed'))).toBeTruthy();
    await waitFor(() => expect(stop.hasAttribute('disabled')).toBe(false));
  });

  it('يحصر «تشغيل مجددًا» في النهايات المعروفة', async () => {
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ id: 'success', state: 'succeeded' }),
      entry({ id: 'failure', state: 'failed' }),
      entry({ id: 'cancelled', state: 'cancelled' }),
      entry({ id: 'preview', state: 'planned' }),
      entry({ id: 'active', state: 'running' }),
    ]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    await screen.findByRole('list');
    expect(screen.getAllByRole('button', { name: t('log.rerun') })).toHaveLength(3);
    expect(screen.getAllByRole('button', { name: t('log.action.cancel') })).toHaveLength(1);
    expect(screen.getAllByRole('button', { name: t('log.action.stop') })).toHaveLength(1);
  });

  it('يعيد العملية المعروفة حتى إن لم يكن لها مدخل واحد', async () => {
    const zeroInput = entry({ id: 'zero-input', op_id: 'system.info', inputs: [] });
    const onRerun = vi.fn();
    vi.mocked(recentRuns).mockResolvedValue([zeroInput]);
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={onRerun} onChanged={vi.fn()} />);

    await user.click(await screen.findByRole('button', { name: 'تشغيل مجددًا' }));
    expect(onRerun).toHaveBeenCalledWith(zeroInput);
  });

  it('يحصر التركيز في تأكيد المسح ويجعل المسح فعلًا صريحًا', async () => {
    const user = userEvent.setup();
    vi.mocked(recentRuns).mockResolvedValue([entry({})]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);
    await screen.findByRole('list');
    const clear = screen.getByRole('button', { name: t('log.clear') });
    expect(clear.classList.contains('log__clear')).toBe(true);
    expect(clear.querySelector('svg')).toBeNull();
    await user.click(clear);

    const dialog = screen.getByRole('alertdialog');
    const safe = within(dialog).getByRole('button', { name: t('log.clear.cancel') });
    const destructive = within(dialog).getByRole('button', { name: t('log.clear.confirm') });
    expect(dialog.querySelector('svg')).toBeNull();
    expect(t('dialog.safe_dismiss')).toBe('Escape والنقر على الخلفية = الإجراء الآمن');
    expect(within(dialog).getByText(t('dialog.safe_dismiss'))).toBeDefined();
    expect(document.activeElement).toBe(safe);
    destructive.focus();
    await user.tab();
    expect(document.activeElement).toBe(safe);

    await user.click(destructive);
    expect(journalClear).toHaveBeenCalledTimes(1);
  });

  it('يجعل Escape والخلفية مخرجين آمنين من تأكيد المسح', async () => {
    const user = userEvent.setup();
    vi.mocked(journalClear).mockClear();
    vi.mocked(recentRuns).mockResolvedValue([entry({})]);
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);
    await screen.findByRole('list');

    await user.click(screen.getByRole('button', { name: t('log.clear') }));
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(journalClear).not.toHaveBeenCalled();

    await user.click(screen.getByRole('button', { name: t('log.clear') }));
    const dialog = screen.getByRole('alertdialog');
    const scrim = dialog.parentElement;
    expect(scrim?.classList.contains('scrim')).toBe(true);
    await user.click(scrim as HTMLElement);
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect(journalClear).not.toHaveBeenCalled();
  });

  it('يعرض فشل المسح ويبقي القيود', async () => {
    vi.mocked(recentRuns).mockResolvedValue([entry({})]);
    vi.mocked(journalClear).mockRejectedValueOnce(new Error('write failed'));
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);
    await screen.findByRole('list');

    await user.click(screen.getByRole('button', { name: t('log.clear') }));
    await user.click(
      within(screen.getByRole('alertdialog')).getByRole('button', {
        name: t('log.clear.confirm'),
      }),
    );

    expect(await screen.findByText(t('log.clear.failed'))).toBeTruthy();
    expect(screen.getByText('run-1')).toBeTruthy();
  });
});

describe('طيّ دورة حياة السجل', () => {
  it('يختار آخر انتقال بترتيب الإلحاق ويبقي المعاينات والتشغيلات المتروكة', () => {
    const terminal = entry({ id: 'a', state: 'succeeded', tail: ['latest'] });
    const abandonedRunning = entry({ id: 'b', state: 'running' });
    const abandonedPreview = entry({ id: 'c', state: 'planned' });

    const coalesced = coalesceRuns([
      entry({ id: 'a', state: 'planned' }),
      abandonedRunning,
      entry({ id: 'a', state: 'running' }),
      abandonedPreview,
      terminal,
    ]);

    expect(coalesced.map(({ id, state }) => [id, state])).toEqual([
      ['b', 'running'],
      ['c', 'planned'],
      ['a', 'succeeded'],
    ]);
    expect(coalesced[0]).toBe(abandonedRunning);
    expect(coalesced[1]).toBe(abandonedPreview);
    expect(coalesced[2]).toBe(terminal);
  });

  it('يطوي قبل الرسم والتصفية فيظهر صفٌّ واحد للتشغيل', async () => {
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ id: 'same', state: 'planned' }),
      entry({ id: 'abandoned', state: 'planned' }),
      entry({ id: 'same', state: 'running' }),
      entry({ id: 'same', state: 'succeeded' }),
    ]);
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const list = await screen.findByRole('list');
    expect(within(list).getAllByRole('listitem')).toHaveLength(2);
    expect(within(list).getAllByText('same')).toHaveLength(1);
    expect(list.querySelector('.entry--succeeded')).not.toBeNull();
    expect(list.querySelector('.entry--planned')).not.toBeNull();

    await user.click(screen.getByRole('button', { name: t('log.filter.succeeded') }));
    expect(within(list).getAllByRole('listitem')).toHaveLength(1);
    expect(list.querySelector('.entry--succeeded')).not.toBeNull();
  });

  it('لا يخترع exit 0 لنتيجة نطاقية ناجحة بخروج غير صفري', async () => {
    const semanticResult: ResultContract = {
      category: 'diff_search',
      semantic: 'no_matches',
      type: 'diff_search',
      kind: 'search',
      items: [],
      notices: [],
    };
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ id: 'domain-exit', state: 'succeeded', result: semanticResult }),
    ]);
    const user = userEvent.setup();
    render(<RunLog onBack={vi.fn()} onRerun={vi.fn()} onChanged={vi.fn()} />);

    const item = within(await screen.findByRole('list')).getByRole('listitem');
    await user.click(within(item).getByRole('button', { name: t('log.details.show') }));
    const technical = item.querySelector('.entry__technical-output')?.textContent ?? '';
    expect(technical).toContain('semantic no_matches');
    expect(technical).not.toContain('exit 0');
  });
});

describe('بحث السجل النقي', () => {
  it('يطوي العربية ويبحث في الحمولة المهيكلة بلا تحليلها', () => {
    const item = entry({ args: ['/Users/sara/أرشيف'] }) as JournalEntry & {
      result: ResultContract;
    };
    item.result = {
      category: 'raw_output',
      semantic: 'completed',
      type: 'raw_output',
      lines: [{ stream: 'stdout', line: 'سَطْرٌ مُشَكَّل' }],
    };
    expect(matchesQuery(item, 'ارشيف')).toBe(true);
    expect(matchesQuery(item, 'سطر مشكل')).toBe(true);
    expect(matchesQuery(item, 'قيمة أخرى')).toBe(false);
  });

  it('يبحث في صفوف الحمولة المهيكلة لا في stdout مفترض', () => {
    const item = entry({}) as JournalEntry & { result: ResultContract };
    item.result = {
      category: 'collection',
      semantic: 'completed',
      type: 'collection',
      kind: 'file_matches',
      columns: ['path', 'size'],
      rows: [{ cells: ['/Users/sara/عقد.pdf', '42 KB'], stream: 'stdout' }],
      notices: [],
    };
    expect(matchesQuery(item, 'عقد')).toBe(true);
    expect(matchesQuery(item, '42 kb')).toBe(true);
  });
});
