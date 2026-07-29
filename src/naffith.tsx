/**
 * نَفِّذ — السطح الهادئ.
 *
 * النموذج يُشتقّ من `OperationSummary` القادم من الفهرس، لا من قائمة حقول
 * مكتوبة هنا: العملية تعلن مدخلاتها، والشاشة ترسمها. وكل تغيير يعيد التخطيط
 * فورًا، فالمعاينة والملخّص وشاشة «سَطْر» كلها ناتج نفس الخطة الحيّة.
 *
 * كان هذا الملف يكتب `source` و`destination` و`archive_name` بيده، فيصدق
 * التوثيق أعلاه على الورق وحده: كل عملية جديدة كانت ستحتاج فرعًا هنا. الآن
 * الحقول تُرسم من `operation.inputs` بترتيب النواة، والدوالُّ التي تقرّر شكل
 * كل حقل تعيش في `operations.ts` — فلا تعرف هذه الشاشة اسم عمليةٍ واحدة.
 */
import { useEffect, useId, useRef, type Ref } from 'react';
import type {
  CoreErrorShape,
  Danger,
  InputSummary,
  OperationSummary,
  PlanResponse,
} from './ipc';
import { asCoreError, pickDirectory } from './ipc';
import {
  extensionHint,
  fieldKeys,
  iconForCategory,
  isComplete,
  isDirectoryInput,
  isDirty,
  isFlagInput,
  isPathInput,
  type FormValues,
} from './operations';
import { errorText, t, tFirst } from './i18n';

type Phase = 'idle' | 'planning' | 'running' | 'cancelling' | 'finished';

/**
 * `FormValues` تعريفها في `operations.ts` مع الدوالّ التي تقرأها وتكتبها.
 * تُصدَّر من هنا كذلك لأن الشاشة الجامعة تستوردها مع المكوّن نفسه.
 */
export type { FormValues } from './operations';

interface Props {
  /** العملية كما أعلنتها النواة. منها يُرسم النموذج كلّه. */
  operation: OperationSummary;
  values: FormValues;
  onChange: (next: FormValues) => void;
  plan: PlanResponse | null;
  error: CoreErrorShape | null;
  phase: Phase;
  outcome: { status: string; produced?: string | null; key?: string } | null;
  onExecute: () => void;
  onCancel: () => void;
  onReveal: () => void;
  onReset: () => void;
  onBack: () => void;
}

/**
 * أيقونة الحقل، مشتقّة لا مكتوبة.
 *
 * المسار يأخذ أيقونته من نوعه (مجلد أو ملف)، وما ليس مسارًا يأخذ أيقونة فئة
 * العملية نفسها: الحقل الذي يسمّي الناتج يحمل صورة ما يُنتَج. خريطةٌ من معرّف
 * الحقل إلى أيقونة كانت ستكون قائمة الحقول المكتوبة يدويًا عائدةً من الباب
 * الخلفي، وتُنسى فيظهر حقلٌ بلا أيقونة في أول عملية جديدة.
 */
function iconFor(op: OperationSummary, input: InputSummary): string {
  if (isDirectoryInput(input)) return '#i-folder';
  if (isPathInput(input)) return '#i-file';
  return iconForCategory(op.category);
}

/** شارة الخطورة: خريطة مغلقة على نوعٍ مغلق، فلا تتقادم بعملية جديدة. */
const DANGER_ICON: Record<Danger, string> = {
  safe: '#i-check',
  creates: '#i-plus',
  modifies: '#i-rename',
  destructive: '#i-delete',
};

const DANGER_CHIP: Record<Danger, string> = {
  safe: 'chip--neutral',
  creates: 'chip--info',
  modifies: 'chip--info',
  destructive: 'chip--danger',
};

/** سطر خطأ الحقل. مفصول لأن كل نوع حقل يعرضه بنفس الشكل والدور. */
function FieldError({ id, text }: { id: string; text: string }) {
  return (
    <p id={id} className="field-error" role="alert">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <use href="#i-error" />
      </svg>
      {text}
    </p>
  );
}

