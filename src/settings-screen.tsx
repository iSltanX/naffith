/**
 * الإعدادات.
 *
 * ## لماذا شاشةٌ فيها خيارٌ واحد — سابقًا
 *
 * «شاشة الترحيب تُعرض مرّةً واحدة» وعدٌ قطعه `onboarding.once`، ووعدٌ كهذا لا
 * يجوز أن يكون طريقًا ذا اتجاه واحد: من أغلقها في ثانيته الأولى قبل أن يقرأ
 * فقد ما لا يستطيع استعادته. فوجودُ هذه الشاشة هو ما يجعل ذلك الوعد صادقًا لا
 * محبِسًا.
 *
 * ومسارا Node.js وCargo أضيفا لسببٍ مختلف: أدواتٌ يثبّتها المستخدم بنفسه في
 * مسارٍ لا مسارٍ نظاميًّا ثابتًا (خلاف كل أداةٍ أخرى في هذا التطبيق)، فتُختار
 * مرّةً هنا — أو في أول عملية أدوات مطوّرين تحتاجها — وتُتذكَّر بعدها.
 *
 * ## لماذا `storageAvailable` مُمرَّرة لا مقروءة هنا
 *
 * الشاشة تعرض ولا تفحص. من يملك المخزن هو من يقرأ الإعداد ويكتبه (`settings.ts`
 * ومن يستدعيه)، وسؤالُه مرّةً ثانيةً من هنا يفتح احتمال أن تقول الشاشة «محفوظ»
 * بينما تعرف الطبقة التي تحفظ أنه ليس كذلك. مصدرُ الحقيقة واحد، وهذه نافذةٌ
 * عليه.
 *
 * وحين يتعذّر الحفظ لا يُعطَّل الزرّ: إعادة عرض الترحيب تعمل في هذه الجلسة
 * كاملةً، وكل ما يضيع أنها لن تُتذكَّر بعد الإغلاق. تعطيلُ فعلٍ يعمل عقوبةٌ على
 * عطلٍ ليس من صنع المستخدم.
 */
import { useEffect, useId, useRef } from 'react';
import { t } from './i18n';
import { pickFile } from './ipc';
import './utility-screen.css';
import './settings-screen.css';

/**
 * قسم مسار أداةٍ واحد — Node.js وCargo كلاهما نفس الشكل بالضبط: عرض المسار
 * المحفوظ أو «لم يُحدَّد بعد»، وزرّ اختيار/تغيير، وزرّ مسح يظهر حين توجد قيمة.
 *
 * مكوّنٌ داخليّ لا ملفٌّ مستقل: لا يُستعمل خارج هذه الشاشة، وفصله كان يعني
 * تمرير سبعة خصائص عبر حدّ ملفّ من أجل قسمٍ واحد يظهر مرّتين فقط.
 */
function ToolPathSection({
  uid,
  keyPrefix,
  path,
  onChange,
}: {
  uid: string;
  keyPrefix: 'settings.node' | 'settings.cargo';
  path: string | null;
  onChange: (path: string | null) => void;
}) {
  const browse = async () => {
    const chosen = await pickFile();
    if (chosen) onChange(chosen);
  };

  const key = (suffix: string): string => `${keyPrefix}.${suffix}`;

  return (
    <section className="card screen__section settings-screen__card" aria-labelledby={`${uid}-${keyPrefix}`}>
      <h3 id={`${uid}-${keyPrefix}`} className="t-card-title">
        {t(key('title'))}
      </h3>
      <p className="t-body-sec">{t(key('body'))}</p>
      <p className="t-body-sec settings-screen__path-value" dir="ltr">
        <bdi>{path ?? t(key('unset'))}</bdi>
      </p>
      <div className="settings-screen__path-actions">
        <button type="button" className="btn btn--primary" onClick={browse}>
          {t(path ? key('change') : key('choose'))}
        </button>
        {path && (
          <button type="button" className="btn btn--quiet" onClick={() => onChange(null)}>
            {t(key('clear'))}
          </button>
        )}
      </div>
    </section>
  );
}

export default function SettingsScreen(props: {
  onBack: () => void;
  onReplayOnboarding: () => void;
  storageAvailable: boolean;
  /** مسار Node.js المحفوظ، أو `null` إن لم يُختر بعد. */
  nodePath: string | null;
  /** `null` يمسح القيمة. الشاشة لا تتحقّق من المسار — ذاك عمل النواة وقت
   *  التخطيط، لا عمل حوار اختيارٍ ولا شاشة إعداد. */
  onNodePathChange: (path: string | null) => void;
  /** مسار Cargo المحفوظ، أو `null`. نفس منطق `nodePath` بالضبط. */
  cargoPath: string | null;
  onCargoPathChange: (path: string | null) => void;
}): JSX.Element {
  const {
    onReplayOnboarding,
    storageAvailable,
    nodePath,
    onNodePathChange,
    cargoPath,
    onCargoPathChange,
  } = props;

  const uid = useId();
  const headingId = `${uid}-heading`;
  const heading = useRef<HTMLHeadingElement>(null);

  // تبديل الشاشة في تطبيق صفحةٍ واحدة لا يحرّك التركيز من تلقائه: من يتنقّل
  // بلوحة المفاتيح يبقى تركيزه على زرٍّ اختفى، فيستأنف Tab من رأس المستند.
  // نقلُه إلى العنوان يجعل أول ما يُنطق اسمَ الشاشة، وأول ما يليه «رجوع».
  useEffect(() => {
    heading.current?.focus();
  }, []);

  return (
    <section className="screen settings-screen" aria-labelledby={headingId}>
      <header className="screen__head">
        {/* ‏h2 لا h1: هويّة المنتج تحمل h1 في ترويسة التطبيق الثابتة، وهذه
            شاشةٌ داخلها. الطبقة `t-page-title` شكلٌ لا رتبة. */}
        <h2 id={headingId} className="t-page-title screen__title" tabIndex={-1} ref={heading}>
          {t('settings.title')}
        </h2>
        <p className="t-body-sec settings-screen__subtitle">{t('settings.subtitle')}</p>
      </header>

      {/* التنبيه فوق الخيار الذي يعنيه لا تحته: من يقرأ «اعرضها الآن متى شئت»
          ينبغي أن يكون قد عرف قبلها أن «متى شئت» لن يُحفظ. */}
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

      <section className="card screen__section settings-screen__card" aria-labelledby={`${uid}-onboarding`}>
        <h3 id={`${uid}-onboarding`} className="t-card-title">
          {t('settings.onboarding.title')}
        </h3>
        <p className="t-body-sec">{t('settings.onboarding.body')}</p>
        <button type="button" className="btn btn--primary settings-screen__replay" onClick={onReplayOnboarding}>
          {t('settings.onboarding.replay')}
        </button>
      </section>

      <ToolPathSection uid={uid} keyPrefix="settings.node" path={nodePath} onChange={onNodePathChange} />
      <ToolPathSection uid={uid} keyPrefix="settings.cargo" path={cargoPath} onChange={onCargoPathChange} />
    </section>
  );
}
