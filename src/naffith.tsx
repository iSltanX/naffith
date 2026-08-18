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
import { useEffect, useId, useRef, useState, type Ref } from "react";
import type {
  CoreErrorShape,
  InputSummary,
  OperationSummary,
  PlanResponse,
  RunFinishedEvent,
} from "./ipc";
import { asCoreError, pickDirectory, pickFile } from "./ipc";
import {
  choiceOptions,
  extensionHint,
  fieldKeys,
  inputKind,
  isComplete,
  isChoiceInput,
  isDirectoryInput,
  isFlagInput,
  isNumberInput,
  isPathInput,
  isUrlInput,
  numberSpec,
  type FormValues,
} from "./operations";
import { errorText, t, tFirst, tOptional } from "./i18n";
import Select from "./select";
import ResultView, {
  type ResultDiagnostic,
  type ResultExecutionDetails,
} from "./result-view";
import './operation-layout.css';
import './operation-screen.css';

type Phase = "idle" | "planning" | "running" | "cancelling" | "finished";

/**
 * `FormValues` تعريفها في `operations.ts` مع الدوالّ التي تقرأها وتكتبها.
 * تُصدَّر من هنا كذلك لأن الشاشة الجامعة تستوردها مع المكوّن نفسه.
 */
export type { FormValues } from "./operations";

interface Props {
  /** العملية كما أعلنتها النواة. منها يُرسم النموذج كلّه. */
  operation: OperationSummary;
  /** أيقونة قسم العملية كما أعلنتها النواة. انظر `iconFor`. */
  categoryIcon: string;
  values: FormValues;
  onChange: (next: FormValues) => void;
  plan: PlanResponse | null;
  /** الخطة التي نُفّذت، لعرض حقائق التنفيذ بعد استهلاك رمز الخطة. */
  executionPlan?: PlanResponse | null;
  error: CoreErrorShape | null;
  phase: Phase;
  /**
   * ناتج التشغيل كما وصل من النواة، بشكله الكامل لا بشكلٍ مختزل هنا.
   *
   * كان النوع مكتوبًا في هذا الملف: `{ status; produced?; key? }` — ثلاثة حقول
   * من ستّة. و`ipc.ts` يحذّر من هذا حرفيًا: «حقلٌ يعلنه ذاك ويغفله هذا لا سبيل
   * لقراءته». وهو ما وقع: `code` و`signal` تصلان الواجهة في كل فشلٍ ولا يمكن
   * قراءتهما، فكانت شاشة الفشل تقول «لم تكتمل العملية» وحدها وتُهمل الرقم الذي
   * يشرح السبب. والنوعُ الواحد يمنع أن يتكرّر ذلك مع حقلٍ يُضاف غدًا.
   */
  outcome: RunFinishedEvent | null;
  onBack: () => void;
  onExecute: () => void;
  onReveal: () => void;
  onReset: () => void;
  onLibrary: () => void;
}

/**
 * أيقونة الحقل، مشتقّة لا مكتوبة.
 *
 * المسار يأخذ أيقونته من نوعه (مجلد أو ملف)، وما ليس مسارًا يأخذ أيقونة فئة
 * العملية نفسها: الحقل الذي يسمّي الناتج يحمل صورة ما يُنتَج. خريطةٌ من معرّف
 * الحقل إلى أيقونة كانت ستكون قائمة الحقول المكتوبة يدويًا عائدةً من الباب
 * الخلفي، وتُنسى فيظهر حقلٌ بلا أيقونة في أول عملية جديدة.
 *
 * وأيقونة الفئة **تُمرَّر** ولا تُشتقّ هنا: القسم يعلن أيقونته في النواة
 * (`categories.rs`)، وخريطةٌ ثانية في الواجهة كانت ستتقادم يوم يُضاف قسم.
 */
function iconFor(categoryIcon: string, input: InputSummary): string {
  if (isDirectoryInput(input) || inputKind(input) === "existing_path") {
    return "#i-folder";
  }
  if (isPathInput(input)) return "#i-file";
  if (isUrlInput(input)) return "#i-network";
  return categoryIcon;
}

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

