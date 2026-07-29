/**
 * سجلّ التشغيل — الأثر القابل للمراجعة.
 *
 * ## لماذا لا تُجمع قيود التشغيل الواحد في صفّ
 *
 * النواة تكتب قيدًا لكل انتقال: `planned` عند حجز الخطة، ثم `running`، ثم
 * النتيجة. فالتشغيل الواحد يظهر هنا صفوفًا لا صفًّا. ضمُّها في صفٍّ واحد يبدو
 * أنظف ويخفي ما جاء المستخدم من أجله: خطةٌ رُوجعت ثم تُركت (‏`planned` بلا ما
 * بعده) حدثٌ حقيقي، وطيُّه داخل «لم يُنفَّذ شيء» يمحو الفرق بين «لم يُطلب» و«طُلب
 * ولم يقع». السجل مراجعةٌ لا ملخّص.
 *
 * ## القيد العالق في «جارية»
 *
 * إن قُتل التطبيق في منتصف تشغيل بقي آخر قيدٍ عند `running` إلى الأبد. لا
 * نستنتج له نهاية: عرضُه ناجحًا كذبٌ عن القرص، وعرضُه فاشلًا كذبٌ عن الأداة
 * التي ربما أتمّت عملها. يُعرض كما هو — «جارية» — بشارةٍ ساكنة لا دوّار. الدوّار
 * يعني «يحدث الآن أمامك»، وهذا قيدٌ تاريخيّ لا يحدث شيء خلفه.
 *
 * ## لماذا يقصّ العرض
 *
 * النواة تحتفظ اليوم بمئتي قيد في الذاكرة، لكن هذا رقمُها هي وقد يتغيّر. سقفُ
 * العرض هنا يحمي التخطيط من قائمةٍ بآلاف الصفوف بصرف النظر عمّا يصل، والقائمة
 * تُمرَّر داخل صندوقها فلا يخرج «رجوع» من المرأى.
 */
import { useEffect, useId, useRef, useState } from 'react';
import { recentRuns } from './ipc';
import type { JournalEntry } from './ipc';
import { t } from './i18n';
import './shell-screens.css';

/** أقصى ما يُرسم من قيود. انظر تعليق الرأس. */
const MAX_SHOWN = 200;

/**
 * الخرائط بمفاتيح نصّية لا `JournalState`.
 *
 * النواة قد تعلن حالةً سادسة قبل أن تعرفها هذه الواجهة، ونوعٌ مغلق يجعل
 * القيمة الجديدة `undefined` صامتة في الصفّ. الاحتياط هنا يعطيها شكلًا محايدًا،
 * و`t` يعرض مفتاحها كما هو — فيبقى الغياب مرئيًا لا مطويًّا.
 */
const STATE_CHIP: Record<string, string> = {
  planned: 'chip--neutral',
  running: 'chip--info',
  succeeded: 'chip--success',
  failed: 'chip--danger',
  cancelled: 'chip--neutral',
};

const STATE_ICON: Record<string, string> = {
  planned: '#i-pending',
  running: '#i-execute',
  succeeded: '#i-success',
  failed: '#i-error',
  cancelled: '#i-cancelled',
};

/**
 * التاريخ بالعربية من `Intl` لا بيدنا.
 *
 * صياغةُ تاريخٍ بأسماء شهورٍ مكتوبة في الشيفرة تُنتج نصًّا عربيًّا خارج
 * `i18n.ts`، وتُخطئ في الأرقام والفواصل وترتيبها. و`dateStyle: 'medium'` لا
 * `full`: القيد صفٌّ في قائمة، واسم اليوم كاملًا يزاحم ما جاء المستخدم يقرؤه.
 */
const WHEN = new Intl.DateTimeFormat('ar', { dateStyle: 'medium', timeStyle: 'short' });

/**
 * المدّة بوحدتها المكتوبة من `Intl` كذلك — «ثانية» و«دقيقة» تأتيان من المحليّة
 * لا من نصٍّ هنا. والوحدة تُختار بالحجم: «‏١٤٠ ثانية» رقمٌ يحتاج قسمةً ذهنية.
 */
