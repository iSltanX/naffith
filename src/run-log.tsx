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
 *
 * ## والحذف: ما يمحوه وما لا يمحوه
 *
 * «حذف القيد» يمحو **الأثر** لا **الأثرَ على القرص**: أرشيفٌ أُنشئ يبقى في
 * مكانه بعد أن يُحذف سطرُه من السجل. هذا مكتوبٌ في نصّ التأكيد لا هنا وحده،
 * لأن الخلط بينهما يجعل المستخدم يظنّ أنه ينظّف قرصه وهو ينظّف تاريخه.
 */
import { useCallback, useEffect, useId, useMemo, useRef, useState } from 'react';
import { journalClear, journalDelete, recentRuns, reveal as revealRun } from './ipc';
import type { CategoryId, JournalEntry } from './ipc';
import { t } from './i18n';
import Select from './select';
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

// ── التصفية ────────────────────────────────────────────────────────────

export type StateFilter = 'all' | 'succeeded' | 'failed' | 'cancelled' | 'planned' | 'running';
export type PeriodFilter = 'all' | 'today' | 'week' | 'month';

const PERIOD_SECONDS: Record<Exclude<PeriodFilter, 'all'>, number> = {
  today: 24 * 60 * 60,
  week: 7 * 24 * 60 * 60,
  month: 30 * 24 * 60 * 60,
};

export interface Filters {
  category: 'all' | CategoryId;
  state: StateFilter;
  period: PeriodFilter;
}

export const NO_FILTERS: Filters = { category: 'all', state: 'all', period: 'all' };

/**
 * يصفّي القيود. دالّةٌ نقيّة مُصدَّرة كي تُختبر بلا رسم.
 *
 * `nowSeconds` مُمرَّر لا مقروءٌ من الساعة: اختبارُ «آخر أسبوع» بساعةٍ حقيقية
 * يمرّ اليوم ويسقط بعد أسبوع، وهو أسوأ أنواع الاختبارات.
 *
 * والقيد الذي لا يحمل قسمًا — كُتب قبل أن تُقيَّد الأقسام — يُعرض تحت «كل
 * الأقسام» ويُسقَط من أي تصفيةٍ بقسمٍ بعينه. البديل — عرضه في كل قسم — كان
 * سيجعل التصفية تكذب.
 */
export function applyFilters(
  entries: JournalEntry[],
  filters: Filters,
  nowSeconds: number,
): JournalEntry[] {
  return entries.filter((e) => {
    if (filters.category !== 'all' && e.category !== filters.category) return false;
    if (filters.state !== 'all' && e.state !== filters.state) return false;
    if (filters.period !== 'all') {
      const window = PERIOD_SECONDS[filters.period];
      if (!Number.isFinite(e.at) || nowSeconds - e.at > window) return false;
    }
    return true;
  });
}

type Load =
  | { status: 'loading' }
  | { status: 'loaded'; entries: JournalEntry[] }
  | { status: 'failed' };

interface Props {
  onBack: () => void;
  /** يملأ نموذج العملية بقيم قيدٍ سابق ثم ينتقل إليه. لا ينفّذ. */
  onRerun: (entry: JournalEntry) => void;
  /** يُخطر المنسّق أن السجل تغيّر، فيعيد قراءة «المستخدَمة حديثًا». */
  onChanged: () => void;
}

