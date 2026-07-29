/**
 * الإعدادات.
 *
 * ## لماذا شاشةٌ فيها خيارٌ واحد
 *
 * «شاشة الترحيب تُعرض مرّةً واحدة» وعدٌ قطعه `onboarding.once`، ووعدٌ كهذا لا
 * يجوز أن يكون طريقًا ذا اتجاه واحد: من أغلقها في ثانيته الأولى قبل أن يقرأ
 * فقد ما لا يستطيع استعادته. فوجودُ هذه الشاشة هو ما يجعل ذلك الوعد صادقًا لا
 * محبِسًا. وخيارٌ واحد اليوم أصدق من قائمة تفضيلاتٍ مخترعة لملء المساحة.
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
import './shell-screens.css';

export default function SettingsScreen(props: {
  onBack: () => void;
  onReplayOnboarding: () => void;
  storageAvailable: boolean;
}): JSX.Element {
  const { onBack, onReplayOnboarding, storageAvailable } = props;

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
    <section className="screen" aria-labelledby={headingId}>
      <header className="screen__head">
        <button type="button" className="btn btn--quiet btn--sm screen__back" onClick={onBack}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-chevron" />
          </svg>
          {t('nav.back')}
        </button>
        {/* ‏h2 لا h1: هويّة المنتج تحمل h1 في ترويسة التطبيق الثابتة، وهذه
            شاشةٌ داخلها. الطبقة `t-page-title` شكلٌ لا رتبة. */}
        <h2 id={headingId} className="t-page-title screen__title" tabIndex={-1} ref={heading}>
          {t('settings.title')}
        </h2>
      </header>

      {/* التنبيه فوق الخيار الذي يعنيه لا تحته: من يقرأ «اعرضها الآن متى شئت»
          ينبغي أن يكون قد عرف قبلها أن «متى شئت» لن يُحفظ. */}
      {!storageAvailable && (
        <div className="notice notice--warning">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-warning" />
          </svg>
          <p>{t('settings.storage.unavailable')}</p>
        </div>
      )}

      <section className="card screen__section" aria-labelledby={`${uid}-onboarding`}>
        <h3 id={`${uid}-onboarding`} className="t-card-title">
          {t('settings.onboarding.title')}
        </h3>
        <p className="t-body-sec">{t('settings.onboarding.body')}</p>
        <button type="button" className="btn btn--quiet" onClick={onReplayOnboarding}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-eye" />
          </svg>
          {t('settings.onboarding.replay')}
        </button>
      </section>
    </section>
  );
}