/**
 * حقل مسار: يُملأ بالحوار أو بالكتابة. الاثنان يمرّان بنفس التحقّق.
 *
 * `onPick` قد تكون `null`: حوار النظام المتاح للواجهة حوار **مجلدات**، فمدخلٌ
 * يطلب ملفًا قائمًا يُعرض بلا زرّ اختيار بدل أن يُفتح له حوارٌ لا يستطيع
 * انتقاءه. الكتابة واللصق يبقيان طريقًا كاملًا في الحالتين.
 */
function PathField({
  id,
  icon,
  label,
  help,
  placeholder,
  value,
  error,
  disabled,
  onPick,
  onType,
  inputRef,
}: {
  id: string;
  icon: string;
  label: string;
  help: string;
  placeholder: string;
  value: string;
  error: string | null;
  disabled: boolean;
  onPick: (() => void) | null;
  onType: (v: string) => void;
  inputRef: Ref<HTMLInputElement> | null;
}) {
  const errorId = `${id}-error`;
  const helpId = `${id}-help`;
  return (
    <div className="row-field">
      <label className="t-label" htmlFor={id}>
        {label}
      </label>
      <div className="row-field__control">
        {/* المسار LTR داخل نموذج عربي: قراءته من اليمين تفسد ترتيب مقاطعه. */}
        <span className="field field--path">
          <svg viewBox="0 0 24 24" aria-hidden="true" className="field__icon">
            <use href={icon} />
          </svg>
          <input
            id={id}
            ref={inputRef}
            type="text"
            dir="ltr"
            spellCheck={false}
            autoComplete="off"
            value={value}
            placeholder={placeholder}
            disabled={disabled}
            aria-describedby={error ? `${errorId} ${helpId}` : helpId}
            aria-invalid={error ? true : undefined}
            onChange={(e) => onType(e.target.value)}
          />
        </span>
        {onPick && (
          <button type="button" className="btn btn--quiet" onClick={onPick} disabled={disabled}>
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-folder-open" />
            </svg>
            {value ? t('field.chosen') : t('field.choose')}
          </button>
        )}
      </div>
      <p id={helpId} className="t-helper">
        {help}
      </p>
      {error && <FieldError id={errorId} text={error} />}
    </div>
  );
}

/** حقل نصّي: اسمٌ جديد أو نصّ حرّ. اللاحقة تأتي من المواصفة لا من نصٍّ مكتوب. */
function TextField({
  id,
  icon,
  label,
  help,
  placeholder,
  suffix,
  value,
  error,
  disabled,
  onType,
  inputRef,
}: {
  id: string;
  icon: string;
  label: string;
  help: string;
  placeholder: string;
  suffix: string | null;
  value: string;
  error: string | null;
  disabled: boolean;
  onType: (v: string) => void;
  inputRef: Ref<HTMLInputElement> | null;
}) {
  const errorId = `${id}-error`;
  const helpId = `${id}-help`;
  return (
    <div className="row-field">
      <label className="t-label" htmlFor={id}>
        {label}
      </label>
      <div className="row-field__control">
        {/* اسم الملف نصّ المستخدم، فيبقى باتجاه الصفحة لا LTR قسرًا. */}
        <span className="field">
          <svg viewBox="0 0 24 24" aria-hidden="true" className="field__icon">
            <use href={icon} />
          </svg>
          <input
            id={id}
            ref={inputRef}
            type="text"
            spellCheck={false}
            autoComplete="off"
            value={value}
            placeholder={placeholder}
            disabled={disabled}
            aria-describedby={error ? `${errorId} ${helpId}` : helpId}
            aria-invalid={error ? true : undefined}
            onChange={(e) => onType(e.target.value)}
          />
        </span>
        {/* اللاحقة عرضٌ لما تضيفه النواة، لا جزءٌ من قيمة الحقل. */}
        {suffix && (
          <span className="suffix-hint lat" aria-hidden="true">
            {suffix}
          </span>
        )}
      </div>
      <p id={helpId} className="t-helper">
        {help}
      </p>
      {error && <FieldError id={errorId} text={error} />}
    </div>
  );
}

/**
 * راية: مربّع اختيار حقيقي.
 *
 * مفتاحٌ مرسوم بـ`div` كان سيحتاج `role` و`aria-checked` ومعالجَ مفاتيح ليقول
 * ما يقوله `input[type=checkbox]` مجانًا، ويظلّ يخطئ في وضع التباين العالي.
 */
