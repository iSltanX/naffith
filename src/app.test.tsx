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
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './app';
import { AR } from './i18n';
import type { OperationSummary } from './ipc';
import { listOperations, onRunFinished, recentRuns } from './ipc';
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
  const root = container.querySelector('.ops, .screen, .page__body');
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
    expect(container.querySelector('.page__body')).toBeTruthy();
    expect(screenRoot(container).closest('.page')).toBeTruthy();
  });
});
