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

export default function Satr({ plan }: { plan: PlanResponse | null }) {
  /**
   * هل الكتابة المائية محمولة؟
   *
   * لا يكفي `!plan`: نزعُ العنصر لحظةَ وصول الخطة يقطع الخبوّ قبل أن يبدأ.
   * فتبقى محمولةً بشفافية صفر طوالَ الانتقال ثم تُنزع — ولو وصلت الشاشة ومعها
   * خطة من البداية لم تُحمل أصلًا، فلا وميض في أول رسم.
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
    <section className="satr" aria-labelledby="satr-heading">
      <div className="satr__head">
        <h2 id="satr-heading" className="t-section-title">
          {t('app.satr')}
        </h2>
        {plan && <p className="t-caption satr__subtitle">{t('satr.subtitle')}</p>}
      </div>

      {/* مسرحٌ واحد للحالتين. ارتفاعه الأدنى يمنع العمود من الانكماش إلى سطرين
          ثم القفز إلى مئات البكسلات حين تصل الخطة — والقفزة تجرّ معها عمود
          «نَفِّذ» المجاور، فتبدو الشاشة كأنها انفجرت لأن المستخدم أكمل حقلًا. */}
      <div className="satr__stage">
        {idleShown && <Watermark leaving={plan !== null} />}
        {plan && <Preview plan={plan} />}
      </div>
    </section>
  );
}

/**
 * الكتابة المائية: ما يُنتظر، مكتوبًا في الفراغ نفسه.
 *
 * بلا حدّ ولا بطاقة ولا أرضية خاصة — صندوقٌ منقّط كان يعلن «هنا عنصر فارغ»،
 * وهذه الرسالة ليست عنصرًا بل حالة المساحة. والعنوان أوضح قليلًا من السطرين
 * تحته (لونٌ ومقاس، لا شفافية أعلى) كي تُقرأ الكتلة كعنوانٍ وشرحه لا كثلاثة
 * أسطر متساوية.
 *
 * **ليست `aria-hidden`.** هي التعليمة الوحيدة المعروضة في هذه المساحة، وإخفاؤها
 * عن قارئ الشاشة يترك من يقرأ بالصوت أمام منطقةٍ صامتة بلا سبب. وليست
 * `aria-live` كذلك: النصّ موجود منذ أول رسم ولا يتبدّل، فإعلانه مقاطعةٌ لا خبر.
 * نصٌّ ساكن في ترتيب المستند: يُقرأ حين يبلغه المستخدم، ولا يقفز إليه.
 */
function Watermark({ leaving }: { leaving: boolean }) {
  return (
    <div className={`satr__watermark${leaving ? ' satr__watermark--leaving' : ''}`}>
      <p className="satr__watermark-title">{t('satr.idle.title')}</p>
      <p className="satr__watermark-line">{t('satr.idle.line1')}</p>
      <p className="satr__watermark-line">{t('satr.idle.line2')}</p>
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

      {/* الوسائط مفصولة: هنا لا يمكن أن تُقرأ مسافةٌ داخل اسمٍ فاصلًا. */}
      <ol className="args" aria-label="وسائط الأمر، كلٌّ على حدة">
        {plan.explain.map((tok, i) => {
          const notes = tokenNotes(tok.token);
          const label =
            i === sourceIndex
              ? 'explain.role.source'
              : i === lastIndex
                ? 'explain.role.temp'
                : tok.key;
          return (
            <li className="arg" key={i}>
              <span className="arg__index lat" aria-hidden="true">
                {i === 0 ? '·' : i}
              </span>
              <div className="arg__main">
                <code className={`arg__token ${ROLE_CLASS[tok.role]}`}>{tok.token}</code>
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
              <span className="arg__role t-caption">{t(ROLE_LABEL[tok.role])}</span>
            </li>
          );
        })}
      </ol>

      <div className="satr__notes">
        <p className="t-body-sec">{t('satr.no_shell')}</p>
        <p className="t-body-sec">{t('satr.promotion')}</p>
        <p className="t-caption satr__fineprint">{t('satr.copy_note')}</p>
      </div>
    </>
  );
}