function PathPicker({
  disabled,
  onPickFile,
  onPickDirectory,
}: {
  disabled: boolean;
  onPickFile: (() => void) | null;
  onPickDirectory: (() => void) | null;
}) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null);
  const initialItem = useRef(0);
  const menuId = useId();
  const hasMenu = onPickFile !== null && onPickDirectory !== null;

  const close = (returnFocus: boolean) => {
    setOpen(false);
    if (returnFocus) trigger.current?.focus();
  };

  const openAt = (index: number) => {
    initialItem.current = index;
    setOpen(true);
  };

  useEffect(() => {
    if (!open) return;
    const items = menu.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]');
    items?.[initialItem.current]?.focus();
  }, [open]);

  useEffect(() => {
    if (!open) return;
    function closeOutside(event: MouseEvent) {
      if (event.target instanceof Node && root.current?.contains(event.target)) {
        return;
      }
      setOpen(false);
    }
    document.addEventListener("mousedown", closeOutside);
    return () => {
      document.removeEventListener("mousedown", closeOutside);
    };
  }, [open]);

  const choose = () => {
    if (!hasMenu) (onPickDirectory ?? onPickFile)?.();
    else if (open) close(false);
    else openAt(0);
  };

  function navigateMenu(event: React.KeyboardEvent<HTMLDivElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      close(true);
      return;
    }
    if (event.key === "Tab") {
      setOpen(false);
      return;
    }
    if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;
    const items = [...(menu.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? [])];
    if (items.length === 0) return;
    event.preventDefault();
    const current = Math.max(0, items.indexOf(document.activeElement as HTMLButtonElement));
    const next =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? items.length - 1
          : (current + (event.key === "ArrowDown" ? 1 : -1) + items.length) % items.length;
    items[next]?.focus();
  }

  return (
    <div className="path-picker" ref={root}>
      <button
        ref={trigger}
        type="button"
        className="btn btn--quiet btn--sm path-picker__trigger"
        onClick={choose}
        disabled={disabled}
        aria-haspopup={hasMenu ? "menu" : undefined}
        aria-expanded={hasMenu ? open : undefined}
        aria-controls={hasMenu ? menuId : undefined}
        onKeyDown={(event) => {
          if (!hasMenu || (event.key !== "ArrowDown" && event.key !== "ArrowUp")) return;
          event.preventDefault();
          openAt(event.key === "ArrowUp" ? 1 : 0);
        }}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <use href="#i-folder-open" />
        </svg>
        {t("field.choose")}
      </button>

      {hasMenu && open && (
        <div
          id={menuId}
          ref={menu}
          className="path-picker__menu"
          role="menu"
          onKeyDown={navigateMenu}
        >
          <button
            type="button"
            role="menuitem"
            tabIndex={-1}
            onClick={() => {
              close(true);
              onPickFile?.();
            }}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-file" />
            </svg>
            {t("field.choose.file")}
          </button>
          <button
            type="button"
            role="menuitem"
            tabIndex={-1}
            onClick={() => {
              close(true);
              onPickDirectory?.();
            }}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-folder" />
            </svg>
            {t("field.choose.folder")}
          </button>
        </div>
      )}
    </div>
  );
}

/**
 * حقل مسار: يُملأ بالحوار أو بالكتابة. الاثنان يمرّان بنفس التحقّق.
 *
 * فعل الاختيار يتبع النوع المعلن: ملف، أو مجلد، أو قائمة صغيرة بالاثنين
 * لـ`existing_path`. وقد يكون أحدهما `null` إن لم يسمح به نوع الحقل؛ الكتابة
 * واللصق يبقيان طريقًا كاملًا في كل الحالات.
 */
function PathField({
  id,
  icon,
  label,
  help,
  placeholder,
  value,
  error,
  pickerError,
  disabled,
  onPickFile,
  onPickDirectory,
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
  pickerError: boolean;
  disabled: boolean;
  onPickFile: (() => void) | null;
  onPickDirectory: (() => void) | null;
  onType: (v: string) => void;
  inputRef: Ref<HTMLInputElement> | null;
}) {
  const errorId = `${id}-error`;
  const helpId = `${id}-help`;
  const displayedError = error ?? (pickerError ? t("err.picker.failed") : null);
  return (
    <div className="row-field">
      <label className="t-label" htmlFor={id}>
        {label}
      </label>
      {/* النصّ المساعد قبل الضابط لا بعده: هو ما يُقرأ لتُعرف قيمةُ الحقل، وقد
          كان يقع تحته — فيصل بعد أن كتب المستخدم. وموضعُه صفُّ الوسم نفسه:
          جملةٌ قصيرة بجانب وسمٍ قصير، فيوفّر الصفُّ المشترك سطرًا في كل حقل. */}
      <p id={helpId} className="t-helper">
        {help}
      </p>
      <div className="row-field__control">
        {/* المسار LTR داخل نموذج عربي: قراءته من اليمين تفسد ترتيب مقاطعه.
            وزرّ الاختيار داخل الضابط لا شقيقًا له: كان شقيقًا، فكان عرض المربّع
            يساوي عرض الصفّ ناقصًا عرض زرٍّ يتبع طول وسمه — فيختلف عن عرض مربّع
            الحقل الذي لا زرّ له، وتتعرّج حوافّ النموذج اليسرى من حقلٍ إلى حقل.
            و`field--with-action` نمطُ نظام التصميم نفسه لهذه الحالة. */}
        <span
          className={`field field--path${
            onPickFile || onPickDirectory ? " field--with-action" : ""
          }${displayedError ? " field--invalid" : ""}${disabled ? " field--disabled" : ""}`}
        >
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
            aria-describedby={displayedError ? `${errorId} ${helpId}` : helpId}
            aria-invalid={displayedError ? true : undefined}
            onChange={(e) => onType(e.target.value)}
          />
          {(onPickFile || onPickDirectory) && (
            <PathPicker
              disabled={disabled}
              onPickFile={onPickFile}
              onPickDirectory={onPickDirectory}
            />
          )}
        </span>
      </div>
      {displayedError && <FieldError id={errorId} text={displayedError} />}
    </div>
  );
}

