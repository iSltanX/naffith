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
import Naffith, { OperationBar } from './naffith';
import Satr from './satr';
import Onboarding from './onboarding';
import LibraryScreen, { type LibraryState } from './library-screen';
import CategoryScreen from './category-screen';
import SettingsScreen from './settings-screen';
import RunLog from './run-log';
import { t } from './i18n';
import './app-shell.css';
import {
  confirmNavigation,
  initialScreen,
  navigate,
  screenKey,
  type ExitCost,
  type NavEvent,
  type Screen,
} from './nav';
import {
  browserStorage,
  loadSettings,
  saveSettings,
  shouldShowOnboarding,
  withFavouriteToggled,
  withLastCategory,
  withOnboardingCompleted,
  withOnboardingReset,
  type Settings,
} from './settings';
import {
  emptyValues,
  isComplete,
  toRawValues,
  type FormValues,
} from './operations';
import {
  favouriteOperations,
  findCategory,
  operationsIn,
  recentOperations,
  toCategoryCard,
  toOperationCard,
} from './library';
import {
  asCoreError,
  cancel as cancelRun,
  execute,
  listCategories,
  listOperations,
  onRunFinished,
  onRunOutput,
  plan as planOperation,
  recentRuns,
  reveal as revealRun,
  type CategorySummary,
  type CoreErrorShape,
  type JournalEntry,
  type OperationSummary,
  type PlanResponse,
  type RunFinishedEvent,
} from './ipc';
import { appendLine, type StreamLine } from './run-stream';

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

  // ── المكتبة: الأقسام والعمليات ───────────────────────────────────────
  //
  // نداءان لا واحد، ويُنتظران معًا: الأقسام تكفي لرسم الشاشة الأولى، لكن
  // البحث والمفضّلة والمستخدَمة حديثًا كلها تحتاج العمليات — فعرضُ الشبكة قبل
  // وصولها كان يعني شاشةً تكتمل على مرحلتين أمام العين.
  const [operations, setOperations] = useState<OperationSummary[] | null>(null);
  const [categories, setCategories] = useState<CategorySummary[] | null>(null);
  const [opsError, setOpsError] = useState<CoreErrorShape | null>(null);

  const loadLibrary = useCallback(() => {
    setOperations(null);
    setCategories(null);
    setOpsError(null);
    Promise.all([listOperations(), listCategories()])
      .then(([ops, cats]) => {
        setOperations(ops);
        setCategories(cats);
      })
      .catch((e) => setOpsError(asCoreError(e)));
  }, []);

  useEffect(loadLibrary, [loadLibrary]);

  /**
   * السجلّ، مقروءًا لأجل «المستخدَمة حديثًا».
   *
   * يُعاد قراءته بعد كل تشغيلٍ ينتهي لا مرّةً عند الإقلاع: صفٌّ اسمه «آخر ما
   * استخدمت» لا يحمل ما استُخدم قبل دقيقة هو صفٌّ يكذب.
   */
  const [journal, setJournal] = useState<JournalEntry[]>([]);
  const loadJournal = useCallback(() => {
    recentRuns()
      .then(setJournal)
      // تعذّرُ قراءة السجل لا يُسقط الشاشة: أسوأ ما يقع أن يفرغ صفٌّ يختفي
      // حين يفرغ أصلًا.
      .catch(() => setJournal([]));
  }, []);
  useEffect(loadJournal, [loadJournal]);

  const categoryCards = useMemo(() => (categories ?? []).map(toCategoryCard), [categories]);
  const operationCards = useMemo(
    () => (operations ?? []).map((op) => toOperationCard(op, categoryCards)),
    [operations, categoryCards],
  );

  const libraryState: LibraryState = opsError
    ? { status: 'failed', error: opsError }
    : operations === null || categories === null
      ? { status: 'loading' }
      : { status: 'ready', categories: categoryCards, operations: operationCards };

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
  /**
   * ما بثّته النواة من خرج الأداة في التشغيل الحالي.
   *
   * تُفرَّغ عند بدء تشغيلٍ جديد لا عند انتهاء السابق: بعد النهاية يُقرأ المجرى —
   * وهو لحظة السؤال «ماذا قالت الأداة؟» — فمحوُه عند `run://finished` يمحوه في
   * اللحظة التي يُحتاج فيها.
   */
  const [stream, setStream] = useState<StreamLine[]>([]);

  /** يبطل ناتج تخطيطٍ سبقه تخطيطٌ أحدث، فلا تظهر معاينة قديمة بعد الجديدة. */
  const planSeq = useRef(0);
  const unlisten = useRef<Array<() => void>>([]);

  useEffect(() => {
    onRunFinished((event) => {
      setOutcome(event);
      setPhase('finished');
      setRunId(null);
      // السجلّ صار فيه قيدٌ جديد، و«المستخدَمة حديثًا» تُقرأ منه.
      loadJournal();
    }).then((u) => unlisten.current.push(u));

    // خرج الأداة. لا تصفية بمعرّف التشغيل: النواة تحجز خانةً واحدة فلا تشغيلان
    // معًا، والأسطر الأولى قد تصل **قبل** أن يعود `execute` بمعرّفه — فتصفيةٌ
    // بالمعرّف كانت تُسقط أوّل ما تقوله الأداة، وهو غالبًا أهمّه.
    onRunOutput((event) => {
      setStream((kept) => appendLine(kept, event));
    }).then((u) => unlisten.current.push(u));

    return () => {
      unlisten.current.forEach((u) => u());
      unlisten.current = [];
    };
  }, [loadJournal]);

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
    // مجرى التشغيل السابق يُمحى هنا: من هذه اللحظة كل سطرٍ يصل يخصّ هذا التشغيل.
    setStream([]);
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
    setStream([]);
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

  /**
   * الشاشة التي فُتحت منها العمليةُ الحالية.
   *
   * العملية تُفتح من ثلاثة مواضع — قسمٌ، ونتائجُ بحث، وصفُّ مستخدَمة حديثًا —
   * والرجوع يجب أن يعيد إلى ما جاء منه. بدونه كان من يفتح عمليةً من البحث
   * يجد نفسه في قسمٍ لم يزره.
   *
   * يُلتقط عند الانتقال لا يُشتقّ من العملية: القسمُ الذي تنتمي إليه العملية
   * ليس بالضرورة الشاشة التي جاء منها المستخدم.
   */
  const [origin, setOrigin] = useState<Screen | null>(null);

  const go = useCallback(
    (event: NavEvent) => {
      const result = navigate(screen, event, exitCost, origin);
      if (result.kind === 'navigate') {
        if (result.screen.name === 'operation-view' && screen.name !== 'operation-view') {
          setOrigin(screen);
        }
        setScreen(result.screen);
        // تشغيلٌ انتهى وغادر المستخدم شاشته: لا يبقى ناتجه معلّقًا على شاشة
        // عمليةٍ أخرى يفتحها بعد قليل.
        if (phase === 'finished') clearRun();
      } else if (result.kind === 'confirm') {
        setPending({ screen: result.pending, reason: result.reason });
      }
    },
    [screen, exitCost, phase, clearRun, origin],
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

  // ── المفضّلة والمستخدَمة حديثًا ──────────────────────────────────────

  const onToggleFavourite = useCallback(
    (opId: string) => persist(withFavouriteToggled(settings, opId)),
    [persist, settings],
  );

  const favourites = useMemo(
    () => favouriteOperations(settings.favourites, operationCards),
    [settings.favourites, operationCards],
  );
  const recents = useMemo(
    () => recentOperations(journal, operationCards),
    [journal, operationCards],
  );

  const openCategory = useCallback(
    (categoryId: string) => {
      persist(withLastCategory(settings, categoryId));
      go({ type: 'category.selected', categoryId });
    },
    [persist, settings, go],
  );

  /**
   * «أعد هذه العملية بالقيم السابقة».
   *
   * يملأ النموذج وينتقل إليه، **ولا ينفّذ**. الفرق كل الفرق: إعادةُ تشغيلٍ
   * بضغطةٍ واحدة من شاشة السجل كانت ستُطلق أمرًا على ملفات المستخدم بلا أن
   * يرى معاينته — وهو بالضبط ما يوجد «نَفِّذ/سَطْر» لمنعه. ما يفعله الزرّ أنه
   * يوفّر إعادة الملء، ثم يقف حيث يقف كل مسارٍ آخر: أمام زرّ «نفِّذ».
   *
   * والقيمة المنقَّحة (`value === null`) تُترك فارغة: الحقل يبقى للمستخدم بدل
   * أن يُملأ بفراغٍ فيبدو كأن كل شيء استُعيد.
   */
  const rerun = useCallback(
    (entry: JournalEntry) => {
      const op = operations?.find((o) => o.id === entry.op_id);
      if (!op) return;
      const restored: FormValues = emptyValues(op);
      for (const input of entry.inputs ?? []) {
        if (input.value === null || input.value === undefined) continue;
        // مدخلٌ لم تعد العملية تعلنه — نواةٌ تغيّرت بين التشغيلين — يُهمَل بدل
        // أن يُحقن في نموذجٍ سيرفضه التحقّق بعد قليل بخطأٍ لا يفهمه المستخدم.
        if (!(input.id in restored)) continue;
        restored[input.id] = input.value;
      }
      setFormsByOp((all) => ({ ...all, [op.id]: restored }));
      go({ type: 'operation.selected', opId: op.id });
    },
    [operations, go],
  );

  // «قيد التخطيط» حالةُ عرضٍ مشتقّة لا حالةُ تشغيل. انظر تعليق `RunPhase`.
  const uiPhase: Phase = phase === 'idle' && planning ? 'planning' : phase;

  // أثناء التشغيل وبعده يعرض «سَطْر» الخطة المستهلكة؛ وفي وضع الخمول لا يعرض
  // إلا الحيّة، فلا يبقى أمرٌ قديم معلّقًا بعد أن يفرّغ المستخدم حقلًا.
  const shownPlan = phase === 'idle' ? plan : (plan ?? executed);

  /**
   * أيقونة قسم العملية المفتوحة.
   *
   * تُحسب هنا وتُمرَّر إلى شاشة العملية: القسم يعلن أيقونته في النواة، وخريطةٌ
   * ثانية داخل `naffith.tsx` كانت ستتقادم يوم يُضاف قسم.
   */
  const categoryIcon = operation
    ? (findCategory(categoryCards, operation.category)?.icon ?? '#i-file')
    : '#i-file';

  /** هويّة الشاشة المعروضة نصًّا. تُحسب في `nav.ts` كي لا تُحسب في موضعين. */
  const currentKey = screenKey(screen);

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
      <Page focusKey={currentKey}>
        <LibraryScreen
          state={libraryState}
          favourites={favourites}
          recents={recents}
          favouriteIds={settings.favourites}
          onOpenCategory={openCategory}
          onOpenOperation={(opId) => go({ type: 'operation.selected', opId })}
          onToggleFavourite={onToggleFavourite}
          onRetry={loadLibrary}
          onOpenLog={() => go({ type: 'log.opened' })}
          onOpenSettings={() => go({ type: 'settings.opened' })}
        />
        {leaveDialog}
      </Page>
    );
  }

  if (screen.name === 'category-view') {
    const category = findCategory(categoryCards, screen.categoryId);
    // القسم قد يختفي بين اختياره وفتحه — نواةٌ أُعيد تحميل فهرسها، أو معرّفٌ
    // محفوظٌ من إصدارٍ أقدم. الحالتان تُعرضان نصًّا واحدًا ومخرجًا واضحًا.
    if (!category) {
      return (
        <Page variant="message" focusKey={currentKey}>
          <p className="t-body">{categories === null ? t('lib.loading') : t('lib.category.gone')}</p>
          <button type="button" className="btn btn--quiet" onClick={() => go({ type: 'back' })}>
            {t('nav.back.library')}
          </button>
        </Page>
      );
    }
    return (
      <Page focusKey={currentKey}>
        <CategoryScreen
          category={category}
          operations={operationsIn(operationCards, category.id)}
          favouriteIds={settings.favourites}
          onOpenOperation={(opId) => go({ type: 'operation.selected', opId })}
          onToggleFavourite={onToggleFavourite}
          onBack={() => go({ type: 'back' })}
        />
        {leaveDialog}
      </Page>
    );
  }

  if (screen.name === 'settings') {
    return (
      <Page focusKey={currentKey}>
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
      <Page focusKey={currentKey}>
          <RunLog
          onBack={() => go({ type: 'back' })}
          onRerun={rerun}
          onChanged={loadJournal}
        />
        {leaveDialog}
      </Page>
    );
  }

  // شاشة العملية. الفهرس قد يكون ما زال يُحمَّل، أو قد تكون العملية اختفت منه
  // بين اختيارها وفتحها — والحالتان مختلفتان ولا يجوز أن تُعرضا نصًّا واحدًا.
  if (!operation) {
    return (
      <Page variant="message" focusKey={currentKey}>
        <p className="t-body">{operations === null ? t('lib.loading') : t('ops.gone')}</p>
        <button type="button" className="btn btn--quiet" onClick={() => go({ type: 'back' })}>
          {t('nav.back')}
        </button>
      </Page>
    );
  }

  return (
    <Page variant="bleed" focusKey={currentKey}>
      {/* شريطٌ يعلو السطحين بعرض النافذة، ثم سطحان متجاوران يملآن ما تحته.
          «نَفِّذ» و«سَطْر» ليسا وضعين يتبادلان: هما قراءتان لخطّةٍ واحدة تُعرضان
          معًا، ورؤيةُ الأمر وهو يتغيّر مع الحقل هي الفكرة كلّها. */}
      <div className="op">
        <OperationBar
            operation={operation}
            categoryIcon={categoryIcon}
            onBack={() => go({ type: 'back' })}
          />
        <main className="op__panes">
          <Naffith
            operation={operation}
            categoryIcon={categoryIcon}
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
          />
          <Satr
            plan={shownPlan}
            stream={stream}
            phase={uiPhase}
            status={outcome?.status}
          />
        </main>
      </div>
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
 *
 * ## ولماذا يقبل الوعاء البؤرة
 *
 * تبديل الشاشة في تطبيق صفحةٍ واحدة لا يحرّك التركيز من تلقائه: الزرّ الذي
 * ضُغط يُنزع من الشجرة، فتسقط البؤرة إلى `body` ويستأنف Tab من رأس المستند —
 * يفقد من يتنقّل بلوحة المفاتيح موضعه في كل انتقال.
 *
 * وأكثر الشاشات تعرف أين تضع البؤرة فتضعها بنفسها: عنوانُ الإعدادات والسجلّ
 * وقائمة العمليات، وأولُ حقلٍ في نموذجٍ بِكر. هذا الوعاء **شبكة أمانٍ تحتها**
 * لا آليةٌ ثانية بجانبها: أثرُ الابن يسبق أثر الأب، فحين يصل الدور إلى هنا
 * تكون الشاشة قد أخذت البؤرة وهذا الأثر لا يفعل شيئًا. ولا يلتقطها إلا إن
 * بقيت على `body` — كنموذجٍ مملوء لا يسحب المؤشّر إلى حقله عمدًا.
 */
function Page({
  children,
  variant,
  focusKey,
}: {
  children: ReactNode;
  /** `message` لسطرٍ وزرّ: يُوسَّط رأسيًا بدل أن يبدأ من الأعلى. */
  variant?: 'message' | 'bleed';
  /** هويّة الشاشة نصًّا. تبدّلُها وحده يحرّك البؤرة، لا كل إعادة رسم. */
  focusKey: string;
}) {
  const frame = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const active = document.activeElement;
    const stranded =
      active === null || active === document.body || active === document.documentElement;
    if (!stranded) return;
    frame.current?.focus();
  }, [focusKey]);

  return (
    // `tabIndex={-1}` هدفٌ للبرنامج لا محطّة لـTab: الوعاء ليس شيئًا يُفعل به
    // شيء، وإقحامه في ترتيب التبويب يزيد ضغطةً بلا مقابل في كل شاشة.
    <div className={variant ? `page page--${variant}` : 'page'} tabIndex={-1} ref={frame}>
      {children}
    </div>
  );
}

