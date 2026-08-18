/**
 * سجلّ التشغيل — الأثر القابل للمراجعة.
 *
 * ## قيدٌ واحد لكل دورة حياة
 *
 * النواة تكتب قيدًا لكل انتقال: `planned` عند حجز الخطة، ثم `running`، ثم
 * النتيجة. الواجهة تطوي هذه الانتقالات بمعرّف التشغيل، وتعرض آخر ما قيّدته
 * النواة. تبقى الخطة المتروكة `planned` والتشغيلة العالقة `running` مرئيتين؛ فلا يُستنتج
 * لهما انتقالٌ لم يصل على السلك.
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
 * ## والإلغاء: ما يمحوه وما لا يمحوه
 *
 * «إلغاء» الخطة المتروكة يمحو **قيد المعاينة** فقط؛ فلم يبدأ تشغيلٌ يحتاج
 * رمز إلغاء. أمّا «إيقاف» القيد الجاري فيمرّ بقدرة الإلغاء الموجودة. المسح الكامل
 * وحده يأخذ تأكيدًا؛ ولا يمحو ملفات النتائج من القرص.
 */
import { useCallback, useEffect, useId, useMemo, useRef, useState } from 'react';
import {
  cancel as cancelRun,
  journalClear,
  journalDelete,
  onRunFinished,
  recentRuns,
  reveal as revealRun,
} from './ipc';
import type {
  JournalEntry,
  RawOutputLine,
  StructuredLine,
} from './ipc';
import { t } from './i18n';
import { fold } from './library';
import StatePanel from './state-panel';
import './utility-screen.css';
import './run-log.css';

/** أقصى ما يُرسم من قيود. انظر تعليق الرأس. */
const MAX_SHOWN = 200;

/**
 * الخرائط بمفاتيح نصّية لا `JournalState`.
 *
 * النواة قد تعلن حالةً سادسة قبل أن تعرفها هذه الواجهة، ونوعٌ مغلق يجعل
 * القيمة الجديدة `undefined` صامتة في الصفّ. الاحتياط هنا يعطيها شكلًا محايدًا
 * واسمًا مفهومًا، بينما يبقى معرّف العملية والأمر التقني معروضين للمراجعة.
 */
const KNOWN_STATES = new Set(['planned', 'running', 'succeeded', 'failed', 'cancelled']);

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
const HOURS = new Intl.NumberFormat('ar', {
  style: 'unit',
  unit: 'hour',
  unitDisplay: 'long',
  maximumFractionDigits: 1,
});
const DAYS = new Intl.NumberFormat('ar', {
  style: 'unit',
  unit: 'day',
  unitDisplay: 'long',
  maximumFractionDigits: 1,
});

