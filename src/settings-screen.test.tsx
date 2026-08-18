// @vitest-environment jsdom
/**
 * الإعدادات — ما يجب أن يبقى صحيحًا مهما تغيّر شكل الشاشة.
 *
 * الاختبار يسأل عن النصوص عبر `t` لا عن حروفها: صياغةٌ تُحسَّن في `i18n.ts`
 * ليست كسرًا في هذه الشاشة، وإسقاطُ الاختبار عليها يعلّم الناسَ تجاهله.
 *
 * وحالات التحديث تُختبر باستهزاء `ipc` وحده: لا شبكة في اختبار وحدة، والفرق
 * بين «تعذّر التحقّق» لعدم ضبط الوجهة وبينه لانقطاع الشبكة فرقٌ في النصّ
 * المعروض — وهو بالضبط ما يجب أن يبقى صحيحًا.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import SettingsScreen from './settings-screen';
import { t } from './i18n';
import { defaultSettings, type Settings } from './settings';

vi.mock('./ipc', () => ({
  APP_VERSION: '9.9.9',
  UPDATER_CONFIGURED: true,
  checkForUpdate: vi.fn(),
  downloadAndInstallUpdate: vi.fn(),
  pickDirectory: vi.fn(),
  pickFile: vi.fn(),
}));

import { checkForUpdate, downloadAndInstallUpdate, pickFile } from './ipc';

afterEach(cleanup);

function view(settingsOver: Partial<Settings> = {}, over: Record<string, unknown> = {}) {
  const settings = { ...defaultSettings(), ...settingsOver };
  const props = {
    settings,
    onSettingsChange: vi.fn(),
    storageAvailable: true,
    onReplayOnboarding: vi.fn(),
    ...over,
  } as Parameters<typeof SettingsScreen>[0] & {
    onSettingsChange: ReturnType<typeof vi.fn>;
    onReplayOnboarding: ReturnType<typeof vi.fn>;
  };
  return { ...render(<SettingsScreen {...props} />), props };
}

/** ينتقل إلى لسانٍ بالاسم المعروض. */
async function openTab(key: string) {
  await userEvent.click(screen.getByRole('tab', { name: t(`settings.tab.${key}`) }));
}

function toolSection(title: string): HTMLElement {
  const heading = screen.getByRole('heading', { name: title });
  const section = heading.closest('section');
  if (!section) throw new Error(`لم يُعثر على قسمٍ للعنوان: ${title}`);
  return section;
}

describe('شاشة الإعدادات — الألسنة', () => {
  it('تعرض الألسنة الأربعة عموديًا، وعام هو المفتوح', () => {
    const { container } = view();
    const list = container.querySelector('[role="tablist"]');
    expect(list?.getAttribute('aria-orientation')).toBe('vertical');

    for (const key of ['general', 'appearance', 'developer', 'about']) {
      expect(screen.getByRole('tab', { name: t(`settings.tab.${key}`) })).toBeDefined();
    }
    expect(
      screen.getByRole('tab', { name: t('settings.tab.general') }).getAttribute('aria-selected'),
    ).toBe('true');
    expect(screen.getByRole('heading', { name: t('settings.general.title') })).toBeDefined();
  });

  it('يبدّل اللسان العنوانَ والمحتوى معًا', async () => {
    view();

    await openTab('appearance');
    expect(screen.getByRole('heading', { name: t('settings.appearance.title') })).toBeDefined();
    expect(screen.getByText(t('settings.theme.title'))).toBeDefined();

    await openTab('developer');
    expect(screen.getByRole('heading', { name: t('settings.developer.title') })).toBeDefined();
  });

  it('لا تكرر مسار رجوع داخل الشاشة مع وجود التنقّل الثابت', () => {
    view();

    expect(screen.queryByRole('button', { name: t('nav.back') })).toBeNull();
  });
});

