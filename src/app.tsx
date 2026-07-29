/**
 * الشاشة — وربطُ الشاشات ببعضها.
 *
 * ## ما يملكه هذا الملف وما لا يملكه
 *
 * يملك: الشاشة الحالية، وحالة أول تشغيل، وفهرس العمليات، ودورة حياة التشغيل.
 * لا يملك: نصًّا عربيًا (في `i18n.ts`)، ولا قائمة عمليات (تأتي من النواة)، ولا
 * قواعد الانتقال (في `nav.ts`)، ولا قراءة الإعداد (في `settings.ts`). ما يفعله
 * هنا هو الوصل وحده، فلا يوجد في التطبيق موضعان يعرفان القاعدة نفسها.
 *
 * ## «نَفِّذ» و«سَطْر» معًا لا بالتناوب
 *
 * ليسا وضعين يبدّل بينهما المستخدم: هما عرضان لشيء واحد معروضان معًا. الخطة
 * الحيّة واحدة، ترسمها اليمنى نموذجًا وملخّصًا، وترسمها اليسرى أمرًا مشروحًا.
 * تغييرٌ في حقل يحرّك الاثنين في اللحظة نفسها، لأنهما قراءتان لنفس
 * `PlanResponse`.
 */
import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import Naffith from './naffith';
import Satr from './satr';
import Onboarding from './onboarding';
import OperationsList, { type OperationsState } from './operations-list';
import SettingsScreen from './settings-screen';
import RunLog from './run-log';
import { t } from './i18n';
import './app-shell.css';
import {
  confirmNavigation,
  initialScreen,
  navigate,
  type ExitCost,
  type NavEvent,
  type Screen,
} from './nav';
import {
  browserStorage,
  loadSettings,
  saveSettings,
  shouldShowOnboarding,
  withOnboardingCompleted,
  withOnboardingReset,
  type Settings,
} from './settings';
import {
  emptyValues,
  isComplete,
  toCards,
  toRawValues,
  type FormValues,
} from './operations';
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

/**
 * مهلة قبل إعادة التخطيط.
 *
 * التخطيط يلمس القرص (يتحقّق من المسارات، ويفحص الكتابة، ويقدّر الحجم)، فربطه
 * بكل ضغطة مفتاح يجعل الكتابة متقطّعة. وأطول من هذا يجعل المعاينة تبدو متأخّرة
 * عن الحقل.
 */
const REPLAN_DELAY_MS = 250;

/** ما تعرضه الشاشة. `planning` مشتقّة، ليست حالة تشغيل. */
export type Phase = 'idle' | 'planning' | 'running' | 'cancelling' | 'finished';

/**
 * دورة حياة التشغيل وحدها.
 *
 * **`planning` ليست منها عمدًا.** أثرُ إعادة التخطيط يقرأ الحالة ويكتبها معًا؛
 * لو كانت «قيد التخطيط» حالةً واحدة مع الباقي لأعاد الأثرُ إطلاقَ نفسه بلا
 * توقّف: يخطّط، فتتغيّر الحالة، فيُلغى المؤقّت ويُعاد، إلى ما لا نهاية — ويظلّ
 * زرّ «نفّذ» يومض بين ممكّن ومعطّل. وقع هذا فعلًا وأمسكه تشغيلُ الشاشة.
 */
type RunPhase = 'idle' | 'running' | 'cancelling' | 'finished';

