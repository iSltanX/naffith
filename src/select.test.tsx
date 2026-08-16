// @vitest-environment jsdom
/**
 * القائمة المنسدلة المشتركة.
 *
 * هذا الملفّ هو الثمن المدفوع صراحةً عن ترك `select` الأصلية: كل ما كان يأتي
 * من المتصفّح مجّانًا — الأسهم وHome/End وEnter وEscape والبحث بالحرف الأوّل
 * ودلالةُ `listbox` — مكتوبٌ الآن بيد، فيُحرَس بيد. سقوطُ اختبارٍ هنا يعني أن
 * المستخدم فقد شيئًا كان يملكه قبل التحويل.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Select from './select';

afterEach(cleanup);

const OPTIONS = [
  { value: 'all', label: 'الكل' },
  { value: 'ok', label: 'نجحت' },
  { value: 'bad', label: 'فشلت' },
  { value: 'gone', label: 'أُلغيت' },
];

function view(over: Partial<Parameters<typeof Select>[0]> = {}) {
  const props = {
    label: 'الحالة',
    value: 'all',
    options: OPTIONS,
    onChange: vi.fn(),
    ...over,
  };
  return { ...render(<Select {...props} />), props };
}

/** الضابط بدوره المُعلَن: `combobox` لا `button` — انظر أوّل اختبارٍ أدناه. */
const trigger = () => screen.getByRole('combobox');

