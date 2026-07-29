// @vitest-environment jsdom
/**
 * النموذج يُشتقّ من المواصفة، لا من حقول مكتوبة في الشاشة.
 *
 * الاختبار الحاسم هنا هو الثاني: عمليةٌ لا وجود لها في هذا البناء، مدخلاتها
 * ملفٌّ قائم ورايةٌ — نوعان لا يظهران في `compress.folder.zip` أصلًا — تُرسم
 * كاملةً. لو عاد أحدٌ يومًا فكتب `if (input.id === 'source')` سقط هذا الاختبار
 * قبل أن يصل التغيير إلى مستخدم.
 *
 * والأول يحرس الاتجاه المعاكس: عملية الضغط يجب أن تبقى كما كانت حرفًا بحرف بعد
 * التعميم — نفس الحقول الثلاثة، بنفس الترتيب، وبنفس النصوص.
 */
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import Naffith from './naffith';
import { AR } from './i18n';
import type { OperationSummary, PlanResponse } from './ipc';
import type { FormValues } from './operations';
import { emptyValues } from './operations';

// `globals: false` يعني أن `afterEach` ليست عامّة، فلا يعمل التنظيف التلقائي
// في مكتبة الاختبار وتتراكم الشجرات بين الاختبارات.
afterEach(cleanup);

/** عملية الضغط كما تعلنها النواة في `ops/compress_ditto.rs`. */
const COMPRESS: OperationSummary = {
  id: 'compress.folder.zip',
  title_key: 'op.compress.folder.zip.title',
  description_key: 'op.compress.folder.zip.description',
  category: 'compress',
  danger: 'creates',
  inputs: [
    { id: 'source', required: true, kind: 'existing_dir' },
    { id: 'destination', required: true, kind: 'target_dir' },
    { id: 'archive_name', required: true, kind: 'new_name', ext: 'zip' },
  ],
};

/** عملية مخترَعة: لا وجود لها في أي بناء، ومدخلاتها من نوعين آخرين. */
const SYNTHETIC: OperationSummary = {
  id: 'invented.op',
  title_key: 'op.invented.title',
  description_key: 'op.invented.description',
  category: 'files',
  danger: 'modifies',
  inputs: [
    { id: 'document', required: true, kind: 'existing_file' },
    { id: 'overwrite', required: false, kind: 'flag' },
  ],
};

function view(op: OperationSummary, over: Partial<Parameters<typeof Naffith>[0]> = {}) {
  const props = {
    operation: op,
    values: emptyValues(op) as FormValues,
    onChange: vi.fn(),
    plan: null,
    error: null,
    phase: 'idle' as const,
    outcome: null,
    onExecute: vi.fn(),
    onCancel: vi.fn(),
    onReveal: vi.fn(),
    onReset: vi.fn(),
    onBack: vi.fn(),
    ...over,
  };
  return { ...render(<Naffith {...props} />), props };
}

describe('عملية الضغط بعد التعميم', () => {
  it('ترسم الحقول الثلاثة بترتيب النواة ونصوصها', () => {
    const { container } = view(COMPRESS);
    const labels = [...container.querySelectorAll('.row-field > label')].map((n) => n.textContent);
    expect(labels).toEqual([
      AR['field.source.label'],
      AR['field.destination.label'],
      AR['field.archive_name.label'],
    ]);
    expect(screen.getByText(AR['field.source.help'])).toBeTruthy();
    expect(screen.getByText(AR['field.archive_name.help'])).toBeTruthy();
  });

  it('تُبقي زرّ الاختيار على المسارين وحدهما', () => {
    view(COMPRESS);
    // مصدر ووجهة: مجلدان يُختاران بالحوار. الاسم لا يُختار.
    expect(screen.getAllByRole('button', { name: AR['field.choose'] })).toHaveLength(2);
  });

  it('تأخذ لاحقة الاسم من المواصفة لا من نصٍّ مكتوب في الشاشة', () => {
    const { container } = view(COMPRESS);
    expect(container.querySelector('.suffix-hint')?.textContent).toBe('.zip');

    cleanup();
    const other: OperationSummary = {
      ...COMPRESS,
      inputs: [{ id: 'archive_name', required: true, kind: 'new_name', ext: 'tar' }],
    };
    const { container: c2 } = view(other);
    expect(c2.querySelector('.suffix-hint')?.textContent).toBe('.tar');
  });

  it('تشتقّ شارة الخطورة من العملية، وتصمت حين لا ترجمة', () => {
    view(COMPRESS);
    expect(screen.getByText(AR['summary.danger.creates'])).toBeTruthy();

    cleanup();
    // `modifies` بلا ترجمة اليوم: شارةٌ تعرض المفتاح خامًا أسوأ من غيابها.
    view(SYNTHETIC);
    expect(screen.queryByText('summary.danger.modifies')).toBeNull();
  });
});