/**
 * ترتيب DOM هو ما يحدّد اليمين واليسار في مستندٍ `dir="rtl"`.
 *
 * الخطأ الذي وقع فعلًا: كُتبت الصفوف بترتيب Figma البصري من اليسار إلى
 * اليمين، فانعكس كل صفٍّ يستعمل `space-between` — صار المفتاح يمينًا والنصّ
 * يسارًا، والعنوان يسارًا والألسنة يمينًا. الحارس يقارن مواضع DOM لا البكسل،
 * لأن jsdom لا يحسب تخطيطًا: أوّل ابنٍ في RTL هو الأيمن، فهذا يكفي.
 */
describe('شاشة الإعدادات — اتجاه RTL', () => {
  function orderWithin(container: Element, a: Element, b: Element): number {
    const kids = [...container.children];
    return kids.indexOf(a) - kids.indexOf(b);
  }

  it('الأقسام تسبق المحتوى فتقع يمينًا والمحتوى يسارًا', () => {
    const { container } = view();
    const body = container.querySelector('.settings-screen__body');
    const sections = container.querySelector('.settings-screen__sections');
    const panel = container.querySelector('.settings-screen__panel');
    expect(body && sections && panel).toBeTruthy();
    expect(orderWithin(body as Element, sections as Element, panel as Element)).toBeLessThan(0);
  });

  it('العنوان يبقى فوق الجسم لا بجانب الأقسام', () => {
    const { container } = view();
    const screen_ = container.querySelector('.settings-screen');
    const head = container.querySelector('.settings-screen__head');
    const body = container.querySelector('.settings-screen__body');
    expect(screen_ && head && body).toBeTruthy();
    expect(orderWithin(screen_ as Element, head as Element, body as Element)).toBeLessThan(0);
    // ولا شريط أقسامٍ داخل الترويسة ينازع العنوان.
    expect((head as Element).querySelector('[role="tablist"]')).toBeNull();
  });

  it('نصّ الصفّ يسبق مفتاحه فيقع النصّ يمينًا والمفتاح يسارًا', () => {
    const { container } = view();
    const row = container.querySelector('.settings-row');
    const labels = row?.querySelector('.settings-row__labels');
    const sw = row?.querySelector('.switch');
    expect(row && labels && sw).toBeTruthy();
    expect(orderWithin(row as Element, labels as Element, sw as Element)).toBeLessThan(0);
  });

  it('المسار يسبق أزراره في بطاقة المسار', () => {
    const { container } = view({ defaultWorkingPath: '/Users/dev/Documents' });
    const row = container.querySelector('.settings-screen__card-row');
    const path = row?.querySelector('.settings-screen__path');
    const actions = row?.querySelector('.settings-screen__card-actions');
    expect(row && path && actions).toBeTruthy();
    expect(orderWithin(row as Element, path as Element, actions as Element)).toBeLessThan(0);
  });

  it('عنوان حجم الأيقونات يسبق قائمته', async () => {
    const { container } = view();
    await openTab('appearance');
    const rows = [...container.querySelectorAll('.settings-screen__card-row')];
    const row = rows.find((r) => r.querySelector('.settings-select'));
    const title = row?.querySelector('.settings-screen__card-title');
    const select = row?.querySelector('.settings-select');
    expect(row && title && select).toBeTruthy();
    expect(orderWithin(row as Element, title as Element, select as Element)).toBeLessThan(0);
  });
});

describe('شاشة الإعدادات — عام', () => {
  it('تعرض التفضيلات الثلاثة بحالتها المحفوظة', () => {
    view({ notificationSound: true, confirmBeforeExecute: false });

    const sound = screen.getByRole('switch', { name: t('settings.sound.title') });
    const confirm = screen.getByRole('switch', { name: t('settings.confirm.title') });
    expect(sound.getAttribute('aria-checked')).toBe('true');
    expect(confirm.getAttribute('aria-checked')).toBe('false');
  });

  it('قلبُ مفتاحٍ يبلّغ الإعداد الجديد ولا يكتب بنفسه', async () => {
    const { props } = view({ notificationSound: true });

    await userEvent.click(screen.getByRole('switch', { name: t('settings.sound.title') }));
    expect(props.onSettingsChange).toHaveBeenCalledTimes(1);
    expect(props.onSettingsChange.mock.calls[0]?.[0].notificationSound).toBe(false);
  });

  it('مفتاح شاشة الترحيب يعيد عرضها حين يُفتح', async () => {
    // مُتمّ الترحيب ⇒ المفتاح مطفأ؛ وفتحه يعني «اعرضها لي».
    const { props } = view({ onboardingCompletedAt: '2026-01-01T00:00:00.000Z' });

    const welcome = screen.getByRole('switch', { name: t('settings.welcome.title') });
    expect(welcome.getAttribute('aria-checked')).toBe('false');
    await userEvent.click(welcome);
    expect(props.onReplayOnboarding).toHaveBeenCalledTimes(1);
  });
});

