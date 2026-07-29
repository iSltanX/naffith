// @vitest-environment jsdom
/**
 * ما يجب أن تصمد له شاشة الترحيب.
 *
 * أكثر هذه الاختبارات يحرس *غيابًا* لا حضورًا: زرّان حُذفا عن قصد، وحركةٌ تُطفأ
 * تحت تفضيل، وتأخيرٌ أُزيل. الغياب المقصود لا يدافع عن نفسه — أول من يمرّ على
 * الملف يقرأ النموذج المرجعي فيعيد الزرّين ظنًّا أنهما سقطا سهوًا، أو يعيد
 * «انطواء» المقتطف ظنًّا أنه زينةٌ مجّانية. هذه الاختبارات هي القرار مكتوبًا
 * بصيغةٍ تفشل حين يُنقَض.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Onboarding from './onboarding';
import { AR } from './i18n';

/** يزرع تفضيل تقليل الحركة. `matchMedia` تُسنَد إسنادًا لأنها قد تغيب أصلًا. */
function setReducedMotion(reduce: boolean): () => void {
  const original = window.matchMedia;
  window.matchMedia = ((query: string) => ({
    matches: reduce && query.includes('reduce'),
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  })) as unknown as typeof window.matchMedia;
  return () => {
    window.matchMedia = original;
  };
}

afterEach(cleanup);

