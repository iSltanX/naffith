/**
 * الشاشة.
 *
 * «نَفِّذ» و«سَطْر» ليسا وضعين يبدّل بينهما المستخدم: هما عرضان لشيء واحد
 * معروضان معًا. الخطة الحيّة واحدة، ترسمها الشاشة اليمنى نموذجًا وملخّصًا،
 * وترسمها اليسرى أمرًا مشروحًا. تغييرٌ في حقل يحرّك الاثنين في اللحظة نفسها،
 * لأنهما قراءتان لنفس `PlanResponse`.
 */
import { useCallback, useEffect, useRef, useState } from 'react';
import Naffith, { type FormValues } from './naffith';
import Satr from './satr';
import { t } from './i18n';
import {
  asCoreError,
  cancel as cancelRun,
  execute,
  listOperations,
  onRunFinished,
  plan as planOperation,
  reveal as revealRun,
  type CoreErrorShape,
  type OperationSummary,
  type PlanResponse,
  type RunFinishedEvent,
} from './ipc';

const OP_ID = 'compress.folder.zip';

/**
 * مهلة قبل إعادة التخطيط.
 *
 * التخطيط يلمس القرص (يتحقّق من المسارات، ويفحص الكتابة، ويقدّر الحجم)، فربطه
 * بكل ضغطة مفتاح يجعل الكتابة متقطّعة. وأطول من هذا يجعل المعاينة تبدو متأخّرة
 * عن الحقل.
 */
const REPLAN_DELAY_MS = 250;

/** ما تعرضه الشاشة. `planning` مشتقّة، ليست حالة تشغيل. */
type Phase = 'idle' | 'planning' | 'running' | 'cancelling' | 'finished';

/**
 * دورة حياة التشغيل وحدها.
 *
 * **`planning` ليست منها عمدًا.** أثرُ إعادة التخطيط يقرأ الحالة ويكتبها معًا؛
 * لو كانت «قيد التخطيط» حالةً واحدة مع الباقي لأعاد الأثرُ إطلاقَ نفسه بلا
 * توقّف: يخطّط، فتتغيّر الحالة، فيُلغى المؤقّت ويُعاد، إلى ما لا نهاية — ويظلّ
 * زرّ «نفّذ» يومض بين ممكّن ومعطّل. وقع هذا فعلًا وأمسكه تشغيلُ الشاشة.
 */
type RunPhase = 'idle' | 'running' | 'cancelling' | 'finished';

const EMPTY: FormValues = { source: '', destination: '', archive_name: '' };