describe('شاشة الإعدادات — المظهر', () => {
  it('تعلّم السمة المختارة وتبلّغ تغييرها', async () => {
    const { props } = view({ theme: 'system' });
    await openTab('appearance');

    const auto = screen.getByRole('radio', { name: t('settings.theme.system') });
    expect(auto.getAttribute('aria-checked')).toBe('true');

    await userEvent.click(screen.getByRole('radio', { name: t('settings.theme.dark') }));
    expect(props.onSettingsChange.mock.calls[0]?.[0].theme).toBe('dark');
  });

  it('تعرض حجم الأيقونات المحفوظ', async () => {
    view({ sidebarIconSize: 'large' });
    await openTab('appearance');

    const select = screen.getByRole('combobox', { name: t('settings.iconsize.title') });
    expect((select as HTMLSelectElement).value).toBe('large');
  });
});

describe('شاشة الإعدادات — أدوات المطوّرين', () => {
  it('يعرض «لم يُحدَّد بعد» ولا زرّ مسح حين لا يوجد مسارٌ محفوظ', async () => {
    view({ nodePath: null });
    await openTab('developer');
    const section = within(toolSection(t('settings.node.title')));

    expect(section.getByText(t('settings.node.unset'))).toBeDefined();
    expect(section.getByRole('button', { name: t('settings.node.choose') })).toBeDefined();
    expect(section.queryByRole('button', { name: t('settings.node.clear') })).toBeNull();
  });

  it('يعرض المسار المحفوظ وزرّ التغيير والمسح حين يوجد', async () => {
    view({ cargoPath: '/Users/dev/.cargo/bin/cargo' });
    await openTab('developer');
    const section = within(toolSection(t('settings.cargo.title')));

    expect(section.getByText('/Users/dev/.cargo/bin/cargo')).toBeDefined();
    expect(section.getByRole('button', { name: t('settings.cargo.change') })).toBeDefined();
    expect(section.getByRole('button', { name: t('settings.cargo.clear') })).toBeDefined();
  });

  it('زرّ المسح يمسح القيمة دون أن يفتح حوار اختيار', async () => {
    const { props } = view({ nodePath: '/usr/local/bin/node' });
    await openTab('developer');
    const section = within(toolSection(t('settings.node.title')));

    await userEvent.click(section.getByRole('button', { name: t('settings.node.clear') }));
    expect(props.onSettingsChange.mock.calls[0]?.[0].nodePath).toBeNull();
    expect(pickFile).not.toHaveBeenCalled();
  });
});

