/**
 * قائمة منسدلة مشتركة — زرٌّ وقائمةٌ من مادّةٍ واحدة.
 *
 * ## لماذا لم تبقَ `select` أصلية
 *
 * كانت مرشّحات السجلّ `select` أصلية، وحجّتُها كانت صحيحة: لوحة المفاتيح وقارئ
 * الشاشة والبحث بالحرف الأوّل تأتي من المتصفّح بلا سطرٍ واحد. وسقطت الحجّة على
 * ما لا يُرى في jsdom: **القائمة المفتوحة في WKWebView قائمةُ النظام**، لا يبلغها
 * سطرُ CSS واحد — لا سطحُها ولا حدُّها ولا ارتفاع صفّها ولا محاذاتها في RTL.
 * فكان في الشاشة نظامان بصريّان: واحدٌ نملكه وهي مغلقة، وآخر لا نملكه وهي
 * مفتوحة. وزادت `select` الخام أنها كانت بلا قاعدة أنماطٍ أصلًا (`class="input"`
 * ولا `.input` في المشروع)، فظهرت بأثاث الوكيل الخام تحته حدُّنا وفوقه حلقتا
 * تركيز.
 *
 * ## والثمن مدفوعٌ صراحةً لا مسكوتٌ عنه
 *
 * كل ما كان يأتي مجّانًا مكتوبٌ هنا: الأسهم، وHome/End، وEnter/Space، وEscape،
 * والبحث بالحرف الأوّل، وإغلاقُ النقر خارجها، وعودةُ البؤرة إلى الزرّ، ودلالةُ
 * `listbox`/`option` مع `aria-activedescendant`. وكلّه مُختبَر في
 * `select.test.tsx` — فما يستطيعه المستخدم بلوحة المفاتيح لم ينقص.
 *
 * ## البؤرة تبقى على الزرّ
 *
 * القائمة المفتوحة لا تأخذ البؤرة، ولا يأخذها صفٌّ منها. الزرّ يحملها ويعلن
 * الصفَّ النشِط بـ`aria-activedescendant` — وهو نمط `listbox` القياسي. البديل
 * (نقلُ البؤرة إلى كل صفّ) يجعل كل ضغطة سهمٍ حدثَ بؤرةٍ كاملًا، ويخرج Escape
 * من مكانٍ غير الذي دخل منه.
 */
import { useCallback, useEffect, useId, useRef, useState } from 'react';
import { fold } from './library';

export interface SelectOption {
  value: string;
  label: string;
}

interface Props {
  /** وسمٌ ظاهر فوق الضابط. يربطه `aria-labelledby` بالزرّ وبالقائمة معًا. */
  label: string;
  value: string;
  options: SelectOption[];
  onChange: (value: string) => void;
  /** نصٌّ حقيقي للحالة غير المختارة؛ لا نعرض أول خيار وكأنه اختير. */
  placeholder?: string;
  disabled?: boolean;
  describedBy?: string;
  invalid?: boolean;
  required?: boolean;
}

