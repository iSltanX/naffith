/**
 * طبقة عرض العمليات.
 *
 * هذا هو الاختبار الذي وعد به رأس `operations.ts`: كل العمليات المستعملة هنا
 * **مخترعة**، ولا واحدة منها موجودة في هذا البناء. لو عاد أحدٌ يومًا فكتب
 * `if (op.id === '…')` أو رتّب القائمة بأسماء مكتوبة بيده، سقط أول اختبار في
 * الملف. الفهرس يأتي من `list_operations()` ولا شيء هنا يضيف إليه.
 *
 * والنصف الثاني عن النموذج: أين تُشذَّب القيمة وأين لا تُشذَّب. الفرق بينهما
 * ليس تفصيلًا شكليًّا — الفراغ في طرف اسم الملف خطأٌ يجب أن تراه النواة، وفي
 * طرف المسار أثرُ لصقٍ لا يعنيه المستخدم.
 */
import { describe, expect, it } from 'vitest';
import {
  choiceOptions,
  emptyValues,
  extensionHint,
  fieldKeys,
  inputKind,
  isChoiceInput,
  isComplete,
  isDirectoryInput,
  isDirty,
  isFlagInput,
  isNumberInput,
  isPathInput,
  isUrlInput,
  numberSpec,
  toRawValues,
  type FormValues,
} from './operations';
import type { Danger, InputSummary, OperationSummary } from './ipc';

/** مدخلٌ كما يصل من النواة: `kind` مسطّح بجانب `id` و`required`. */
function input(
  id: string,
  kind: string,
  extra: Record<string, unknown> = {},
  required = true,
): InputSummary {
  return { ...extra, id, required, kind } as InputSummary;
}

function op(
  id: string,
  category: OperationSummary['category'],
  danger: Danger,
  inputs: InputSummary[],
): OperationSummary {
  return {
    id,
    title_key: `op.${id}.title`,
    description_key: `op.${id}.desc`,
    category,
    danger,
    // حقولٌ لا يقرؤها هذا الملف: طبقةُ النموذج لا تعنيها الإتاحة ولا الترتيب
    // ولا كلمات البحث. تُملأ بقيمٍ صالحة كي يبقى النوع كاملًا، ويحرسها
    // `library.test.ts` حيث تُقرأ فعلًا.
    conflict: 'no_artifact',
    tool: 'owlctl',
    availability: { state: 'available' },
    sort_order: 10,
    search_terms: ['owlctl'],
    inputs,
  };
}

/** أنواع المدخلات التي تعلنها `InputKind` في `ipc.ts`. */
const KNOWN_KINDS = [
  'existing_dir',
  'existing_file',
  'existing_path',
  'target_dir',
  'new_name',
  'new_dir_name',
  'text',
  'choice',
  'number',
  'url',
  'flag',
];

describe('مفاتيح نصوص الحقل', () => {
  it('الأخصّ ثم العام', () => {
    // `tFirst` تأخذ أول موجود، فالترتيب هو القاعدة كلها: تخصيصٌ حين يلزم،
    // ونصٌّ عامّ حين لا يلزم.
    expect(fieldKeys('alpha.fold.paper', 'source')).toEqual({
      label: ['field.alpha.fold.paper.source.label', 'field.source.label'],
      help: ['field.alpha.fold.paper.source.help', 'field.source.help'],
      placeholder: ['field.alpha.fold.paper.source.placeholder', 'field.source.placeholder'],
    });
  });
});

describe('النموذج الفارغ', () => {
  it('مدخلٌ واحد لكل مدخلٍ معلَن', () => {
    const o = op('nu.form', 'files', 'safe', [
      input('source', 'existing_dir'),
      input('name', 'new_name', { ext: 'zip' }),
      input('quiet', 'flag', {}, false),
    ]);
    expect(emptyValues(o)).toEqual({ source: '', name: '', quiet: '' });
  });

  it('عمليةٌ بلا مدخلات نموذجها فارغ', () => {
    expect(emptyValues(op('mu.count.stars', 'system', 'safe', []))).toEqual({});
  });

  it('الرقم يبدأ بالقيمة الافتراضية المعلنة', () => {
    const o = op('nu.numbered', 'files', 'safe', [
      input('limit', 'number', { min: 1, max: 1000, default: 100 }),
    ]);
    expect(emptyValues(o)).toEqual({ limit: '100' });
  });

  it('الراية تُخزَّن نصًّا كبقية الحقول', () => {
    // سجلٌّ متجانس: التحويل إلى منطقيّ يقع في موضع واحد هو `toRawValues`.
    const o = op('nu.flagged', 'files', 'safe', [input('quiet', 'flag')]);
    expect(emptyValues(o)['quiet']).toBe('');
  });
});

