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
import { shellCommand, tokenNotes } from './shell-quote';

const ROLE_CLASS: Record<TokenRole, string> = {
  tool: 'tok-name',
  flag: 'tok-flag',
  path: 'tok-path',
  value: 'tok-string',
};

const ROLE_LABEL: Record<TokenRole, string> = {
  tool: 'satr.legend.tool',
  flag: 'satr.legend.flag',
  path: 'satr.legend.path',
  value: 'satr.arg',
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
}: {
  plan: PlanResponse | null;
  /** ما بثّته النواة من خرج الأداة في التشغيل الحالي. */
  stream?: readonly StreamLine[];
  phase?: StreamPhase;
  status?: string | undefined;
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
            <p className="t-caption satr__subtitle">{t('satr.subtitle')}</p>
          </div>
          <div className="satr__stage">
            <Preview plan={plan} />
            {/* مجرى التشغيل بعد الأمر ووسائطه وقبل الملاحظات: الأمرُ ما
                سيُنفَّذ، والمجرى ما قالته الأداة وهي تُنفَّذ، والملاحظات سياسةٌ
                ثابتة لا تتبدّل بتشغيل. فالترتيب زمنيّ: خطّة، فتنفيذ، فسياسة. */}
            <RunStream lines={stream} phase={phase} status={status} />
            <Notes />
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
      {/* خطٌّ فيه عقدة: سَطْرٌ لم يُكتب بعد. */}
      <span className="satr__rail-glyph" aria-hidden="true">
        <svg viewBox="0 0 24 24">
          <use href="#i-git-commit" />
        </svg>
      </span>
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

  async function copy() {
    await navigator.clipboard.writeText(shellCommand(plan.argv_display));
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  }

  // آخر وسيطين هما المصدر والمؤقّت. نعطيهما شرحًا خاصًا لأنهما بيانات لا رايات.
  const lastIndex = plan.explain.length - 1;
  const sourceIndex = lastIndex - 1;

  return (
    <>
      <div className="command">
        <div className="command__bar">
          <span className="command__label">{t('satr.title')}</span>
          <button
            type="button"
            className="btn btn--quiet btn--sm"
            onClick={copy}
            aria-live="polite"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href={copied ? '#i-check' : '#i-copy'} />
            </svg>
            {copied ? t('action.copied') : t('action.copy')}
          </button>
        </div>

        {/* سطر واحد للقراءة السريعة، بنفس ترتيب الوسائط ولونها. */}
        <div className="command__body">
          <span className="tok-prompt">$ </span>
          {plan.argv_display.map((token, i) => (
            <span key={i}>
              <span className={ROLE_CLASS[plan.explain[i]?.role ?? 'value']}>{token}</span>
              {i < plan.argv_display.length - 1 ? ' ' : ''}
            </span>
          ))}
        </div>
      </div>

      {/* الوسائط مفصولة: هنا لا يمكن أن تُقرأ مسافةٌ داخل اسمٍ فاصلًا.
          والشرح مطويّ في كلٍّ منها. كانت السبعة تُفتح معًا، فتصير اللوحة جدارًا
          من فقراتٍ عربية يعلو ألفَ بكسل — والأمر، وهو ما جاء المستخدم لأجله،
          يبقى في أعلاها سطرين. الآن الصفوف قائمةٌ تُمسح بالبصر (رقم، رمز، دور)
          ويُفتح منها ما يُسأل عنه. */}
      <h3 className="t-label satr__section">{t('satr.args.heading')}</h3>
      <ol className="args" aria-label="وسائط الأمر، كلٌّ على حدة">
        {plan.explain.map((tok, i) => {
          const notes = tokenNotes(tok.token);
          const label =
            i === sourceIndex
              ? 'explain.role.source'
              : i === lastIndex
                ? 'explain.role.temp'
                : tok.key;
          const line = (
            // الرقم والرمز والدور في صفٍّ واحد، والشرح تحته.
            //
            // كان الدور شقيقًا لكتلة الشرح كلّها، فكان يحاذي رأس صفٍّ يعلو
            // ثلاثة أسطر — وسمٌ صغير معلّق بعيدًا عن الرمز الذي يسمّيه. وفي
            // لوحةٍ أضيق صار ذلك يزاحم الرمز على العرض. موضعُه الصحيح طرفُ
            // سطر الرمز: هما معًا «هذا الوسيط، وهذا نوعه».
            <>
              <span className="arg__index lat" aria-hidden="true">
                {i === 0 ? '·' : i}
              </span>
              <code className={`arg__token ${ROLE_CLASS[tok.role]}`}>{tok.token}</code>
              <span className="arg__role t-caption">{t(ROLE_LABEL[tok.role])}</span>
            </>
          );

          // رمزٌ بلا شرحٍ ولا ملاحظة لا يُلفّ في `details`: قرصُ فتحٍ لا يفتح
          // شيئًا يَعِد بما ليس هناك.
          if (!label && notes.length === 0) {
            return (
              <li key={i}>
                <div className="arg">
                  <div className="arg__line">{line}</div>
                </div>
              </li>
            );
          }

          return (
            <li key={i}>
              {/* ‏`details` أصليّ لا زرٌّ وحالة: يعطي المفتاح والدور
                  و`aria-expanded` مجانًا، ويُفتح حين يبحث المتصفّح في الصفحة.
                  و`open` حين توجد ملاحظة: الملاحظة تنبيهٌ على محرفٍ مريب في
                  الاسم، وتنبيهٌ مطويّ تنبيهٌ لم يقع. */}
              <details className="arg" open={notes.length > 0}>
                <summary className="arg__line">
                  {line}
                  <svg className="arg__caret" viewBox="0 0 24 24" aria-hidden="true">
                    <use href="#i-chevron-down" />
                  </svg>
                </summary>
                <div className="arg__main">
                  {label && <p className="t-caption arg__note">{t(label)}</p>}
                  {notes.length > 0 && (
                    <p className="t-caption arg__flagged">
                      <svg viewBox="0 0 24 24" aria-hidden="true">
                        <use href="#i-info" />
                      </svg>
                      {notes.join(' · ')}
                    </p>
                  )}
                </div>
              </details>
            </li>
          );
        })}
      </ol>

    </>
  );
}

/**
 * سياسة التطبيق: قسمٌ مطويّ كلّه.
 *
 * ثلاث فقرات لا تتبدّل بين تخطيطٍ وآخر — تُقرأ مرّةً ويُرجَع إليها عند السؤال.
 * ومكانها آخر اللوحة لأن ترتيب اللوحة زمنيّ: أمرٌ سيُنفَّذ، فما قالته الأداة وهي
 * تُنفَّذ، فسياسةٌ ثابتة تحكم الاثنين. وفصلُها عن `Preview` هو ما يجعل هذا
 * الترتيب ممكنًا: كانت داخله، فكان كل ما يُضاف بعده يقع بعد السياسة.
 */
function Notes() {
  return (
    <details className="satr__notes">
      <summary className="t-label satr__section satr__section--fold">
        {t('satr.notes.heading')}
        <svg className="arg__caret" viewBox="0 0 24 24" aria-hidden="true">
          <use href="#i-chevron-down" />
        </svg>
      </summary>
      <div className="satr__notes-body">
        <p className="t-body-sec">{t('satr.no_shell')}</p>
        <p className="t-body-sec">{t('satr.promotion')}</p>
        <p className="t-caption satr__fineprint">{t('satr.copy_note')}</p>
      </div>
    </details>
  );
}
