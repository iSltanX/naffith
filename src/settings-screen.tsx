/**
 * الإعدادات — أربعة أقسام في شاشة واحدة.
 *
 * ## لماذا ألسنة لا شاشات
 *
 * الأقسام الأربعة (عام، المظهر، أدوات المطوّرين، حول) تشترك في ترويسةٍ واحدة
 * وتُقرأ في جلسةٍ واحدة: من يفتح الإعدادات ليضبط السمة قد يضبط بعدها مسار
 * Node. جعلُها أربع وجهات في `nav.ts` كان يعني أربعة مسارات رجوع وأربعة
 * عناوين في السجل من أجل تنقّلٍ داخل شاشةٍ واحدة. اللسان حالةٌ محلّية هنا،
 * والشاشة كلها وجهةٌ واحدة كما كانت.
 *
 * ## لماذا `settings` كاملةً لا سبعة خصائص
 *
 * الشاشة صارت تقرأ تسعة حقول وتكتب ستّة. تمريرُها حقلًا حقلًا كان يعني ثمانية
 * عشر خاصيّة عبر حدّ ملفّ، وكل حقلٍ جديد يمسّ ثلاثة مواضع. الشاشة تعرض ولا
 * تحفظ: `onSettingsChange` هي الطريق الوحيد إلى المخزن، ومن يملكه هو `app.tsx`
 * كما كان — وهذا هو نفس السبب الذي جعل `storageAvailable` مُمرَّرة لا مقروءة
 * هنا.
 *
 * ## لماذا حالة التحديث محلّية
 *
 * «جارٍ الفحص» و«تعذّر التحقّق» ليستا إعدادًا يُحفظ بل واقعةَ جلسة: من أغلق
 * التطبيق وفتحه لا يجب أن يجد رسالة فشلٍ من الأمس. المحفوظ هو التفضيل وحده
 * (`autoUpdate`)، والنتيجة تُعاد بالسؤال لا بالقراءة.
 */
import { useCallback, useEffect, useId, useRef, useState } from 'react';
import { t, tFormat } from './i18n';
import {
  APP_VERSION,
  checkForUpdate,
  downloadAndInstallUpdate,
  pickDirectory,
  pickFile,
  UPDATER_CONFIGURED,
  type UpdateInfo,
} from './ipc';
import {
  shouldShowOnboarding,
  withAutoUpdate,
  withCargoPath,
  withConfirmBeforeExecute,
  withDefaultWorkingPath,
  withNodePath,
  withNotificationSound,
  withOnboardingCompleted,
  withSidebarIconSize,
  withTheme,
  type Settings,
  type SidebarIconSize,
  type ThemePreference,
} from './settings';
import './utility-screen.css';
import './settings-screen.css';

export type SettingsTab = 'general' | 'appearance' | 'developer' | 'about';

const TABS: readonly SettingsTab[] = ['general', 'appearance', 'developer', 'about'];

/** عنوان القسم المعروض في الترويسة — واحد لكل لسان. */
const TITLE_KEY: Record<SettingsTab, string> = {
  general: 'settings.general.title',
  appearance: 'settings.appearance.title',
  developer: 'settings.developer.title',
  about: 'settings.about.title',
};

const THEME_OPTIONS: readonly ThemePreference[] = ['light', 'dark', 'system'];
const ICON_SIZES: readonly SidebarIconSize[] = ['small', 'medium', 'large'];

/**
 * حالة سؤال التحديث.
 *
 * `unconfigured` حالةٌ قائمة بذاتها لا نوعٌ من الخطأ: لا شيء انكسر، ولم
 * تُضبط وجهة التحديث في هذا البناء بعد. فصلُها يمنع عرض نبرة الخطر وزرّ
 * «إعادة المحاولة» على شيءٍ لا تُصلحه إعادةُ محاولة.
 */