describe('الاكتمال', () => {
  const form = op('nu.complete', 'files', 'creates', [
    input('source', 'existing_dir'),
    input('name', 'new_name', { ext: 'zip' }),
    input('note', 'text', { max_len: 40 }, false),
    input('quiet', 'flag'),
  ]);

  it('المطلوب مملوءًا يكتمل', () => {
    expect(isComplete(form, { source: '/Users/a/x', name: 'x', note: '', quiet: '' })).toBe(true);
  });

  it('حقلٌ مطلوب فارغ يمنع', () => {
    expect(isComplete(form, { source: '', name: 'x', note: '', quiet: '' })).toBe(false);
  });

  it('المسافات وحدها ليست قيمة', () => {
    // هذا ما يقرّر ظهور المعاينة: تخطيطٌ على مسارٍ من مسافات نداءٌ ضائع.
    expect(isComplete(form, { source: '   ', name: 'x', note: '', quiet: '' })).toBe(false);
    expect(isComplete(form, { source: '/x', name: ' \t\n ', note: '', quiet: '' })).toBe(false);
  });

  it('مفتاحٌ غائب أصلًا كالفارغ', () => {
    // النموذج قد يصل ناقصًا بعد تغيّر الفهرس؛ الغياب نقصٌ لا اكتمال.
    expect(isComplete(form, { name: 'x', note: '', quiet: '' })).toBe(false);
    expect(isComplete(form, {})).toBe(false);
  });

  it('الاختياري الفارغ لا يمنع', () => {
    expect(isComplete(form, { source: '/x', name: 'x', quiet: '' })).toBe(true);
  });

  it('الراية المطلوبة المطفأة لا تمنع', () => {
    // غياب الراية قيمةٌ صالحة (مطفأة)، فلا تكون «ناقصة» أبدًا — وإلا لتعطّل
    // «نفِّذ» حتى يشعل المستخدم خيارًا لا يريده.
    const flagOnly = op('nu.flag', 'files', 'safe', [input('quiet', 'flag')]);
    expect(isComplete(flagOnly, { quiet: '' })).toBe(true);
    expect(isComplete(flagOnly, {})).toBe(true);
  });

  it('عمليةٌ بلا مدخلات مكتملة دائمًا', () => {
    expect(isComplete(op('mu.count.stars', 'system', 'safe', []), {})).toBe(true);
  });
});

describe('التحويل إلى قيم الحدّ', () => {
  const form = op('nu.raw', 'compress', 'creates', [
    input('source', 'existing_dir'),
    input('file', 'existing_file'),
    input('either', 'existing_path'),
    input('destination', 'target_dir'),
    input('name', 'new_name', { ext: 'zip' }),
    input('note', 'text', { max_len: 40 }),
    input('quiet', 'flag'),
  ]);

  const values: FormValues = {
    source: '  /Users/a/src  ',
    file: '/Users/a/f.txt',
    either: ' /Users/a/thing ',
    destination: '/Users/a/dst\n',
    name: 'archive ',
    note: ' note ',
    quiet: '1',
  };

  it('المسار يُشذَّب من الفراغ المحيط', () => {
    // اللصق يجرّ معه مسافةً أو سطرًا؛ لا معنى لأن يراها المستخدم خطأً.
    const raw = toRawValues(form, values);
    expect(raw['source']).toEqual({ kind: 'path', value: '/Users/a/src' });
    expect(raw['file']).toEqual({ kind: 'path', value: '/Users/a/f.txt' });
    expect(raw['either']).toEqual({ kind: 'path', value: '/Users/a/thing' });
    expect(raw['destination']).toEqual({ kind: 'path', value: '/Users/a/dst' });
  });

  it('النصّ والاسم لا يُشذَّبان', () => {
    // الفراغ في طرف الاسم خطأٌ يجب أن تراه النواة وترفضه برسالتها، لا أن تُخفيه
    // الواجهة فيُنشأ ملفٌ باسمٍ غير الذي كُتب.
    const raw = toRawValues(form, values);
    expect(raw['name']).toEqual({ kind: 'text', value: 'archive ' });
    expect(raw['note']).toEqual({ kind: 'text', value: ' note ' });
  });

  it('الراية «1» اشتعال وما سواها إطفاء', () => {
    expect(toRawValues(form, values)['quiet']).toEqual({ kind: 'flag', value: true });
    expect(toRawValues(form, { ...values, quiet: '' })['quiet']).toEqual({ kind: 'flag', value: false });
    expect(toRawValues(form, { ...values, quiet: '0' })['quiet']).toEqual({ kind: 'flag', value: false });
    expect(toRawValues(form, { ...values, quiet: 'true' })['quiet']).toEqual({
      kind: 'flag',
      value: false,
    });
  });

  it('كل مدخلٍ معلَن يظهر ولو لم يُملأ', () => {
    // النواة تنتظر المدخلات كما أعلنتها هي؛ حذفُ المفقود يجعل الخطأ «مفتاح
    // ناقص» بدل «قيمة فارغة».
    const raw = toRawValues(form, {});
    expect(Object.keys(raw).sort()).toEqual(
      ['destination', 'either', 'file', 'name', 'note', 'quiet', 'source'].sort(),
    );
    expect(raw['source']).toEqual({ kind: 'path', value: '' });
    expect(raw['quiet']).toEqual({ kind: 'flag', value: false });
  });

  it('قيمةٌ لم تعلنها العملية لا تعبر الحدّ', () => {
    // ما يُرسَل يقرّره `op.inputs` وحده، فلا يمرّ حقلٌ بقي في الحالة من عمليةٍ
    // سابقة.
    const raw = toRawValues(form, { ...values, leftover: 'من عملية أخرى' });
    expect(raw['leftover']).toBeUndefined();
    expect(Object.keys(raw)).toHaveLength(form.inputs.length);
  });

  it('النوع المجهول يعبر نصًّا لا مسارًا', () => {
    // النواة هي التي ترفض، لا الواجهة تخمّن. والمسار يُشذَّب فلا يجوز افتراضه.
    const raw = toRawValues(op('psi.unknown', 'files', 'safe', [input('a', 'colour')]), { a: ' x ' });
    expect(raw['a']).toEqual({ kind: 'text', value: ' x ' });
  });
});

