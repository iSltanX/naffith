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
import { cleanup, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import Naffith, { OperationBar } from './naffith';
import { AR } from './i18n';
import type { OperationSummary, PlanResponse } from './ipc';
import type { FormValues } from './operations';
import { emptyValues } from './operations';

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }));

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
  conflict: 'refuse',
  tool: 'ditto',
  availability: { state: 'available' },
  sort_order: 10,
  search_terms: ['ditto', 'zip'],
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
  conflict: 'no_artifact',
  tool: 'invented',
  availability: { state: 'available' },
  sort_order: 10,
  search_terms: ['invented'],
  inputs: [
    { id: 'document', required: true, kind: 'existing_file' },
    { id: 'overwrite', required: false, kind: 'flag' },
  ],
};

function view(op: OperationSummary, over: Partial<Parameters<typeof Naffith>[0]> = {}) {
  const props = {
    operation: op,
    categoryIcon: '#i-compress',
    values: emptyValues(op) as FormValues,
    onChange: vi.fn(),
    plan: null,
    error: null,
    phase: 'idle' as const,
    outcome: null,
    onBack: vi.fn(),
    onExecute: vi.fn(),
    onReveal: vi.fn(),
    onReset: vi.fn(),
    onLibrary: vi.fn(),
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
    const name = screen.getByLabelText(AR['field.archive_name.label']);
    expect(name.getAttribute('type')).toBe('text');
    expect(name.getAttribute('dir')).toBe('ltr');
    expect(name.classList.contains('field__technical-value')).toBe(true);

    cleanup();
    const other: OperationSummary = {
      ...COMPRESS,
      inputs: [{ id: 'archive_name', required: true, kind: 'new_name', ext: 'tar' }],
    };
    const { container: c2 } = view(other);
    expect(c2.querySelector('.suffix-hint')?.textContent).toBe('.tar');
  });

  it('تشتقّ شارة الخطورة من العملية وتربط نبرتها بنوع الأثر', () => {
    // الشارة في رأس نَفِّذ الداخلي: هي وسمُ العملية لا وسمُ النموذج.
    render(<OperationBar categoryIcon="#i-compress" operation={COMPRESS} onBack={vi.fn()} />);
    expect(screen.getByText(AR['summary.danger.creates'])).toBeTruthy();
    expect(document.querySelector('.opintro__danger--creates')).toBeTruthy();

    cleanup();
    render(<OperationBar categoryIcon="#i-compress" operation={SYNTHETIC} onBack={vi.fn()} />);
    expect(screen.getByText(AR['summary.danger.modifies'])).toBeTruthy();
    expect(document.querySelector('.opintro__danger--modifies')).toBeTruthy();
  });
});