type UpdateState =
  | { k: 'idle' }
  | { k: 'unconfigured' }
  | { k: 'checking' }
  | { k: 'uptodate' }
  | { k: 'available'; info: UpdateInfo }
  | { k: 'installing' }
  // ‏H-7: الحالة النهائية بعد نجاح `downloadAndInstallUpdate` فعلًا — بدونها
  // كانت الشاشة تبقى على `installing` («جارٍ تنزيل التحديث…») إلى الأبد،
  // فلا شيء يقول للمستخدم إن التحديث انتهى أو أنه بحاجة لإعادة التشغيل.
  | { k: 'installed' }
  | { k: 'error' };

/** مفتاح تبديل — نفس هندسة `Switch` في التصميم (٤٤×٢٤، المقبض ٢٠). */
function Switch({
  on,
  onChange,
  labelledBy,
  describedBy,
}: {
  on: boolean;
  onChange: (next: boolean) => void;
  labelledBy: string;
  describedBy?: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-labelledby={labelledBy}
      aria-describedby={describedBy}
      className={`switch${on ? ' switch--on' : ''}`}
      onClick={() => onChange(!on)}
    >
      <span className="switch__knob" />
    </button>
  );
}

/** صفٌّ في بطاقة التفضيلات: عنوان وشرح على اليمين، ومفتاح على اليسار. */
function ToggleRow({
  uid,
  name,
  on,
  onChange,
}: {
  uid: string;
  name: string;
  on: boolean;
  onChange: (next: boolean) => void;
}) {
  const titleId = `${uid}-${name}-title`;
  const bodyId = `${uid}-${name}-body`;
  return (
    // النصّ أوّلًا فيقع يمينًا، والمفتاح آخرًا فيقع يسارًا — كما في التصميم.
    <div className="settings-row">
      <div className="settings-row__labels">
        <span id={titleId} className="settings-row__title">
          {t(`settings.${name}.title`)}
        </span>
        <span id={bodyId} className="settings-row__body">
          {t(`settings.${name}.body`)}
        </span>
      </div>
      <Switch on={on} onChange={onChange} labelledBy={titleId} describedBy={bodyId} />
    </div>
  );
}

/**
 * بطاقة مسار: عنوان، والمسار المحفوظ أو «لم يُحدَّد بعد»، وأزرار.
 *
 * تخدم مسار العمل الافتراضي (مجلد) ومساري Node وCargo (ملفّان)، والفرق بينها
 * `pick` وحدها — فحوار المجلد وحوار الملف كلاهما يعيد نصًّا غير موثوق يفحصه
 * النواة لاحقًا، ولا شيء في هذه الشاشة يعتمد على أيّهما كان.
 */
function PathCard({
  uid,
  name,
  keyPrefix,
  labelKey,
  path,
  pick,
  onChange,
}: {
  uid: string;
  name: string;
  keyPrefix: string;
  labelKey: string;
  path: string | null;
  pick: () => Promise<string | null>;
  onChange: (path: string | null) => void;
}) {
  const titleId = `${uid}-${name}`;
  const browse = async () => {
    const chosen = await pick();
    if (chosen) onChange(chosen);
  };
  return (
    <section className="settings-screen__card" aria-labelledby={titleId}>
      <h3 id={titleId} className="settings-screen__card-title">
        {t(`${keyPrefix}.title`)}
      </h3>
      {/* المسار أوّلًا فيقع يمينًا، والأزرار آخرًا فتقع يسارًا. */}
      <div className="settings-screen__card-row">
        <p className="settings-screen__path">
          <span className="settings-screen__path-label">{t(labelKey)}</span>{' '}
          <bdi className="settings-screen__path-value" dir="ltr">
            {path ?? t(`${keyPrefix}.unset`)}
          </bdi>
        </p>
        <div className="settings-screen__card-actions">
          <button type="button" className="btn btn--outline" onClick={browse}>
            {t(path ? `${keyPrefix}.change` : `${keyPrefix}.choose`)}
          </button>
          {path && (
            <button type="button" className="btn btn--quiet" onClick={() => onChange(null)}>
              {t(`${keyPrefix}.clear`)}
            </button>
          )}
        </div>
      </div>
    </section>
  );
}