/**
 * اختيارٌ مغلق من مواصفة النواة. لا تعرض الشاشة أول خيارٍ وكأنه مختار، ولا
 * تحتفظ بقائمةٍ ثانية للخيارات: القيمة والوسم كلاهما يأتيان من `InputKind`.
 */
function ChoiceField({
  id,
  label,
  help,
  placeholder,
  value,
  options,
  error,
  disabled,
  required,
  onChoose,
}: {
  id: string;
  label: string;
  help: string;
  placeholder: string;
  value: string;
  options: Array<{ value: string; label: string }>;
  error: string | null;
  disabled: boolean;
  required: boolean;
  onChoose: (value: string) => void;
}) {
  const errorId = `${id}-error`;
  const helpId = `${id}-help`;
  return (
    <div className="row-field row-field--choice">
      <div className="row-field__control operation-choice">
        <Select
          label={label}
          value={value}
          options={options}
          placeholder={placeholder}
          disabled={disabled}
          describedBy={error ? `${errorId} ${helpId}` : helpId}
          invalid={error !== null}
          required={required}
          onChange={onChoose}
        />
      </div>
      <p id={helpId} className="t-helper operation-choice__help">
        {help}
      </p>
      {error && <FieldError id={errorId} text={error} />}
    </div>
  );
}

/**
 * رقمٌ قابل للكتابة مع خطوتين صريحتين. الأزرار لا تحمل منطق نطاقٍ مستقلًا:
 * نفس `min` و`max` اللذين أعلنتْهما النواة يضبطان الإدخال ويحدّان الزيادة.
 */