describe('شاشة الإعدادات — حول', () => {
  it('تعرض الإصدار الحقيقي والوصف والاعتماد', async () => {
    vi.mocked(checkForUpdate).mockResolvedValue(null);
    view({ autoUpdate: false });
    await openTab('about');

    expect(screen.getByText(/9\.9\.9/)).toBeDefined();
    expect(screen.getByText(t('settings.about.description'))).toBeDefined();
    expect(screen.getByText(t('settings.about.credit'))).toBeDefined();
  });

  it('«لا توجد تحديثات» حين يجيب الفحص بلا شيء', async () => {
    vi.mocked(checkForUpdate).mockResolvedValue(null);
    view({ autoUpdate: false });
    await openTab('about');

    await userEvent.click(screen.getByRole('button', { name: t('settings.update.check') }));
    await waitFor(() =>
      expect(screen.getByText(t('settings.update.uptodate'))).toBeDefined(),
    );
  });

  it('يعرض الإصدار المتوفّر وزرّ التحميل حين يوجد تحديث', async () => {
    vi.mocked(checkForUpdate).mockResolvedValue({
      rid: 7,
      currentVersion: '9.9.9',
      version: '10.0.0',
    });
    view({ autoUpdate: false });
    await openTab('about');

    await userEvent.click(screen.getByRole('button', { name: t('settings.update.check') }));
    const download = await screen.findByRole('button', { name: t('settings.update.download') });
    expect(screen.getByText(/10\.0\.0/)).toBeDefined();

    await userEvent.click(download);
    expect(downloadAndInstallUpdate).toHaveBeenCalledWith(7);
  });

  /**
   * H-7 regression: a successful install must reach a terminal state, not
   * stay on "جارٍ تنزيل التحديث…" (downloading…) forever. Before the fix,
   * `install()` had no state transition after `downloadAndInstallUpdate`
   * resolved, so the screen was stuck on the in-progress copy even though
   * nothing was still happening.
   */
  it('يعرض حالة «ثُبِّت التحديث» بعد نجاح التثبيت، لا «جارٍ التنزيل» إلى الأبد', async () => {
    vi.mocked(checkForUpdate).mockResolvedValue({
      rid: 9,
      currentVersion: '9.9.9',
      version: '10.0.0',
    });
    vi.mocked(downloadAndInstallUpdate).mockResolvedValue(undefined);
    view({ autoUpdate: false });
    await openTab('about');

    await userEvent.click(screen.getByRole('button', { name: t('settings.update.check') }));
    const download = await screen.findByRole('button', { name: t('settings.update.download') });
    await userEvent.click(download);

    await waitFor(() => expect(screen.getByText(t('settings.update.installed'))).toBeDefined());
    expect(screen.getByText(t('settings.update.installed.hint'))).toBeDefined();
    expect(screen.queryByText(t('settings.update.installing'))).toBeNull();

    // وزرّ الفحص لا يعرض «سؤالًا جديدًا»: التطبيق ما زال يشغّل النسخة القديمة.
    expect(screen.getByRole('button', { name: t('settings.update.check') }).hasAttribute('disabled')).toBe(
      true,
    );
  });

  /**
   * الحالة التي يشحن بها المنتج اليوم: لا وجهة تحديث مضبوطة.
   *
   * الرسالة يجب أن تقول ذلك لا أن تنصح بفحص الاتصال، ويجب ألّا تدّعي الشاشة
   * أن النسخة محدَّثة — «لا أعرف» أصدق من «أنت على أحدث إصدار» بلا سؤال.
   */
  it('فشلٌ حقيقي يعرض نبرة الخطر و«إعادة المحاولة»', async () => {
    vi.mocked(checkForUpdate).mockRejectedValue(new Error('network unreachable'));
    view({ autoUpdate: false });
    await openTab('about');

    await userEvent.click(screen.getByRole('button', { name: t('settings.update.check') }));
    await waitFor(() =>
      expect(screen.getByText(t('settings.update.failed.network'))).toBeDefined(),
    );
    expect(screen.getByText(t('settings.update.failed'))).toBeDefined();
    expect(screen.getByRole('button', { name: t('settings.update.retry') })).toBeDefined();
  });

  it('يفحص تلقائيًا حين يكون التحديث التلقائي مفعّلًا', async () => {
    vi.mocked(checkForUpdate).mockResolvedValue(null);
    view({ autoUpdate: true });
    await openTab('about');

    await waitFor(() => expect(checkForUpdate).toHaveBeenCalled());
  });
});

describe('شاشة الإعدادات — تحذير التخزين', () => {
  it('لا يظهر حين يكون الحفظ متاحًا', () => {
    view();

    expect(screen.queryByText(t('settings.storage.unavailable.title'))).toBeNull();
  });

  it('يظهر بعنوان ومتن مستقلين حين يتعذّر الحفظ', () => {
    view({}, { storageAvailable: false });

    const notice = screen.getByRole('status');
    const title = within(notice).getByText(t('settings.storage.unavailable.title'));
    expect(title.tagName).toBe('STRONG');
    expect(within(notice).getByText(t('settings.storage.unavailable.body'))).toBeDefined();
    expect(notice.querySelector('svg')).toBeNull();
  });
});
