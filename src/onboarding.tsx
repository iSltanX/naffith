/**
 * الترحيب — الشاشة التي تُعرض مرّةً واحدة.
 *
 * ## لماذا فعلٌ واحد لا ثلاثة
 *
 * النموذج المرجعي يعرض ثلاثة أزرار: «ابدأ الآن» و«جولة سريعة» و«تخطّي». اثنان
 * منها سقطا هنا عن قصد:
 *
 * - **«جولة سريعة»** تَعِد بتلميحاتٍ داخل التطبيق، ولا نظام تلميحات في هذا
 *   الإصدار. زرٌّ يَعِد بما لا يقع أسوأ من غيابه: أول تفاعل في التطبيق يصير
 *   وعدًا مُخلَفًا، وهذا نقيض ما تبيعه الشاشة كلّها.
 * - **«تخطّي»** مغادرةٌ ثانية لنفس الوجهة. «ابدأ الآن» *هي* المضيّ قُدُمًا، ولا
 *   شيء هنا يُتخطّى — لا حساب يُنشأ ولا إعداد يُملأ. وجودُ مخرجين لمخرجٍ واحد
 *   يُضعف الأساسي ويجعل المستخدم يسأل عن فرقٍ لا وجود له.
 *
 * فبقي فعلٌ واحد، وهو أيضًا عنصر التبويب الوحيد في الشاشة.
 *
 * ## لماذا الأمر المعروض هو `ditto` نفسه
 *
 * النموذج المرجعي يعرض `mv -v …` تحت الخطوة الثالثة. عرضُ أمرٍ لا يستطيع
 * التطبيق تنفيذه ينقض دعواه الوحيدة — «هذا هو الأمر نفسه، لا تمثيلٌ له» — في
 * أول شاشة يراها المستخدم. فالمعروض هنا شكل `argv` الحقيقي الذي تبنيه النواة:
 * الأداة ورايتها الأربع كما هي، ومسارَان نائبان يوضّحان الموضع لا أكثر.
 * وبما أن المسارين نائبان، فالمقتطف كلّه محجوب عن القارئ الصوتي: هو رسمُ سطحٍ
 * لا محتوى، ونصّ الخطوة الثالثة يقول ما يقوله كاملًا.
 *
 * ## لماذا البؤرة على الإطار لا على الزر
 *
 * تركيز الزر تلقائيًا يجعل القارئ الصوتي ينطق «ابدأ الآن، زر» قبل أن يسمع
 * المستخدم فيمَ يُرحَّب به — وشاشةٌ تُقرأ مرّةً واحدة لا تحتمل أن يُقفَز عن
 * عنوانها. فالبؤرة تقع على الإطار (`tabindex="-1"`) ليُقرأ من العنوان، والزر —
 * وهو العنصر القابل للتبويب الوحيد — أول محطّة لـ Tab وآخرها.
 *
 * ## لماذا سطحٌ داكن في الوضعين
 *
 * الداكن في هذه الهوية ليس «وضع النظام ليليّ»، بل «الطبقة التي تحت». والشاشة
 * تُعرّف سَطْر — تلك الطبقة بعينها — فتلبس مادّتها. رموز `--*-on-inverse` ثابتة
 * بين الوضعين لهذا السبب، فالتباين محسوب مرّةً واحدة ويصحّ في الحالتين.
 */
import { useEffect, useId, useRef, useState } from 'react';
import { t } from './i18n';
import './onboarding.css';

/** الخطوات تُشتقّ من المفاتيح لا تُكتب صفًّا صفًّا: إضافة رابعة سطرٌ واحد. */
const STEPS = ['step1', 'step2', 'step3'] as const;

/**
 * شكل الأمر الحقيقي، لا مثالٌ مخترع.
 *
 * الأداة والرايات منقولة حرفيًا عمّا تبنيه النواة (انظر `shell-quote.test.ts`)؛
 * المسارَان وحدهما نائبان لأن لا مصدر ولا وجهة بعد.
 */
const PEEK_ARGV: ReadonlyArray<readonly [token: string, role: string]> = [
  ['/usr/bin/ditto', 'tok-name'],
  ['-c', 'tok-flag'],
  ['-k', 'tok-flag'],
  ['--sequesterRsrc', 'tok-flag'],
  ['--keepParent', 'tok-flag'],
  ['~/Documents/Reports', 'tok-path'],
  ['~/Desktop/Reports.zip', 'tok-path'],
];