const SECONDS = new Intl.NumberFormat('ar', {
  style: 'unit',
  unit: 'second',
  unitDisplay: 'long',
  maximumFractionDigits: 1,
});
const MINUTES = new Intl.NumberFormat('ar', {
  style: 'unit',
  unit: 'minute',
  unitDisplay: 'long',
  maximumFractionDigits: 1,
});

function durationText(ms: number): string {
  return ms >= 60_000 ? MINUTES.format(ms / 60_000) : SECONDS.format(ms / 1000);
}

/**
 * `at` **ثوانٍ** منذ الحقبة لا أجزاء ألفٍ من الثانية.
 *
 * هذا مذكور في `journal.rs` وحده، والنوع في `ipc.ts` رقمٌ لا يفرّق. تمريرُه إلى
 * `Date` كما هو يضع كل تشغيل في كانون الثاني ١٩٧٠ — خطأٌ لا يكشفه المُصرِّف ولا
 * يبدو خطأً في الشاشة: تاريخٌ معقول الشكل، خاطئ تمامًا.
 */
function whenOf(atSeconds: number): { text: string; iso: string } | null {
  if (!Number.isFinite(atSeconds)) return null;
  const date = new Date(atSeconds * 1000);
  if (Number.isNaN(date.getTime())) return null;
  return { text: WHEN.format(date), iso: date.toISOString() };
}

type Load =
  | { status: 'loading' }
  | { status: 'loaded'; entries: JournalEntry[] }
  | { status: 'failed' };

export default function RunLog(props: { onBack: () => void }): JSX.Element {
  const { onBack } = props;

  const uid = useId();
  const headingId = `${uid}-heading`;
  const heading = useRef<HTMLHeadingElement>(null);

  const [load, setLoad] = useState<Load>({ status: 'loading' });
  /** يزداد عند «إعادة المحاولة» فيعيد إطلاق أثر القراءة. */
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    heading.current?.focus();
  }, []);

  useEffect(() => {
    // الشاشة قد تُغادَر قبل أن يردّ الوعد، فكتابةُ الحالة بعدها تحذيرٌ في
    // الطرفية وعملٌ بلا فائدة.
    let live = true;
    setLoad({ status: 'loading' });
    recentRuns()
      .then((entries) => {
        if (!live) return;
        setLoad({ status: 'loaded', entries: Array.isArray(entries) ? entries : [] });
      })
      .catch(() => {
        // سبب الفشل لا يُعرض: تعذّر الوصول إلى النواة ليس خطأً يُصلحه المستخدم،
        // ومفتاحٌ تقنيّ في وجهه ضجيج. `log.failed` يقول ما يعنيه، و«إعادة
        // المحاولة» هي كل ما يمكن فعله.
        if (live) setLoad({ status: 'failed' });
      });
    return () => {
      live = false;
    };
  }, [attempt]);

  // الأحدث أولًا: النواة تُلحق القيود، فآخر ما وقع آخر المصفوفة — وأولُ ما
  // يُسأل عنه. والقصّ بعد العكس كي يُقصّ الأقدم لا الأحدث.
  const shown =
    load.status === 'loaded' ? [...load.entries].reverse().slice(0, MAX_SHOWN) : [];

  return (
    <section className="screen log" aria-labelledby={headingId}>
      <header className="screen__head">
        <button type="button" className="btn btn--quiet btn--sm screen__back" onClick={onBack}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-chevron" />
          </svg>
          {t('nav.back')}
        </button>
        <h2 id={headingId} className="t-page-title screen__title" tabIndex={-1} ref={heading}>
          {t('log.title')}
        </h2>
      </header>

      {/* منطقة الحالة حيّة: القراءة غير متزامنة، وما يظهر بعدها (فراغٌ أو
          تعذّرٌ) تغيّرٌ يستحقّ أن يُنطق لا أن يُرى وحده. */}
      <div className="log__status" role="status" aria-busy={load.status === 'loading'}>
        {load.status === 'loading' && <span className="spinner" aria-hidden="true" />}

        {load.status === 'loaded' && shown.length === 0 && (
          <p className="t-body-sec">{t('log.empty')}</p>
        )}

        {load.status === 'failed' && (
          <>
            <div className="notice notice--warning">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <use href="#i-warning" />
              </svg>
              <p>{t('log.failed')}</p>
            </div>
            <button
              type="button"
              className="btn btn--quiet btn--sm"
              onClick={() => setAttempt((n) => n + 1)}
            >
              {t('ops.retry')}
            </button>
          </>
        )}
      </div>

      {shown.length > 0 && (
        <ol className="log__list">
          {shown.map((entry, i) => (
            // المفتاح ليس `entry.id` وحده: قيود التشغيل الواحد تتشارك المعرّف
            // (‏planned ثم running ثم النتيجة)، فمفتاحٌ به وحده يتكرّر ويُربك
            // المطابقة. المركّب يميّز الصفّ داخل القائمة المقصوصة.
            <Entry key={`${entry.id}:${entry.state}:${i}`} entry={entry} />
          ))}
        </ol>
      )}
    </section>
  );
}

