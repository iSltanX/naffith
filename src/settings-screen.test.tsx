// @vitest-environment jsdom
/**
 * الإعدادات — ما يجب أن يبقى صحيحًا مهما تغيّر شكل الشاشة.
 *
 * الاختبار يسأل عن النصوص عبر `t` لا عن حروفها: صياغةٌ تُحسَّن في `i18n.ts`
 * ليست كسرًا في هذه الشاشة، وإسقاطُ الاختبار عليها يعلّم الناسَ تجاهله.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import SettingsScreen from './settings-screen';
import { t } from './i18n';

// `globals: false` في إعداد Vite، فلا `afterEach` عامًّا يلتقطه التنظيف
// التلقائي في مكتبة الاختبار. بدون هذا السطر تتراكم الشاشات في المستند
// فتُطابق الاستعلامات عنصرين.
afterEach(cleanup);

describe('شاشة الإعدادات', () => {
  it('تعرض عنوانها وقسم الترحيب', () => {
    render(
      <SettingsScreen onBack={vi.fn()} onReplayOnboarding={vi.fn()} storageAvailable={true} />,
    );

    expect(screen.getByRole('heading', { name: t('settings.title') })).toBeDefined();
    expect(screen.getByRole('heading', { name: t('settings.onboarding.title') })).toBeDefined();
    expect(screen.getByText(t('settings.onboarding.body'))).toBeDefined();
  });

  it('تستدعي إعادة عرض الترحيب عند الضغط على زرّها', async () => {
    const onReplayOnboarding = vi.fn();
    render(
      <SettingsScreen
        onBack={vi.fn()}
        onReplayOnboarding={onReplayOnboarding}
        storageAvailable={true}
      />,
    );

    await userEvent.click(screen.getByRole('button', { name: t('settings.onboarding.replay') }));
    expect(onReplayOnboarding).toHaveBeenCalledTimes(1);
  });

  it('تستدعي الرجوع عند الضغط على «رجوع»', async () => {
    const onBack = vi.fn();
    render(
      <SettingsScreen onBack={onBack} onReplayOnboarding={vi.fn()} storageAvailable={true} />,
    );

    await userEvent.click(screen.getByRole('button', { name: t('nav.back') }));
    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it('لا تعرض تحذير التخزين حين يكون الحفظ متاحًا', () => {
    render(
      <SettingsScreen onBack={vi.fn()} onReplayOnboarding={vi.fn()} storageAvailable={true} />,
    );

    expect(screen.queryByText(t('settings.storage.unavailable'))).toBeNull();
  });

  it('تعرض تحذير التخزين حين يتعذّر الحفظ', () => {
    render(
      <SettingsScreen onBack={vi.fn()} onReplayOnboarding={vi.fn()} storageAvailable={false} />,
    );

    expect(screen.getByText(t('settings.storage.unavailable'))).toBeDefined();
  });

  it('يبقى زرّ إعادة العرض عاملًا حين يتعذّر الحفظ', async () => {
    // تعذّر الحفظ يمنع التذكّر لا الفعل: من ضغط الزرّ يرى الترحيب في هذه
    // الجلسة. تعطيلُ الزرّ كان سيعاقب المستخدم على عطلٍ ليس من صنعه.
    const onReplayOnboarding = vi.fn();
    render(
      <SettingsScreen
        onBack={vi.fn()}
        onReplayOnboarding={onReplayOnboarding}
        storageAvailable={false}
      />,
    );

    const button = screen.getByRole('button', { name: t('settings.onboarding.replay') });
    expect(button.hasAttribute('disabled')).toBe(false);
    await userEvent.click(button);
    expect(onReplayOnboarding).toHaveBeenCalledTimes(1);
  });
});