function NumberField({
  id,
  label,
  help,
  value,
  min,
  max,
  defaultValue,
  error,
  disabled,
  onType,
  inputRef,
}: {
  id: string;
  label: string;
  help: string;
  value: string;
  min: number;
  max: number;
  defaultValue: number;
  error: string | null;
  disabled: boolean;
  onType: (value: string) => void;
  inputRef: Ref<HTMLInputElement> | null;
}) {
  const errorId = `${id}-error`;
  const helpId = `${id}-help`;
  const parsed = Number(value);
  const validNumber = value.trim() !== "" && Number.isFinite(parsed);

  function step(delta: number) {
    const base = validNumber ? parsed : min;
    onType(String(Math.min(max, Math.max(min, base + delta))));
  }

  return (
    <div className="row-field row-field--number">
      <label className="t-label" htmlFor={id}>
        {label}
      </label>
      <div id={helpId} className="t-helper number-field__description">
        <span>{help}</span>
        <span className="number-field__meta">
          {t("field.number.range.label")}{" "}
          <bdi className="num">{min}</bdi>
          {"–"}
          <bdi className="num">{max}</bdi>
          <span aria-hidden="true"> · </span>
          {t("field.number.default.label")}{" "}
          <bdi className="num">{defaultValue}</bdi>
        </span>
      </div>
      <div className="row-field__control">
        <span
          className={`number-field${error ? " number-field--invalid" : ""}${
            disabled ? " number-field--disabled" : ""
          }`}
        >
          <button
            type="button"
            className="number-field__step"
            onClick={() => step(-1)}
            disabled={disabled || (validNumber && parsed <= min)}
            aria-label={`− ${label}`}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-minus" />
            </svg>
          </button>
          <input
            id={id}
            ref={inputRef}
            className="number-field__input"
            type="number"
            dir="ltr"
            inputMode="numeric"
            min={min}
            max={max}
            step={1}
            value={value}
            disabled={disabled}
            aria-describedby={error ? `${errorId} ${helpId}` : helpId}
            aria-invalid={error ? true : undefined}
            onChange={(event) => onType(event.target.value)}
          />
          <button
            type="button"
            className="number-field__step"
            onClick={() => step(1)}
            disabled={disabled || (validNumber && parsed >= max)}
            aria-label={`+ ${label}`}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-plus" />
            </svg>
          </button>
        </span>
      </div>
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
  technical = false,
  url = false,
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
  technical?: boolean;
  url?: boolean;
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
      <p id={helpId} className="t-helper">
        {help}
      </p>
      <div className="row-field__control">
        {/* عناوين URL وأسماء الملفات محتوى تقني LTR وفق Page 14؛ النص الحر
            وحده يرث اتجاه الصفحة. ويبقى نوع URL مستقلًا عن اتجاه الاسم حتى لا
            تفرض دلالات التحقّق/لوحة المفاتيح الخاصة بعنوان على اسم ملف. */}
        <span
          className={`field${error ? " field--invalid" : ""}${
            disabled ? " field--disabled" : ""
          }`}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true" className="field__icon">
            <use href={icon} />
          </svg>
          <input
            id={id}
            ref={inputRef}
            className={technical ? "field__technical-value" : undefined}
            type={url ? "url" : "text"}
            dir={technical ? "ltr" : undefined}
            inputMode={url ? "url" : undefined}
            spellCheck={false}
            autoComplete="off"
            value={value}
            placeholder={placeholder}
            disabled={disabled}
            aria-describedby={error ? `${errorId} ${helpId}` : helpId}
            aria-invalid={error ? true : undefined}
            onChange={(e) => onType(e.target.value)}
          />
          {/* اللاحقة عرضٌ لما تضيفه النواة، لا جزءٌ من قيمة الحقل. وموضعها داخل
              الحقل لا بعده: هي طرفُ الاسم الذي يُكتب، وسطحُ الحقل هو ما يقول
              ذلك. كانت شقيقةً للحقل تطفو خارجه فتُقرأ عنصرًا ثالثًا في الصفّ. */}
          {suffix && (
            <span className="suffix-hint lat" aria-hidden="true">
              {suffix}
            </span>
          )}
        </span>
      </div>
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
    <div className={`row-field row-field--flag${disabled ? " row-field--disabled" : ""}`}>
      <label className="t-label row-field__flag" htmlFor={id}>
        <input
          id={id}
          className="flag-toggle"
          type="checkbox"
          aria-label={label}
          checked={checked}
          disabled={disabled}
          aria-describedby={error ? `${errorId} ${helpId}` : helpId}
          aria-invalid={error ? true : undefined}
          onChange={(e) => onToggle(e.target.checked)}
        />
        <span className="row-field__flag-copy">
          <span className="t-label">{label}</span>
          <span id={helpId} className="t-helper">{help}</span>
        </span>
      </label>
      {error && <FieldError id={errorId} text={error} />}
    </div>
  );
}

/**
 * شريط الشاشة — زرّ الرجوع، ثم هويّة العملية، في صفٍّ واحد مدمج.
 *
 * ## ما كان، ولماذا سقط
 *
 * كان الشريط يعطي العنوان `--fs-page-title` (‏28px بوزن ‎800) ويضع تحته الوصف
 * فقرةً، ويدفع شارة الخطورة إلى أقصى الشريط بـ`margin-inline-start: auto`.
 * الحصيلة رأسٌ يأكل ‎170px‎ من ارتفاع النافذة قبل أن يبدأ النموذج، وشارةٌ تطفو
 * وحدها في فراغٍ بعيدٍ عن الاسم الذي تصفه، وزرُّ رجوعٍ مؤطَّر يحاذي رأس كتلةٍ
 * أطول منه بثلاثة أضعاف فيبدو ملصقًا فوق الواجهة لا جزءًا منها.
 *
 * ## القاعدة الآن
 *
 * كل عنصر يبرّر حجمه وموضعه:
 *
 * - **الرجوع** زرٌّ شفّاف بشيفرون واحد ووسمٍ من كلمة، بمقاس عناصر التحكّم
 *   الصغيرة، محاذٍ لمركز الشريط رأسيًا — كزرّ شريط أدوات في macOS. ووسمُه
 *   المعروض كلمة، واسمُه المقروء الجملة كاملة (`nav.back`)، فهو يبقى مفهومًا
 *   بالصوت كما هو مفهوم بالعين.
 * - **فاصلٌ شعرة** بعده: يفصل التنقّل عن الهويّة بلا أن يرسم صندوقًا حول أيّهما.
 * - **الأيقونة والاسم والوصف** كتلةٌ واحدة: الأيقونة تحاذي سطر الاسم، والشارة
 *   تجاور الاسم في صفّه لأنها وسمٌ عليه — لا تُنبذ إلى الطرف المقابل.
 */
