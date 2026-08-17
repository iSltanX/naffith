/**
 * الترحيب — الشاشة التي تُعرض مرّةً واحدة.
 *
 * ## لماذا لوحتان لا بطاقة واحدة
 *
 * الشاشة مكتوبة بلغة mockups-v3: سطحان متجاوران، أحدهما أعمق من الآخر. اليمنى
 * (`__visual`) تُعرِّف: العلامة والاسم والجملة والمقتطف — أي *ما هذا*. واليسرى
 * (`__copy`) تُشغِّل: الخطوات الثلاث والفعل — أي *ماذا أفعل الآن*. البطاقة
 * الواحدة التي كانت هنا رصّت الستّة في عمودٍ واحد، فصار الفعل في أسفل عمودٍ
 * طويل، وصارت النافذة العريضة فراغًا على الجانبين. اللوحتان تحوّلان العرض
 * الفائض إلى محتوى بدل أن تتركاه هامشًا.
 *
 * وتبقيان عمودين عند كل مقاسٍ تبلغه النافذة. كانتا ترجعان عمودًا واحدًا حين
 * يضيق العرض، وذلك هو ما دفع «ابدأ الآن» خارج النافذة: عمودان متجاوران
 * ارتفاعُهما أكبرهما، ومرصوصان ارتفاعُهما مجموعهما — والفرق المقيس عند ‎680px
 * عرضًا ‎230.7px، أي أكثر من العجز كلّه. انظر `onboarding.css`: القرار هناك
 * بأرقامه، لا هنا، ولا سمة في هذا الملف تعرف مقاس النافذة.
 *
 * والمقتطف وحده هو ما يُخفى حين يضيق العرض — لأنه `white-space: pre` فلا
 * يضيق — ونصّ الخطوة الثالثة يبقى يقول دعواه كاملةً عند كل مقاس.
 *
 * ## لماذا انتقل المقتطف من الخطوة الثالثة إلى لوحة التعريف
 *
 * كان تحت نصّ الخطوة الثالثة. في تخطيطٍ ذي عمودين هذا يضع صندوقًا لاتينيًّا
 * (`direction: ltr`) في وسط قائمةٍ عربية قصيرة الأسطر، فينكسر إيقاع الخطوات
 * الثلاث ويضيق الصندوق إلى نصف ما يحتاج. وموضعه الجديد هو موضعه في النموذج
 * المرجعي: تحت الجملة مباشرةً، لأنه *برهانها* — الجملة تدّعي أن الأمر يبقى
 * معروضًا، والمقتطف هو ذلك الأمر. ونصّ الخطوة الثالثة يظل يقول ما يقوله
 * كاملًا، فلم يُفقَد شيء بنقله.
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
 * ## لماذا البرهان سطرٌ واحد
 *
 * هذا ليس سجلّ تنفيذ ولا معاينة خطة؛ هو برهان بصري قصير من Page 15 على أن
 * الأمر يظل ظاهرًا. لذلك يبقى في سطرٍ واحد بلا شريط عنوان ولا تفكيك رايات.
 * والمساران نائبان، فالبطاقة كلها محجوبة عن القارئ الصوتي: نصّ الخطوة الثالثة
 * يشرح المعنى، ولا يُطلب من المستخدم تفسير مثالٍ لم يُنشئه.
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

const PEEK_COMMAND = '$ ditto ~/Documents ~/Desktop/Backup';

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
      <div className="onboarding__titlebar" data-tauri-drag-region aria-hidden="true" />
      <div className="onboarding__panes">
        <div className="onboarding__visual">
          <h1 id={titleId} className="onboarding__title">{t('app.naffith')}</h1>

          <p className="t-body onboarding__lede">{t('onboarding.lede')}</p>

          <Peek />
        </div>

        <div className="onboarding__copy">
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
                </div>
              </li>
            ))}
          </ol>

          <div className="onboarding__actions">
            <button type="button" className="btn btn--primary btn--lg" onClick={onStart}>
              {t('onboarding.start')}
            </button>
          </div>

          <p className="t-caption onboarding__once">{t('onboarding.once')}</p>
        </div>
      </div>
    </main>
  );
}

/** برهان سَطْر المختصر من Page 15، ومحجوب عن القارئ الصوتي. */
function Peek() {
  return (
    <figure className="onboarding__peek" aria-hidden="true">
      {/* قد يضيق العمود في نافذة 680px. يبقى المثال سطرًا واحدًا كما صُمّم،
          والصندوق قابل للتمرير برمجيًا لكن خارج ترتيب Tab لأنه رسمٌ محجوب. */}
      <code className="onboarding__proof-command" dir="ltr" tabIndex={-1}>
        {PEEK_COMMAND}
      </code>
      <figcaption className="onboarding__proof-caption">
        {t('onboarding.proof.caption')}
      </figcaption>
    </figure>
  );
}
