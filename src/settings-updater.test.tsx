// @vitest-environment jsdom
/**
 * حالة «التحديثات غير مهيأة بعد» — وهي الحالة التي يشحن بها المنتج اليوم.
 *
 * تعيش في ملفٍّ مستقل لأنها تحتاج `UPDATER_CONFIGURED: false` في الاستهزاء،
 * وبقيةُ اختبارات الشاشة تحتاجها `true` كي تصل إلى مسارات الفحص. استهزاءُ
 * وحدةٍ واحد لا يحتمل القيمتين في ملفٍّ واحد.
 *
 * وما يجب أن يبقى صحيحًا هنا شيئان: أن الشاشة **لا تسأل الشبكة** حين تعرف
 * أن لا وجهة، وأنها لا تعرض ما لم يحدث — لا فشلًا، ولا ادّعاءَ أن النسخة
 * محدَّثة.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import SettingsScreen from './settings-screen';
import { t } from './i18n';
import { defaultSettings } from './settings';

vi.mock('./ipc', () => ({
  APP_VERSION: '0.2.0',
  UPDATER_CONFIGURED: false,
  checkForUpdate: vi.fn(),
  downloadAndInstallUpdate: vi.fn(),
  pickDirectory: vi.fn(),
  pickFile: vi.fn(),
}));

import { checkForUpdate } from './ipc';

afterEach(cleanup);

async function openAbout(autoUpdate: boolean) {
  render(
    <SettingsScreen
      settings={{ ...defaultSettings(), autoUpdate }}
      onSettingsChange={vi.fn()}
      storageAvailable
      onReplayOnboarding={vi.fn()}
    />,
  );
  await userEvent.click(screen.getByRole('tab', { name: t('settings.tab.about') }));
}

describe('التحديثات غير مهيأة', () => {
  it('تُعرض فورًا بلا سؤال الشبكة', async () => {
    await openAbout(false);

    expect(screen.getByText(t('settings.update.unconfigured'))).toBeDefined();
    expect(screen.getByText(t('settings.update.unconfigured.hint'))).toBeDefined();
    expect(checkForUpdate).not.toHaveBeenCalled();
  });

  it('لا تُعرض بوصفها فشلًا ولا بوصفها «محدَّث»', async () => {
    await openAbout(false);

    expect(screen.queryByText(t('settings.update.failed'))).toBeNull();
    expect(screen.queryByText(t('settings.update.failed.network'))).toBeNull();
    expect(screen.queryByText(t('settings.update.uptodate'))).toBeNull();
  });

  it('لا تعرض «إعادة المحاولة» — التكرار لا يغيّر شيئًا', async () => {
    await openAbout(false);

    expect(screen.queryByRole('button', { name: t('settings.update.retry') })).toBeNull();
    expect(screen.getByRole('button', { name: t('settings.update.check') })).toBeDefined();
  });

  it('التحديث التلقائي لا يستدعي فحصًا يُعرف فشله سلفًا', async () => {
    await openAbout(true);

    // مهلةٌ كافية لوقوع أي أثرٍ مؤجَّل لو كان سيقع.
    await waitFor(() => expect(screen.getByText(t('settings.update.unconfigured'))).toBeDefined());
    expect(checkForUpdate).not.toHaveBeenCalled();
  });

  it('والضغط على الفحص يدويًا يبقيها على حالها بلا نداء', async () => {
    await openAbout(false);

    await userEvent.click(screen.getByRole('button', { name: t('settings.update.check') }));
    expect(checkForUpdate).not.toHaveBeenCalled();
    expect(screen.getByText(t('settings.update.unconfigured'))).toBeDefined();
  });
});