function durationText(ms: number): string {
  if (ms >= 86_400_000) return DAYS.format(ms / 86_400_000);
  if (ms >= 3_600_000) return HOURS.format(ms / 3_600_000);
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

/**
 * يطوي انتقالات دورة الحياة إلى آخر قيد لكل معرّف.
 *
 * الترتيب هو ترتيب الإلحاق في السجل، لا `at`: حدثان في الثانية نفسها
 * يشتركان الطابع الزمني، والقيد الأخير في JSONL هو الحقيقة الأحدث. وتبقى
 * الخطط والتشغيلات العالقة لأنها هي آخر قيد لمعرّفها.
 */
export function coalesceRuns(entries: JournalEntry[]): JournalEntry[] {
  const latest = new Map<string, { entry: JournalEntry; index: number }>();
  entries.forEach((entry, index) => latest.set(entry.id, { entry, index }));
  return [...latest.values()]
    .sort((left, right) => left.index - right.index)
    .map(({ entry }) => entry);
}

// ── التصفية ────────────────────────────────────────────────────────────

export type StateFilter = 'all' | 'succeeded' | 'failed' | 'cancelled';
export type PeriodFilter = 'all' | 'today' | 'week' | 'month';

const PERIOD_SECONDS: Record<Exclude<PeriodFilter, 'all'>, number> = {
  today: 24 * 60 * 60,
  week: 7 * 24 * 60 * 60,
  month: 30 * 24 * 60 * 60,
};

export interface Filters {
  state: StateFilter;
  period: PeriodFilter;
}

export const NO_FILTERS: Filters = { state: 'all', period: 'all' };

/**
 * يصفّي القيود. دالّةٌ نقيّة مُصدَّرة كي تُختبر بلا رسم.
 *
 * `nowSeconds` مُمرَّر لا مقروءٌ من الساعة: اختبارُ «آخر أسبوع» بساعةٍ حقيقية
 * يمرّ اليوم ويسقط بعد أسبوع، وهو أسوأ أنواع الاختبارات.
 *
 * حالات التخطيط والتشغيل تبقى مرئية تحت «الكل»؛ شاشة Figma لا تمنحهما مرشّحًا
 * منفصلًا لأنهما انتقالان تاريخيان نادران وليسا نتيجتين نهائيتين.
 */
export function applyFilters(
  entries: JournalEntry[],
  filters: Filters,
  nowSeconds: number,
): JournalEntry[] {
  return entries.filter((e) => {
    if (filters.state !== 'all' && e.state !== filters.state) return false;
    if (filters.period !== 'all') {
      const window = PERIOD_SECONDS[filters.period];
      if (!Number.isFinite(e.at) || nowSeconds - e.at > window) return false;
    }
    return true;
  });
}

/**
 * بحثٌ على ما يملكه القيد فعلًا، بلا تخمينٍ من خرج الأداة. طيُّ العربية هو
 * نفسه المستعمل في المكتبة، والمسارات تبقى نصوصًا للبحث لا حقولًا مستنتجة.
 */
export function matchesQuery(entry: JournalEntry, query: string): boolean {
  const needle = fold(query.trim());
  if (!needle) return true;
  const result = entry.result;
  const structured = result ? resultSearchValues(result) : [];
  const values = [
    entry.op_id,
    entry.category ?? '',
    entry.program,
    ...entry.args,
    entry.cwd ?? '',
    entry.produced ?? '',
    entry.reason ?? '',
    ...(entry.tail ?? []),
    ...(entry.inputs ?? []).flatMap((input) => [input.id, input.value ?? '']),
    ...structured,
  ];
  return fold(values.join(' ')).includes(needle);
}

/** Search the typed payload as data; never reinterpret stdout into structure. */
function resultSearchValues(result: NonNullable<JournalEntry['result']>): string[] {
  const textLines = (lines: RawOutputLine[]) =>
    lines.flatMap((line) =>
      line.stream === 'stdout' || line.stream === 'stderr' ? [line.line] : [],
    );
  const structuredLines = (lines: StructuredLine[]) => lines.map((line) => line.value);

  switch (result.type) {
    case 'artifact':
      return [result.path, ...textLines(result.output ?? [])];
    case 'acknowledgement':
      return [result.message_key, ...structuredLines(result.details)];
    case 'collection':
      return [...result.columns, ...result.rows.flatMap((row) => row.cells)];
    case 'properties_report':
      return result.properties.flatMap((property) => [t(property.label_key), property.value]);
    case 'metrics':
      return result.metrics.flatMap((metric) => [t(metric.label_key), metric.value, metric.unit ?? '']);
    case 'digest':
      return [result.algorithm, result.value ?? '', ...structuredLines(result.details)];
    case 'comparison':
      return [
        result.reference ?? '',
        result.comparison ?? '',
        ...structuredLines(result.details),
      ];
    case 'verdict':
      return [result.value, ...structuredLines(result.details)];
    case 'diff_search':
      return structuredLines(result.items);
    case 'diagnostic':
    case 'raw_output':
      return textLines(result.lines);
  }
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
  const [mutationError, setMutationError] = useState<string | null>(null);
  const [stoppingRuns, setStoppingRuns] = useState<ReadonlySet<string>>(() => new Set());
  const [acknowledgedUnknownRuns, setAcknowledgedUnknownRuns] = useState<ReadonlySet<string>>(
    () => new Set(),
  );

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

  // إن انتهى تشغيلٌ بينما السجل مفتوح، تصل حالته النهائية ثم يُعاد طيّ دورته.
  // هذا هو ما ينقل صفّ «جارية» الذي أوقفه المستخدم إلى «أُلغيت» بلا تخمين
  // متفائل في الواجهة.
  useEffect(() => {
    let live = true;
    let dispose: (() => void) | undefined;
    onRunFinished((event) => {
      if (!live) return;
      setStoppingRuns((current) => {
        if (!current.has(event.run_id)) return current;
        const next = new Set(current);
        next.delete(event.run_id);
        return next;
      });
      reload();
    })
      .then((unlisten) => {
        if (live) dispose = unlisten;
        else unlisten();
      })
      // تعذّر تسجيل المستمع لا يمنع قراءة السجل أو أفعاله؛ إعادة فتح الشاشة
      // تعيد قراءته من القرص في كل الأحوال.
      .catch(() => {});
    return () => {
      live = false;
      dispose?.();
    };
  }, [reload]);

  const onCancelPreview = useCallback((entry: JournalEntry) => {
    setMutationError(null);
    journalDelete(entry.id)
      .then(reload)
      .catch(() => setMutationError(t('log.preview.cancel.failed')));
  }, [reload]);

  const onStop = useCallback((entry: JournalEntry) => {
    setMutationError(null);
    setStoppingRuns((current) => new Set(current).add(entry.id));
    cancelRun(entry.id).catch(() => {
      setStoppingRuns((current) => {
        const next = new Set(current);
        next.delete(entry.id);
        return next;
      });
      setMutationError(t('log.stop.failed'));
    });
  }, []);

  const onReveal = useCallback((entry: JournalEntry) => {
    setMutationError(null);
    revealRun(entry.id).catch(() => setMutationError(t('err.reveal.failed')));
  }, []);

  const onDelete = useCallback((entry: JournalEntry) => {
    setMutationError(null);
    journalDelete(entry.id)
      .then(reload)
      .catch(() => setMutationError(t('log.delete.failed')));
  }, [reload]);

  const onClearAll = useCallback(() => {
    setConfirmingClear(false);
    setMutationError(null);
    journalClear()
      .then(reload)
      .catch(() => setMutationError(t('log.clear.failed')));
  }, [reload]);

  // الأحدث أولًا: النواة تُلحق القيود، فآخر ما وقع آخر المصفوفة — وأولُ ما
  // يُسأل عنه. والتصفية قبل القصّ كي يُقصّ من المُصفّى لا من الكلّ.
  const nowSeconds = Math.floor(Date.now() / 1000);
  const all = useMemo(
    () => (load.status === 'loaded' ? coalesceRuns(load.entries) : []),
    [load],
  );
  const matched = useMemo(
    () => applyFilters([...all].reverse(), filters, nowSeconds),
    // `nowSeconds` عمدًا خارج الاعتماديات: إعادةُ الحساب مع كل ثانية تعيد رسم
    // القائمة كلها بلا سبب، والفرق بين ثانيةٍ وأخرى لا يغيّر «آخر أسبوع».
    [all, filters],
  );
  const shown = matched.slice(0, MAX_SHOWN);
  const gatedUnknown = shown.filter(
    (entry) =>
      visualStateOf(String(entry.state)) === 'unknown' && !acknowledgedUnknownRuns.has(entry.id),
  );
  const visibleShown = shown.filter(
    (entry) =>
      visualStateOf(String(entry.state)) !== 'unknown' || acknowledgedUnknownRuns.has(entry.id),
  );
  // عقد النواة لا يحمل عدّادًا إجماليًّا؛ وصول السقف نفسه هو الإشارة الوحيدة
  // الآمنة إلى أن الأقدم قد لا يكون ضمن الحمولة. لذلك لا نخفي حالة Page 18
  // عند 200 بالضبط منتظرين عنصرًا 201 لن يصل أصلًا من `recent_runs`.
  const capped = matched.length >= MAX_SHOWN;

  const filtered = filters.state !== 'all' || filters.period !== 'all';

  const resetFilters = useCallback(() => {
    setFilters(NO_FILTERS);
  }, []);

  return (
    <section className="screen log" aria-labelledby={headingId}>
      <header className="screen__head">
        <div className="log__heading-copy">
          <h2 id={headingId} className="t-page-title screen__title" tabIndex={-1} ref={heading}>
            {t('log.title')}
          </h2>
          <p className="t-body-sec log__subtitle">{t('log.subtitle')}</p>
        </div>
        {all.length > 0 && (
          <button
            type="button"
            className="btn log__clear"
            onClick={() => setConfirmingClear(true)}
          >
            {t('log.clear')}
          </button>
        )}
      </header>

      {mutationError && (
        <div className="notice notice--warning" role="status" aria-live="polite">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-warning" />
          </svg>
          <p>{mutationError}</p>
        </div>
      )}

      {/* مرشّحات Page 18 غير تدميرية: أربع شرائح للحالة وشريحةٌ واحدة للمدّة.
          البحث والقوائم المنسدلة ليسا في شاشة الإنتاج المعتمدة، فلا نضيف
          سطحًا موازيًا إلى مواصفة Figma. الضغط على شريحة المدّة مرةً ثانية
          يلغيها، بينما «الكل» يعيد الحالة وحدها. */}
      {all.length > 0 && (
        <div className="log__filters" role="group" aria-label={t('log.filters')}>
          {(
            [
              ['all', 'log.filter.any'],
              ['succeeded', 'log.filter.succeeded'],
              ['failed', 'log.filter.failed'],
              ['cancelled', 'log.filter.cancelled'],
            ] as const
          ).map(([value, label]) => (
            <button
              key={value}
              type="button"
              className={`filter-chip${filters.state === value ? ' filter-chip--selected' : ''}`}
              aria-pressed={filters.state === value}
              onClick={() => setFilters((current) => ({ ...current, state: value }))}
            >
              {t(label)}
            </button>
          ))}
          <button
            type="button"
            className={`filter-chip filter-chip--period${filters.period === 'month' ? ' filter-chip--selected' : ''}`}
            aria-pressed={filters.period === 'month'}
            onClick={() =>
              setFilters((current) => ({
                ...current,
                period: current.period === 'month' ? 'all' : 'month',
              }))
            }
          >
            {t('log.filter.month')}
          </button>
        </div>
      )}

      {/* كل حالة تملك دلالتها الحيّة داخل مكوّنها: StatePanel يعلن التحميل
          والعطل، وحالة الفراغ تعلن نفسها. تجنّبُ role على الغلاف يمنع مناطق
          status متداخلة حين يكون العطل alert. */}
      <div className="log__status" aria-busy={load.status === 'loading'}>
        {load.status === 'loading' && (
          <StatePanel
            title={t('log.loading')}
            body={t('log.loading.body')}
            busy
            headingLevel={2}
          />
        )}

        {load.status === 'loaded' && shown.length === 0 && (
          <div className="log-empty" role="status">
            <h2 className="t-section-title">
              {filtered ? t('log.empty.filtered') : t('log.empty')}
            </h2>
            <p className="t-body-sec">
              {filtered ? t('log.empty.filtered.body') : t('log.empty.body')}
            </p>
            <button
              type="button"
              className={filtered ? 'btn btn--quiet' : 'btn btn--primary'}
              onClick={filtered ? resetFilters : onBack}
            >
              {filtered ? t('log.filter.reset') : t('log.start')}
            </button>
          </div>
        )}

        {load.status === 'failed' && (
          <StatePanel
            title={t('log.failed')}
            body={t('log.failed.body')}
            tone="danger"
            action={t('ops.retry')}
            onAction={() => setAttempt((n) => n + 1)}
            headingLevel={2}
          />
        )}
      </div>

      {shown.length > 0 && (
        <>
          {capped && (
            <aside className="log-cap" role="status" aria-labelledby={`${uid}-cap-title`}>
              <h2 id={`${uid}-cap-title`} className="t-section-title">{t('log.cap.title')}</h2>
              <p className="t-body-sec">{t('log.cap.body')}</p>
            </aside>
          )}
          {gatedUnknown.length > 0 && (
            <aside
              className="log-unknown"
              role="status"
              aria-labelledby={`${uid}-unknown-title`}
            >
              <h2 id={`${uid}-unknown-title`} className="t-section-title">
                {t('log.unknown.title')}
              </h2>
              <p className="t-body-sec">{t('log.unknown.body')}</p>
              <button
                type="button"
                className="btn btn--primary"
                onClick={() =>
                  setAcknowledgedUnknownRuns((current) => {
                    const next = new Set(current);
                    gatedUnknown.forEach((entry) => next.add(entry.id));
                    return next;
                  })
                }
              >
                {t('log.unknown.continue')}
              </button>
            </aside>
          )}
          {visibleShown.length > 0 && <ol className="log__list">
            {visibleShown.map((entry) => (
              // الطيّ أعلاه يضمن صفًّا واحدًا للمعرّف، فيعود `id` مفتاحًا صادقًا.
              <Entry
                key={entry.id}
                entry={entry}
                nowSeconds={nowSeconds}
                stopping={stoppingRuns.has(entry.id)}
                onRerun={onRerun}
                onCancelPreview={onCancelPreview}
                onStop={onStop}
                onReveal={onReveal}
                onDelete={onDelete}
              />
            ))}
          </ol>}
        </>
      )}

      {confirmingClear && (
        <ConfirmClear onCancel={() => setConfirmingClear(false)} onConfirm={onClearAll} />
      )}
    </section>
  );
}

type EntryVisualState = 'planned' | 'running' | 'succeeded' | 'failed' | 'cancelled' | 'unknown';

function visualStateOf(state: string): EntryVisualState {
  return KNOWN_STATES.has(state) ? (state as EntryVisualState) : 'unknown';
}

function entryStatus(entry: JournalEntry, visualState: EntryVisualState, nowSeconds: number): string {
  switch (visualState) {
    case 'planned':
      return t('log.entry.status.planned');
    case 'running': {
      const elapsed = Number.isFinite(entry.at) ? Math.max(0, nowSeconds - entry.at) * 1000 : null;
      return elapsed === null
        ? t('log.entry.status.running')
        : `${t('log.entry.status.running')} · ${t('log.entry.since')} ${durationText(elapsed)}`;
    }
    case 'succeeded':
      return typeof entry.duration_ms === 'number'
        ? `${t('log.entry.status.succeeded')} · ${t('log.entry.completed_in')} ${durationText(entry.duration_ms)}`
        : t('log.entry.status.succeeded');
    case 'failed':
      if (typeof entry.code !== 'number') return t('log.entry.status.failed');
      return entry.reason === 'signal'
        ? `${t('log.entry.status.failed')} · ${t('state.failed.signal')} ${entry.code}`
        : `${t('log.entry.status.failed')} · ${t('state.failed.code')} ${entry.code}`;
    case 'cancelled':
      return t('log.entry.status.cancelled');
    case 'unknown':
      return t('log.entry.status.unknown');
  }
}

/** وسم تقني LTR كما في Figma: الفئة ثم آخر جزء من معرّف العملية. */
function entryCategory(entry: JournalEntry): string {
  const parts = entry.op_id.split('.').filter(Boolean);
  const category = entry.category ?? parts[0] ?? '';
  const kind = parts[parts.length - 1] ?? '';
  return [...new Set([category, kind].filter(Boolean))].join(' · ').toLocaleUpperCase('en-US');
}

function entryCommand(entry: JournalEntry): string {
  return [entry.program, ...entry.args].join(' ');
}

function entryDetailTarget(entry: JournalEntry, visualState: EntryVisualState): string {
  if (visualState === 'unknown') return entryCommand(entry);
  if (entry.result?.type === 'artifact') return entry.result.path;
  if (entry.produced) return entry.produced;
  if (entry.cwd) return entry.cwd;
  return entry.args.find((argument) => argument.startsWith('/')) ?? entry.program;
}

function entryDetailNote(entry: JournalEntry, visualState: EntryVisualState): string {
  if (visualState === 'unknown') return entry.id;
  const tail = entry.tail ?? [];
  return tail[tail.length - 1] ?? t('log.entry.output.retained');
}

function entryTechnicalState(entry: JournalEntry, visualState: EntryVisualState): string {
  const retained = (entry.tail?.length ?? 0) > 0 ? 'output retained' : 'output unavailable';
  switch (visualState) {
    case 'succeeded': {
      // A terminal domain answer is not necessarily exit 0: grep no-match,
      // diff-found, unsigned code, and a Gatekeeper rejection are successful
      // semantic results with non-zero tool exits. The journal does not persist
      // a success exit code, so state the typed semantic instead of inventing one.
      const semantic = entry.result?.semantic ?? 'completed';
      return `semantic ${semantic} · ${retained}`;
    }
    case 'failed':
      return `exit ${entry.code ?? '—'} · ${retained}`;
    case 'cancelled':
      return `cancelled · ${retained}`;
    case 'running':
      return 'running · output pending';
    case 'planned':
      return 'planned · command retained';
    case 'unknown':
      return `${String(entry.state)} · ${retained}`;
  }
}

function Entry({
  entry,
  nowSeconds,
  stopping,
  onRerun,
  onCancelPreview,
  onStop,
  onReveal,
  onDelete,
}: {
  entry: JournalEntry;
  nowSeconds: number;
  stopping: boolean;
  onRerun: (entry: JournalEntry) => void;
  onCancelPreview: (entry: JournalEntry) => void;
  onStop: (entry: JournalEntry) => void;
  onReveal: (entry: JournalEntry) => void;
  onDelete: (entry: JournalEntry) => void;
}): JSX.Element {
  // نصٌّ لا `JournalState`: قيمةٌ لا تعرفها هذه النسخة يجب أن تمرّ لا أن تختفي.
  const state: string = entry.state;
  const visualState = visualStateOf(state);
  const when = whenOf(entry.at);
  // عنوان العملية إن عرفته هذه النسخة. `t` يعيد المفتاح نفسه حين يغيب، وهو
  // نصٌّ لاتينيّ لا يصلح عنوانًا — فالمقارنة تفرّق بين ترجمةٍ وغيابها، ويبقى
  // المعرّف معروضًا في الحالتين لأنه ما يربط الصفّ بالنواة.
  const titleKey = `op.${entry.op_id}.title`;
  const title = t(titleKey);
  const named = title !== titleKey;
  const result = entry.result;
  const terminal = visualState === 'succeeded' || visualState === 'failed' || visualState === 'cancelled';
  const canRerun = terminal && named;
  const revealable =
    visualState === 'succeeded' && (result?.reveal === 'file' || result?.reveal === 'directory');
  const detailsId = `${useId()}-details`;
  const [expanded, setExpanded] = useState(false);

  return (
    <li className={`entry entry--${visualState}${expanded ? ' is-expanded' : ''}`}>
      <div className="entry__summary">
        <div className="entry__technical" dir="ltr">
          <bdi dir="ltr" className="entry__run-id">{entry.id}</bdi>
          <bdi dir="ltr" className="entry__category">{entryCategory(entry)}</bdi>
        </div>
        <div className="entry__identity" dir="rtl">
          <span className="entry__state-mark" aria-hidden="true"><span /></span>
          <div className="entry__identity-copy">
            <p className="entry__title">{named ? title : entry.op_id}</p>
            <p className="entry__status">{entryStatus(entry, visualState, nowSeconds)}</p>
          </div>
          {when && <time className="visually-hidden" dateTime={when.iso}>{when.text}</time>}
        </div>
      </div>

      <div className="entry__divider" aria-hidden="true" />

      <div className="entry__lower">
        <div id={detailsId} className="entry__details" hidden={!expanded}>
          <bdi dir="ltr" className="entry__target">{entryDetailTarget(entry, visualState)}</bdi>
          <p
            className={`entry__detail-note${visualState === 'unknown' ? ' entry__detail-note--technical' : ''}`}
            dir={visualState === 'unknown' ? 'ltr' : 'auto'}
          >
            {entryDetailNote(entry, visualState)}
          </p>
          <bdi dir="ltr" className="entry__technical-output">
            {entryTechnicalState(entry, visualState)}
          </bdi>
        </div>

        <div className="entry__footer">
          <button
            type="button"
            className="entry__action entry__disclosure"
            aria-expanded={expanded}
            aria-controls={detailsId}
            onClick={() => setExpanded((open) => !open)}
          >
            {expanded ? t('log.details.hide') : t('log.details.show')}
          </button>

          <div className="entry__lifecycle-actions">
            {visualState === 'planned' && (
              <button type="button" className="entry__action entry__action--accent" onClick={() => onCancelPreview(entry)}>
                {t('log.action.cancel')}
              </button>
            )}
            {visualState === 'running' && (
              <button
                type="button"
                className="entry__action entry__action--accent"
                disabled={stopping}
                aria-busy={stopping}
                onClick={() => onStop(entry)}
              >
                {t('log.action.stop')}
              </button>
            )}
            {canRerun && (
              <button type="button" className="entry__action entry__action--accent" onClick={() => onRerun(entry)}>
                {t('log.rerun')}
              </button>
            )}
            {revealable && (
              <button type="button" className="entry__action entry__action--accent" onClick={() => onReveal(entry)}>
                {t('action.reveal')}
              </button>
            )}
            {(terminal || visualState === 'unknown') && (
              <button
                type="button"
                className="entry__action entry__action--danger"
                onClick={() => onDelete(entry)}
              >
                {t('log.delete')}
              </button>
            )}
            {visualState === 'unknown' && (
              <button type="button" className="entry__action entry__action--accent" onClick={() => setExpanded(true)}>
                {t('log.action.details')}
              </button>
            )}
          </div>
        </div>
      </div>
    </li>
  );
}

function ConfirmClear({ onCancel, onConfirm }: { onCancel: () => void; onConfirm: () => void }) {
  return (
    <JournalConfirm
      title={t('log.clear.title')}
      body={t('log.clear.body')}
      cancelLabel={t('log.clear.cancel')}
      confirmLabel={t('log.clear.confirm')}
      onCancel={onCancel}
      onConfirm={onConfirm}
    />
  );
}

/**
 * تأكيد مسح السجل كاملًا، وهو الفعل التدميري الوحيد في هذه الشاشة.
 *
 * إلغاء معاينةٍ متروكة فعلٌ فوري محلّي؛ أمّا المسح الكامل فيأخذ هذا الحوار.
 * الإبقاء هو الفعل الافتراضي ويأخذ التركيز، والنص يصرّح بأن ملفات
 * النتائج لا تُمسّ.
 */
function JournalConfirm({
  title,
  body,
  cancelLabel,
  confirmLabel,
  onCancel,
  onConfirm,
}: {
  title: string;
  body: string;
  cancelLabel: string;
  confirmLabel: string;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const cancel = useRef<HTMLButtonElement>(null);
  const dialog = useRef<HTMLDivElement>(null);
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

  function keepFocus(event: React.KeyboardEvent<HTMLDivElement>) {
    if (event.key !== 'Tab') return;
    const focusable = [...(dialog.current?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? [])];
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (!first || !last) return;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  return (
    <div className="scrim" role="presentation" onClick={onCancel}>
      <div
        ref={dialog}
        className="dialog card"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={`${uid}-t`}
        aria-describedby={`${uid}-b`}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={keepFocus}
      >
        <h2 id={`${uid}-t`} className="t-section-title dialog__title">
          {title}
        </h2>
        <p id={`${uid}-b`} className="t-body-sec dialog__body">
          {body}
        </p>
        <div className="dialog__actions">
          <button ref={cancel} type="button" className="btn btn--primary" onClick={onCancel}>
            {cancelLabel}
          </button>
          <button type="button" className="btn btn--danger" onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
        <p className="dialog__hint">{t('dialog.safe_dismiss')}</p>
      </div>
    </div>
  );
}
