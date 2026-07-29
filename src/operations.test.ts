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
  availabilityOf,
  emptyValues,
  extensionHint,
  fieldKeys,
  findCard,
  iconForCategory,
  isComplete,
  isDirectoryInput,
  isDirty,
  isFlagInput,
  isPathInput,
  toCard,
  toCards,
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
  return { ...extra, id, required, kind };
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
    inputs,
  };
}

/** أنواع المدخلات الستة التي تعرفها الواجهة، كما تعلنها `InputKind` في `ipc.ts`. */
const KNOWN_KINDS = ['existing_dir', 'existing_file', 'target_dir', 'new_name', 'text', 'flag'];

/**
 * فهرسٌ مخترع بالكامل.
 *
 * المعرّفات والفئات لا تطابق أيّ عملية في هذا البناء عمدًا، والترتيب هنا ليس
 * أبجديًّا كي يُمسَك أيّ فرزٍ يُضاف في الواجهة.
 */
const INVENTED: OperationSummary[] = [
  op('zeta.summon.owl', 'compress', 'creates', [input('source', 'existing_dir')]),
  op('alpha.fold.paper', 'files', 'modifies', [input('target', 'target_dir')]),
  op('mu.count.stars', 'system', 'safe', []),
  op('beta.erase.map', 'files', 'destructive', [input('victim', 'existing_file')]),
];

describe('الفهرس يأتي من النواة', () => {
  it('يعرض عملياتٍ لا وجود لها في هذا البناء', () => {
    const cards = toCards(INVENTED);
    expect(cards).toHaveLength(INVENTED.length);
    expect(cards[0]).toEqual({
      id: 'zeta.summon.owl',
      titleKey: 'op.zeta.summon.owl.title',
      descriptionKey: 'op.zeta.summon.owl.desc',
      category: 'compress',
      danger: 'creates',
      icon: '#i-compress',
      availability: { state: 'available' },
    });
  });

  it('يحفظ ترتيب النواة ولا يفرز', () => {
    // فرضُ ترتيبٍ هنا يجعل الواجهة تقرّر أولويةَ عرضٍ ليست لها، وينفصل عن نيّة
    // الفهرس حين يُرتَّب في النواة يومًا.
    expect(toCards(INVENTED).map((c) => c.id)).toEqual([
      'zeta.summon.owl',
      'alpha.fold.paper',
      'mu.count.stars',
      'beta.erase.map',
    ]);
  });

  it('الفهرس الفارغ قائمةٌ فارغة لا حالةٌ خاصة', () => {
    expect(toCards([])).toEqual([]);
  });

  it('البطاقة لا تحمل نصًّا بل مفاتيح نصوص', () => {
    // النصّ كلّه في `i18n.ts`. بطاقةٌ تحمل نصًّا عربيًّا تعني نسخةً ثانية منه.
    const card = toCard(op('nu.invented.thing', 'files', 'safe', []));
    expect(card.titleKey).toBe('op.nu.invented.thing.title');
    expect(card.descriptionKey).toBe('op.nu.invented.thing.desc');
  });

  it('يجد البطاقة بمعرّفها', () => {
    const found = findCard(toCards(INVENTED), 'beta.erase.map');
    expect(found?.danger).toBe('destructive');
  });

  it('لا يجد ما لم يعد في الفهرس', () => {
    // العملية قد تختفي بين تشغيلين: شاشةٌ محفوظة على معرّفٍ قديم يجب أن تتلقّى
    // `undefined` لا استثناءً.
    expect(findCard(toCards(INVENTED), 'gone.forever')).toBeUndefined();
    expect(findCard([], 'anything')).toBeUndefined();
  });
});

