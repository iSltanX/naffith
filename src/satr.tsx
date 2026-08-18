/**
 * سَطْر — الطبقة المكشوفة تحت السطح الهادئ.
 *
 * ثلاث قواعد تحكم هذه الشاشة:
 *
 * 1. **تعرض الأمر نفسه، لا تمثيلًا له.** كل رمز هنا يأتي من `plan.explain`،
 *    وهي مبنيّة في Rust من `argv` نفسها التي ستذهب إلى `execve`. اختبارٌ ذهبي
 *    يثبت التطابق رمزًا رمزًا.
 *
 * 2. **الوسائط مفصولة بصريًا.** أمرٌ معروض سطرًا واحدًا يجعل المسافة داخل اسم
 *    ملف تبدو كفاصل بين وسيطين. هنا كل وسيط صفّ مرقّم، فلا التباس.
 *
 * 3. **النسخ ليس تنفيذًا.** الأمر المنسوخ يمرّ بهروب صدفة لأن Terminal يحتاجه؛
 *    التطبيق لا يستعمل ذلك النص أبدًا. انظر `shell-quote.ts`.
 */
import { useEffect, useState } from 'react';
import type { PlanResponse, TokenRole } from './ipc';
import { t } from './i18n';
import RunStream, { type StreamLine, type StreamPhase } from './run-stream';
import { shellCommand } from './shell-quote';

const ROLE_CLASS: Record<TokenRole, string> = {
  tool: 'tok-name',
  flag: 'tok-flag',
  path: 'tok-path',
  value: 'tok-string',
};

/**
 * مهلة بقاء الكتابة المائية بعد وصول الخطة.
 *
 * أطول قليلًا من `--duration-fast` كي ينتهي الخبوّ قبل أن يُنزع العنصر، فلا
 * يُقتطع الانتقال في منتصفه. ومع `prefers-reduced-motion` تنهار مدّة الانتقال
 * في نظام التصميم إلى ١ms، فتختفي الكتابة فورًا وتبقى هذه المهلة بلا أثر مرئي.
 */
const IDLE_FADE_MS = 160;

/**
 * اللوحة: مطويّةٌ حتى يوجد أمر، ثم تتوسّع إلى لوحةٍ جانبية حقيقية.
 *
 * ## العطل البنيوي الذي عولج هنا
 *
 * كانت اللوحة تحجز عمودها كاملًا في كل حال: عرضٌ ثابت وارتفاعٌ بارتفاع النافذة،
 * وفيه — قبل اكتمال الحقول — اسمٌ وثلاثة أسطر. أي أن ثلث الشاشة كان محجوزًا
 * لمحتوًى لا يوجد. وضبطُ ألوانه لا يغيّر ذلك: مساحةٌ محجوزةٌ لغائب تُقرأ عطلًا
 * أو نقصًا، ولو كانت بلون التطبيق تمامًا.
 *
 * والعرضُ الآن دالّةُ المحتوى لا دالّةُ النافذة:
 *
 * | الحالة | العرض | ما يُعرض |
 * |---|---|---|
 * | لا خطة | ‏96px‎ ثابتة | الاسم والحالة، لا أكثر |
 * | خطة حاضرة | ‏34% بسقف ‎464px‎ | الأمر، ثم الوسائط، ثم الملاحظات |
 *
 * والشرط `plan !== null` وحده — لا اسم عملية ولا عدد حقول — فالسلوك واحد في كل
 * عملية، الحاضرة اليوم والمضافة غدًا.
 */
export default function Satr({
  plan,
  stream = [],
  phase = 'idle',
  status,
  onCancel,
}: {
  plan: PlanResponse | null;
  /** ما بثّته النواة من خرج الأداة في التشغيل الحالي. */
  stream?: readonly StreamLine[];
  phase?: StreamPhase;
  status?: string | undefined;
  onCancel?: (() => void) | undefined;
}) {
  /**
   * هل الشريط المطويّ محمول؟
   *
   * لا يكفي `!plan`: نزعُ العنصر لحظةَ وصول الخطة يقطع الخبوّ قبل أن يبدأ.
   * فيبقى محمولًا بشفافية صفر طوالَ الانتقال ثم يُنزع — ولو وصلت الشاشة ومعها
   * خطة من البداية لم يُحمل أصلًا، فلا وميض في أول رسم.
   */
  const [idleShown, setIdleShown] = useState(plan === null);

  useEffect(() => {
    if (plan === null) {
      setIdleShown(true);
      return;
    }
    if (!idleShown) return;
    const timer = window.setTimeout(() => setIdleShown(false), IDLE_FADE_MS);
    return () => window.clearTimeout(timer);
  }, [plan, idleShown]);

  const stateKey =
    phase === 'running'
      ? 'satr.state.running'
      : phase === 'cancelling'
        ? 'state.cancelling'
        : phase === 'finished'
          ? status === 'success'
            ? 'satr.state.succeeded'
            : 'satr.state.failed'
          : 'satr.state.planned';
  const stateClass =
    phase === 'cancelling'
      ? ' satr__state--warning'
      : phase === 'finished' && status !== 'success'
        ? ' satr__state--danger'
        : phase === 'finished'
          ? ' satr__state--success'
          : '';

  return (
    <section
      className={`satr${plan ? ' satr--live' : ''}`}
      aria-labelledby="satr-heading"
    >
      {/* العنوان في الحالتين، وبمعرّفٍ واحد: هو ما يسمّي المنطقة لقارئ الشاشة،
          فلا يجوز أن يغيب في إحداهما ويحمل `aria-labelledby` مرجعًا معلَّقًا. */}
      {plan ? (
        <>
          <div className="satr__head">
            <h2 id="satr-heading" className="t-card-title satr__name">
              {t('app.satr')}
            </h2>
            <p className={`t-caption satr__state${stateClass}`}>{t(stateKey)}</p>
          </div>
          <div className="satr__stage">
            <Preview plan={plan} />
            {/* مجرى التشغيل بعد الأمر ووسائطه وقبل الملاحظات: الأمرُ ما
                سيُنفَّذ، والمجرى ما قالته الأداة وهي تُنفَّذ، والملاحظات سياسةٌ
                ثابتة لا تتبدّل بتشغيل. فالترتيب زمنيّ: خطّة، فتنفيذ، فسياسة. */}
            <RunStream lines={stream} phase={phase} />
            {(phase === 'running' || phase === 'cancelling') && onCancel && (
              <button
                type="button"
                className={`btn satr__cancel${phase === 'cancelling' ? ' satr__cancel--waiting' : ''}`}
                onClick={onCancel}
                disabled={phase === 'cancelling'}
              >
                {t(phase === 'cancelling' ? 'state.cancelling' : 'satr.action.cancel')}
              </button>
            )}
          </div>
        </>
      ) : null}

      {idleShown && <Rail leaving={plan !== null} labelled={plan === null} />}
    </section>
  );
}