describe('عملية مخترَعة بمدخلات أخرى', () => {
  it('ترسم الملف القائم والراية دون أن تعرف اسم العملية', () => {
    view(SYNTHETIC);
    // لا ترجمة لهذين الحقلين، فيظهر المفتاح خامًا — غيابٌ مرئي لا صامت.
    expect(screen.getByRole('textbox', { name: 'field.document.label' })).toBeTruthy();
    expect(screen.getByRole('checkbox', { name: 'field.overwrite.label' })).toBeTruthy();
  });

  it('تعطي الملف القائم فعل اختيار ملف واحدًا وتحفظ اتجاه المسار', () => {
    view(SYNTHETIC);
    // أصبح للواجهة حوار ملفٍ أصلي؛ لا يحتاج المستخدم إلى لصق المسار يدويًا.
    expect(screen.getByRole('button', { name: AR['field.choose'] })).toBeTruthy();
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

describe('الأنواع المعلنة في فهرس العمليات الكامل', () => {
  const TYPED: OperationSummary = {
    ...SYNTHETIC,
    id: 'invented.typed',
    inputs: [
      {
        id: 'format',
        required: true,
        kind: 'choice',
        options: [
          { value: 'png', label_key: 'choice.format.png' },
          { value: 'jpeg', label_key: 'choice.format.jpeg' },
        ],
      },
      { id: 'count', required: true, kind: 'number', min: 1, max: 5, default: 3 },
      { id: 'address', required: true, kind: 'url' },
      { id: 'source', required: true, kind: 'existing_path' },
    ],
  };

  it('يرسم الاختيار والرقم والرابط بضوابطها لا كحقول نص متشابهة', async () => {
    const user = userEvent.setup();
    const { props } = view(TYPED);

    const choice = screen.getByRole('combobox', {
      name: new RegExp(AR['field.format.label']),
    });
    expect(document.activeElement).toBe(
      screen.getByRole('heading', { name: 'op.invented.title', level: 2 }),
    );
    expect(choice.textContent).toContain(AR['field.format.placeholder']);
    await user.click(choice);
    await user.click(screen.getByRole('option', { name: AR['choice.format.png'] }));
    expect(props.onChange).toHaveBeenCalledWith({
      format: 'png',
      count: '3',
      address: '',
      source: '',
    });

    const number = screen.getByRole('spinbutton', { name: AR['field.count.label'] });
    expect((number as HTMLInputElement).value).toBe('3');
    const metadata = document.querySelector('.number-field__meta');
    expect(metadata?.textContent).toContain('1');
    expect(metadata?.textContent).toContain('5');
    expect(metadata?.textContent).toContain('3');
    await user.click(screen.getByRole('button', { name: `+ ${AR['field.count.label']}` }));
    expect(props.onChange).toHaveBeenCalledWith({
      format: '',
      count: '4',
      address: '',
      source: '',
    });

    const url = screen.getByLabelText('field.address.label');
    expect(url.getAttribute('type')).toBe('url');
    expect(url.getAttribute('dir')).toBe('ltr');
  });

  it('يبقي existing_path فعلًا واحدًا ثم يتيح ملفًا أو مجلدًا', async () => {
    const user = userEvent.setup();
    view(TYPED);
    const choose = screen.getByRole('button', { name: AR['field.choose'] });
    expect(choose.getAttribute('aria-haspopup')).toBe('menu');
    await user.click(choose);
    const file = screen.getByRole('menuitem', { name: AR['field.choose.file'] });
    const folder = screen.getByRole('menuitem', { name: AR['field.choose.folder'] });
    expect(document.activeElement).toBe(file);
    await user.keyboard('{ArrowDown}');
    expect(document.activeElement).toBe(folder);
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('menu')).toBeNull();
    expect(document.activeElement).toBe(choose);
  });

  it('يفتح حوار المجلد مرة واحدة عند اختيار عنصر القائمة', async () => {
    vi.mocked(openDialog).mockResolvedValueOnce('/Users/x/folder');
    const user = userEvent.setup();
    view(TYPED);

    await user.click(screen.getByRole('button', { name: AR['field.choose'] }));
    await user.click(screen.getByRole('menuitem', { name: AR['field.choose.folder'] }));

    expect(openDialog).toHaveBeenCalledTimes(1);
    expect(openDialog).toHaveBeenCalledWith({ directory: true, multiple: false });
  });
});

describe('زرّ «نفِّذ»', () => {
  it('يظهر دائمًا معطّلًا بسببٍ مكتوب حين تنقص الحقول', () => {
    view(COMPRESS);
    const button = screen.getByRole('button', { name: AR['action.execute.incomplete'] });
    expect((button as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText(AR['action.execute.why.incomplete'])).toBeTruthy();
  });

  it('يقول «يجري التحقّق» لا «أكمل الحقول» حين تكتمل الحقول والتخطيط جارٍ', () => {
    view(COMPRESS, {
      values: { source: '/a', destination: '/b', archive_name: 'n' },
      phase: 'planning' as const,
    });
    expect(screen.getByText(AR['action.execute.why.planning'])).toBeTruthy();
    expect(screen.getByRole('button', { name: AR['state.checking'] })).toBeTruthy();
    expect(screen.getByText(AR['state.checking.note'])).toBeTruthy();
    expect(document.querySelector('.runstate--planning .spinner')).toBeTruthy();
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

describe('وصل النتيجة المهيكلة', () => {
  it('يستبدل حالة النهاية القديمة ويصل أفعال النتيجة بعقد النواة', async () => {
    const user = userEvent.setup();
    const { props, container } = view(COMPRESS, {
      phase: 'finished' as const,
      outcome: {
        run_id: 'run-1',
        status: 'success',
        produced: '/Users/x/out.zip',
        result: {
          category: 'artifact',
          semantic: 'completed',
          type: 'artifact',
          path: '/Users/x/out.zip',
          name: 'out.zip',
          parent: '/Users/x',
          reveal: 'file',
        },
      },
    });

    expect(container.querySelector('.result-workspace')).toBeTruthy();
    expect(container.querySelector('.runstate--ok')).toBeNull();
    expect(screen.queryByLabelText(AR['field.source.label'])).toBeNull();
    expect(container.querySelector('.summary')).toBeNull();
    expect(screen.queryByRole('button', { name: AR['action.execute'] })).toBeNull();
    const actions = container.querySelector('.result-actions');
    expect(actions).toBeTruthy();
    await user.click(within(actions as HTMLElement).getByRole('button', { name: AR['action.reveal'] }));
    await user.click(within(actions as HTMLElement).getByRole('button', { name: AR['action.again'] }));
    await user.click(
      within(actions as HTMLElement).getByRole('button', { name: AR['action.return.library'] }),
    );
    expect(props.onReveal).toHaveBeenCalledTimes(1);
    expect(props.onReset).toHaveBeenCalledTimes(1);
    expect(props.onLibrary).toHaveBeenCalledTimes(1);
  });

  it('لا يعرض Reveal إن لم يمنحه عقد النواة', () => {
    view(COMPRESS, {
      phase: 'finished' as const,
      outcome: {
        run_id: 'run-2',
        status: 'success',
        produced: null,
        result: {
          category: 'acknowledgement',
          semantic: 'completed',
          type: 'acknowledgement',
          message_key: 'result.semantic.completed',
          details: [],
          notices: [],
        },
      },
    });
    expect(screen.queryByRole('button', { name: AR['action.reveal'] })).toBeNull();
  });
});

describe('تعذّر حوار اختيار المسار', () => {
  it('يشرح الفشل ويبقي الكتابة اليدوية طريقًا ظاهرًا', async () => {
    vi.mocked(openDialog).mockRejectedValueOnce(new Error('dialog unavailable'));
    const user = userEvent.setup();
    view(COMPRESS);

    await user.click(screen.getAllByRole('button', { name: AR['field.choose'] })[0] as HTMLElement);
    expect(await screen.findByText(AR['err.picker.failed'])).toBeTruthy();

    const source = screen.getByLabelText(AR['field.source.label']);
    expect(source.getAttribute('aria-invalid')).toBe('true');
    const describedBy = source.getAttribute('aria-describedby') ?? '';
    expect(describedBy).toContain('-error');
    expect(screen.getByLabelText(AR['field.destination.label']).getAttribute('aria-invalid')).toBeNull();

    await user.type(source, '/Users/x/source');
    expect(screen.queryByText(AR['err.picker.failed'])).toBeNull();
  });
});

describe('ملخّص «ما الذي سيحدث»', () => {
  const PLAN: PlanResponse = {
    token: 'tok',
    plan_id: 'p1',
    op_id: COMPRESS.id,
    title_key: COMPRESS.title_key,
    description_key: COMPRESS.description_key,
    category: 'compress',
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

  it('يعرض قالب إنشاء الملف والمسار المهيكل دون تخمين argv', () => {
    const { container } = view(COMPRESS, { plan: PLAN });
    const summary = container.querySelector('.summary--creates-file');
    expect(summary).toBeTruthy();
    expect(within(summary as HTMLElement).getByText(AR['summary.plan.creates-file.title'])).toBeTruthy();
    expect(summary?.querySelector('.summary__path')?.textContent).toBe(PLAN.produces);
    expect(summary?.textContent).not.toContain('/Users/x/src');
  });

  it('يفصل الحجم وعدد العناصر في كتلة التقدير', () => {
    const { container } = view(COMPRESS, { plan: PLAN });
    const estimate = container.querySelector('.summary__estimate');
    expect(estimate).toBeTruthy();
    expect(estimate?.textContent).toContain(AR['summary.estimate']);
    expect(estimate?.textContent).toContain(AR['unit.mb']);
    expect(estimate?.textContent).toContain(AR['summary.estimate.entries']);
    expect(estimate?.textContent).toContain(String(PLAN.estimate?.scanned_entries));
  });

  it('يميّز إنشاء المجلد من نوع المدخل لا من اسم العملية', () => {
    const directoryOp: OperationSummary = {
      ...SYNTHETIC,
      danger: 'creates',
      inputs: [{ id: 'name', required: true, kind: 'new_dir_name' }],
    };
    const { container } = view(directoryOp, { plan: { ...PLAN, danger: 'creates' } });
    expect(container.querySelector('.summary--creates-directory')).toBeTruthy();
    expect(screen.getByText(AR['summary.plan.creates-directory.title'])).toBeTruthy();
  });

  it('يعرض القراءة فقط والتعديل كقالبين مستقلين ولا يكشف وسائط الأمر', () => {
    const safePlan: PlanResponse = {
      ...PLAN,
      danger: 'safe',
      conflict: 'no_artifact',
      produces: null,
      writes_to: null,
      working_directory: '/Users/x/project',
      argv_display: ['/usr/bin/tool', '--password', 'do-not-expose'],
      estimate: null,
    };
    const safe = view(COMPRESS, { plan: safePlan });
    expect(safe.container.querySelector('.summary--safe')).toBeTruthy();
    expect(screen.getByText(AR['summary.plan.safe.title'])).toBeTruthy();
    expect(safe.container.querySelector('.summary')?.textContent).not.toContain('do-not-expose');

    cleanup();
    const modified = view(SYNTHETIC, {
      plan: { ...PLAN, danger: 'modifies', produces: null, estimate: null },
    });
    expect(modified.container.querySelector('.summary--modifies')).toBeTruthy();
    expect(screen.getByText(AR['summary.plan.modifies.title'])).toBeTruthy();
  });

  it('يعرض التحذير الفعلي داخل قالب الخطة', () => {
    const { container } = view(COMPRESS, {
      plan: { ...PLAN, warnings: ['warn.size.partial'] },
    });
    expect(container.querySelector('.summary--warning')).toBeTruthy();
    expect(container.querySelector('.summary__warning')?.textContent).toContain(
      AR['warn.size.partial'],
    );
  });

  it('يعرض قالب عدم الإتاحة واسم الأداة حتى بلا خطة', () => {
    const unavailable: OperationSummary = {
      ...COMPRESS,
      availability: { state: 'tool_missing', tool: 'ditto' },
    };
    const { container } = view(unavailable);
    expect(container.querySelector('.summary--unavailable')).toBeTruthy();
    expect(screen.getByText(AR['summary.plan.unavailable.title'])).toBeTruthy();
    expect(container.querySelector('.summary--unavailable')?.textContent).toContain('ditto');
  });
});

describe('رأس العملية الداخلي', () => {
  it('يعرض مسار المكتبة والقسم والعملية ويصل فعلي الرجوع', async () => {
    const user = userEvent.setup();
    const onBack = vi.fn();
    const onLibrary = vi.fn();
    render(
      <OperationBar
        categoryIcon="#i-compress"
        operation={COMPRESS}
        onBack={onBack}
        onLibrary={onLibrary}
      />,
    );
    const crumbs = screen.getByRole('navigation', { name: AR['nav.breadcrumbs'] });
    expect(
      within(crumbs).getByText(AR['op.compress.folder.zip.title']).getAttribute('aria-current'),
    ).toBeNull();
    expect(screen.getByRole('heading', { name: AR['op.compress.folder.zip.title'] })).toBeTruthy();
    expect(screen.queryByText(AR['op.compress.folder.zip.description'])).toBeNull();
    await user.click(within(crumbs).getByRole('button', { name: AR['cat.compress.title'] }));
    expect(onBack).toHaveBeenCalledTimes(1);
    await user.click(within(crumbs).getByRole('button', { name: AR['nav.operations'] }));
    expect(onLibrary).toHaveBeenCalledTimes(1);
  });

  it('يسكن داخل مجرى نَفِّذ ولا يعيد شريط الهوية القديم', () => {
    const { container } = view(COMPRESS);
    expect(container.querySelector('.naffith__scroll > .opintro')).toBeTruthy();
    expect(container.querySelectorAll('.opintro')).toHaveLength(1);
    expect(container.querySelector('.opbar')).toBeNull();
  });
});