function FlagField({
  id,
  label,
  help,
  checked,
  error,
  disabled,
  onToggle,
}: {
  id: string;
  label: string;
  help: string;
  checked: boolean;
  error: string | null;
  disabled: boolean;
  onToggle: (on: boolean) => void;
}) {
  const errorId = `${id}-error`;
  const helpId = `${id}-help`;
  return (
    <div className="row-field row-field--flag">
      <label className="t-label row-field__flag" htmlFor={id}>
        <input
          id={id}
          type="checkbox"
          checked={checked}
          disabled={disabled}
          aria-describedby={error ? `${errorId} ${helpId}` : helpId}
          onChange={(e) => onToggle(e.target.checked)}
        />
        <span>{label}</span>
      </label>
      <p id={helpId} className="t-helper">
        {help}
      </p>
      {error && <FieldError id={errorId} text={error} />}
    </div>
  );
}

export default function Naffith(props: Props) {
  const {
    operation,
    values,
    onChange,
    plan,
    error,
    phase,
    outcome,
    onExecute,
    onCancel,
    onReveal,
    onReset,
    onBack,
  } = props;

  const uid = useId();
  const whyId = `${uid}-why`;
  const firstField = useRef<HTMLInputElement>(null);

  const busy = phase === 'running' || phase === 'cancelling';
  const errorFor = (field: string): string | null =>
    error && error.input === field ? errorText(error.key, error.detail) : null;
  // خطأ لا ينسبه النواةُ إلى حقل يُعرض فوق الأزرار لا تحت حقل عشوائي.
  const generalError = error && !error.input ? errorText(error.key, error.detail) : null;

  useEffect(() => {
    // النموذج البِكر يضع المؤشّر في أول حقل معلَن، لا في حقلٍ بعينه بالاسم.
    if (phase === 'idle' && !isDirty(values)) firstField.current?.focus();
  }, [phase, values]);

  async function pick(inputId: string) {
    try {
      const chosen = await pickDirectory();
      if (chosen) onChange({ ...values, [inputId]: chosen });
    } catch {
      // فشل الحوار لا يُعطّل الحقل: الكتابة واللصق يبقيان طريقًا كاملًا.
    }
  }

  /**
   * سبب تعطّل «نفِّذ»، أو `null` إن كان ممكنًا.
   *
   * الترتيب مقصود: النقص أوّلًا لأنه أكثر الأسباب وقوعًا وأقلّها إثارةً للقلق،
   * ثم التحقّق الجاري، ثم الخطأ. وغيابُ خطةٍ مع اكتمال الحقول وسلامتها لحظةٌ
   * عابرة بين ضغطة المفتاح وبدء التخطيط، فتُقرأ «يجري التحقّق» لا شيئًا آخر.
   */
  const blocked: string | null = !isComplete(operation, values)
    ? 'action.execute.why.incomplete'
    : phase === 'planning'
      ? 'action.execute.why.planning'
      : error
        ? 'action.execute.why.invalid'
        : plan
          ? null
          : 'action.execute.why.planning';

  // النصّ يُعرض ما دام الأمر بيد المستخدم. أثناء التشغيل تتكلّم `RunState`،
  // فإبقاؤه هناك كان سيقول «يجري التحقّق» بينما الأداة تعمل فعلًا.
  const showWhy = blocked !== null && !busy && phase !== 'finished';

  const dangerLabelKey = `summary.danger.${operation.danger}`;
  // مفتاحٌ غير مترجَم يعود كما هو من `t`. شارةٌ تعرض `summary.danger.modifies`
  // أسوأ من غياب الشارة: الأولى تشوّش، والثانية تترك الوصف يقول ما تقوله.
  const dangerLabel = t(dangerLabelKey);
  const showDanger = dangerLabel !== dangerLabelKey;

  return (
    <section className="naffith card card--naffith" aria-labelledby="naffith-heading">
      {/* الرجوع أوّل ما يُبلَغ بالمفتاح، كما هو أوّل ما تقع عليه العين في أعلى
          البداية. والشيفرون يُترك بلا `data-directional`: رأسه يشير يمينًا،
          واليمين في صفحة RTL هو جهة الرجوع أصلًا، فقلبُه كان سيعكس المعنى. */}
      <div className="naffith__nav">
        <button type="button" className="btn btn--quiet btn--sm" onClick={onBack}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-chevron" />
          </svg>
          {t('nav.back')}
        </button>
      </div>

      <header className="naffith__head">
        <div>
          <h2 id="naffith-heading" className="t-section-title">
            {t('app.naffith')}
          </h2>
          <p className="t-card-title naffith__op">{t(operation.title_key)}</p>
        </div>
        {showDanger && (
          <span className={`chip ${DANGER_CHIP[operation.danger]}`}>
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href={DANGER_ICON[operation.danger]} />
            </svg>
            {dangerLabel}
          </span>
        )}
      </header>

      <p className="t-body-sec naffith__desc">{t(operation.description_key)}</p>

      <div className="naffith__form">
        {operation.inputs.map((input, index) => {
          const id = `${uid}-${input.id}`;
          const keys = fieldKeys(operation.id, input.id);
          const label = tFirst(keys.label);
          const help = tFirst(keys.help);
          const value = values[input.id] ?? '';
          const fieldError = errorFor(input.id);
          // المؤشّر يقع في أول حقل، وحقل الراية لا يصلح مقصدًا: مربّعُ اختيارٍ
          // مركَّزٌ تلقائيًا يوحي بأن الطريق يبدأ منه وهو غالبًا خيارٌ جانبي.
          const ref = index === 0 && !isFlagInput(input) ? firstField : null;

          if (isFlagInput(input)) {
            return (
              <FlagField
                key={input.id}
                id={id}
                label={label}
                help={help}
                checked={value === '1'}
                error={fieldError}
                disabled={busy}
                onToggle={(on) => onChange({ ...values, [input.id]: on ? '1' : '' })}
              />
            );
          }

          if (isPathInput(input)) {
            return (
              <PathField
                key={input.id}
                id={id}
                icon={iconFor(operation, input)}
                label={label}
                help={help}
                placeholder={tFirst(keys.placeholder)}
                value={value}
                error={fieldError}
                disabled={busy}
                onPick={isDirectoryInput(input) ? () => pick(input.id) : null}
                onType={(v) => onChange({ ...values, [input.id]: v })}
                inputRef={ref}
              />
            );
          }

          return (
            <TextField
              key={input.id}
              id={id}
              icon={iconFor(operation, input)}
              label={label}
              help={help}
              placeholder={tFirst(keys.placeholder)}
              suffix={extensionHint(input)}
              value={value}
              error={fieldError}
              disabled={busy}
              onType={(v) => onChange({ ...values, [input.id]: v })}
              inputRef={ref}
            />
          );
        })}
      </div>

      <Summary plan={plan} phase={phase} />

      {generalError && (
        <p className="field-error field-error--block" role="alert">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-error" />
          </svg>
          {generalError}
        </p>
      )}

      <div className="naffith__act">
        <div className="naffith__actions">
          {phase !== 'finished' && (
            <>
              {/* الزرّ ظاهر دائمًا ومعطّل بسببٍ مكتوب تحته. زرٌّ يظهر ويختفي مع
                  صلاحية الخطة يجعل الفعل الأساسي يرقص مع كل ضغطة مفتاح، ويحرم
                  من لم تكتمل حقوله من معرفة أن ثمّة زرًّا أصلًا. */}
              <button
                type="button"
                className="btn btn--primary btn--lg"
                onClick={onExecute}
                disabled={blocked !== null || busy}
                aria-describedby={showWhy ? whyId : undefined}
              >
                <svg viewBox="0 0 24 24" aria-hidden="true">
                  <use href="#i-execute" />
                </svg>
                {t('action.execute')}
              </button>
              {busy && (
                <button
                  type="button"
                  className="btn btn--danger"
                  onClick={onCancel}
                  disabled={phase === 'cancelling'}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <use href="#i-close" />
                  </svg>
                  {t('action.cancel')}
                </button>
              )}
            </>
          )}
          {phase === 'finished' && (
            <button type="button" className="btn btn--quiet btn--lg" onClick={onReset}>
              {t('action.again')}
            </button>
          )}
        </div>

        {/* حالة لا خطأ: `t-helper` خافتة صغيرة، بلا لون الخطر وبلا `role=alert`.
            و`aria-live` مهذّبة لأن السبب يتبدّل تحت يد المستخدم وهو يكتب. */}
        {showWhy && blocked && (
          <p id={whyId} className="t-helper naffith__why" aria-live="polite">
            {t(blocked)}
          </p>
        )}
      </div>

      <RunState phase={phase} outcome={outcome} onReveal={onReveal} />
    </section>
  );
}