/**
 * ما يمكن أن يقع عليه Tab داخل الحوار.
 *
 * قائمةٌ مكتوبة لا مكتبة: الحوار زرّان لا أكثر، وحبسُ البؤرة فيهما لا يستحقّ
 * تبعيةً كاملة بشجرتها وتحديثاتها. والمعطَّل مستثنًى لأنه ليس محطّة أصلًا،
 * و`tabindex="-1"` كذلك لأنه هدفٌ للبرنامج لا للمفتاح.
 */
const FOCUS_STOPS =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]),' +
  ' textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/**
 * قرار المغادرة أثناء تشغيل نشط.
 *
 * حوارٌ مقاطع لا إشعارٌ عابر: الانتقال الصامت أثناء تشغيلٍ يكتب على قرص
 * المستخدم يخفي عنه أن شيئًا ما زال يجري. و«البقاء» هو الفعل الافتراضي —
 * يأخذ التركيز — لأن الخطأ في اتجاه البقاء غير مكلف.
 *
 * ## `aria-modal` دعوى، وهذه براهينها الثلاثة
 *
 * الوسم وحده لا يفعل شيئًا؛ من يقرأه ينتظر سلوكًا:
 *
 * - **Escape يُغلق** بمعنى «البقاء»، وهو ما يفعله النقر على العتمة أصلًا.
 *   المخرج الآمن هو غير المكلف دائمًا، فمن ضغط Escape هربًا من الحوار لا
 *   يجوز أن يجد تشغيله وقد غُودر.
 * - **البؤرة محبوسة**: بلا حبسٍ يخرج Tab إلى الشاشة التي خلف العتمة، فيملأ
 *   المستخدم حقولًا لا يراها ويظنّ الحوار ما زال يحاوره.
 * - **البؤرة تُعاد** إلى ما كان يحملها عند الفتح، فلا تُدفع إلى `body` بعد
 *   قرارٍ اتُّخذ ويستأنف Tab من رأس المستند.
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
  const box = useRef<HTMLDivElement>(null);

  // مرّةً واحدة عند الفتح: يُحفظ حاملُ البؤرة ثم تُنقل إلى «البقاء»، وتُعاد
  // إليه عند الإغلاق. وشرطُ `isConnected` لأن الحوار قد يُغلق بمغادرةٍ تنزع
  // الشاشة كلها — وتركيزُ عنصرٍ خارج الشجرة يترك البؤرة على `body` صامتًا.
  useEffect(() => {
    const opener = document.activeElement;
    stay.current?.focus();
    return () => {
      if (opener instanceof HTMLElement && opener.isConnected) opener.focus();
    };
  }, []);

  // المستمع على المستند لا على الحوار: Escape يجب أن يُغلق أينما وقعت البؤرة،
  // ولو كانت خارج الحوار في اللحظة التي تسبق حبسها.
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        event.preventDefault();
        onStay();
        return;
      }
      if (event.key !== 'Tab') return;

      const stops = box.current?.querySelectorAll<HTMLElement>(FOCUS_STOPS) ?? [];
      const first = stops[0];
      const last = stops[stops.length - 1];
      if (!first || !last) return;

      const active = document.activeElement;
      const inside = box.current?.contains(active) ?? false;
      // الخروج من الطرف يُلفّ إلى الطرف الآخر؛ والبؤرة الشاردة خارج الحوار
      // تُعاد إلى أوّله بدل أن تتابع في الصفحة المحجوبة تحته.
      const edge = event.shiftKey ? first : last;
      const wrap = event.shiftKey ? last : first;
      if (!inside || active === edge) {
        event.preventDefault();
        wrap.focus();
      }
    }

    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, [onStay]);

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
        ref={box}
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