/**
 * تفضيل تقليل الحركة، مقروءًا مرّةً واحدة.
 *
 * قراءته في JS لا في CSS وحدها لأن الحركة تُطفأ بسمةٍ على الجذر، فيبقى إطفاؤها
 * قابلًا للاختبار بلا محرّك أنماط. و`matchMedia` قد تغيب في بيئة اختبار قديمة،
 * فغيابها يُقرأ «لا تفضيل» بدل أن يُسقط الشاشة.
 */
function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export default function Onboarding({ onStart }: { onStart: () => void }): JSX.Element {
  const uid = useId();
  const titleId = `${uid}-title`;
  const frame = useRef<HTMLElement>(null);

  // تُقرأ عند التركيب فقط: الشاشة تعيش ثوانيَ معدودة، ومراقبة تغيّر التفضيل
  // أثناءها تشتري حالةً حيّة مقابل حدثٍ لا يقع.
  const [still] = useState(prefersReducedMotion);

  useEffect(() => {
    frame.current?.focus();
  }, []);

  return (
    <main
      className="onboarding"
      data-motion={still ? 'still' : 'enter'}
      tabIndex={-1}
      ref={frame}
      aria-labelledby={titleId}
    >
      <div className="onboarding__card">
        <svg className="onboarding__mark" viewBox="0 0 64 64" aria-hidden="true">
          <use href="#mark" />
        </svg>

        <h1 id={titleId} className="t-page-title onboarding__title">
          {t('onboarding.title')} {t('app.naffith')}{' '}
          <span className="onboarding__sep" aria-hidden="true">
            —
          </span>{' '}
          {t('app.satr')}
        </h1>

        <p className="t-body onboarding__lede">{t('onboarding.lede')}</p>

        {/* `ol` لأن الترتيب معنى لا زينة: لا تُراجَع خطةٌ قبل اختيار عملية. */}
        <ol className="onboarding__steps">
          {STEPS.map((step, i) => (
            <li className="onboarding__step" key={step}>
              {/* الرقم مرسوم، والقائمة المرقّمة تعطيه للقارئ الصوتي أصلًا. */}
              <span className="onboarding__n num lat" aria-hidden="true">
                {i + 1}
              </span>
              <div className="onboarding__step-text">
                <h2 className="t-card-title onboarding__step-title">
                  {t(`onboarding.${step}.title`)}
                </h2>
                <p className="t-body-sec onboarding__step-body">{t(`onboarding.${step}.body`)}</p>
                {step === 'step3' && <Peek />}
              </div>
            </li>
          ))}
        </ol>

        <div className="onboarding__actions">
          <button type="button" className="btn btn--primary btn--lg" onClick={onStart}>
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-execute" />
            </svg>
            {t('onboarding.start')}
          </button>
        </div>

        <p className="t-caption onboarding__once">{t('onboarding.once')}</p>
      </div>
    </main>
  );
}

/** مقتطف سَطْر. انظر تعليق الرأس: شكلٌ حقيقي، ومحجوب عن القارئ الصوتي. */
function Peek() {
  return (
    <div className="onboarding__peek" aria-hidden="true">
      <div className="command">
        <div className="command__bar">
          <span className="command__label">{t('app.satr')}</span>
        </div>
        {/* `white-space: pre` في `.command__body`: لا مسافة مصدرٍ بين الوسوم.

            و`tabindex="-1"` ليس زينة: الصندوق يمرّر أفقيًا (`overflow-x: auto`)
            لأن الأمر أطول من البطاقة، والمتصفّحات الحديثة تجعل كل صندوق تمريرٍ
            بلا ابنٍ قابلٍ للتبويب محطّةَ تبويبٍ من تلقائها كي يبقى تمريرُه
            بالمفتاح ممكنًا. فكانت أوّل ضغطة Tab في الشاشة تقع على هذا الرسم لا
            على «ابدأ الآن» — وهو `aria-hidden`، فيقع التركيز على ما لا ينطقه
            القارئ الصوتي: صمتٌ يظنّه المستخدم عطلًا. لا شيء هنا يُقرأ ولا
            يُمرَّر طلبًا لمعنى: نصّ الخطوة الثالثة يقول ما يقوله كاملًا. */}
        <div className="command__body" tabIndex={-1}>
          <span className="tok-prompt">$ </span>
          {PEEK_ARGV.map(([token, role], i) => (
            <span key={token}>
              <span className={role}>{token}</span>
              {i < PEEK_ARGV.length - 1 ? ' ' : ''}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}
