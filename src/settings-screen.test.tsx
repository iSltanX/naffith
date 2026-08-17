// @vitest-environment jsdom
/**
 * الإعدادات — ما يجب أن يبقى صحيحًا مهما تغيّر شكل الشاشة.
 *
 * الاختبار يسأل عن النصوص عبر `t` لا عن حروفها: صياغةٌ تُحسَّن في `i18n.ts`
 * ليست كسرًا في هذه الشاشة، وإسقاطُ الاختبار عليها يعلّم الناسَ تجاهله.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import SettingsScreen from './settings-screen';
import { t } from './i18n';

// `globals: false` في إعداد Vite، فلا `afterEach` عامًّا يلتقطه التنظيف
// التلقائي في مكتبة الاختبار. بدون هذا السطر تتراكم الشاشات في المستند
// فتُطابق الاستعلامات عنصرين.
afterEach(cleanup);

function view(over: Partial<Parameters<typeof SettingsScreen>[0]> = {}) {
  const props = {
    onBack: vi.fn(),
    onReplayOnboarding: vi.fn(),
    storageAvailable: true,
    nodePath: null,
    onNodePathChange: vi.fn(),
    cargoPath: null,
    onCargoPathChange: vi.fn(),
    ...over,
  };
  return { ...render(<SettingsScreen {...props} />), props };
}

describe('شاشة الإعدادات', () => {
  it('تعرض عنوانها وقسم الترحيب', () => {
    view();

    expect(screen.getByRole('heading', { name: t('settings.title') })).toBeDefined();
    expect(screen.getByText(t('settings.subtitle'))).toBeDefined();
    expect(screen.getByRole('heading', { name: t('settings.onboarding.title') })).toBeDefined();
    expect(screen.getByText(t('settings.onboarding.body'))).toBeDefined();
  });

  it('تستدعي إعادة عرض الترحيب عند الضغط على زرّها', async () => {
    const { props } = view();

    await userEvent.click(screen.getByRole('button', { name: t('settings.onboarding.replay') }));
    expect(props.onReplayOnboarding).toHaveBeenCalledTimes(1);
  });

  it('لا تكرر مسار رجوع داخل الشاشة مع وجود التنقّل الثابت', () => {
    view();

    expect(screen.queryByRole('button', { name: t('nav.back') })).toBeNull();
  });

  it('لا تعرض تحذير التخزين حين يكون الحفظ متاحًا', () => {
    view();

    expect(screen.queryByText(t('settings.storage.unavailable.title'))).toBeNull();
    expect(screen.queryByText(t('settings.storage.unavailable.body'))).toBeNull();
  });

  it('تعرض تحذير التخزين بعنوان ومتن مستقلين حين يتعذّر الحفظ', () => {
    view({ storageAvailable: false });

    expect(t('settings.storage.unavailable.title')).toBe('تعذّر حفظ الإعدادات على هذا الجهاز');
    expect(t('settings.storage.unavailable.body')).toBe('لن تُحفظ هذه الخيارات بين التشغيلات.');

    const notice = screen.getByRole('status');
    const title = within(notice).getByText(t('settings.storage.unavailable.title'));
    expect(title.tagName).toBe('STRONG');
    expect(within(notice).getByText(t('settings.storage.unavailable.body'))).toBeDefined();
    expect(notice.querySelector('svg')).toBeNull();
  });

  it('يبقى زرّ إعادة العرض عاملًا حين يتعذّر الحفظ', async () => {
    // تعذّر الحفظ يمنع التذكّر لا الفعل: من ضغط الزرّ يرى الترحيب في هذه
    // الجلسة. تعطيلُ الزرّ كان سيعاقب المستخدم على عطلٍ ليس من صنعه.
    const { props } = view({ storageAvailable: false });

    const button = screen.getByRole('button', { name: t('settings.onboarding.replay') });
    expect(button.classList.contains('btn--primary')).toBe(true);
    expect(button.hasAttribute('disabled')).toBe(false);
    expect(button.querySelector('svg')).toBeNull();
    await userEvent.click(button);
    expect(props.onReplayOnboarding).toHaveBeenCalledTimes(1);
  });
});

// القسمان يعرضان معًا دومًا (Node.js وCargo)، ونصّ «لم يُحدَّد بعد» واحدٌ
// حرفيًا بينهما عمدًا — فتفريقه بصياغتين كان سيوهم أن أحد المسارين أهمّ من
// الآخر. لذا تُحصر كل استعلامات هذا القسم داخل بطاقته عبر `within`، لا في
// المستند كله؛ وإلا طابق `getByText` القيمة الافتراضية للقسم الآخر أيضًا.
function toolSection(title: string): HTMLElement {
  const heading = screen.getByRole('heading', { name: title });
  const section = heading.closest('section');
  if (!section) throw new Error(`لم يُعثر على قسمٍ للعنوان: ${title}`);
  return section;
}

describe('قسم مسار Node.js', () => {
  it('يعرض «لم يُحدَّد بعد» ولا زرّ مسح حين لا يوجد مسارٌ محفوظ', () => {
    view({ nodePath: null });
    const section = within(toolSection(t('settings.node.title')));

    expect(section.getByText(t('settings.node.unset'))).toBeDefined();
    expect(
      section.getByRole('button', { name: t('settings.node.choose') }),
    ).toBeDefined();
    expect(section.queryByRole('button', { name: t('settings.node.clear') })).toBeNull();
  });

  it('يعرض المسار المحفوظ وزرّ التغيير والمسح حين يوجد', () => {
    view({ nodePath: '/usr/local/bin/node' });
    const section = within(toolSection(t('settings.node.title')));

    expect(section.getByText('/usr/local/bin/node')).toBeDefined();
    expect(section.getByRole('button', { name: t('settings.node.change') })).toBeDefined();
    expect(section.getByRole('button', { name: t('settings.node.clear') })).toBeDefined();
    expect(section.queryByText(t('settings.node.unset'))).toBeNull();
  });

  it('زرّ المسح يمسح القيمة دون أن يفتح حوار اختيار', async () => {
    const { props } = view({ nodePath: '/usr/local/bin/node' });
    const section = within(toolSection(t('settings.node.title')));

    await userEvent.click(section.getByRole('button', { name: t('settings.node.clear') }));
    expect(props.onNodePathChange).toHaveBeenCalledExactlyOnceWith(null);
  });
});

describe('قسم مسار Cargo', () => {
  it('يعرض «لم يُحدَّد بعد» ولا زرّ مسح حين لا يوجد مسارٌ محفوظ', () => {
    view({ cargoPath: null });
    const section = within(toolSection(t('settings.cargo.title')));

    expect(section.getByText(t('settings.cargo.unset'))).toBeDefined();
    expect(
      section.getByRole('button', { name: t('settings.cargo.choose') }),
    ).toBeDefined();
    expect(section.queryByRole('button', { name: t('settings.cargo.clear') })).toBeNull();
  });

  it('يعرض المسار المحفوظ وزرّ التغيير والمسح حين يوجد', () => {
    view({ cargoPath: '/Users/dev/.cargo/bin/cargo' });
    const section = within(toolSection(t('settings.cargo.title')));

    expect(section.getByText('/Users/dev/.cargo/bin/cargo')).toBeDefined();
    expect(section.getByRole('button', { name: t('settings.cargo.change') })).toBeDefined();
    expect(section.getByRole('button', { name: t('settings.cargo.clear') })).toBeDefined();
    expect(section.queryByText(t('settings.cargo.unset'))).toBeNull();
  });

  it('زرّ المسح يمسح القيمة دون أن يفتح حوار اختيار', async () => {
    const { props } = view({ cargoPath: '/Users/dev/.cargo/bin/cargo' });
    const section = within(toolSection(t('settings.cargo.title')));

    await userEvent.click(section.getByRole('button', { name: t('settings.cargo.clear') }));
    expect(props.onCargoPathChange).toHaveBeenCalledExactlyOnceWith(null);
  });
});