function Entry({ entry }: { entry: JournalEntry }): JSX.Element {
  // نصٌّ لا `JournalState`: قيمةٌ لا تعرفها هذه النسخة يجب أن تمرّ لا أن تختفي.
  const state: string = entry.state;
  const when = whenOf(entry.at);
  // عنوان العملية إن عرفته هذه النسخة. `t` يعيد المفتاح نفسه حين يغيب، وهو
  // نصٌّ لاتينيّ لا يصلح عنوانًا — فالمقارنة تفرّق بين ترجمةٍ وغيابها، ويبقى
  // المعرّف معروضًا في الحالتين لأنه ما يربط الصفّ بالنواة.
  const titleKey = `op.${entry.op_id}.title`;
  const title = t(titleKey);
  const named = title !== titleKey;

  return (
    <li className={`entry entry--${state}`}>
      <div className="entry__head">
        <span className={`chip ${STATE_CHIP[state] ?? 'chip--neutral'}`}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href={STATE_ICON[state] ?? '#i-info'} />
          </svg>
          {t(`log.state.${state}`)}
        </span>

        {named && <p className="t-card-title entry__title">{title}</p>}

        {when && (
          <time className="t-caption entry__time" dateTime={when.iso}>
            <bdi>{when.text}</bdi>
          </time>
        )}
      </div>

      {/* الأمر كما شُغّل. سطحه سطح «سَطْر» نفسه لأنه المادّة نفسها. */}
      <div className="command">
        <div className="command__body" dir="ltr">
          <span className="tok-prompt">$ </span>
          <bdi dir="ltr" className="tok-name">
            {entry.program}
          </bdi>
          {entry.args.map((arg, i) => (
            <span key={i}>
              {' '}
              {/* كل وسيط معزول على حدة لا الأمرُ كتلةً: وسيطٌ يحمل نصًّا عربيًّا
                  (اسم ملف مثلًا) بين وسيطين لاتينيّين يقفز إلى طرف السطر إن
                  كان العزل حول الكتلة وحدها. */}
              <bdi dir="ltr" className="tok-string">
                {arg}
              </bdi>
            </span>
          ))}
        </div>
      </div>

      {/* المعرّف يُعرض دائمًا وإن عُرض العنوان فوقه: العنوان نصٌّ قد يتغيّر
          بتغيّر الصياغة، والمعرّف هو ما يربط الصفّ بالنواة وبالسجل على القرص.
          ولاتينيّته تحتاج عزلًا وإلا التصقت نقطته بالعربية حوله. */}
      <p className="t-caption entry__meta">
        <bdi dir="ltr" className="entry__op">
          {entry.op_id}
        </bdi>
        {typeof entry.duration_ms === 'number' && (
          <bdi>{durationText(entry.duration_ms)}</bdi>
        )}
      </p>
    </li>
  );
}