/**
 * حجم مقروء بالعربية.
 *
 * القاعدة العشرية (١٠٠٠) لا الثنائية: الرقم يُقارَن بمساحة قرص، وFinder نفسه
 * يعرض القرص عشريًا. عرضُ ٩٥٤ م.ب لما يسمّيه النظام غيغابايت ارتباك لا دقّة.
 * والرقم يبقى LTR داخل الجملة العربية عبر `<bdi>` في موضع الاستدعاء.
 */
export function readableSize(bytes: number): string {
  const units: [number, string][] = [
    [1e9, 'unit.gb'],
    [1e6, 'unit.mb'],
    [1e3, 'unit.kb'],
  ];
  for (const [scale, key] of units) {
    if (bytes >= scale) {
      const n = bytes / scale;
      // منزلة واحدة تحت ١٠، وعدد صحيح فوقها: «١٢٫٤ م.ب» مفيد، «١٢٤٫٧ م.ب» ضجيج.
      return `${n < 10 ? n.toFixed(1) : Math.round(n)} ${t(key)}`;
    }
  }
  return `${bytes} ${t('unit.bytes')}`;
}

/**
 * ملخّص بلغة المستخدم: ماذا يُنشأ، من أين، وما الذي لن يحدث.
 *
 * بلا خطةٍ لا يُعرض شيء. كان هنا صندوقٌ يقول «أكمل الحقول أعلاه لتظهر المعاينة»
 * ثم صار سبب تعطّل «نفِّذ» يقول الجملة نفسها بصياغة أخرى تحت الزرّ مباشرة —
 * وجملتان متجاورتان تقولان أمرًا واحدًا تجعلان المستخدم يبحث عن الفرق بينهما.
 * السبب تحت الزرّ هو الموضع الصحيح لأنه ملتصق بالفعل المعطَّل.
 */