export default function Select({
  label,
  value,
  options,
  onChange,
  placeholder = '',
  disabled = false,
  describedBy,
  invalid = false,
  required = false,
}: Props): JSX.Element {
  const uid = useId();
  const labelId = `${uid}-label`;
  const listId = `${uid}-list`;
  const optionId = (i: number) => `${uid}-opt-${i}`;

  const [open, setOpen] = useState(false);
  /** الصفّ النشِط بلوحة المفاتيح — مؤشّرٌ لا قيمة: قائمةٌ فيها قيمتان متساويتان
      لا تحدث هنا، لكن المؤشّر يبقى صحيحًا لو حدثت. */
  const [active, setActive] = useState(0);

  const root = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLUListElement>(null);
  /** حروفُ البحث المتراكمة وآخر وقتٍ كُتب فيه حرف. انظر `onType`. */
  const typed = useRef<{ text: string; at: number }>({ text: '', at: 0 });

  const selected = options.findIndex((o) => o.value === value);
  const current = selected >= 0 ? options[selected] : undefined;

  const close = useCallback((focusButton: boolean) => {
    setOpen(false);
    if (focusButton) button.current?.focus();
  }, []);

  /** يفتح ويضع النشِط على المختار — لا على أوّل القائمة: من يفتح ليغيّر يبدأ
      من حيث هو، لا من حيث بدأت القائمة. */
  const openAt = useCallback(() => {
    // قائمةٌ بلا خيارات لا تُفتح: صندوقٌ فارغ يطفو فوق الشاشة، و`aria-activedescendant`
    // يشير إلى معرّفٍ لا وجود له فيصمت قارئ الشاشة.
    if (!options.length || disabled) return;
    setActive(selected >= 0 ? selected : 0);
    setOpen(true);
  }, [selected, options.length, disabled]);

  const choose = useCallback(
    (index: number) => {
      const option = options[index];
      if (!option) return;
      onChange(option.value);
      close(true);
    },
    [options, onChange, close],
  );

  // فتحُ القائمة أو إغلاقها يمسح حروف البحث المتراكمة، كما تفعل القائمة
  // الأصلية. وبلا هذا يلتصق آخرُ حرفٍ كُتب قبل الفتح بأوّل حرفٍ بعده، فيصير
  // «ف» ثم «ن» بحثًا عن «فن» — ولا شيء يبدأ بها، فتبدو لوحة المفاتيح معطّلة.
  useEffect(() => {
    typed.current = { text: '', at: 0 };
  }, [open]);

  // النقر خارج الضابط يغلق بلا اختيار، والبؤرة تبقى حيث وقع النقر: من ينقر
  // على شيءٍ آخر يقصده هو، فسحبُ البؤرة إلى الزرّ يخالفه.
  useEffect(() => {
    if (!open) return;
    function onPointerDown(event: MouseEvent) {
      const target = event.target;
      if (target instanceof Node && root.current?.contains(target)) return;
      setOpen(false);
    }
    document.addEventListener('mousedown', onPointerDown);
    return () => document.removeEventListener('mousedown', onPointerDown);
  }, [open]);

  // وخروجُ البؤرة يغلق كذلك.
  //
  // ‏Tab لا يُعترض هنا: من ضغطها يريد الضابط التالي، ومنعُه حبسٌ للبؤرة في
  // قائمةٍ لا حوار. لكن تركها بلا شيء كان يبقي القائمة مفتوحةً معلّقةً فوق
  // الشاشة والبؤرةُ في مكانٍ آخر — قائمةٌ لا يملكها أحد، لا تُغلق إلا بنقرة.
  // فالإغلاق يقع على خروج البؤرة من الضابط كلّه، أيًّا كان سبب الخروج.
  useEffect(() => {
    if (!open) return;
    function onFocusOut(event: FocusEvent) {
      const next = event.relatedTarget;
      if (next instanceof Node && root.current?.contains(next)) return;
      setOpen(false);
    }
    const node = root.current;
    node?.addEventListener('focusout', onFocusOut);
    return () => node?.removeEventListener('focusout', onFocusOut);
  }, [open]);

  // الصفّ النشِط يبقى في المرأى داخل القائمة الممرَّرة: بلا هذا يتحرّك النشِط
  // بالسهم إلى ما تحت الحافّة فيبدو أن السهم لا يفعل شيئًا.
  useEffect(() => {
    if (!open) return;
    // `CSS.escape` لازم لا احتياط: `useId` يُنتج معرّفًا فيه نقطتان (`:r0:`)،
    // وهو محرف تسلسلٍ في محدِّدات CSS — فبلا تهريبه يرمي `querySelector`.
    const row = list.current?.querySelector(`#${CSS.escape(`${uid}-opt-${active}`)}`);
    row?.scrollIntoView({ block: 'nearest' });
  }, [open, active, uid]);

  /**
   * البحث بالحرف الأوّل، كما تفعله القائمة الأصلية.
   *
   * الحروف تتراكم ما دام بينها أقلّ من ثانية، فـ«ال» تجد «الكل» ولا تقف عند
   * أوّل ما يبدأ بألف. والبحث يبدأ من الصفّ التالي للنشِط حين يتكرّر الحرف
   * نفسه، فضغطُ «ن» مرّتين يتنقّل بين ما يبدأ بها.
   */
  const onType = useCallback(
    (key: string) => {
      if (!options.length) return;
      const now = Date.now();
      const memo = typed.current;
      const fresh = now - memo.at > 1000;
      // `fold` هي دالّة المقارنة العربية نفسها التي يستعملها بحث المكتبة، لا
      // `toLowerCase` وحدها. بدونها لا يصل المستخدم إلى «أُلغيت» إلا إن كتب
      // الهمزة والضمّة كما كُتبت — و«ا» لا تجد «أ»، والشدّة تمنع المطابقة.
      // فكان أكثرُ خيارات المرشّحات غيرَ قابلٍ للبلوغ بلوحة المفاتيح.
      const text = (fresh ? '' : memo.text) + fold(key);
      typed.current = { text, at: now };

      const repeat = text.length > 1 && text.split('').every((c) => c === text[0]);
      const needle = repeat ? text[0]! : text;
      // نقطةُ البدء مفتوحةً هي الصفُّ النشِط، ومغلقةً هي **المختار**: `active`
      // مغلقةً بقيّةُ فتحةٍ سابقة، فالبحث كان ينطلق من مكانٍ لا يراه المستخدم.
      const base = open ? active : selected >= 0 ? selected : 0;
      const from = fresh || repeat ? base + 1 : base;

      for (let i = 0; i < options.length; i += 1) {
        const at = (from + i) % options.length;
        if (fold(options[at]!.label).startsWith(needle)) {
          setActive(at);
          if (!open) onChange(options[at]!.value);
          return;
        }
      }
    },
    [active, selected, open, options, onChange],
  );

  const onKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      const { key } = event;

      if (key === 'Escape') {
        if (!open) return;
        event.preventDefault();
        close(true);
        return;
      }

      if (key === 'Enter' || key === ' ') {
        event.preventDefault();
        if (open) choose(active);
        else openAt();
        return;
      }

      if (key === 'ArrowDown' || key === 'ArrowUp') {
        event.preventDefault();
        if (!open) return openAt();
        const step = key === 'ArrowDown' ? 1 : -1;
        setActive((i) => Math.min(options.length - 1, Math.max(0, i + step)));
        return;
      }

      // ‏Home وEnd مفتوحةً تنقلان النشِط، ومغلقةً **تختاران** الطرف — كما تفعل
      // القائمة الأصلية. وتركُهما مغلقةً بلا معالجة كان يمرّرهما إلى الصفحة
      // فتقفز إلى أعلاها بينما التركيز على مرشّح.
      if (key === 'Home' || key === 'End') {
        if (!options.length) return;
        event.preventDefault();
        const edge = key === 'Home' ? 0 : options.length - 1;
        if (open) setActive(edge);
        else choose(edge);
        return;
      }

      // حرفٌ واحد مطبوع، بلا مُعدِّلات: مفاتيح الاختصار (⌘K) ليست بحثًا.
      if (key.length === 1 && !event.metaKey && !event.ctrlKey && !event.altKey) {
        event.preventDefault();
        onType(key);
      }
    },
    [open, active, options.length, choose, openAt, close, onType],
  );

  return (
    <div className="select" ref={root}>
      <span className="t-caption select__label" id={labelId}>
        {label}
      </span>

      {/* ‏`role="combobox"` على الزرّ، وهو ليس زخرفةً في الوسم.
          ‏`aria-activedescendant` **لا يُقرأ على `role=button`**: المواصفة لا
          تسمح به إلا على `combobox` و`textbox` و`listbox` وأشباهها، فيتجاهله
          واجهُ الإتاحة. أثرُه أن التنقّل بالأسهم داخل القائمة كان صامتًا تمامًا
          لقارئ الشاشة — يتحرّك التمييز في الشاشة ولا يُنطق شيء، وهو أسوأ من
          قائمةٍ لا تُفتح لأنه عطلٌ لا يُرى بالعين.
          و`combobox` مغلقٌ للاختيار (بلا حقل نصّ) هو نمط WAI-ARIA لهذا الضابط
          بعينه، والعنصر يبقى `button` فتبقى المسافة وEnter من المتصفّح. */}
      <button
        type="button"
        ref={button}
        className="select__button"
        role="combobox"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-controls={open ? listId : undefined}
        aria-activedescendant={open ? optionId(active) : undefined}
        aria-labelledby={`${labelId} ${uid}-value`}
        aria-describedby={describedBy}
        aria-invalid={invalid || undefined}
        aria-required={required || undefined}
        disabled={disabled}
        onClick={() => (open ? close(false) : openAt())}
        onKeyDown={onKeyDown}
      >
        <span
          className={`select__value${current ? '' : ' select__value--placeholder'}`}
          id={`${uid}-value`}
        >
          {current?.label ?? placeholder}
        </span>
        {/* القرص لا يُقلب مع الاتجاه: سهمٌ إلى أسفل يعني «تنزل قائمة» في
            الاتجاهين، ولا جهة له تُعكس. */}
        <svg className="select__caret" viewBox="0 0 24 24" aria-hidden="true">
          <use href="#i-chevron-down" />
        </svg>
      </button>

      {open && (
        <ul className="select__menu" role="listbox" id={listId} aria-labelledby={labelId}>
          {options.map((option, i) => (
            <li
              key={option.value}
              id={optionId(i)}
              role="option"
              aria-selected={option.value === value}
              className={
                'select__option' +
                (i === active ? ' select__option--active' : '') +
                (option.value === value ? ' select__option--on' : '')
              }
              /* الاختيار على `click` لا على `mousedown`.
                 كان على `mousedown` كي لا يسبقه مستمعُ الإغلاق؛ وذلك المستمع
                 يستثني ما بداخل الضابط أصلًا، فلم تكن للحيلة حاجة. وثمنُها كان
                 حقيقيًّا: تفعيلُ قارئ الشاشة (VoiceOver: ⌃⌥مسافة) وكلُّ تفعيلٍ
                 برمجيّ يُرسل `click` وحده بلا `mousedown` — فكان الصفّ لا
                 يُختار بأي وسيلةٍ غير الفأرة.
                 و`mousedown` يبقى لغرضٍ واحد: منعُ الضغطة من سحب البؤرة من
                 الزرّ إلى `body`، فلا تُغلق القائمة تحت الإصبع. */
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => choose(i)}
              onMouseEnter={() => setActive(i)}
            >
              <svg className="select__check" viewBox="0 0 24 24" aria-hidden="true">
                <use href="#i-check" />
              </svg>
              {option.label}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