export function OperationBar({
  operation,
  onBack,
  onLibrary,
  titleRef,
}: {
  operation: OperationSummary;
  categoryIcon?: string;
  onBack: () => void;
  onLibrary?: () => void;
  titleRef?: Ref<HTMLHeadingElement>;
}) {
  // «ماذا سيحدث عندما أُنفّذها؟» — جوابُ صفحة التنفيذ، مقابلَ «ماذا تفعل؟»
  // الذي تجيبه البطاقة في المكتبة. اختياريٌّ لأن العملية الداخلية بلا نصّ.
  const execution = tOptional(`op.${operation.id}.execution`);
  return (
    <header className="opintro">
      <nav className="opintro__crumbs" aria-label={t("nav.breadcrumbs")}>
        <button type="button" onClick={onLibrary ?? onBack}>
          {t("nav.operations")}
        </button>
        <span aria-hidden="true">‹</span>
        <button
          type="button"
          onClick={onBack}
          aria-label={t(`cat.${operation.category}.title`)}
        >
          <span className="opintro__category-label">
            {t(`cat.${operation.category}.title`)}
          </span>
          <span className="opintro__category-ellipsis" aria-hidden="true">…</span>
        </button>
        <span aria-hidden="true">‹</span>
        <strong>{t(operation.title_key)}</strong>
      </nav>
      <div className="opintro__heading">
        <h2
          id="naffith-heading"
          className="t-section-title opintro__title"
          tabIndex={-1}
          ref={titleRef}
        >
          {t(operation.title_key)}
        </h2>
        <span className={`opintro__danger opintro__danger--${operation.danger}`}>
          {t(`summary.danger.${operation.danger}`)}
        </span>
      </div>
      {execution && <p className="t-body-sec opintro__execution">{execution}</p>}
    </header>
  );
}