export default function RunLog(props: Props): JSX.Element {
  const { onBack, onRerun, onChanged } = props;

  const uid = useId();
  const headingId = `${uid}-heading`;
  const heading = useRef<HTMLHeadingElement>(null);

  const [load, setLoad] = useState<Load>({ status: 'loading' });
  /** يزداد عند «إعادة المحاولة» وبعد كل حذف، فيعيد إطلاق أثر القراءة. */
  const [attempt, setAttempt] = useState(0);
  const [filters, setFilters] = useState<Filters>(NO_FILTERS);
  const [confirmingClear, setConfirmingClear] = useState(false);

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

  const reload = useCallback(() => {
    setAttempt((n) => n + 1);
    onChanged();
  }, [onChanged]);

  const onDelete = useCallback(
    (runId: string) => {
      // الفشل لا يُعرض حوارًا: أسوأ ما يقع أن يبقى الصفّ، وإعادة القراءة تُظهر
      // ذلك بنفسها. حوارُ خطأٍ على فعلٍ ثانوي يقاطع أكثر مما يفيد.
      journalDelete(runId).finally(reload);
    },
    [reload],
  );

  const onClearAll = useCallback(() => {
    setConfirmingClear(false);
    journalClear().finally(reload);
  }, [reload]);

  // الأحدث أولًا: النواة تُلحق القيود، فآخر ما وقع آخر المصفوفة — وأولُ ما
  // يُسأل عنه. والتصفية قبل القصّ كي يُقصّ من المُصفّى لا من الكلّ.
  const nowSeconds = Math.floor(Date.now() / 1000);
  const all = load.status === 'loaded' ? load.entries : [];
  const shown = useMemo(
    () => applyFilters([...all].reverse(), filters, nowSeconds).slice(0, MAX_SHOWN),
    // `nowSeconds` عمدًا خارج الاعتماديات: إعادةُ الحساب مع كل ثانية تعيد رسم
    // القائمة كلها بلا سبب، والفرق بين ثانيةٍ وأخرى لا يغيّر «آخر أسبوع».
    [all, filters],
  );

  /** الأقسام التي يذكرها السجلّ فعلًا — لا كل أقسام المكتبة. */
  const categoriesPresent = useMemo(() => {
    const seen = new Set<CategoryId>();
    for (const e of all) if (e.category) seen.add(e.category);
    return [...seen];
  }, [all]);

  const filtered = filters !== NO_FILTERS && (
    filters.category !== 'all' || filters.state !== 'all' || filters.period !== 'all'
  );

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
        {all.length > 0 && (
          <button
            type="button"
            className="btn btn--quiet btn--sm log__clear"
            onClick={() => setConfirmingClear(true)}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-delete" />
            </svg>
            {t('log.clear')}
          </button>
        )}
      </header>

      {/* ── المرشّحات ───────────────────────────────────────────────
          `control-row` صنفٌ مشترك من `controls.css` لا تخطيطٌ خاصّ بهذه الشاشة:
          أي شاشةٍ تصفّ ضوابط متجاورة تأخذ الصفّ نفسه بفجوته ومقاساته والتفافه،
          فلا تُكتب ثلاثة صفوفٍ متشابهة في ثلاثة ملفّات. */}
      {all.length > 0 && (
        <div className="control-row log__filters" role="group" aria-label={t('log.filters')}>
          <Select
            label={t('log.filter.state')}
            value={filters.state}
            onChange={(v) => setFilters((f) => ({ ...f, state: v as StateFilter }))}
            options={[
              { value: 'all', label: t('log.filter.any') },
              { value: 'succeeded', label: t('log.state.succeeded') },
              { value: 'failed', label: t('log.state.failed') },
              { value: 'cancelled', label: t('log.state.cancelled') },
              { value: 'planned', label: t('log.state.planned') },
              { value: 'running', label: t('log.state.running') },
            ]}
          />
          {categoriesPresent.length > 1 && (
            <Select
              label={t('log.filter.category')}
              value={filters.category}
              onChange={(v) => setFilters((f) => ({ ...f, category: v as Filters['category'] }))}
              options={[
                { value: 'all', label: t('log.filter.any') },
                ...categoriesPresent.map((id) => ({ value: id, label: t(`cat.${id}.title`) })),
              ]}
            />
          )}
          <Select
            label={t('log.filter.period')}
            value={filters.period}
            onChange={(v) => setFilters((f) => ({ ...f, period: v as PeriodFilter }))}
            options={[
              { value: 'all', label: t('log.filter.any') },
              { value: 'today', label: t('log.filter.today') },
              { value: 'week', label: t('log.filter.week') },
              { value: 'month', label: t('log.filter.month') },
            ]}
          />
          {filtered && (
            <button
              type="button"
              className="btn btn--ghost btn--sm"
              onClick={() => setFilters(NO_FILTERS)}
            >
              {t('log.filter.reset')}
            </button>
          )}
        </div>
      )}

      {/* منطقة الحالة حيّة: القراءة غير متزامنة، وما يظهر بعدها (فراغٌ أو
          تعذّرٌ) تغيّرٌ يستحقّ أن يُنطق لا أن يُرى وحده. */}
      <div className="log__status" role="status" aria-busy={load.status === 'loading'}>
        {load.status === 'loading' && <span className="spinner" aria-hidden="true" />}

        {load.status === 'loaded' && shown.length === 0 && (
          <p className="t-body-sec">{filtered ? t('log.empty.filtered') : t('log.empty')}</p>
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
            <Entry
              key={`${entry.id}:${entry.state}:${i}`}
              entry={entry}
              onRerun={onRerun}
              onDelete={onDelete}
            />
          ))}
        </ol>
      )}

      {confirmingClear && (
        <ConfirmClear onCancel={() => setConfirmingClear(false)} onConfirm={onClearAll} />
      )}
    </section>
  );
}