export default function App() {
  // ── الإعداد وأول تشغيل ───────────────────────────────────────────────
  //
  // يُقرأ مرّة واحدة قبل أول رسم، لا في أثر: قراءتُه في `useEffect` كانت ترسم
  // قائمة العمليات للحظة ثم تقفز إلى الترحيب — ووميضُ الشاشة الخطأ في أول
  // تشغيل هو أول ما يراه المستخدم من التطبيق.
  const storage = useMemo(() => browserStorage(), []);
  const [settings, setSettings] = useState<Settings>(() => loadSettings(storage).settings);
  const storageAvailable = storage !== null;

  const [screen, setScreen] = useState<Screen>(() =>
    initialScreen(shouldShowOnboarding(loadSettings(storage).settings)),
  );

  /** انتقالٌ ينتظر قرار المستخدم. انظر `nav.ts`. */
  const [pending, setPending] = useState<{ screen: Screen; reason: 'dirty' | 'busy' } | null>(null);

  // ── فهرس العمليات ────────────────────────────────────────────────────
  const [operations, setOperations] = useState<OperationSummary[] | null>(null);
  const [opsError, setOpsError] = useState<CoreErrorShape | null>(null);

  const loadOperations = useCallback(() => {
    setOperations(null);
    setOpsError(null);
    listOperations()
      .then((ops) => setOperations(ops))
      .catch((e) => setOpsError(asCoreError(e)));
  }, []);

  useEffect(loadOperations, [loadOperations]);

  const opsState: OperationsState = opsError
    ? { status: 'failed', error: opsError }
    : operations === null
      ? { status: 'loading' }
      : { status: 'ready', cards: toCards(operations) };

  // ── حالة النموذج، محفوظة لكل عملية ───────────────────────────────────
  //
  // خريطةٌ بمعرّف العملية لا نموذجٌ واحد: الرجوع إلى القائمة ثم العودة يجب أن
  // يعيد ما كُتب. هذا ما يجعل كلفة مغادرة شاشة العملية `free` في الحالة
  // العادية — لا يُفقد شيء فلا معنى لسؤال المستخدم عن فقدٍ لن يقع. يبقى
  // `dirty` في `nav.ts` لشاشةٍ تُتلف حالتها فعلًا.
  const [formsByOp, setFormsByOp] = useState<Record<string, FormValues>>({});

  // ── دورة حياة التشغيل ────────────────────────────────────────────────
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

  const openOpId = screen.name === 'operation-view' ? screen.opId : null;
  const operation = useMemo(
    () => (openOpId ? (operations?.find((o) => o.id === openOpId) ?? null) : null),
    [openOpId, operations],
  );

  const values: FormValues = useMemo(() => {
    if (!operation) return {};
    return formsByOp[operation.id] ?? emptyValues(operation);
  }, [operation, formsByOp]);

  const setValues = useCallback(
    (next: FormValues) => {
      if (!operation) return;
      setFormsByOp((all) => ({ ...all, [operation.id]: next }));
    },
    [operation],
  );

  const complete = operation ? isComplete(operation, values) : false;

  /** بعد الضغط على «نفّذ» لا يعود النموذج يخطّط: التشغيل جارٍ أو انتهى. */
  const locked = phase !== 'idle';

  // إعادة التخطيط عند كل تغيير: المعاينة والأمر المعروض ناتج الخطة نفسها،
  // فإبقاؤها قديمةً يعني عرض أمرٍ غير الذي سيُنفَّذ.
  //
  // اعتماديات هذا الأثر لا تشمل `planning`، وهذا مقصود: كتابةُ ما يُقرأ في
  // الاعتماديات تُنتج حلقة لا تنتهي. `locked` تتغيّر مرّتين لا أكثر في كل
  // تشغيل، فالأثر يستقرّ.
  useEffect(() => {
    if (locked || !operation) return;

    if (!complete) {
      setPlan(null);
      setError(null);
      setPlanning(false);
      return;
    }

    const seq = ++planSeq.current;
    setPlanning(true);
    const timer = window.setTimeout(() => {
      planOperation(operation.id, toRawValues(operation, values))
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
  }, [operation, values, complete, locked]);

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

  /** يعيد شاشة العملية إلى الخمول بعد تشغيل انتهى. */
  const clearRun = useCallback(() => {
    setOutcome(null);
    setPlan(null);
    setExecuted(null);
    setError(null);
    setPhase('idle');
  }, []);

  const onReset = useCallback(() => {
    clearRun();
    // الاسم وحده يُفرَّغ: المصدر والوجهة غالبًا يبقيان كما هما لعمليةٍ تالية،
    // وإفراغهما يجبر المستخدم على اختيار المجلد نفسه مرّة أخرى.
    if (!operation) return;
    setFormsByOp((all) => {
      const current = all[operation.id] ?? emptyValues(operation);
      const cleared: FormValues = { ...current };
      for (const input of operation.inputs) {
        if (String((input as { kind?: unknown }).kind) === 'new_name') cleared[input.id] = '';
      }
      return { ...all, [operation.id]: cleared };
    });
  }, [clearRun, operation]);

  // ── التنقّل ──────────────────────────────────────────────────────────

  /**
   * كلفة مغادرة الشاشة الحالية.
   *
   * تشغيلٌ جارٍ وحده يستحق سؤالًا. الحقول المملوءة لا تُفقد بالمغادرة — تبقى
   * في `formsByOp` وتعود كما هي — فسؤال المستخدم عنها كان سيكون إنذارًا كاذبًا،
   * وهو أسوأ من لا إنذار: من يتعلّم أن التحذير لا يعني شيئًا يتجاوزه حين يعني.
   */
  const exitCost: ExitCost = phase === 'running' || phase === 'cancelling' ? 'busy' : 'free';

  const go = useCallback(
    (event: NavEvent) => {
      const result = navigate(screen, event, exitCost);
      if (result.kind === 'navigate') {
        setScreen(result.screen);
        // تشغيلٌ انتهى وغادر المستخدم شاشته: لا يبقى ناتجه معلّقًا على شاشة
        // عمليةٍ أخرى يفتحها بعد قليل.
        if (phase === 'finished') clearRun();
      } else if (result.kind === 'confirm') {
        setPending({ screen: result.pending, reason: result.reason });
      }
    },
    [screen, exitCost, phase, clearRun],
  );

  const confirmLeave = useCallback(() => {
    if (!pending) return;
    const result = confirmNavigation(pending.screen);
    if (result.kind === 'navigate') setScreen(result.screen);
    setPending(null);
  }, [pending]);

  const persist = useCallback(
    (next: Settings) => {
      setSettings(next);
      saveSettings(storage, next);
    },
    [storage],
  );

  const finishOnboarding = useCallback(() => {
    // الحفظ قد يفشل، والانتقال يقع رغم ذلك: أسوأ ما يقع أن يرى المستخدم
    // الترحيب مرّةً أخرى، وهو أهون من أن يُحبس في شاشة لا تُغادَر.
    persist(withOnboardingCompleted(settings, new Date()));
    go({ type: 'onboarding.finished' });
  }, [persist, settings, go]);

  const replayOnboarding = useCallback(() => {
    persist(withOnboardingReset(settings));
    go({ type: 'onboarding.replay' });
  }, [persist, settings, go]);

  // «قيد التخطيط» حالةُ عرضٍ مشتقّة لا حالةُ تشغيل. انظر تعليق `RunPhase`.
  const uiPhase: Phase = phase === 'idle' && planning ? 'planning' : phase;

  // أثناء التشغيل وبعده يعرض «سَطْر» الخطة المستهلكة؛ وفي وضع الخمول لا يعرض
  // إلا الحيّة، فلا يبقى أمرٌ قديم معلّقًا بعد أن يفرّغ المستخدم حقلًا.
  const shownPlan = phase === 'idle' ? plan : (plan ?? executed);

  // ── الرسم ────────────────────────────────────────────────────────────

  if (screen.name === 'onboarding') {
    return <Onboarding onStart={finishOnboarding} />;
  }

  const leaveDialog = pending && (
    <ConfirmLeave
      reason={pending.reason}
      onStay={() => setPending(null)}
      onLeave={confirmLeave}
    />
  );

  if (screen.name === 'operations-list') {
    return (
      <Page>
        <OperationsList
          state={opsState}
          onSelect={(opId) => go({ type: 'operation.selected', opId })}
          onRetry={loadOperations}
          onOpenLog={() => go({ type: 'log.opened' })}
          onOpenSettings={() => go({ type: 'settings.opened' })}
        />
        {leaveDialog}
      </Page>
    );
  }

  if (screen.name === 'settings') {
    return (
      <Page>
        <SettingsScreen
          onBack={() => go({ type: 'back' })}
          onReplayOnboarding={replayOnboarding}
          storageAvailable={storageAvailable}
        />
        {leaveDialog}
      </Page>
    );
  }

  if (screen.name === 'run-log') {
    return (
      <Page>
        <RunLog onBack={() => go({ type: 'back' })} />
        {leaveDialog}
      </Page>
    );
  }

  // شاشة العملية. الفهرس قد يكون ما زال يُحمَّل، أو قد تكون العملية اختفت منه
  // بين اختيارها وفتحها — والحالتان مختلفتان ولا يجوز أن تُعرضا نصًّا واحدًا.
  if (!operation) {
    return (
      <Page variant="message">
        <p className="t-body">{operations === null ? t('ops.loading') : t('ops.gone')}</p>
        <button type="button" className="btn btn--quiet" onClick={() => go({ type: 'back' })}>
          {t('nav.back')}
        </button>
      </Page>
    );
  }

  return (
    <Page>
      <main className="page__body">
        <Naffith
          operation={operation}
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
          onBack={() => go({ type: 'back' })}
        />
        <Satr plan={shownPlan} />
      </main>
      {leaveDialog}
    </Page>
  );
}

/**
 * وعاء الشاشة — الحشوة والعرض الأقصى والتوسيط.
 *
 * مكوّنٌ في الغلاف لا صنفٌ تكتبه كل شاشة في وسمها: الشاشات الثلاث الثانوية
 * كانت تُعاد عاريةً من `App` بينما شاشة العملية وحدها ملفوفة في وعاءٍ مبطّن،
 * فظهرت قائمة العمليات والإعدادات والسجلّ ملاصقةً لحافّتي النافذة — يُقصّ
 * العنوان عند الحافة، وتُقصّ حلقةُ التركيز حول البطاقة المحاذية لها.
 *
 * الوعاء هنا يجعل الحشوة أثرًا للتركيب لا شيئًا يُتذكَّر: شاشةٌ تُضاف غدًا
 * تمرّ من الموضع نفسه، فترث ما ورثته أخواتها.
 *
 * وحوارُ المغادرة ابنٌ له بلا ضرر: `.scrim` مثبّت `fixed` بلا محوّل على
 * الوعاء، فلا يقيّده عرضه الأقصى ولا يشارك في تخطيطه المرن.
 */
function Page({
  children,
  variant,
}: {
  children: ReactNode;
  /** `message` لسطرٍ وزرّ: يُوسَّط رأسيًا بدل أن يبدأ من الأعلى. */
  variant?: 'message';
}) {
  return <div className={variant ? `page page--${variant}` : 'page'}>{children}</div>;
}

/**
 * قرار المغادرة أثناء تشغيل نشط.
 *
 * حوارٌ مقاطع لا إشعارٌ عابر: الانتقال الصامت أثناء تشغيلٍ يكتب على قرص
 * المستخدم يخفي عنه أن شيئًا ما زال يجري. و«البقاء» هو الفعل الافتراضي —
 * يأخذ التركيز — لأن الخطأ في اتجاه البقاء غير مكلف.
 */
function ConfirmLeave({
  reason,
  onStay,
  onLeave,
}: {
  reason: 'dirty' | 'busy';
  onStay: () => void;
  onLeave: () => void;
}) {
  const stay = useRef<HTMLButtonElement>(null);
  useEffect(() => stay.current?.focus(), []);

  const titleId = `leave-${reason}-title`;
  const bodyId = `leave-${reason}-body`;

  return (
    <div className="scrim" role="presentation" onClick={onStay}>
      <div
        className="dialog card"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={bodyId}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 id={titleId} className="t-section-title dialog__title">
          {t(`nav.leave.${reason}.title`)}
        </h2>
        <p id={bodyId} className="t-body-sec dialog__body">
          {t(`nav.leave.${reason}.body`)}
        </p>
        <div className="dialog__actions">
          <button ref={stay} type="button" className="btn btn--primary" onClick={onStay}>
            {t(`nav.leave.${reason}.stay`)}
          </button>
          <button type="button" className="btn btn--quiet" onClick={onLeave}>
            {t(`nav.leave.${reason}.leave`)}
          </button>
        </div>
      </div>
    </div>
  );
}