function Summary({ plan, phase }: { plan: PlanResponse | null; phase: Phase }) {
  if (phase === 'finished' || !plan) return null;

  return (
    <div className="summary" aria-live="polite">
      <h3 className="t-label summary__title">{t('summary.title')}</h3>

      <p className="summary__line">
        <span className="t-body-sec">{t('summary.will_create')}</span>
        <bdi className="path summary__path">{plan.produces}</bdi>
      </p>
      <p className="summary__line">
        <span className="t-body-sec">{t('summary.from')}</span>
        <bdi className="path summary__path">{plan.argv_display[plan.argv_display.length - 2]}</bdi>
      </p>

      {/* الأداة والتقدير: حقائق قبل التنفيذ، لا بعده. */}
      {/* `dt`/`dd` أبناءٌ مباشرون للشبكة لا مغلَّفون، وإلا صار كل صفّ شبكةً
          مستقلّة فاختلف عرض عموده الأول وتعرّجت القيم. */}
      <dl className="summary__meta">
        <dt className="t-body-sec">{t('summary.tool')}</dt>
        <dd>
          <code className="lat">{plan.tool.id}</code>{' '}
          <bdi className="path summary__path">{plan.tool.path}</bdi>
        </dd>

        {plan.estimate && (
          <>
            <dt className="t-body-sec">{t('summary.estimate')}</dt>
            <dd>
              {/* عزلٌ بلا فرض اتجاه: العبارة تخلط رقمًا لاتينيًا بوحدة عربية،
                  فإجبارها LTR كان يقلبها إلى «م.ب ١٢ ≈». «≈» ليست زينة — هي
                  وسم التقدير نفسه، ومكانها صدر العبارة. */}
              <bdi>≈ {readableSize(plan.estimate.approx_source_bytes)}</bdi>
              <span className="t-caption summary__estimate-note">
                {' '}
                {t(
                  plan.estimate.complete ? 'summary.estimate.note' : 'summary.estimate.partial',
                )}
              </span>
            </dd>

            {/* عدد ما مُسح: هو ما يجعل «تقدير ناقص» رقمًا لا شعورًا — المستخدم
                يرى كم عنصرًا دخل في الحساب قبل أن يقف المسح عند حدّه.

                المفتاح واحدٌ بلا سلسلة رجوع. كانت السلسلة
                `[…entries, summary.estimate]`، ورجوعُها ليس صياغةً أعمّ للمعنى
                نفسه — بل معنى آخر: الرقم ١٨٤٣ كان يُعرض تحت «حجم المصدر
                تقديريًا». `tFirst` تخدم نصوص الحقول حيث الخاصّ والعامّ يقولان
                الشيء ذاته؛ وهنا كان غيابُ الترجمة يصير كذبًا لا فجوة. ومفتاحٌ
                غير مترجَم يعود كما هو من `t` — غيابٌ مرئي يُصلَح، وهو أهون من
                رقمٍ يحمل اسم غيره. */}
            <dt className="t-body-sec">{t('summary.estimate.entries')}</dt>
            <dd>
              <bdi className="num">{plan.estimate.scanned_entries}</bdi>
            </dd>
          </>
        )}
      </dl>

      <ul className="summary__facts">
        <li>{t('summary.untouched')}</li>
        {/* سياسة التضارب تُقرأ من الخطة لا تُكتب هنا: نصٌّ ثابت كان سيبقى
            معروضًا لو تغيّرت السياسة يومًا، فيَعِد بما لا تفعله النواة. */}
        <li>{t(`summary.conflict.${plan.conflict}`)}</li>
        <li>{t('summary.atomic')}</li>
      </ul>

      {plan.warnings.length > 0 && (
        <ul className="summary__warnings">
          {plan.warnings.map((w) => (
            <li key={w} className="warning">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <use href="#i-warning" />
              </svg>
              <span>{t(w)}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/**
 * حالة التشغيل.
 *
 * لا شريط تقدّم ولا نسبة مئوية: `ditto` لا تعطي تقدّمًا موثوقًا، ورقمٌ مخترع
 * يزحف إلى ٩٩٪ ثم يقف أسوأ من غياب الرقم. ما يُعرض دوّار وحالة عمل، مع سبب
 * غيابه مكتوبًا.
 */
function RunState({
  phase,
  outcome,
  onReveal,
}: {
  phase: Phase;
  outcome: { status: string; produced?: string | null; key?: string } | null;
  onReveal: () => void;
}) {
  if (phase === 'running' || phase === 'cancelling') {
    return (
      <div className="runstate runstate--busy" role="status">
        <span className="spinner" aria-hidden="true" />
        <div>
          <p className="t-body">
            {phase === 'cancelling' ? t('state.cancelling') : t('state.running')}
          </p>
          <p className="t-caption">{t('state.running.note')}</p>
        </div>
      </div>
    );
  }

  if (phase !== 'finished' || !outcome) return null;

  if (outcome.status === 'success') {
    return (
      <div className="runstate runstate--ok" role="status">
        <span className="chip chip--success">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-success" />
          </svg>
          {t('state.succeeded')}
        </span>
        <bdi className="path runstate__path">{outcome.produced}</bdi>
        <button type="button" className="btn btn--quiet" onClick={onReveal}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-external" />
          </svg>
          {t('action.reveal')}
        </button>
      </div>
    );
  }

  if (outcome.status === 'cancelled') {
    return (
      <div className="runstate runstate--neutral" role="status">
        <span className="chip chip--neutral">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-cancelled" />
          </svg>
          {t('state.cancelled')}
        </span>
        <p className="t-body-sec">{t('state.cancelled.note')}</p>
      </div>
    );
  }

  return (
    <div className="runstate runstate--bad" role="alert">
      <span className="chip chip--danger">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <use href="#i-error" />
        </svg>
        {t('state.failed')}
      </span>
      <p className="t-body-sec">
        {outcome.key ? errorText(outcome.key) : t('state.failed.note')}
      </p>
      {outcome.key && <p className="t-caption">{t('state.failed.note')}</p>}
    </div>
  );
}

export { asCoreError };