function Entry({
  entry,
  onRerun,
  onDelete,
}: {
  entry: JournalEntry;
  onRerun: (entry: JournalEntry) => void;
  onDelete: (runId: string) => void;
}): JSX.Element {
  // نصٌّ لا `JournalState`: قيمةٌ لا تعرفها هذه النسخة يجب أن تمرّ لا أن تختفي.
  const state: string = entry.state;
  const when = whenOf(entry.at);
  // عنوان العملية إن عرفته هذه النسخة. `t` يعيد المفتاح نفسه حين يغيب، وهو
  // نصٌّ لاتينيّ لا يصلح عنوانًا — فالمقارنة تفرّق بين ترجمةٍ وغيابها، ويبقى
  // المعرّف معروضًا في الحالتين لأنه ما يربط الصفّ بالنواة.
  const titleKey = `op.${entry.op_id}.title`;
  const title = t(titleKey);
  const named = title !== titleKey;

  // «أعد بهذه القيم» لا يظهر إلا حين يكون له معنى: قيدٌ بلا مدخلات محفوظة —
  // كُتب قبل أن تُقيَّد المدخلات، أو لعمليةٍ بلا حقول — لا شيء يُعاد به.
  const canRerun = (entry.inputs?.length ?? 0) > 0;
  const produced = typeof entry.produced === 'string' && entry.produced !== '';

  const onReveal = useCallback(() => {
    // الفشل صامت: أسوأ ما يقع أن الناتج نُقل أو حُذف بعد التشغيل، وهي حالةٌ
    // يفهمها المستخدم من Finder نفسه أسرع مما يفهمها من حوارٍ هنا.
    revealRun(entry.id).catch(() => {});
  }, [entry.id]);

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

        {entry.category && (
          <span className="chip chip--neutral entry__cat">{t(`cat.${entry.category}.title`)}</span>
        )}

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

      {/* آخر ما طبعته الأداة. النواة تحفظ اثني عشر سطرًا مقصوصة لا الخرج كلّه —
          انظر `MAX_TAIL_LINES`. ومراجعةُ فشلٍ بلا سطرٍ واحد ممّا قالته الأداة
          ليست مراجعة: يبقى «رمز الخروج ‎1‎» ولا شيء غيره. */}
      {(entry.tail?.length ?? 0) > 0 && (
        <details className="entry__tail">
          <summary className="t-caption">{t('log.tail')}</summary>
          <pre className="entry__tail-body" dir="ltr">
            {entry.tail?.join('\n')}
          </pre>
        </details>
      )}

      {/* المعرّف يُعرض دائمًا وإن عُرض العنوان فوقه: العنوان نصٌّ قد يتغيّر
          بتغيّر الصياغة، والمعرّف هو ما يربط الصفّ بالنواة وبالسجل على القرص.
          ولاتينيّته تحتاج عزلًا وإلا التصقت نقطته بالعربية حوله. */}
      <p className="t-caption entry__meta">
        <bdi dir="ltr" className="entry__op">
          {entry.op_id}
        </bdi>
        {typeof entry.duration_ms === 'number' && <bdi>{durationText(entry.duration_ms)}</bdi>}
        {typeof entry.code === 'number' && (
          <bdi>
            {t('state.failed.code')} <span className="num">{entry.code}</span>
          </bdi>
        )}
      </p>

      <div className="entry__actions">
        {canRerun && (
          <button type="button" className="btn btn--quiet btn--sm" onClick={() => onRerun(entry)}>
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-execute" />
            </svg>
            {t('log.rerun')}
          </button>
        )}
        {produced && (
          <button type="button" className="btn btn--quiet btn--sm" onClick={onReveal}>
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-folder-open" />
            </svg>
            {t('action.reveal')}
          </button>
        )}
        <button
          type="button"
          className="btn btn--ghost btn--sm entry__delete"
          onClick={() => onDelete(entry.id)}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-delete" />
          </svg>
          {t('log.delete')}
        </button>
      </div>
    </li>
  );
}

/**
 * تأكيد مسح السجلّ كلّه.
 *
 * حوارٌ مقاطع لا زرٌّ مباشر: الفعل لا يُتراجَع عنه. و«الإلغاء» هو الفعل
 * الافتراضي — يأخذ التركيز — لأن الخطأ في اتجاه الإبقاء غير مكلف.
 *
 * ونصُّه يفرّق صراحةً بين الأثر وما أنتجه: من يمسح سجلّه لا يمسح ملفاته.
 */
function ConfirmClear({ onCancel, onConfirm }: { onCancel: () => void; onConfirm: () => void }) {
  const cancel = useRef<HTMLButtonElement>(null);
  const uid = useId();

  useEffect(() => {
    const opener = document.activeElement;
    cancel.current?.focus();
    return () => {
      if (opener instanceof HTMLElement && opener.isConnected) opener.focus();
    };
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        event.preventDefault();
        onCancel();
      }
    }
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, [onCancel]);

  return (
    <div className="scrim" role="presentation" onClick={onCancel}>
      <div
        className="dialog card"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={`${uid}-t`}
        aria-describedby={`${uid}-b`}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id={`${uid}-t`} className="t-section-title dialog__title">
          {t('log.clear.title')}
        </h2>
        <p id={`${uid}-b`} className="t-body-sec dialog__body">
          {t('log.clear.body')}
        </p>
        <div className="dialog__actions">
          <button ref={cancel} type="button" className="btn btn--primary" onClick={onCancel}>
            {t('log.clear.cancel')}
          </button>
          <button type="button" className="btn btn--quiet" onClick={onConfirm}>
            {t('log.clear.confirm')}
          </button>
        </div>
      </div>
    </div>
  );
}