describe('عملية مخترَعة بمدخلات أخرى', () => {
  it('ترسم الملف القائم والراية دون أن تعرف اسم العملية', () => {
    const { container } = view(SYNTHETIC);
    const labels = [...container.querySelectorAll('.row-field label')].map((n) => n.textContent);
    // لا ترجمة لهذين الحقلين، فيظهر المفتاح خامًا — غيابٌ مرئي لا صامت.
    expect(labels).toEqual(['field.document.label', 'field.overwrite.label']);
  });

  it('تحرم الملف القائم من زرّ اختيار المجلد', () => {
    view(SYNTHETIC);
    // حوار النظام المتاح حوار مجلدات، فلا يُعرض زرّ لا يستطيع انتقاء ملف.
    expect(screen.queryByRole('button', { name: AR['field.choose'] })).toBeNull();
    const path = screen.getByLabelText('field.document.label');
    expect(path.getAttribute('dir')).toBe('ltr');
  });

  it('ترسم الراية مربّعَ اختيار حقيقيًا وتخزّنها نصًّا', async () => {
    const user = userEvent.setup();
    const { props } = view(SYNTHETIC);
    const box = screen.getByRole('checkbox', { name: 'field.overwrite.label' });
    expect((box as HTMLInputElement).checked).toBe(false);
    await user.click(box);
    expect(props.onChange).toHaveBeenCalledWith({ document: '', overwrite: '1' });
  });
});

describe('زرّ «نفِّذ»', () => {
  it('يظهر دائمًا معطّلًا بسببٍ مكتوب حين تنقص الحقول', () => {
    view(COMPRESS);
    const button = screen.getByRole('button', { name: AR['action.execute'] });
    expect((button as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(AR['action.execute.why.incomplete'])).toBeTruthy();
  });

  it('يقول «يجري التحقّق» لا «أكمل الحقول» حين تكتمل الحقول والتخطيط جارٍ', () => {
    view(COMPRESS, {
      values: { source: '/a', destination: '/b', archive_name: 'n' },
      phase: 'planning' as const,
    });
    expect(screen.getByText(AR['action.execute.why.planning'])).toBeTruthy();
    expect(screen.queryByText(AR['action.execute.why.incomplete'])).toBeNull();
  });

  it('لا يكرّر الرسالة نفسها في الملخّص وتحت الزرّ', () => {
    const { container } = view(COMPRESS);
    // بلا خطة لا ملخّص: السبب تحت الزرّ هو الصوت الوحيد.
    expect(container.querySelector('.summary')).toBeNull();
    expect(container.querySelectorAll('.naffith__why')).toHaveLength(1);
  });

  it('يستعمل نبرة الإرشاد لا نبرة الخطأ', () => {
    const { container } = view(COMPRESS);
    const why = container.querySelector('.naffith__why');
    expect(why?.className).toContain('t-helper');
    expect(why?.className).not.toContain('field-error');
    expect(why?.getAttribute('role')).not.toBe('alert');
  });
});

describe('ملخّص «ما الذي سيحدث»', () => {
  /** خطةٌ كما تعيدها النواة، بتقديرٍ كامل. */
  const PLAN: PlanResponse = {
    token: 'tok',
    plan_id: 'p1',
    op_id: COMPRESS.id,
    title_key: COMPRESS.title_key,
    description_key: COMPRESS.description_key,
    danger: 'creates',
    argv_display: ['/usr/bin/ditto', '-c', '-k', '/Users/x/src', '/Users/x/dst/.n.part'],
    explain: [],
    warnings: [],
    tool: { id: 'ditto', path: '/usr/bin/ditto' },
    conflict: 'refuse',
    estimate: { approx_source_bytes: 12_400_000, scanned_entries: 1843, complete: true },
    produces: '/Users/x/dst/n.zip',
    writes_to: '/Users/x/dst/.n.part',
    working_directory: null,
  };

  /** كل صفٍّ في `dl` الملخّص: عنوانه وقيمته. */
  function metaRows(container: HTMLElement): Array<{ label: string; value: string }> {
    const meta = container.querySelector('.summary__meta');
    const dts = [...(meta?.querySelectorAll('dt') ?? [])];
    const dds = [...(meta?.querySelectorAll('dd') ?? [])];
    return dts.map((dt, i) => ({
      label: dt.textContent ?? '',
      value: dds[i]?.textContent ?? '',
    }));
  }

  it('لا يعطي عدد العناصر المفحوصة عنوان الحجم', () => {
    // العطل الذي حرسه هذا الاختبار: سلسلة رجوعٍ إلى `summary.estimate` كانت
    // تُلبس الرقم ١٨٤٣ عنوانَ «حجم المصدر تقديريًا»، فيقرأ المستخدم عدّادًا
    // على أنه حجم. رجوعٌ إلى معنى آخر ليس صياغةً أعمّ — هو خبرٌ كاذب.
    const { container } = view(COMPRESS, { plan: PLAN });
    const rows = metaRows(container);
    const labels = rows.map((r) => r.label);
    expect(new Set(labels).size, `عنوان مكرّر في الملخّص: ${labels.join(' | ')}`).toBe(
      labels.length,
    );

    const entries = rows.find((r) => r.value.includes(String(PLAN.estimate?.scanned_entries)));
    expect(entries, 'عدد العناصر المفحوصة غير معروض').toBeTruthy();
    expect(entries?.label).not.toBe(AR['summary.estimate']);
  });

  it('يبقي عنوان الحجم على الحجم وحده', () => {
    const { container } = view(COMPRESS, { plan: PLAN });
    const size = metaRows(container).find((r) => r.label === AR['summary.estimate']);
    expect(size?.value).toContain('≈');
    expect(size?.value).toContain(AR['unit.mb']);
  });
});

describe('الرجوع', () => {
  it('يُعرض ويستدعي `onBack`', async () => {
    const user = userEvent.setup();
    const { props } = view(COMPRESS);
    await user.click(screen.getByRole('button', { name: AR['nav.back'] }));
    expect(props.onBack).toHaveBeenCalledTimes(1);
  });
});