describe('لاحقة الاسم', () => {
  it('تأتي من المواصفة بنقطةٍ واحدة', () => {
    expect(extensionHint(input('name', 'new_name', { ext: 'zip' }))).toBe('.zip');
  });

  it('لا تتضاعف النقطة إن كانت في المواصفة', () => {
    // النواة قد تعلنها بنقطة أو بغيرها؛ المعروض واحد في الحالين.
    expect(extensionHint(input('name', 'new_name', { ext: '.zip' }))).toBe('.zip');
  });

  it('لا لاحقة إن لم تعلن المواصفة واحدة', () => {
    expect(extensionHint(input('name', 'new_name', { ext: null }))).toBeNull();
    expect(extensionHint(input('name', 'new_name', { ext: '' }))).toBeNull();
    expect(extensionHint(input('name', 'new_name'))).toBeNull();
  });

  it('الحقول الأخرى لا لاحقة لها', () => {
    for (const kind of [
      'existing_dir',
      'existing_file',
      'existing_path',
      'target_dir',
      'text',
      'flag',
    ]) {
      expect(extensionHint(input('x', kind, { ext: 'zip' }))).toBeNull();
    }
  });
});

describe('تصنيف المدخل', () => {
  it('أنواع المسار الأربعة والاسم ليس منها', () => {
    // التصنيف يقرّر اتجاه النصّ LTR وزرّ الاختيار؛ الاسم يُكتب لا يُختار.
    expect(KNOWN_KINDS.filter((k) => isPathInput(input('x', k)))).toEqual([
      'existing_dir',
      'existing_file',
      'existing_path',
      'target_dir',
    ]);
  });

  it('حوار المجلد لا يختار ملفًا قائمًا', () => {
    expect(KNOWN_KINDS.filter((k) => isDirectoryInput(input('x', k)))).toEqual([
      'existing_dir',
      'target_dir',
    ]);
    expect(isDirectoryInput(input('x', 'existing_file'))).toBe(false);
  });

  it('الراية وحدها راية', () => {
    expect(KNOWN_KINDS.filter((k) => isFlagInput(input('x', k)))).toEqual(['flag']);
  });

  it('النوع المجهول ليس شيئًا من هذه', () => {
    const odd = input('x', 'colour_picker');
    expect([isPathInput(odd), isDirectoryInput(odd), isFlagInput(odd)]).toEqual([
      false,
      false,
      false,
    ]);
  });

  it('يميّز الاختيار والرقم والرابط ويقرأ مواصفاتها', () => {
    const choice = input('format', 'choice', {
      options: [
        { value: 'png', label_key: 'choice.image.png' },
        { value: 'jpeg', label_key: 'choice.image.jpeg' },
      ],
    });
    const number = input('count', 'number', { min: 1, max: 20, default: 5 });
    const url = input('address', 'url');

    expect(inputKind(choice)).toBe('choice');
    expect(isChoiceInput(choice)).toBe(true);
    expect(choiceOptions(choice)).toHaveLength(2);
    expect(isNumberInput(number)).toBe(true);
    expect(numberSpec(number)).toEqual({ min: 1, max: 20, default: 5 });
    expect(isUrlInput(url)).toBe(true);
  });
});

describe('هل في النموذج ما يُفقد', () => {
  it('النموذج الفارغ لا يُفقد شيئًا', () => {
    expect(isDirty({})).toBe(false);
    expect(isDirty({ source: '', name: '', quiet: '' })).toBe(false);
  });

  it('المسافات وحدها ليست قيمة تُفقد', () => {
    // سؤالُ «أتترك ما كتبت؟» على مسافةٍ عابرة سؤالٌ يُدرَّب المستخدم على تجاهله.
    expect(isDirty({ source: '   ', name: '\n\t' })).toBe(false);
  });

  it('أيّ قيمة حقيقية تجعل المغادرة مكلفة', () => {
    expect(isDirty({ source: '/Users/a/x', name: '' })).toBe(true);
    expect(isDirty({ source: '', quiet: '1' })).toBe(true);
    expect(isDirty({ name: '٠' })).toBe(true);
  });
});