export default function SettingsScreen(props: {
  settings: Settings;
  onSettingsChange: (next: Settings) => void;
  storageAvailable: boolean;
  onReplayOnboarding: () => void;
}): JSX.Element {
  const { settings, onSettingsChange, storageAvailable, onReplayOnboarding } = props;

  const uid = useId();
  const headingId = `${uid}-heading`;
  const heading = useRef<HTMLHeadingElement>(null);
  const [tab, setTab] = useState<SettingsTab>('general');
  const [update, setUpdate] = useState<UpdateState>(
    UPDATER_CONFIGURED ? { k: 'idle' } : { k: 'unconfigured' },
  );

  // تبديل الشاشة في تطبيق صفحةٍ واحدة لا يحرّك التركيز من تلقائه.
  useEffect(() => {
    heading.current?.focus();
  }, []);

  const runCheck = useCallback(async () => {
    // بناءٌ بلا وجهة لا يسأل الشبكة: الجواب معروفٌ قبل السؤال.
    if (!UPDATER_CONFIGURED) {
      setUpdate({ k: 'unconfigured' });
      return;
    }
    setUpdate({ k: 'checking' });
    try {
      const info = await checkForUpdate();
      setUpdate(info ? { k: 'available', info } : { k: 'uptodate' });
    } catch {
      setUpdate({ k: 'error' });
    }
  }, []);

  const install = useCallback(async (info: UpdateInfo) => {
    setUpdate({ k: 'installing' });
    try {
      await downloadAndInstallUpdate(info.rid);
      // نجح التثبيت فعليًا — حالةٌ نهائية تقوله، لا بقاءٌ على «جارٍ التنزيل».
      setUpdate({ k: 'installed' });
    } catch {
      setUpdate({ k: 'error' });
    }
  }, []);

  // «التحديث التلقائي» تعني السؤال بلا أن يُطلب — وأوّل فرصةٍ لذلك هي فتح
  // القسم الذي يعرض الجواب. لا تُسأل مرّتين: `idle` وحدها تسمح.
  useEffect(() => {
    if (tab === 'about' && settings.autoUpdate && update.k === 'idle') void runCheck();
  }, [tab, settings.autoUpdate, update.k, runCheck]);

  const showWelcome = shouldShowOnboarding(settings);

  return (
    <section className="screen settings-screen" aria-labelledby={headingId}>
      {/* العنوان وحده فوق المحتوى: شريطُ أقسامٍ أفقيّ بجانبه كان ينازعه
          الصدارة في نفس السطر. */}
      <header className="settings-screen__head">
        {/* ‏h2 لا h1: هويّة المنتج تحمل h1 في ترويسة التطبيق الثابتة. */}
        <h2 id={headingId} className="t-page-title settings-screen__title" tabIndex={-1} ref={heading}>
          {t(TITLE_KEY[tab])}
        </h2>
      </header>

      {!storageAvailable && (
        <div className="notice notice--warning settings-screen__storage-notice" role="status">
          <div className="settings-screen__notice-copy">
            <strong className="settings-screen__notice-title">
              {t('settings.storage.unavailable.title')}
            </strong>
            <p>{t('settings.storage.unavailable.body')}</p>
          </div>
        </div>
      )}

      {/* الأقسام أوّلًا فتقع يمينًا، والمحتوى بعدها فيقع يسارًا. وهي تنبع من
          داخل مساحة الإعدادات لا من حافة النافذة، فتبقى مفصولةً بصريًا عن
          تنقّل التطبيق الرئيسي الذي يسكن الحافة. */}
      <div className="settings-screen__body">
        <div
          className="settings-screen__sections"
          role="tablist"
          aria-orientation="vertical"
          aria-label={t('settings.tabs.label')}
        >
          {TABS.map((id) => (
            <button
              key={id}
              type="button"
              role="tab"
              id={`${uid}-tab-${id}`}
              aria-selected={tab === id}
              aria-controls={`${uid}-panel-${id}`}
              className={`settings-screen__section${tab === id ? ' is-current' : ''}`}
              onClick={() => setTab(id)}
            >
              {t(`settings.tab.${id}`)}
            </button>
          ))}
        </div>

        <div
          className="settings-screen__panel"
          role="tabpanel"
          id={`${uid}-panel-${tab}`}
          aria-labelledby={`${uid}-tab-${tab}`}
        >
        {tab === 'general' && (
          <>
            <div className="settings-screen__card settings-screen__card--rows">
              <ToggleRow
                uid={uid}
                name="welcome"
                on={showWelcome}
                onChange={(next) => {
                  if (next) onReplayOnboarding();
                  else onSettingsChange(withOnboardingCompleted(settings, new Date()));
                }}
              />
              <ToggleRow
                uid={uid}
                name="sound"
                on={settings.notificationSound}
                onChange={(next) => onSettingsChange(withNotificationSound(settings, next))}
              />
              <ToggleRow
                uid={uid}
                name="confirm"
                on={settings.confirmBeforeExecute}
                onChange={(next) => onSettingsChange(withConfirmBeforeExecute(settings, next))}
              />
            </div>
            <PathCard
              uid={uid}
              name="workpath"
              keyPrefix="settings.workpath"
              labelKey="settings.workpath.active"
              path={settings.defaultWorkingPath}
              pick={pickDirectory}
              onChange={(p) => onSettingsChange(withDefaultWorkingPath(settings, p))}
            />
          </>
        )}

        {tab === 'appearance' && (
          <>
            <section className="settings-screen__card" aria-labelledby={`${uid}-theme`}>
              <h3 id={`${uid}-theme`} className="settings-screen__card-title">
                {t('settings.theme.title')}
              </h3>
              <div className="theme-grid" role="radiogroup" aria-labelledby={`${uid}-theme`}>
                {THEME_OPTIONS.map((option) => (
                  <button
                    key={option}
                    type="button"
                    role="radio"
                    aria-checked={settings.theme === option}
                    className={`theme-option${settings.theme === option ? ' is-current' : ''}`}
                    onClick={() => onSettingsChange(withTheme(settings, option))}
                  >
                    <span className={`theme-option__specimen theme-option__specimen--${option}`} />
                    <span className="theme-option__label">{t(`settings.theme.${option}`)}</span>
                  </button>
                ))}
              </div>
            </section>
            <section className="settings-screen__card" aria-labelledby={`${uid}-iconsize`}>
              {/* العنوان أوّلًا فيقع يمينًا، والقائمة آخرًا فتقع يسارًا. */}
              <div className="settings-screen__card-row">
                <h3 id={`${uid}-iconsize`} className="settings-screen__card-title">
                  {t('settings.iconsize.title')}
                </h3>
                <select
                  className="settings-select"
                  aria-labelledby={`${uid}-iconsize`}
                  value={settings.sidebarIconSize}
                  onChange={(e) =>
                    onSettingsChange(withSidebarIconSize(settings, e.target.value as SidebarIconSize))
                  }
                >
                  {ICON_SIZES.map((size) => (
                    <option key={size} value={size}>
                      {t(`settings.iconsize.${size}`)}
                    </option>
                  ))}
                </select>
              </div>
            </section>
          </>
        )}

        {tab === 'developer' && (
          <>
            <PathCard
              uid={uid}
              name="node"
              keyPrefix="settings.node"
              labelKey="settings.toolpath.selected"
              path={settings.nodePath}
              pick={pickFile}
              onChange={(p) => onSettingsChange(withNodePath(settings, p))}
            />
            <PathCard
              uid={uid}
              name="cargo"
              keyPrefix="settings.cargo"
              labelKey="settings.toolpath.selected"
              path={settings.cargoPath}
              pick={pickFile}
              onChange={(p) => onSettingsChange(withCargoPath(settings, p))}
            />
          </>
        )}

        {tab === 'about' && (
          <div className="about">
            <div className="about__mark" aria-hidden="true">
              ن
            </div>
            <p className="about__name">نَفِّذ</p>
            <p className="about__version">{tFormat('settings.about.version', { version: APP_VERSION })}</p>

            <UpdateBadge state={update} />

            <div className="about__toggle">
              <Switch
                on={settings.autoUpdate}
                onChange={(next) => onSettingsChange(withAutoUpdate(settings, next))}
                labelledBy={`${uid}-autoupdate`}
              />
              <span id={`${uid}-autoupdate`} className="about__toggle-label">
                {t('settings.update.auto')}
              </span>
            </div>

            {update.k === 'available' ? (
              <button
                type="button"
                className="btn btn--primary"
                onClick={() => void install(update.info)}
              >
                {t('settings.update.download')}
              </button>
            ) : (
              <button
                type="button"
                className="btn btn--outline"
                // ‏`installed` أيضًا: التطبيق ما زال يشغّل الثنائيّة القديمة
                // حتى تُعاد تشغيله، فسؤالٌ جديد الآن يقارن بنفس الإصدار القديم.
                disabled={
                  update.k === 'checking' ||
                  update.k === 'installing' ||
                  update.k === 'installed'
                }
                onClick={() => void runCheck()}
              >
                {/* «إعادة المحاولة» تَعِد بأن التكرار قد ينجح — وهو صحيحٌ في
                    فشل الشبكة، وكاذبٌ حين لا وجهة مضبوطة أصلًا. */}
                {t(update.k === 'error' ? 'settings.update.retry' : 'settings.update.check')}
              </button>
            )}

            <hr className="about__rule" />
            <p className="about__description">{t('settings.about.description')}</p>
            <p className="about__credit">{t('settings.about.credit')}</p>
          </div>
        )}
        </div>
      </div>
    </section>
  );
}