describe('الأيقونة من الفئة', () => {
  it('لكل فئة أيقونتها', () => {
    expect(iconForCategory('files')).toBe('#i-file');
    expect(iconForCategory('compress')).toBe('#i-compress');
    expect(iconForCategory('system')).toBe('#i-system');
    expect(iconForCategory('internal')).toBe('#i-admin');
  });

  it('الأيقونات متمايزة فلا تتشابه فئتان في القائمة', () => {
    const icons = (['files', 'compress', 'system', 'internal'] as const).map(iconForCategory);
    expect(new Set(icons).size).toBe(icons.length);
  });

  it('فئةٌ لا نعرفها تُعطى أيقونةً لا فراغًا', () => {
    // النواة قد تُدرج فئةً أحدث من الواجهة. `<use href="">` يرسم لا شيء،
    // والبطاقة بلا أيقونة تبدو عطبًا لا جديدًا.
    const unseen = 'quantum' as unknown as OperationSummary['category'];
    expect(iconForCategory(unseen)).toBe('#i-file');
  });
});

describe('التوفّر', () => {
  it('كل الأنواع المعروفة تُرسم', () => {
    const all = op(
      'omega.everything',
      'files',
      'safe',
      KNOWN_KINDS.map((kind, i) => input(`in${i}`, kind)),
    );
    expect(availabilityOf(all)).toEqual({ state: 'available' });
  });

  it('عمليةٌ بلا مدخلات متاحة', () => {
    expect(availabilityOf(op('mu.count.stars', 'system', 'safe', []))).toEqual({
      state: 'available',
    });
  });

  it('نوعٌ لا تعرفه الواجهة يُعطّل العملية ويسمّي السبب', () => {
    // تطبيقٌ يشحن نواةً أحدث من واجهته يعرض العملية معطّلةً بسببٍ مفهوم، بدل أن
    // يفتح لها شاشةً فارغة.
    const future = op('psi.time.travel', 'files', 'safe', [
      input('a', 'existing_dir'),
      input('b', 'colour_picker'),
    ]);
    expect(availabilityOf(future)).toEqual({
      state: 'unsupported',
      unknownKinds: ['colour_picker'],
    });
  });

  it('الأنواع المجهولة تُذكر مرّةً واحدة ومرتّبة', () => {
    // الرسالة تُعرض للمستخدم فلا تتكرّر فيها كلمة ولا يتغيّر ترتيبها بين تشغيلين.
    const messy = op('psi.mess', 'files', 'safe', [
      input('a', 'zeta_kind'),
      input('b', 'alpha_kind'),
      input('c', 'zeta_kind'),
      input('d', 'text'),
    ]);
    expect(availabilityOf(messy)).toEqual({
      state: 'unsupported',
      unknownKinds: ['alpha_kind', 'zeta_kind'],
    });
  });

  it('مدخلٌ بلا نوع أصلًا يُعطّل لا يُتجاهَل', () => {
    // قيمةٌ ناقصة من الحدّ لا تُفترض سليمة: التعطيل هو الاتجاه الآمن.
    const broken: OperationSummary = op('psi.broken', 'files', 'safe', [
      { id: 'a', required: true },
    ]);
    expect(availabilityOf(broken).state).toBe('unsupported');
  });

  it('التوفّر جزء من البطاقة لا حساب ثانٍ في الشاشة', () => {
    const card = toCard(op('psi.one', 'files', 'safe', [input('a', 'nonsense')]));
    expect(card.availability).toEqual({ state: 'unsupported', unknownKinds: ['nonsense'] });
  });
});

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
    input('destination', 'target_dir'),
    input('name', 'new_name', { ext: 'zip' }),
    input('note', 'text', { max_len: 40 }),
    input('quiet', 'flag'),
  ]);

  const values: FormValues = {
    source: '  /Users/a/src  ',
    file: '/Users/a/f.txt',
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
      ['destination', 'file', 'name', 'note', 'quiet', 'source'].sort(),
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
    for (const kind of ['existing_dir', 'existing_file', 'target_dir', 'text', 'flag']) {
      expect(extensionHint(input('x', kind, { ext: 'zip' }))).toBeNull();
    }
  });
});

describe('تصنيف المدخل', () => {
  it('المسارات ثلاثة والاسم ليس منها', () => {
    // التصنيف يقرّر اتجاه النصّ LTR وزرّ الاختيار؛ الاسم يُكتب لا يُختار.
    expect(KNOWN_KINDS.filter((k) => isPathInput(input('x', k)))).toEqual([
      'existing_dir',
      'existing_file',
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
