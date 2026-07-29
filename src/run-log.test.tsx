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
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { JournalEntry } from './ipc';
import { recentRuns } from './ipc';
import { t } from './i18n';
import RunLog from './run-log';

// النواة لا تُستدعى: `@tauri-apps/api` نفسه غير موجود في jsdom، والمصنع هنا
// يمنع تحميل الوحدة الحقيقية أصلًا لا يستبدل دالّةً فيها.
vi.mock('./ipc', () => ({ recentRuns: vi.fn() }));

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
  // `restoreMocks: true` يمسح التنفيذ قبل كل اختبار، فبلا هذا السطر تعيد
  // الدالّة `undefined` ويسقط `.then` بخطأ لا علاقة له بما يُختبر.
  vi.mocked(recentRuns).mockResolvedValue([]);
});

describe('سجلّ التشغيل', () => {
  it('يعرض كل حالة باسمها', async () => {
    vi.mocked(recentRuns).mockResolvedValue(ALL_STATES);
    render(<RunLog onBack={vi.fn()} />);

    for (const state of ['planned', 'running', 'succeeded', 'failed', 'cancelled']) {
      expect(await screen.findByText(t(`log.state.${state}`))).toBeDefined();
    }
  });

  it('يعرض القيد العالق في «جارية» كما هو لا ناجحًا ولا فاشلًا', async () => {
    // تطبيقٌ قُتل في منتصف تشغيل يترك هذا القيد إلى الأبد. استنتاجُ نهايةٍ له
    // كذبٌ في الاتجاهين.
    vi.mocked(recentRuns).mockResolvedValue([entry({ state: 'running' })]);
    render(<RunLog onBack={vi.fn()} />);

    expect(await screen.findByText(t('log.state.running'))).toBeDefined();
    expect(screen.queryByText(t('log.state.succeeded'))).toBeNull();
    expect(screen.queryByText(t('log.state.failed'))).toBeNull();
  });

  it('يعرض الفراغ حين لا تشغيلات', async () => {
    vi.mocked(recentRuns).mockResolvedValue([]);
    render(<RunLog onBack={vi.fn()} />);

    expect(await screen.findByText(t('log.empty'))).toBeDefined();
  });

  it('يعرض تعذّر القراءة حين ترفض النواة، ويُبقي إعادة المحاولة ممكنة', async () => {
    vi.mocked(recentRuns).mockRejectedValue({ key: 'err.io', input: null, detail: null });
    render(<RunLog onBack={vi.fn()} />);

    expect(await screen.findByText(t('log.failed'))).toBeDefined();

    vi.mocked(recentRuns).mockResolvedValue([entry({ state: 'succeeded' })]);
    await userEvent.click(screen.getByRole('button', { name: t('ops.retry') }));

    expect(await screen.findByText(t('log.state.succeeded'))).toBeDefined();
    await waitFor(() => expect(screen.queryByText(t('log.failed'))).toBeNull());
  });

  it('يعزل المسارات فلا ينقلب ترتيبها داخل الصفّ العربي', async () => {
    vi.mocked(recentRuns).mockResolvedValue([entry({})]);
    const { container } = render(<RunLog onBack={vi.fn()} />);

    await screen.findByText(t('log.state.succeeded'));

    const isolated = [...container.querySelectorAll('bdi[dir="ltr"]')].map((n) => n.textContent);
    expect(isolated).toContain(SOURCE);
    expect(isolated).toContain(TEMP);
    expect(isolated).toContain('/usr/bin/ditto');
  });

  it('يقرأ الوقت ثوانيَ لا أجزاءَ ألفٍ من الثانية', async () => {
    // ‏`at` بالثواني في `journal.rs`. لو مُرّر إلى `Date` كما هو لَظهر ١٩٧٠،
    // وهو تاريخ لا يبدو خطأً في الشاشة.
    vi.mocked(recentRuns).mockResolvedValue([entry({})]);
    const { container } = render(<RunLog onBack={vi.fn()} />);

    await screen.findByText(t('log.state.succeeded'));

    const time = container.querySelector('time');
    expect(time?.getAttribute('datetime')?.startsWith('2026')).toBe(true);
  });

  it('يعرض الأحدث أوّلًا', async () => {
    // النواة تُلحق القيود، وأوّل ما يُسأل عنه آخرُ ما وقع.
    vi.mocked(recentRuns).mockResolvedValue([
      entry({ id: 'old', state: 'cancelled' }),
      entry({ id: 'new', state: 'succeeded' }),
    ]);
    const { container } = render(<RunLog onBack={vi.fn()} />);

    await screen.findByText(t('log.state.succeeded'));

    const labels = [...container.querySelectorAll('.chip')].map((n) => n.textContent);
    expect(labels[0]).toBe(t('log.state.succeeded'));
    expect(labels[1]).toBe(t('log.state.cancelled'));
  });

  it('يستدعي الرجوع عند الضغط على «رجوع»', async () => {
    const onBack = vi.fn();
    render(<RunLog onBack={onBack} />);

    await userEvent.click(screen.getByRole('button', { name: t('nav.back') }));
    expect(onBack).toHaveBeenCalledTimes(1);
  });
});