/** شارة الحالة فوق زرّ الفحص — نبرتها من الحالة لا من نصّها. */
function UpdateBadge({ state }: { state: UpdateState }): JSX.Element | null {
  if (state.k === 'idle') return null;

  // «غير مهيأة» حالةٌ محايدة لا خطر: لا شيء فشل، والوجهة لم تُضبط بعد.
  if (state.k === 'unconfigured') {
    return (
      <>
        <p className="update-badge update-badge--neutral" role="status">
          {t('settings.update.unconfigured')}
        </p>
        <p className="update-badge__hint update-badge__hint--neutral">
          {t('settings.update.unconfigured.hint')}
        </p>
      </>
    );
  }

  if (state.k === 'error') {
    return (
      <>
        <p className="update-badge update-badge--danger" role="status">
          {t('settings.update.failed')}
        </p>
        <p className="update-badge__hint">{t('settings.update.failed.network')}</p>
      </>
    );
  }

  if (state.k === 'available') {
    return (
      <p className="update-badge update-badge--warning" role="status">
        {tFormat('settings.update.available', { version: state.info.version })}
      </p>
    );
  }

  // ‏H-7: حالةٌ نهائية إيجابية — التحديث ثُبِّت فعلًا — لا تختلط بـ`uptodate`
  // (لم يوجد تحديثٌ أصلًا) رغم مشاركتهما النبرة: الرسالتان مختلفتان، والثانية
  // وحدها تحتاج تلميحًا بإعادة التشغيل.
  if (state.k === 'installed') {
    return (
      <>
        <p className="update-badge update-badge--ok" role="status">
          {t('settings.update.installed')}
        </p>
        {/* `--neutral` لا اللون الافتراضي: ذاك أحمر مصمَّمٌ لتلميحٍ يتبع خطأً
            («تحقّق من اتصالك»)، وهذا تعليمات متابعةٍ بعد نجاحٍ لا إنذار. */}
        <p className="update-badge__hint update-badge__hint--neutral">
          {t('settings.update.installed.hint')}
        </p>
      </>
    );
  }

  const tone = state.k === 'uptodate' ? 'ok' : 'neutral';
  const key =
    state.k === 'uptodate'
      ? 'settings.update.uptodate'
      : state.k === 'installing'
        ? 'settings.update.installing'
        : 'settings.update.checking';
  return (
    <p className={`update-badge update-badge--${tone}`} role="status">
      {t(key)}
    </p>
  );
}