export default function Naffith(props: Props) {
  const {
    operation,
    categoryIcon,
    values,
    onChange,
    plan,
    executionPlan = null,
    error,
    phase,
    outcome,
    onBack,
    onExecute,
    onReveal,
    onReset,
    onLibrary,
  } = props;

  const uid = useId();
  const whyId = `${uid}-why`;
  const form = useRef<HTMLDivElement>(null);
  const title = useRef<HTMLHeadingElement>(null);
  const previousOperation = useRef<string | null>(null);
  const previousPhase = useRef<Phase>(phase);
  const [pickerError, setPickerError] = useState<string | null>(null);

  const busy = phase === "running" || phase === "cancelling";
  const errorFor = (field: string): string | null =>
    error && error.input === field ? errorText(error.key, error.detail) : null;
  // خطأ لا ينسبه النواةُ إلى حقل يُعرض فوق الأزرار لا تحت حقل عشوائي.
  const generalError =
    error && !error.input ? errorText(error.key, error.detail) : null;
  const executionDetails: ResultExecutionDetails | undefined = outcome
    ? {
        runId: outcome.run_id,
        status: outcome.status,
        exitCode: outcome.code ?? null,
        signal: outcome.signal ?? null,
        ...(executionPlan
          ? {
              executable: executionPlan.tool.path,
              warnings: executionPlan.warnings,
            }
          : {}),
      }
    : undefined;
  useEffect(() => {
    const returningFromResult = previousPhase.current === "finished" && phase === "idle";
    const operationChanged = previousOperation.current !== operation.id;
    previousPhase.current = phase;
    previousOperation.current = operation.id;
    // الانتقال إلى عملية جديدة يعلن عنوان الشاشة أولًا. «تشغيل مجددًا» وحده
    // يعيد المؤشّر إلى أول حقل لأن الشاشة لم تتبدّل، وإنما عاد المستخدم إلى
    // مهمة الإدخال صراحةً من النتيجة.
    if (operationChanged) {
      title.current?.focus();
    } else if (phase === "idle" && returningFromResult) {
      const firstControl = form.current?.querySelector<HTMLElement>(
        'input:not(:disabled), button[role="combobox"]:not(:disabled), button:not(:disabled)',
      );
      firstControl?.focus();
    }
  }, [phase, operation.id]);

  async function pick(input: InputSummary, choice: "file" | "directory") {
    setPickerError(null);
    try {
      const chosen =
        choice === "directory" ? await pickDirectory() : await pickFile();
      if (chosen) onChange({ ...values, [input.id]: chosen });
    } catch {
      // فشل الحوار لا يُعطّل الحقل: الكتابة واللصق يبقيان طريقًا كاملًا، لكن
      // الصمت هنا كان يجعل الزر يبدو كأنه لم يعمل بلا أي تفسير.
      setPickerError(input.id);
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
    ? "action.execute.why.incomplete"
    : phase === "planning"
      ? "action.execute.why.planning"
      : error
        ? "action.execute.why.invalid"
        : plan
          ? null
          : "action.execute.why.planning";

  // النصّ يُعرض ما دام الأمر بيد المستخدم. أثناء التشغيل تتكلّم `RunState`،
  // فإبقاؤه هناك كان سيقول «يجري التحقّق» بينما الأداة تعمل فعلًا.
  const showWhy = blocked !== null && !busy && phase !== "finished";

  return (
    <section
      className="naffith"
      aria-labelledby={phase === "finished" && outcome ? "result-view-heading" : "naffith-heading"}
    >
      <div
        className={`naffith__body${phase === "finished" && outcome ? " naffith__body--result" : ""}`}
      >
        <div className="naffith__col">
          {phase === "finished" && outcome ? (
            <div className="result-page">
              <ResultView
                result={outcome.result}
                operationId={operation.id}
                diagnostic={diagnosticFor(outcome)}
                execution={executionDetails}
                onReveal={onReveal}
                onRunAgain={onReset}
                onLibrary={onLibrary}
              />
              {generalError && (
                <div className="notice notice--warning" role="status" aria-live="polite">
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <use href="#i-warning" />
                  </svg>
                  <p>{generalError}</p>
                </div>
              )}
            </div>
          ) : (
            <>
          <div className="naffith__scroll">
          <OperationBar
            operation={operation}
            categoryIcon={categoryIcon}
            onBack={onBack}
            onLibrary={onLibrary}
            titleRef={title}
          />

          <div className="naffith__form" ref={form}>
            {operation.inputs.map((input) => {
              const id = `${uid}-${input.id}`;
              const keys = fieldKeys(operation.id, input.id);
              const label = tFirst(keys.label);
              const help = tFirst(keys.help);
              const value = values[input.id] ?? "";
              const fieldError = errorFor(input.id);
              if (isFlagInput(input)) {
                return (
                  <FlagField
                    key={input.id}
                    id={id}
                    label={label}
                    help={help}
                    checked={value === "1"}
                    error={fieldError}
                    disabled={busy}
                    onToggle={(on) =>
                      onChange({ ...values, [input.id]: on ? "1" : "" })
                    }
                  />
                );
              }

              if (isChoiceInput(input)) {
                return (
                  <ChoiceField
                    key={input.id}
                    id={id}
                    label={label}
                    help={help}
                    placeholder={tFirst([
                      ...keys.placeholder,
                      "field.choice.placeholder",
                    ])}
                    value={value}
                    options={choiceOptions(input).map((option) => ({
                      value: option.value,
                      label: t(option.label_key),
                    }))}
                    error={fieldError}
                    disabled={busy}
                    required={input.required}
                    onChoose={(next) =>
                      onChange({ ...values, [input.id]: next })
                    }
                  />
                );
              }

              if (isNumberInput(input)) {
                const spec = numberSpec(input);
                if (spec) {
                  return (
                    <NumberField
                      key={input.id}
                      id={id}
                      label={label}
                      help={
                        help === keys.help[keys.help.length - 1]
                          ? t("field.number.range")
                          : help
                      }
                      value={value}
                      min={spec.min}
                      max={spec.max}
                      defaultValue={spec.default}
                      error={fieldError}
                      disabled={busy}
                      onType={(next) =>
                        onChange({ ...values, [input.id]: next })
                      }
                      inputRef={null}
                    />
                  );
                }
              }

              if (isPathInput(input)) {
                return (
                  <PathField
                    key={input.id}
                    id={id}
                    icon={iconFor(categoryIcon, input)}
                    label={label}
                    help={help}
                    placeholder={tFirst(keys.placeholder)}
                    value={value}
                    error={fieldError}
                    pickerError={pickerError === input.id}
                    disabled={busy}
                    onPickFile={
                      input.kind === "existing_file" ||
                      input.kind === "existing_path"
                        ? () => pick(input, "file")
                        : null
                    }
                    onPickDirectory={
                      isDirectoryInput(input) || input.kind === "existing_path"
                        ? () => pick(input, "directory")
                        : null
                    }
                    onType={(v) => {
                      if (pickerError === input.id) setPickerError(null);
                      onChange({ ...values, [input.id]: v });
                    }}
                    inputRef={null}
                  />
                );
              }

              return (
                <TextField
                  key={input.id}
                  id={id}
                  icon={iconFor(categoryIcon, input)}
                  label={label}
                  help={help}
                  placeholder={tFirst(keys.placeholder)}
                  suffix={extensionHint(input)}
                  value={value}
                  error={fieldError}
                  disabled={busy}
                  technical={
                    isUrlInput(input) ||
                    inputKind(input) === "new_name" ||
                    inputKind(input) === "new_dir_name"
                  }
                  url={isUrlInput(input)}
                  onType={(v) => onChange({ ...values, [input.id]: v })}
                  inputRef={null}
                />
              );
            })}
          </div>

          <Summary operation={operation} plan={plan} phase={phase} />

          {generalError && (
            <p className="field-error field-error--block" role="alert">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <use href="#i-error" />
              </svg>
              {generalError}
            </p>
          )}
          {showWhy && blocked && (
            <p id={whyId} className="t-helper naffith__why" aria-live="polite">
              {t(blocked)}
            </p>
          )}
          </div>

          {/* Page 16 يفصل مجرى النموذج القابل للتمرير عن شريط الفعل الثابت.
              بذلك يبقى التنفيذ/الإلغاء ظاهرًا في نافذة 520px من غير أن يحجب
              الحقول: الحقول والملخّص وحدهما يمرّان، والفعل شقيقٌ للمجرى. */}
          <div className="naffith__act">
            {/* الحالة قبل الفعل التالي: بعد تشغيلٍ انتهى يُقرأ الناتج أوّلًا ثم
                يُقرَّر ما بعده. كانت بعده، فكان زرّ «ضغط مجلد آخر» يعلو خبرَ
                النجاح — سؤالٌ عن الخطوة التالية قبل الجواب عن السابقة. */}
            <RunningState phase={phase} />

            <div className="naffith__actions">
              {phase !== "finished" && (
                <>
                  {/* الزرّ ظاهر دائمًا ومعطّل بسببٍ مكتوب تحته. زرٌّ يظهر ويختفي مع
                    صلاحية الخطة يجعل الفعل الأساسي يرقص مع كل ضغطة مفتاح، ويحرم
                    من لم تكتمل حقوله من معرفة أن ثمّة زرًّا أصلًا. */}
                  <button
                    type="button"
                    className="btn btn--primary btn--lg naffith__go"
                    onClick={onExecute}
                    disabled={blocked !== null || busy}
                    aria-describedby={showWhy ? whyId : undefined}
                  >
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                      <use href="#i-execute" />
                    </svg>
                    {t(
                      blocked === "action.execute.why.incomplete"
                        ? "action.execute.incomplete"
                        : phase === "planning"
                          ? "state.checking"
                          : "action.execute",
                    )}
                  </button>
                </>
              )}
            </div>
          </div>
            </>
          )}
        </div>
      </div>
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
    [1e9, "unit.gb"],
    [1e6, "unit.mb"],
    [1e3, "unit.kb"],
  ];
  for (const [scale, key] of units) {
    if (bytes >= scale) {
      const n = bytes / scale;
      // منزلة واحدة تحت ١٠، وعدد صحيح فوقها: «١٢٫٤ م.ب» مفيد، «١٢٤٫٧ م.ب» ضجيج.
      return `${n < 10 ? n.toFixed(1) : Math.round(n)} ${t(key)}`;
    }
  }
  return `${bytes} ${t("unit.bytes")}`;
}

/**
 * ملخّص بلغة المستخدم، مبنيّ على حقائق الخطة المهيكلة وحدها.
 *
 * بلا خطةٍ لا يُعرض شيء. كان هنا صندوقٌ يقول «أكمل الحقول أعلاه لتظهر المعاينة»
 * ثم صار سبب تعطّل «نفِّذ» يقول الجملة نفسها بصياغة أخرى تحت الزرّ مباشرة —
 * وجملتان متجاورتان تقولان أمرًا واحدًا تجعلان المستخدم يبحث عن الفرق بينهما.
 * السبب تحت الزرّ هو الموضع الصحيح لأنه ملتصق بالفعل المعطَّل.
 *
 * لا نقرأ `argv_display` هنا مطلقًا. ترتيب الوسائط يخصّ كل أداة على حدة، وقد
 * كان افتراض «قبل الأخير هو المصدر» يعرض رايةً أو سرًّا أو مسارًا آخر تحت اسم
 * المصدر في عمليات كثيرة. سَطْر يملك عرض الأمر؛ هذه البطاقة لا تعرض إلا الحقول
 * المسماة في `PlanResponse`.
 */
function Summary({
  operation,
  plan,
  phase,
}: {
  operation: OperationSummary;
  plan: PlanResponse | null;
  phase: Phase;
}) {
  if (phase === "finished") return null;
  if (operation.availability.state === "tool_missing") {
    return (
      <div className="summary summary--unavailable" aria-live="polite">
        <div className="summary__head">
          <h3 className="t-card-title summary__title">
            {t("summary.plan.unavailable.title")}
          </h3>
          <span className="summary__chip">
            {t("summary.plan.unavailable.chip")}
          </span>
        </div>
        <p className="t-body-sec summary__body">
          {t("summary.plan.unavailable.body")}{" "}
          <bdi dir="ltr" className="technical-value">
            {operation.availability.tool}
          </bdi>
        </p>
      </div>
    );
  }
  if (!plan) return null;
  const createsDirectory = operation.inputs.some(
    (input) => inputKind(input) === "new_dir_name",
  );
  const kind =
    plan.produces && plan.danger === "creates"
      ? createsDirectory
        ? "creates-directory"
        : "creates-file"
      : plan.danger === "safe"
        ? "safe"
        : "modifies";
  const warning = plan.warnings.length > 0;
  const titleKey = `summary.plan.${kind}.title`;
  const chipKey = `summary.plan.${kind}.chip`;
  const bodyKey = `summary.plan.${kind}.body`;

  return (
    <div
      className={`summary summary--${kind}${warning ? " summary--warning" : ""}`}
      aria-live="polite"
    >
      <div className="summary__head">
        <h3 className="t-card-title summary__title">{t(titleKey)}</h3>
        <span className="summary__chip">{t(chipKey)}</span>
      </div>
      <p className="t-body-sec summary__body">{t(bodyKey)}</p>

      {plan.produces && (
        <bdi dir="ltr" className="path summary__path">{plan.produces}</bdi>
      )}

      {plan.estimate && (
        <div
          className={`summary__estimate${plan.estimate.complete ? "" : " summary__estimate--partial"}`}
        >
          <span>{t("summary.estimate")}</span>
          <bdi dir="ltr">{readableSize(plan.estimate.approx_source_bytes)}</bdi>
          <span>{t("summary.estimate.entries")}</span>
          <bdi dir="ltr">{plan.estimate.scanned_entries}</bdi>
          <p>
            {t(
              plan.estimate.complete
                ? "summary.estimate.note"
                : "summary.estimate.partial",
            )}
          </p>
        </div>
      )}

      {warning && (
        <div className="summary__warning" role="status">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-warning" />
          </svg>
          <span>{plan.warnings.map((key) => t(key)).join(" · ")}</span>
        </div>
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
function RunningState({ phase }: { phase: Phase }) {
  if (phase === "planning" || phase === "running" || phase === "cancelling") {
    return (
      <div
        className={`runstate runstate--busy${phase === "planning" ? " runstate--planning" : ""}`}
        role="status"
      >
        <span className="spinner" aria-hidden="true" />
        <div>
          <p className="t-body">
            {phase === "planning"
              ? t("state.checking")
              : phase === "cancelling"
              ? t("state.cancelling")
              : t("state.running")}
          </p>
          <p className="t-caption">
            {t(phase === "planning" ? "state.checking.note" : "state.running.note")}
          </p>
        </div>
      </div>
    );
  }

  return null;
}

/** Outcome detail remains visible inside the typed diagnostic presentation. */
function diagnosticFor(outcome: RunFinishedEvent): ResultDiagnostic | undefined {
  if (outcome.status === "signalled" && outcome.signal != null) {
    return { label: t("state.failed.signal"), value: outcome.signal };
  }
  if (outcome.code != null) {
    return { label: t("state.failed.code"), value: outcome.code };
  }
  if (outcome.key) return { label: errorText(outcome.key) };
  return undefined;
}

export { asCoreError };