export default function App() {
  const [operation, setOperation] = useState<OperationSummary | null>(null);
  const [values, setValues] = useState<FormValues>(EMPTY);
  const [plan, setPlan] = useState<PlanResponse | null>(null);
  const [error, setError] = useState<CoreErrorShape | null>(null);
  const [phase, setPhase] = useState<RunPhase>('idle');
  const [planning, setPlanning] = useState(false);
  const [runId, setRunId] = useState<string | null>(null);
  /**
   * الخطة التي استُهلكت.
   *
   * الرمز أحادي الاستخدام فتُنزع الخطة الحيّة عند التنفيذ، لكن «سَطْر» يجب أن
   * يظلّ يعرض ما شُغِّل أثناء التشغيل وبعده. إخفاء الأمر في اللحظة التي يسأل
   * فيها المستخدم «ماذا جرى؟» ينقض غرض الشاشة كلّه. للعرض فقط — الرمز مُستهلك.
   */
  const [executed, setExecuted] = useState<PlanResponse | null>(null);
  const [outcome, setOutcome] = useState<RunFinishedEvent | null>(null);

  /** يبطل ناتج تخطيطٍ سبقه تخطيطٌ أحدث، فلا تظهر معاينة قديمة بعد الجديدة. */
  const planSeq = useRef(0);
  const unlisten = useRef<Array<() => void>>([]);

  useEffect(() => {
    listOperations()
      .then((ops) => setOperation(ops.find((o) => o.id === OP_ID) ?? null))
      .catch((e) => setError(asCoreError(e)));

    onRunFinished((event) => {
      setOutcome(event);
      setPhase('finished');
      setRunId(null);
    }).then((u) => unlisten.current.push(u));

    return () => {
      unlisten.current.forEach((u) => u());
      unlisten.current = [];
    };
  }, []);

  const complete =
    values.source.trim() !== '' &&
    values.destination.trim() !== '' &&
    values.archive_name.trim() !== '';

  /** بعد الضغط على «نفّذ» لا يعود النموذج يخطّط: التشغيل جارٍ أو انتهى. */
  const locked = phase !== 'idle';

  // إعادة التخطيط عند كل تغيير: المعاينة والأمر المعروض ناتج الخطة نفسها،
  // فإبقاؤها قديمةً يعني عرض أمرٍ غير الذي سيُنفَّذ.
  //
  // اعتماديات هذا الأثر لا تشمل `planning`، وهذا مقصود: كتابةُ ما يُقرأ في
  // الاعتماديات تُنتج حلقة لا تنتهي. `locked` تتغيّر مرّتين لا أكثر في كل
  // تشغيل، فالأثر يستقرّ.
  useEffect(() => {
    if (locked) return;

    if (!complete) {
      setPlan(null);
      setError(null);
      setPlanning(false);
      return;
    }

    const seq = ++planSeq.current;
    setPlanning(true);
    const timer = window.setTimeout(() => {
      planOperation(OP_ID, {
        source: { kind: 'path', value: values.source.trim() },
        destination: { kind: 'path', value: values.destination.trim() },
        archive_name: { kind: 'text', value: values.archive_name },
      })
        .then((response) => {
          // خطّةٌ سبقتها أحدثُ منها تُهمَل، فلا تظهر معاينة قديمة بعد الجديدة.
          if (seq !== planSeq.current) return;
          setPlan(response);
          setError(null);
          setPlanning(false);
        })
        .catch((e) => {
          if (seq !== planSeq.current) return;
          setPlan(null);
          setError(asCoreError(e));
          setPlanning(false);
        });
    }, REPLAN_DELAY_MS);

    return () => window.clearTimeout(timer);
  }, [values, complete, locked]);

  const onExecute = useCallback(async () => {
    if (!plan) return;
    setError(null);
    setOutcome(null);
    setPhase('running');
    try {
      setExecuted(plan);
      setRunId(await execute(plan.token));
      // الرمز أحادي الاستخدام: الخطة استُهلكت، فلا معنى لإبقائها قابلة للتنفيذ.
      setPlan(null);
    } catch (e) {
      setError(asCoreError(e));
      setPhase('idle');
    }
  }, [plan]);

  const onCancel = useCallback(async () => {
    if (!runId) return;
    setPhase('cancelling');
    try {
      await cancelRun(runId);
    } catch (e) {
      setError(asCoreError(e));
    }
  }, [runId]);

  const onReveal = useCallback(async () => {
    if (!outcome?.run_id) return;
    try {
      await revealRun(outcome.run_id);
    } catch (e) {
      setError(asCoreError(e));
    }
  }, [outcome]);

  const onReset = useCallback(() => {
    setOutcome(null);
    setPlan(null);
    setExecuted(null);
    setError(null);
    setPhase('idle');
    setValues((v) => ({ ...v, archive_name: '' }));
  }, []);

  // «قيد التخطيط» حالةُ عرضٍ مشتقّة لا حالةُ تشغيل. انظر تعليق `RunPhase`.
  const uiPhase: Phase = phase === 'idle' && planning ? 'planning' : phase;

  // أثناء التشغيل وبعده يعرض «سَطْر» الخطة المستهلكة؛ وفي وضع الخمول لا يعرض
  // إلا الحيّة، فلا يبقى أمرٌ قديم معلّقًا بعد أن يفرّغ المستخدم حقلًا.
  const shownPlan = phase === 'idle' ? plan : (plan ?? executed);

  return (
    <div className="app">
      <header className="app__head">
        <svg viewBox="0 0 64 64" aria-hidden="true" className="app__mark">
          <use href="#mark" />
        </svg>
        <div>
          <h1 className="t-page-title app__title">
            {t('app.naffith')} <span aria-hidden="true">—</span> {t('app.satr')}
          </h1>
          <p className="t-helper">{t('app.tagline')}</p>
        </div>
      </header>

      <main className="app__body">
        <Naffith
          values={values}
          onChange={setValues}
          plan={plan}
          error={error}
          phase={uiPhase}
          outcome={outcome}
          onExecute={onExecute}
          onCancel={onCancel}
          onReveal={onReveal}
          onReset={onReset}
          opTitleKey={operation?.title_key ?? 'op.compress.folder.zip.title'}
          opDescriptionKey={operation?.description_key ?? 'op.compress.folder.zip.description'}
        />
        <Satr plan={shownPlan} />
      </main>
    </div>
  );
}