describe('شاشة الترحيب', () => {
  it('تعرض الخطوات الثلاث ونصّ الافتتاح', () => {
    render(<Onboarding onStart={() => {}} />);

    expect(screen.getByText(AR['onboarding.lede'])).toBeTruthy();
    expect(screen.getByRole('heading', { name: AR['onboarding.step1.title'] })).toBeTruthy();
    expect(screen.getByRole('heading', { name: AR['onboarding.step2.title'] })).toBeTruthy();
    expect(screen.getByRole('heading', { name: AR['onboarding.step3.title'] })).toBeTruthy();
  });

  it('«ابدأ الآن» زرٌّ حقيقي لا عنصرٌ مُتنكِّر', () => {
    render(<Onboarding onStart={() => {}} />);
    const start = screen.getByRole('button', { name: AR['onboarding.start'] });

    // `div` بدَور `button` يخسر التفعيل بالمسافة وبـ Enter معًا، ويخسر معهما كل
    // ما يعطيه المتصفّح مجّانًا. الدَور وحده لا يكفي: يُطلب الوسم نفسه.
    expect(start.tagName).toBe('BUTTON');
    expect(start.getAttribute('type')).toBe('button');
  });

  it('تنادي onStart مرّةً واحدة عند الضغط على «ابدأ الآن»', async () => {
    const onStart = vi.fn();
    const user = userEvent.setup();
    render(<Onboarding onStart={onStart} />);

    await user.click(screen.getByRole('button', { name: AR['onboarding.start'] }));

    expect(onStart).toHaveBeenCalledTimes(1);
  });

  it('لا تعرض «جولة سريعة» ولا «تخطّي»: فعلٌ واحد لا ثلاثة', () => {
    render(<Onboarding onStart={() => {}} />);

    // الزرّان محذوفان بقرار، لا بسهو. انظر تعليق الرأس في `onboarding.tsx`.
    expect(screen.getAllByRole('button')).toHaveLength(1);
    expect(screen.queryByText(/جولة/)).toBeNull();
    expect(screen.queryByText(/تخطّ/)).toBeNull();
  });

  it('لا يتسرّب مفتاح ترجمة غير مترجَم إلى النص المعروض', () => {
    const { container } = render(<Onboarding onStart={() => {}} />);

    // `t()` تعيد المفتاح كما هو حين يغيب. ظهورُ «.onboarding» في النص يعني أن
    // مفتاحًا غُيِّر اسمه في أحد الطرفين دون الآخر.
    const text = container.textContent ?? '';
    expect(text).not.toMatch(/onboarding\./);
    expect(text).not.toMatch(/\bapp\.(naffith|satr|tagline)\b/);
  });

  it('لا حركة إطلاقًا تحت تفضيل تقليل الحركة', () => {
    const restore = setReducedMotion(true);
    try {
      const { container } = render(<Onboarding onStart={() => {}} />);
      expect(container.querySelector('.onboarding')?.getAttribute('data-motion')).toBe('still');
    } finally {
      restore();
    }
  });

  it('تتحرّك عند الدخول حين لا تفضيل', () => {
    const restore = setReducedMotion(false);
    try {
      const { container } = render(<Onboarding onStart={() => {}} />);
      expect(container.querySelector('.onboarding')?.getAttribute('data-motion')).toBe('enter');
    } finally {
      restore();
    }
  });

  it('المقتطف حاضرٌ في أول رسم: لا مؤقّت ولا انتظار', () => {
    // حارس العطل الأصلي: كان للمقتطف `animation-delay` قدره ٢٠٠ms مع
    // `fill-mode: both`، فيبقى معدوم العتامة بعد ظهور كل ما حوله. الاستعلام
    // هنا متزامنٌ عمدًا — لا `await` ولا تقديم مؤقّتات ولا `findBy*` — فلو عاد
    // المقتطف عنصرًا يُركَّب لاحقًا سقط هذا السطر.
    const { container } = render(<Onboarding onStart={() => {}} />);

    const peek = container.querySelector('.onboarding__peek');
    expect(peek).not.toBeNull();
    expect(peek?.querySelector('.command__body')?.textContent ?? '').toContain('/usr/bin/ditto');
  });

  it('المقتطف داخل البنية نفسها التي يظهر بها كل شيء آخر', () => {
    const { container } = render(<Onboarding onStart={() => {}} />);

    // ليس شقيقًا مؤجّلًا خارج اللوحتين: هو ابنٌ للوحة التعريف، يرتفع معها في
    // الحركة نفسها. أي عنصرٍ يُحرَّك على حدة يعود بنا إلى الوصول المتأخّر.
    const panes = container.querySelector('.onboarding__panes');
    const peek = container.querySelector('.onboarding__peek');
    expect(panes?.contains(peek ?? null)).toBe(true);
    expect(peek?.closest('.onboarding__visual')).not.toBeNull();
  });

  it('لا نمط سطريّ يؤخّر المقتطف أو يُخفيه', () => {
    const { container } = render(<Onboarding onStart={() => {}} />);
    const peek = container.querySelector<HTMLElement>('.onboarding__peek');
    const command = peek?.querySelector<HTMLElement>('.command');

    // الطريق الثاني إلى العطل نفسه: تأخيرٌ يُكتب في JSX بدل CSS. النمط السطري
    // يغلب كل ورقة، فيُحرَس هنا حيث يُقرأ.
    for (const node of [peek, command]) {
      expect(node?.style.animationDelay ?? '').toBe('');
      expect(node?.style.transitionDelay ?? '').toBe('');
      expect(node?.style.opacity ?? '').toBe('');
    }
  });

  it('الأمر مكسورٌ على أربعة أسطر بالضبط: عليها يقوم حجز الارتفاع', () => {
    const { container } = render(<Onboarding onStart={() => {}} />);
    const body = container.querySelector('.onboarding__peek .command__body');

    // `--peek-lines` في `onboarding.css` مضروبٌ في ارتفاع السطر ليحجز ارتفاع
    // الصندوق قبل وصول خطّ `mono`. زيادةُ سطرٍ هنا دون تحديث الرمز هناك تعيد
    // القفزة بالضبط. الرقم مكتوبٌ حرفيًا في الطرفين ليصطدما.
    const lines = (body?.textContent ?? '').split('\n');
    expect(lines).toHaveLength(4);

    // ولا سطرٌ منها يطول فيفيض عن الصندوق: الفيضان يُظهر شريط تمريرٍ أفقيًّا
    // يأكل من الارتفاع، وهو المصدر الثاني للقفزة.
    for (const line of lines) expect(line.length).toBeLessThanOrEqual(40);
  });

  it('المقتطف خارج ترتيب التبويب: أول Tab للفعل لا للرسم المحجوب', () => {
    const { container } = render(<Onboarding onStart={() => {}} />);
    const peek = container.querySelector('.onboarding__peek');

    expect(peek?.getAttribute('aria-hidden')).toBe('true');

    // منطقةٌ محجوبة عن القارئ الصوتي وفيها محطّة تبويب مخالفةٌ صريحة: يقف
    // المستخدم على عنصرٍ لا يُنطق له اسم. فلا محطّة داخلها البتّة.
    const stops = peek?.querySelectorAll(
      'a[href], button, input, select, textarea, [tabindex]:not([tabindex="-1"])',
    );
    expect(stops?.length).toBe(0);

    // وهذا هو الموضع الذي جاءت منه المحطّة: صندوقٌ يمرّر أفقيًا تجعله
    // المتصفّحات الحديثة قابلًا للتبويب من تلقائها — لا يمنعه إلا تصريح.
    // (‏jsdom لا يفعل ذلك، فيُحرَس التصريح نفسه لا أثره.)
    expect(peek?.querySelector('.command__body')?.getAttribute('tabindex')).toBe('-1');
  });

  it('الفعل الأساسي يُبلَغ ويُفعَّل بلوحة المفاتيح وحدها', async () => {
    const onStart = vi.fn();
    const user = userEvent.setup();
    const { container } = render(<Onboarding onStart={onStart} />);

    // البؤرة تبدأ على الإطار ليُقرأ من العنوان، لا على الزر.
    expect(document.activeElement).toBe(container.querySelector('.onboarding'));

    await user.tab();
    expect(document.activeElement).toBe(
      screen.getByRole('button', { name: AR['onboarding.start'] }),
    );

    await user.keyboard('{Enter}');
    expect(onStart).toHaveBeenCalledTimes(1);
  });

  it('الزر أول محطّة تبويبٍ وآخرها: لا محطّة ثانية في الشاشة', async () => {
    const user = userEvent.setup();
    render(<Onboarding onStart={() => {}} />);
    const start = screen.getByRole('button', { name: AR['onboarding.start'] });

    await user.tab();
    expect(document.activeElement).toBe(start);

    // الضغطة الثانية تغادر المستند كلّه (‏jsdom يعيدها إلى `body`). أي عنصرٍ
    // آخر يظهر هنا يعني محطّةً تسلّلت إلى شاشةٍ فعلُها واحد.
    await user.tab();
    expect(document.activeElement).toBe(document.body);
  });

  it('المسافة تفعّل الزر كما يفعل Enter', async () => {
    const onStart = vi.fn();
    const user = userEvent.setup();
    render(<Onboarding onStart={onStart} />);

    await user.tab();
    await user.keyboard('[Space]');

    expect(onStart).toHaveBeenCalledTimes(1);
  });
});
