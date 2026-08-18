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
import LibraryScreen, { type LibraryState } from './library-screen';
import CategoryScreen from './category-screen';
import SettingsScreen from './settings-screen';
import RunLog from './run-log';
import AppShell from './app-navigation';
import StatePanel from './state-panel';
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
  withCargoPath,
  withFavouriteToggled,
  withLastCategory,
  withNodePath,
  withOnboardingCompleted,
  withOnboardingReset,
  type Settings,
} from './settings';
import { applyTheme } from './theme';
import { playSuccessChime } from './notification-sound';
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
/**
 * حقول مسار أداةٍ يُختار مرّةً في الإعدادات ويُتذكَّر — Node.js وCargo
 * كلاهما بالضبط، لا فرق بينهما هنا سوى الاسم والقيمة المحفوظة.
 *
 * دالّةٌ لا ثابتًا: القيم تتغيّر مع الإعداد، والمعرّفان (`inputId`) وحدهما
 * ثابتان — وهما نفس المعرّفَين اللذين تُعلنهما عمليات `dev.npm.*` و`dev.cargo.*`
 * في النواة، فتغييرهما هنا بلا تغييرهما هناك يكسر التعبئة التلقائية صامتًا.
 */
function toolPathFields(
  nodePath: string | null,
  cargoPath: string | null,
): Array<{ inputId: 'node_path' | 'cargo_path'; value: string | null }> {
  return [
    { inputId: 'node_path', value: nodePath },
    { inputId: 'cargo_path', value: cargoPath },
  ];
}

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
  // بحث المكتبة جزءٌ من سياق الفتح لا حالةٌ مؤقتة داخل شاشة تُفكّ عند فتح
  // نتيجة. إبقاؤه هنا يجعل «رجوع» يعيد شاشة النتائج نفسها واستعلامها.
  const [libraryQuery, setLibraryQuery] = useState('');

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
  // خريطةٌ بمعرّف العملية لا نموذجٌ واحد: التنقّل داخل شاشة العملية لا يخلط
  // حقول عمليتين. مغادرة نموذجٍ معدّل تمرّ بحوار Page 18، والمغادرة المؤكدة
  // تمحو قيم تلك العملية وحدها كي يطابق الفعل نصّ الحوار.
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

  /**
   * مرآةٌ لآخر قيمة لإعداد الصوت، يقرؤها مستمعٌ لا يُعاد تسجيله عند تبدّلها.
   *
   * لو كان `settings.notificationSound` في مصفوفة اعتماديات الأثر أدناه —
   * كما كانت قبل هذا التعليق — لأعاد الأثر تسجيل `onRunFinished`/`onRunOutput`
   * عند كل ضغطة على مفتاح الصوت في الإعدادات. وتشغيلٌ يمكن أن يبقى جاريًا في
   * الخلفية (زرّ «المغادرة على أي حال» في حوار المغادرة أثناء تشغيل لا يُلغيه)
   * بينما يفتح المستخدم الإعدادات ويبدّل الصوت — ففكّ الاستماع وإعادته حينها
   * نافذةٌ حقيقية تضيع فيها `run://finished` نفسها لا صدى فقط: لا طابور إعادة
   * تشغيلٍ في أحداث Tauri، وضياع تلك النتيجة يُجمّد `phase` على `'running'`
   * بلا مخرجٍ آخر. القراءة من مرجعٍ تتفادى هذا كله: يُسجَّل المستمع مرّةً عند
   * التركيب ويقرأ أحدث قيمةٍ دائمًا دون أن يُعاد تسجيله أبدًا.
   */
  const notificationSoundRef = useRef(settings.notificationSound);
  useEffect(() => {
    notificationSoundRef.current = settings.notificationSound;
  }, [settings.notificationSound]);

  useEffect(() => {
    onRunFinished((event) => {
      setOutcome(event);
      // نهاية التشغيل هي الحالة الأحدث. خطأُ محاولة إلغاءٍ سابقة لا يبقى
      // معلّقًا تحت ResultView بعد أن وصلت النتيجة الفعلية.
      setError(null);
      setPhase('finished');
      setRunId(null);
      // السجلّ صار فيه قيدٌ جديد، و«المستخدَمة حديثًا» تُقرأ منه.
      loadJournal();
      // ‏`status` هنا مستوى النقل — «نجح التشغيل كعملية» — لا تفسير جوابها:
      // بحثٌ بلا نتائج أو عملية `unsigned` كلاهما `success` أيضًا، وكلاهما
      // يستحقّ النغمة. الفشل والإشارة والإلغاء وحدها لا تُشغِّلها.
      if (notificationSoundRef.current && event.status === 'success') {
        playSuccessChime();
      }
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
    const saved = formsByOp[operation.id];
    if (saved) return saved;

    // مسارا Node.js وCargo يُختاران مرّةً — من الإعدادات، أو من أول عملية
    // أدوات مطوّرين تحتاجهما — ويُتذكَّران بعدها. القيمة تُعرض كأي نموذجٍ
    // ابتدائي آخر ويظلّ الحقل قابلًا للتعديل، لا فرضًا صامتًا: التحقّق
    // الحقيقي يبقى في النواة وقت التخطيط كأي مسارٍ آخر.
    const initial = emptyValues(operation);
    for (const { inputId, value } of toolPathFields(settings.nodePath, settings.cargoPath)) {
      if (value && operation.inputs.some((input) => input.id === inputId)) {
        initial[inputId] = value;
      }
    }
    return initial;
  }, [operation, formsByOp, settings.nodePath, settings.cargoPath]);

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

  /** خطّةٌ تنتظر تأكيدًا صريحًا قبل التنفيذ — انظر توثيق `confirmBeforeExecute`
   *  في `settings.ts`: حاجزٌ ثانٍ فوق المعاينة الدائمة، للعمليات التي تعدّل
   *  أو تتلف وحدها. */
  const [confirmExecutePlan, setConfirmExecutePlan] = useState<{
    plan: PlanResponse;
    danger: 'creates' | 'modifies' | 'destructive';
  } | null>(null);

  const runExecute = useCallback(async (toRun: PlanResponse) => {
    setError(null);
    setOutcome(null);
    // مجرى التشغيل السابق يُمحى هنا: من هذه اللحظة كل سطرٍ يصل يخصّ هذا التشغيل.
    setStream([]);
    setPhase('running');
    try {
      setExecuted(toRun);
      setRunId(await execute(toRun.token));
      // الرمز أحادي الاستخدام: الخطة استُهلكت، فلا معنى لإبقائها قابلة للتنفيذ.
      setPlan(null);
    } catch (e) {
      setError(asCoreError(e));
      setPhase('idle');
    }
  }, []);

  const onExecute = useCallback(() => {
    if (!plan) return;
    // القراءة فقط لا تُسأل عنها أبدًا: لا شيء فيها يُعدَّل أو يُتلَف كي
    // يستحقّ حاجزًا ثانيًا فوق المعاينة التي تُعرض في كل حال.
    if (settings.confirmBeforeExecute && plan.danger !== 'safe') {
      setConfirmExecutePlan({ plan, danger: plan.danger });
      return;
    }
    void runExecute(plan);
  }, [plan, settings.confirmBeforeExecute, runExecute]);

  const confirmExecuteNow = useCallback(() => {
    if (!confirmExecutePlan) return;
    const { plan: toRun } = confirmExecutePlan;
    setConfirmExecutePlan(null);
    void runExecute(toRun);
  }, [confirmExecutePlan, runExecute]);

  const cancelConfirmExecute = useCallback(() => setConfirmExecutePlan(null), []);

  // حارسٌ صريح لا اتّكال على أن العتمة تمنع كل شيء: العتمة تمنع اليوم فعلًا —
  // لا مسار حاليّ يبدّل العملية المفتوحة أو يفرغها والحوار قائم — لكنها ليست
  // القاعدة المعلَنة، بل أثرًا جانبيًا لتغطيتها الشاشة. نفس منطق `planSeq`
  // أعلاه الذي يُبطل معاينةً سبقتها أحدث: خطّةُ تأكيدٍ تخصّ عمليةً لم تعد
  // مفتوحة يجب أن تُبطَل صراحةً، لا أن تبقى صالحةً بمصادفة تخطيطٍ حاليّ.
  useEffect(() => {
    setConfirmExecutePlan(null);
  }, [operation?.id]);

  const onCancel = useCallback(async () => {
    if (!runId) return;
    setPhase('cancelling');
    try {
      await cancelRun(runId);
    } catch (e) {
      setError(asCoreError(e));
      // رفض طلب الإلغاء لا يعني أن التشغيل توقف. نعود إلى «جارٍ» كي يبقى
      // زر الإلغاء قابلًا للمحاولة، ولا تُحبس الشاشة في `cancelling` المعطّلة.
      setPhase('running');
    }
  }, [runId]);

  const onReveal = useCallback(async () => {
    if (!outcome?.run_id) return;
    // Reveal فعلٌ تالٍ مستقل عن نتيجة التشغيل. نمحو خطأ محاولةٍ سابقة قبل
    // الجديدة، ولا نبدّل النتيجة الناجحة نفسها إن كان الملف قد نُقل بعد ذلك.
    setError(null);
    try {
      await revealRun(outcome.run_id);
    } catch (e) {
      setError({ key: 'err.reveal.failed', input: null, detail: e });
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

  /** كلفة المغادرة كما تعرضها حوارات Page 18: تشغيل جارٍ، أو نموذج لم يُنفّذ. */
  const formDirty = Boolean(
    operation &&
      phase === 'idle' &&
      operation.inputs.some(
        (input) =>
          (values[input.id] ?? '') !== (emptyValues(operation)[input.id] ?? ''),
      ),
  );
  const exitCost: ExitCost =
    phase === 'running' || phase === 'cancelling'
      ? 'busy'
      : formDirty
        ? 'dirty'
        : 'free';

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
    if (pending.reason === 'dirty' && operation) {
      setFormsByOp((all) => {
        const next = { ...all };
        delete next[operation.id];
        return next;
      });
    }
    const result = confirmNavigation(pending.screen);
    if (result.kind === 'navigate') setScreen(result.screen);
    setPending(null);
  }, [pending, operation]);

  const persist = useCallback(
    (next: Settings) => {
      setSettings(next);
      saveSettings(storage, next);
    },
    [storage],
  );

  // السمة تُنفَّذ من هنا لا من شاشة الإعدادات: لو عاشت هناك لَما طُبِّقت إلا
  // بعد أن يفتح المستخدم الشاشة، فيبدأ التطبيق فاتحًا ثم يقفز إلى الداكن.
  // هنا تُطبَّق عند الإقلاع بالقيمة المحمَّلة، وعند كل تغيير بعدها.
  useEffect(() => {
    applyTheme(settings.theme, document.documentElement);
  }, [settings.theme]);

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

  // مسارا Node.js وCargo: يُختاران مرّةً — من الإعدادات، أو من أول حقل
  // `node_path`/`cargo_path` تملؤه شاشة عمليةٍ من أدوات المطوّرين — ويُتذكَّران
  // لكل عملياتهما بعده.
  const setNodePath = useCallback(
    (path: string | null) => {
      persist(withNodePath(settings, path));
    },
    [persist, settings],
  );
  const setCargoPath = useCallback(
    (path: string | null) => {
      persist(withCargoPath(settings, path));
    },
    [persist, settings],
  );

  // يعكس التعديل اليدوي على حقل مسار أداةٍ إلى الإعداد المحفوظ، فتُملأ به كل
  // عملية أدوات مطوّرين لاحقة لا هذه وحدها. لا يكتب حين تكون القيمة نفسها
  // (تجنّبًا لكتابةٍ عند كل رسم)، ولا حين يكون الحقل فارغًا (المستخدم في
  // منتصف الكتابة — القيمة الفارغة ليست اختيارًا يستحقّ أن يُنسى به المحفوظ).
  useEffect(() => {
    if (!operation) return;
    const setters: Record<'node_path' | 'cargo_path', (path: string) => void> = {
      node_path: setNodePath,
      cargo_path: setCargoPath,
    };
    for (const { inputId, value: saved } of toolPathFields(settings.nodePath, settings.cargoPath)) {
      if (!operation.inputs.some((input) => input.id === inputId)) continue;
      const current = values[inputId];
      if (!current || current === saved) continue;
      setters[inputId](current);
    }
  }, [operation, values, settings.nodePath, settings.cargoPath, setNodePath, setCargoPath]);

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
      // «السجل» بطاقةٌ في مكتبة الفئات كي يبقى الوصول إليه متوقّعًا في
      // البحث/الشبكة، لكنه ليس قسم عمليات. نوعه المعلن من النواة هو القرار؛
      // فلا نفتح له شاشة فئة فارغة قبل شاشة السجل الحقيقية.
      const category = findCategory(categoryCards, categoryId);
      go(category?.kind === 'journal'
        ? { type: 'log.opened' }
        : { type: 'category.selected', categoryId });
    },
    [persist, settings, go, categoryCards],
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

  const activeCategoryId =
    screen.name === 'category-view'
      ? screen.categoryId
      : screen.name === 'operation-view'
        ? operation?.category
        : undefined;

  const inShell = (content: ReactNode, viewportMode: 'scroll' | 'fixed' = 'scroll') => (
    <AppShell
      screen={screen}
      categories={categoryCards}
      activeCategoryId={activeCategoryId}
      viewportMode={viewportMode}
      iconSize={settings.sidebarIconSize}
      onOpenLibrary={() => go({ type: 'library.opened' })}
      onOpenCategory={openCategory}
      onOpenLog={() => go({ type: 'log.opened' })}
      onOpenSettings={() => go({ type: 'settings.opened' })}
    >
      {content}
    </AppShell>
  );

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
    return inShell(
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
          initialQuery={libraryQuery}
          onQueryChange={setLibraryQuery}
        />
        {leaveDialog}
      </Page>,
    );
  }

  if (screen.name === 'category-view') {
    const category = findCategory(categoryCards, screen.categoryId);
    // القسم قد يختفي بين اختياره وفتحه — نواةٌ أُعيد تحميل فهرسها، أو معرّفٌ
    // محفوظٌ من إصدارٍ أقدم. الحالتان تُعرضان نصًّا واحدًا ومخرجًا واضحًا.
    if (!category) {
      return inShell(
        <Page variant="message" focusKey={currentKey}>
          {categories === null ? (
            <StatePanel
              title={t('lib.loading')}
              body={t('lib.category.loading.body')}
              busy
              headingLevel={2}
            />
          ) : (
            <StatePanel
              title={t('lib.category.gone.title')}
              body={t('lib.category.gone.body')}
              tone="danger"
              action={t('action.return.library')}
              onAction={() => go({ type: 'back' })}
              headingLevel={2}
            />
          )}
        </Page>,
      );
    }
    return inShell(
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
      </Page>,
    );
  }

  if (screen.name === 'settings') {
    return inShell(
      <Page focusKey={currentKey}>
        <SettingsScreen
          settings={settings}
          onSettingsChange={persist}
          storageAvailable={storageAvailable}
          onReplayOnboarding={replayOnboarding}
        />
        {leaveDialog}
      </Page>,
    );
  }

  if (screen.name === 'run-log') {
    return inShell(
      <Page focusKey={currentKey}>
          <RunLog
          onBack={() => go({ type: 'back' })}
          onRerun={rerun}
          onChanged={loadJournal}
        />
        {leaveDialog}
      </Page>,
    );
  }

  // شاشة العملية. الفهرس قد يكون ما زال يُحمَّل، أو قد تكون العملية اختفت منه
  // بين اختيارها وفتحها — والحالتان مختلفتان ولا يجوز أن تُعرضا نصًّا واحدًا.
  if (!operation) {
    return inShell(
      <Page variant="message" focusKey={currentKey}>
        {operations === null ? (
          <StatePanel
            title={t('lib.loading')}
            body={t('ops.loading.body')}
            badge="Loading"
            busy
            headingLevel={2}
          />
        ) : (
          <StatePanel
            title={t('ops.gone.title')}
            body={t('ops.gone.body')}
            badge="Gone"
            tone="danger"
            action={t('action.return')}
            onAction={() => go({ type: 'back' })}
            headingLevel={2}
          />
        )}
      </Page>,
    );
  }

  return inShell(
    <Page variant="bleed" focusKey={currentKey}>
      <div className="op">
        <main className={uiPhase === 'finished' && outcome ? 'op__result' : 'op__panes'}>
          <Naffith
            operation={operation}
            categoryIcon={categoryIcon}
            values={values}
            onChange={setValues}
            plan={plan}
            executionPlan={shownPlan}
            error={error}
            phase={uiPhase}
            outcome={outcome}
            onBack={() => go({ type: 'back' })}
            onExecute={onExecute}
            onReveal={onReveal}
            onReset={onReset}
            onLibrary={() => go({ type: 'library.opened' })}
          />
          {!(uiPhase === 'finished' && outcome) && (
            <Satr
              plan={shownPlan}
              stream={stream}
              phase={uiPhase}
              status={outcome?.status}
              onCancel={onCancel}
            />
          )}
        </main>
      </div>
      {confirmExecutePlan && (
        <ConfirmExecute
          danger={confirmExecutePlan.danger}
          onCancel={cancelConfirmExecute}
          onConfirm={confirmExecuteNow}
        />
      )}
      {leaveDialog}
    </Page>,
    'fixed',
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
 * القالب المشترك لكل حوارٍ يقاطع — يعرض قرارًا بخيارين ويحبس البؤرة حتى
 * يُتّخذ.
 *
 * استُخرج من حوار المغادرة (الاستعمال الأول) حين احتاج حوار تأكيد التنفيذ
 * (الثاني) الآلية نفسها حرفًا بحرف: بؤرةٌ محبوسة، Escape يعني الخيار الآمن،
 * والبؤرة تعود إلى حاملها عند الإغلاق. نسخُ هذا في كل حوارٍ جديد كان يعني
 * عطلًا يُصلَح في نسخةٍ وينسى في الأخرى — وهو بالضبط ما يمنعه استخراجه هنا.
 *
 * ## `aria-modal` دعوى، وهذه براهينها الثلاثة
 *
 * الوسم وحده لا يفعل شيئًا؛ من يقرأه ينتظر سلوكًا:
 *
 * - **Escape يُغلق** بمعنى `onPrimary` — الخيار الآمن دائمًا، وهو ما يفعله
 *   النقر على العتمة أصلًا. المخرج الآمن هو غير المكلف دائمًا، فمن ضغط
 *   Escape هربًا من الحوار لا يجوز أن يقع في الخيار الآخر.
 * - **البؤرة محبوسة**: بلا حبسٍ يخرج Tab إلى الشاشة التي خلف العتمة، فيملأ
 *   المستخدم حقولًا لا يراها ويظنّ الحوار ما زال يحاوره.
 * - **البؤرة تُعاد** إلى ما كان يحملها عند الفتح، فلا تُدفع إلى `body` بعد
 *   قرارٍ اتُّخذ ويستأنف Tab من رأس المستند.
 *
 * و`onPrimary` — الزرّ الأول، الذي يأخذ التركيز، والذي يعنيه Escape والنقر
 * على العتمة — هو **الخيار الآمن دائمًا** لا الخيار الأول بالمصادفة: الخطأ
 * في اتجاهه غير مكلف، وهذا ما يبرّر منحه الفعل الافتراضي.
 */
function ConfirmDialog({
  dialogId,
  title,
  body,
  primaryLabel,
  onPrimary,
  secondaryLabel,
  onSecondary,
  secondaryTone,
  hint,
}: {
  dialogId: string;
  title: string;
  body: string;
  primaryLabel: string;
  onPrimary: () => void;
  secondaryLabel: string;
  onSecondary: () => void;
  secondaryTone: 'danger' | 'quiet' | 'primary';
  hint?: string;
}) {
  const primary = useRef<HTMLButtonElement>(null);
  const box = useRef<HTMLDivElement>(null);

  // مرّةً واحدة عند الفتح: يُحفظ حاملُ البؤرة ثم تُنقل إلى الخيار الآمن،
  // وتُعاد إليه عند الإغلاق. وشرطُ `isConnected` لأن الحوار قد يُغلق
  // بمغادرةٍ تنزع الشاشة كلها — وتركيزُ عنصرٍ خارج الشجرة يترك البؤرة على
  // `body` صامتًا.
  useEffect(() => {
    const opener = document.activeElement;
    primary.current?.focus();
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
        onPrimary();
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
  }, [onPrimary]);

  const titleId = `${dialogId}-title`;
  const bodyId = `${dialogId}-body`;

  return (
    <div className="scrim" role="presentation" onClick={onPrimary}>
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
          {title}
        </h2>
        <p id={bodyId} className="t-body-sec dialog__body">
          {body}
        </p>
        <div className="dialog__actions">
          <button ref={primary} type="button" className="btn btn--primary" onClick={onPrimary}>
            {primaryLabel}
          </button>
          <button type="button" className={`btn btn--${secondaryTone}`} onClick={onSecondary}>
            {secondaryLabel}
          </button>
        </div>
        {hint && <p className="dialog__hint">{hint}</p>}
      </div>
    </div>
  );
}

/**
 * قرار المغادرة أثناء تشغيل نشط أو نموذجٍ لم يُنفَّذ — واجهةٌ رقيقة فوق
 * `ConfirmDialog`. «البقاء» هو الخيار الآمن دائمًا؛ انظر توثيق `ConfirmDialog`
 * لسبب ذلك.
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
  return (
    <ConfirmDialog
      dialogId={`leave-${reason}`}
      title={t(`nav.leave.${reason}.title`)}
      body={t(`nav.leave.${reason}.body`)}
      primaryLabel={t(`nav.leave.${reason}.stay`)}
      onPrimary={onStay}
      secondaryLabel={t(`nav.leave.${reason}.leave`)}
      onSecondary={onLeave}
      secondaryTone={reason === 'dirty' ? 'danger' : 'quiet'}
      hint={t('dialog.safe_dismiss')}
    />
  );
}

/**
 * تأكيد التنفيذ لعمليةٍ تعدّل أو تتلف — واجهةٌ رقيقة فوق `ConfirmDialog`،
 * تُعرض فقط حين يفعّل المستخدم `confirmBeforeExecute` في الإعدادات.
 *
 * «الإلغاء» هو الخيار الآمن هنا — لا «عام» كما في `ConfirmLeave`: أن يُنفَّذ
 * أمرٌ لم يُقصد تنفيذه هو الخطأ المكلف، لا العكس. ولذلك يأخذ التركيز ويعنيه
 * Escape والنقر على العتمة، مطابقًا نفس القاعدة التي طبّقها `ConfirmLeave`
 * على «البقاء» — الاتجاه الآمن يتغيّر، والقاعدة (امنحه الافتراضي) لا تتغيّر.
 *
 * وزرّ «نفِّذ» لا يأخذ `btn--primary` أبدًا هنا — ولو بدا ذلك متّسقًا مع كونه
 * الفعل المطلوب: `ConfirmDialog` يحجز `btn--primary` لزرّها الأول (الآمن)
 * دائمًا، فمنحُ الثاني الصنف نفسه في درجتي «ينشئ»/«يعدّل» كان يُلغي التمييز
 * البصري الذي هذا الحوار وُجد من أجله — كلا الزرّين يبدوان الفعل الرئيسي.
 * ‏`ConfirmLeave` لا يقع في هذا: زرّها الثاني إمّا `btn--danger` أو
 * `btn--quiet`، لا `btn--primary` أبدًا. فالدرجات الثلاث هنا: تلقائي/خافت
 * لـ«ينشئ» و«يعدّل» (فعلٌ متاحٌ لا مُلحّ)، وخطرٌ لـ«تلف» وحدها.
 */
function ConfirmExecute({
  danger,
  onCancel,
  onConfirm,
}: {
  danger: 'creates' | 'modifies' | 'destructive';
  onCancel: () => void;
  onConfirm: () => void;
}) {
  return (
    <ConfirmDialog
      dialogId="run-confirm"
      title={t(`run.confirm.${danger}.title`)}
      body={t(`run.confirm.${danger}.body`)}
      primaryLabel={t('action.cancel')}
      onPrimary={onCancel}
      secondaryLabel={t('action.execute')}
      onSecondary={onConfirm}
      secondaryTone={danger === 'destructive' ? 'danger' : 'quiet'}
      hint={t('dialog.safe_dismiss')}
    />
  );
}