describe('القائمة المنسدلة', () => {
  it('تُعلن نفسها قائمةً مغلقة، والاسم المقروء يحمل الوسم والقيمة', () => {
    view();
    // ‏`combobox` لا `button`: على `button` يتجاهل واجهُ الإتاحة
    // ‏`aria-activedescendant`، فيصير التنقّل بالأسهم صامتًا.
    const button = screen.getByRole('combobox');
    expect(button.getAttribute('aria-haspopup')).toBe('listbox');
    expect(button.getAttribute('aria-expanded')).toBe('false');

    // الاسم المقروء يُركَّب فعلًا من الوسم ثم القيمة — لا مجرّد «شيءٍ غير فارغ».
    const named = (button.getAttribute('aria-labelledby') ?? '')
      .split(' ')
      .map((id) => document.getElementById(id)?.textContent?.trim())
      .join(' ');
    expect(named).toBe('الحالة الكل');

    // مغلقةً لا وجود للقائمة في المستند: خياراتُها ليست نصًّا معروضًا في الشاشة.
    expect(screen.queryByRole('listbox')).toBeNull();
  });

  it('قائمةٌ بلا خيارات لا تُفتح على فراغ', async () => {
    // صندوقٌ فارغ يطفو فوق الشاشة، و`aria-activedescendant` يشير إلى معرّفٍ لا
    // وجود له — فيصمت قارئ الشاشة بلا سببٍ ظاهر.
    const user = userEvent.setup();
    view({ options: [], value: '' });

    await user.click(trigger());
    expect(screen.queryByRole('listbox')).toBeNull();

    trigger().focus();
    await user.keyboard('{ArrowDown}');
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(trigger().getAttribute('aria-expanded')).toBe('false');
    expect(trigger().getAttribute('aria-activedescendant')).toBeNull();
  });

  it('البحث بالحرف الأوّل يطابق العربية بعد التوحيد لا حرفًا بحرف', async () => {
    // «أُلغيت» تبدأ بهمزةٍ فوقها ضمّة. بلا `fold` لا يبلغها من يكتب «ا»، وهي
    // دالّة المقارنة نفسها التي يستعملها بحث المكتبة.
    const user = userEvent.setup();
    const { props } = view();
    trigger().focus();
    await user.keyboard('ا');
    expect(props.onChange).toHaveBeenCalledWith('gone');
  });

  it('النقر يفتحها، والنقر ثانيةً يغلقها', async () => {
    const user = userEvent.setup();
    view();
    await user.click(trigger());
    expect(screen.getByRole('listbox')).toBeTruthy();
    expect(trigger().getAttribute('aria-expanded')).toBe('true');

    await user.click(trigger());
    expect(screen.queryByRole('listbox')).toBeNull();
  });

  it('تُفتح على المختار لا على أوّل القائمة', async () => {
    // من يفتح ليغيّر يبدأ من حيث هو.
    const user = userEvent.setup();
    view({ value: 'bad' });
    await user.click(trigger());

    const options = within(screen.getByRole('listbox')).getAllByRole('option');
    expect(options[2]!.className).toContain('select__option--active');
    expect(options[2]!.getAttribute('aria-selected')).toBe('true');
    expect(trigger().getAttribute('aria-activedescendant')).toBe(options[2]!.id);
  });

  it('السهمان يتنقّلان، وEnter يختار', async () => {
    const user = userEvent.setup();
    const { props } = view();
    trigger().focus();

    await user.keyboard('{ArrowDown}'); // يفتح على المختار (‏0)
    expect(screen.getByRole('listbox')).toBeTruthy();
    await user.keyboard('{ArrowDown}{ArrowDown}');
    await user.keyboard('{Enter}');

    expect(props.onChange).toHaveBeenCalledWith('bad');
    // وتُغلق، وتعود البؤرة إلى الزرّ — لا إلى جسم المستند.
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(document.activeElement).toBe(trigger());
  });

  it('السهم لا يتجاوز طرفَي القائمة', async () => {
    // اللفُّ من الآخر إلى الأوّل يجعل ضغطةً زائدة تقفز بالمستخدم إلى الطرف
    // المقابل بلا أن يقصد.
    const user = userEvent.setup();
    view();
    trigger().focus();
    await user.keyboard('{ArrowDown}{ArrowUp}{ArrowUp}');

    const options = within(screen.getByRole('listbox')).getAllByRole('option');
    expect(options[0]!.className).toContain('select__option--active');
  });

  it('‏Home وEnd يبلغان الطرفين', async () => {
    const user = userEvent.setup();
    view();
    trigger().focus();
    await user.keyboard('{ArrowDown}{End}');

    let options = within(screen.getByRole('listbox')).getAllByRole('option');
    expect(options[3]!.className).toContain('select__option--active');

    await user.keyboard('{Home}');
    options = within(screen.getByRole('listbox')).getAllByRole('option');
    expect(options[0]!.className).toContain('select__option--active');
  });

  it('‏Escape يغلق بلا اختيار ويعيد البؤرة إلى الزرّ', async () => {
    const user = userEvent.setup();
    const { props } = view();
    trigger().focus();
    await user.keyboard('{ArrowDown}{ArrowDown}');
    await user.keyboard('{Escape}');

    expect(screen.queryByRole('listbox')).toBeNull();
    expect(props.onChange).not.toHaveBeenCalled();
    expect(document.activeElement).toBe(trigger());
  });

  it('البحث بالحرف الأوّل يبلغ الخيار، مفتوحةً كانت أو مغلقة', async () => {
    // القائمة الأصلية تفعل هذا، ففقدُه تراجعٌ لا تبسيط.
    const user = userEvent.setup();
    const { props } = view();
    trigger().focus();

    // مغلقةً: الحرف يبدّل القيمة مباشرة، كما تفعل `select`.
    await user.keyboard('ف');
    expect(props.onChange).toHaveBeenCalledWith('bad');

    // مفتوحةً: ينتقل النشِط ولا يختار حتى يُضغط Enter.
    await user.keyboard('{ArrowDown}ن');
    const options = within(screen.getByRole('listbox')).getAllByRole('option');
    expect(options[1]!.className).toContain('select__option--active');
  });

  it('النقر على خيار يختاره', async () => {
    const user = userEvent.setup();
    const { props } = view();
    await user.click(trigger());
    await user.click(within(screen.getByRole('listbox')).getByText('فشلت'));

    expect(props.onChange).toHaveBeenCalledWith('bad');
    expect(screen.queryByRole('listbox')).toBeNull();
  });

  it('‏Tab يخرج إلى الضابط التالي ويغلقها خلفه', async () => {
    // البؤرة لا تُحبَس: هذه قائمةٌ لا حوار. لكن ترْكَها مفتوحةً والبؤرةُ في
    // مكانٍ آخر يعني قائمةً معلّقة فوق الشاشة لا يملكها أحد.
    const user = userEvent.setup();
    const { props } = view();
    trigger().focus();
    await user.keyboard('{ArrowDown}');
    expect(screen.getByRole('listbox')).toBeTruthy();

    await user.tab();
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(props.onChange).not.toHaveBeenCalled();
    expect(document.activeElement).not.toBe(trigger());
  });

  it('النقر خارجها يغلقها بلا اختيار', async () => {
    const user = userEvent.setup();
    const { props } = view();
    await user.click(trigger());
    await user.click(document.body);

    expect(screen.queryByRole('listbox')).toBeNull();
    expect(props.onChange).not.toHaveBeenCalled();
  });

  it('كل خيار يعلن اختياره أو عدمه، لا اللون وحده', async () => {
    const user = userEvent.setup();
    view({ value: 'ok' });
    await user.click(trigger());

    const options = within(screen.getByRole('listbox')).getAllByRole('option');
    expect(options.map((o) => o.getAttribute('aria-selected'))).toEqual([
      'false',
      'true',
      'false',
      'false',
    ]);
  });
});