/**
 * الشريط المطويّ: الاسم والحالة، لا أكثر.
 *
 * ثلاث صياغات سبقته وسقطت كلّها لسببٍ واحد — أنها كانت **محتوًى بديلًا** يشغل
 * مساحةَ المحتوى الغائب: صندوقٌ منقّط، ثم كتابةٌ مائية موسَّطة بعنوان ‎28px‎، ثم
 * كتلةٌ صغيرة من ثلاثة أسطر في أعلى عمودٍ كامل. الثلاثة تختلف في الحجم وتتّفق
 * في الخطأ: عمودٌ محجوز لما لا يوجد.
 *
 * وهذا ليس محتوًى بديلًا بل **حالةُ اللوحة نفسها**: شريطٌ بعرض ‎96px‎ يقول ما هي
 * وأنها تنتظر. لا تعليمات فيه — التعليمة مكانها النموذج المجاور، وشريطٌ بهذا
 * العرض لا يحمل جملة أصلًا.
 *
 * ‏`labelled` تحمل العنوان `h2` وقتَ الطيّ وحده: عند التوسّع يحمله رأس اللوحة،
 * ووجودُ معرّفٍ واحد في عنصرين معًا — ولو للحظة الخبوّ — يجعل `aria-labelledby`
 * يشير إلى أوّلهما في المستند لا إلى ما يراه المستخدم.
 *
 * **ليس `aria-hidden`.** هو ما تقوله هذه المنطقة كلّها، وإخفاؤه يترك من يقرأ
 * بالصوت أمام منطقةٍ صامتة بلا سبب. وليس `aria-live`: النصّ ساكن منذ أول رسم،
 * فإعلانه مقاطعةٌ لا خبر.
 */
function Rail({ leaving, labelled }: { leaving: boolean; labelled: boolean }) {
  return (
    <div className={`satr__rail${leaving ? ' satr__rail--leaving' : ''}`}>
      <h2
        id={labelled ? 'satr-heading' : undefined}
        className="t-label satr__rail-name"
      >
        {t('app.satr')}
      </h2>
      <p className="t-caption satr__rail-state">{t('satr.idle.title')}</p>
    </div>
  );
}

/** الأمر كما سيُنفَّذ: سطرًا واحدًا للقراءة السريعة، ثم وسيطًا وسيطًا للفهم. */
function Preview({ plan }: { plan: PlanResponse }) {
  const [copied, setCopied] = useState(false);
  const suspicious = plan.argv_display.some(
    (token, index) =>
      index > 0 && /\s/u.test(token) && plan.explain[index]?.role !== 'tool',
  );

  async function copy() {
    await navigator.clipboard.writeText(shellCommand(plan.argv_display));
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  }

  return (
      <div className={`command${suspicious ? ' command--suspicious' : ''}`}>
        <div className="command__bar">
          <span className="command__label">{t('satr.title')}</span>
          <button
            type="button"
            className="command__copy"
            onClick={copy}
            aria-live="polite"
          >
            {copied ? t('action.copied') : t('action.copy')}
          </button>
        </div>

        {/* سطر واحد للقراءة السريعة، بنفس ترتيب الوسائط ولونها. */}
        <div className={`command__body${plan.argv_display.length === 0 ? ' command__body--empty' : ''}`}>
          {plan.argv_display.length === 0 ? (
            <span>{t('satr.command.empty')}</span>
          ) : plan.argv_display.map((token, i) => {
            const role = plan.explain[i]?.role ?? 'value';
            return (
            <span key={i} className={`command__token command__token--${role}`}>
              <span className={ROLE_CLASS[role]}>{token}</span>
              {i < plan.argv_display.length - 1 ? ' ' : ''}
            </span>
            );
          })}
        </div>
        {suspicious && (
          <p className="command__warning" role="status">
            {t('satr.command.suspicious')}
          </p>
        )}
      </div>
  );
}
