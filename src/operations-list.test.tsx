// @vitest-environment jsdom
/**
 * ما تحرسه هذه الاختبارات ليس شكل الشاشة بل خلوّها من فهرسٍ ثانٍ.
 *
 * البطاقات المحقونة هنا **لا وجود لها في هذا البناء**: معرّفاتها ومفاتيحها
 * مخترعة. فلو عاد أحدٌ يومًا فكتب في المكوّن خريطةً من معرّف إلى عنوان أو
 * أيقونة — أو `if` واحدة على معرّف — سقطت هذه الاختبارات لأن العمليات الخمس
 * ستُرسم ناقصةً أو لن تُرسم أصلًا.
 *
 * ومفتاحٌ لا ترجمة له يُعرض كما هو (انظر `t` في `i18n.ts`)، وهذه ليست ثغرةً
 * في الاختبار بل ما يجعله دقيقًا: النصّ المتوقَّع هو المفتاح حرفًا بحرف.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import OperationsList, { type OperationsState } from './operations-list';
import type { OperationCard } from './operations';
import { t } from './i18n';

// التنظيف يدويّ: `globals: false` في إعداد vitest يعني أن `afterEach` ليست
// عامّة، فلا يستطيع testing-library تركيب تنظيفه التلقائي.
afterEach(cleanup);

/** بطاقة موهومة. القيم الافتراضية تجعل كل اختبار يذكر ما يهمّه وحده. */
function card(over: Partial<OperationCard> & { id: string }): OperationCard {
  return {
    titleKey: `title.${over.id}`,
    descriptionKey: `desc.${over.id}`,
    category: 'files',
    danger: 'safe',
    icon: '#i-file',
    availability: { state: 'available' },
    ...over,
  };
}

/** خمس عمليات لا تعرفها هذه النسخة ولا نواتها. */
const INVENTED: OperationCard[] = [
  card({ id: 'tape.archive.dump', icon: '#i-disk', category: 'system' }),
  card({ id: 'mirror.tree.sync', icon: '#i-file-move', danger: 'modifies' }),
  card({ id: 'checksum.tree.verify', icon: '#i-security', category: 'system' }),
  card({ id: 'bundle.folder.tar', icon: '#i-compress', category: 'compress', danger: 'creates' }),
  card({ id: 'share.volume.mount', icon: '#i-network', category: 'system' }),
];

function noop() {
  /* لا شيء */
}

function show(state: OperationsState, handlers: Partial<Parameters<typeof OperationsList>[0]> = {}) {
  return render(
    <OperationsList
      state={state}
      onSelect={handlers.onSelect ?? noop}
      onRetry={handlers.onRetry ?? noop}
      onOpenLog={handlers.onOpenLog ?? noop}
      onOpenSettings={handlers.onOpenSettings ?? noop}
    />,
  );
}

/** الزرّ الذي يحمل عنوانًا بعينه. */
function tileOf(titleKey: string): HTMLButtonElement {
  const button = screen.getByText(titleKey).closest('button');
  if (!button) throw new Error(`لا زرّ للبطاقة ${titleKey}`);
  return button;
}

/** معرّفات الرموز المُشار إليها داخل بطاقة. */
function iconsIn(button: HTMLButtonElement): string[] {
  return [...button.querySelectorAll('use')].map((u) => u.getAttribute('href') ?? '');
}

describe('قائمة العمليات — التوسّع بلا تعديل', () => {
  it('ترسم خمس عملياتٍ مخترعة كاملةً: اسمًا ووصفًا وأيقونة', () => {
    show({ status: 'ready', cards: INVENTED });

    expect(screen.getAllByRole('listitem')).toHaveLength(5);
    for (const c of INVENTED) {
      expect(screen.getByText(c.titleKey)).toBeTruthy();
      expect(screen.getByText(c.descriptionKey)).toBeTruthy();
      expect(iconsIn(tileOf(c.titleKey))).toContain(c.icon);
    }
  });

  it('ترسم عمليةً واحدة كما ترسم اثنتي عشرة', () => {
    const one = card({ id: 'solo.op.only', icon: '#i-export' });
    const { unmount } = show({ status: 'ready', cards: [one] });

    expect(screen.getAllByRole('listitem')).toHaveLength(1);
    expect(screen.getByText(one.titleKey)).toBeTruthy();
    expect(screen.getByText(one.descriptionKey)).toBeTruthy();
    expect(screen.getByText(t('ops.count.one'))).toBeTruthy();
    unmount();

    const many = Array.from({ length: 12 }, (_, i) => card({ id: `bulk.op.n${i}` }));
    show({ status: 'ready', cards: many });
    expect(screen.getAllByRole('listitem')).toHaveLength(12);
  });
});

describe('قائمة العمليات — الاختيار', () => {
  it('تمرّر معرّف النواة حرفيًّا عند النقر', async () => {
    const onSelect = vi.fn();
    show({ status: 'ready', cards: INVENTED }, { onSelect });

    await userEvent.setup().click(tileOf('title.checksum.tree.verify'));

    expect(onSelect).toHaveBeenCalledTimes(1);
    expect(onSelect).toHaveBeenCalledWith('checksum.tree.verify');
  });

  it('تفتح الشاشة من لوحة المفاتيح كما من الفأرة', async () => {
    const onSelect = vi.fn();
    show({ status: 'ready', cards: [card({ id: 'solo.op.only' })] }, { onSelect });

    const user = userEvent.setup();
    await user.tab();
    await user.tab();
    await user.tab();
    await user.keyboard('{Enter}');

    expect(onSelect).toHaveBeenCalledWith('solo.op.only');
  });
});

describe('قائمة العمليات — عمليةٌ لا تدعمها هذه النسخة', () => {
  const blocked = card({
    id: 'future.op.unknown',
    availability: { state: 'unsupported', unknownKinds: ['duration'] },
  });

  it('تعطّل الزرّ فعليًّا لا بالمظهر', () => {
    show({ status: 'ready', cards: [blocked, ...INVENTED] });
    expect(tileOf(blocked.titleKey).disabled).toBe(true);
    expect(tileOf('title.tape.archive.dump').disabled).toBe(false);
  });

  it('لا تنادي onSelect مهما نُقر عليها', async () => {
    const onSelect = vi.fn();
    show({ status: 'ready', cards: [blocked] }, { onSelect });

    await userEvent.setup().click(tileOf(blocked.titleKey));

    expect(onSelect).not.toHaveBeenCalled();
  });

  it('تقول إنها غير متاحة وتقول لماذا', () => {
    show({ status: 'ready', cards: [blocked] });
    expect(screen.getByText(t('ops.unavailable'))).toBeTruthy();
    expect(screen.getByText(t('ops.unavailable.why'))).toBeTruthy();
  });

  it('لا تُحسب في عدد المتاح', () => {
    show({ status: 'ready', cards: [blocked, card({ id: 'a.b.c' })] });
    // بطاقتان معروضتان، وواحدة متاحة: العدد يصف ما يمكن فتحه لا ما يُرى.
    expect(screen.getAllByRole('listitem')).toHaveLength(2);
    expect(screen.getByText(t('ops.count.one'))).toBeTruthy();
  });
});

describe('قائمة العمليات — الحالات', () => {
  it('تعلن التحميل في منطقةٍ حيّة', () => {
    const { container } = show({ status: 'loading' });
    expect(screen.getByText(t('ops.loading'))).toBeTruthy();
    expect(container.querySelector('.spinner')).toBeTruthy();
    expect(screen.queryByRole('list')).toBeNull();
  });

  it('تقول إن الفهرس خالٍ بدل أن تعرض شبكةً فارغة', () => {
    show({ status: 'ready', cards: [] });
    expect(screen.getByText(t('ops.empty'))).toBeTruthy();
    expect(screen.queryByRole('list')).toBeNull();
  });

  it('تعرض العطل وسببه وتتيح إعادة المحاولة', async () => {
    const onRetry = vi.fn();
    show(
      { status: 'failed', error: { key: 'err.io', input: null, detail: null } },
      { onRetry },
    );

    expect(screen.getByRole('alert')).toBeTruthy();
    expect(screen.getByText(t('ops.failed'))).toBeTruthy();
    expect(screen.getByText(t('err.io'))).toBeTruthy();

    await userEvent.setup().click(screen.getByRole('button', { name: t('ops.retry') }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it('تعرض السؤال وعنوانه الفرعي في كل حال', () => {
    show({ status: 'loading' });
    expect(screen.getByRole('heading', { name: t('ops.heading') })).toBeTruthy();
    expect(screen.getByText(t('ops.subheading'))).toBeTruthy();
  });
});

describe('قائمة العمليات — السجلّ والإعدادات', () => {
  it('يبقيان في المتناول ولو لم تُقرأ العمليات', async () => {
    const onOpenLog = vi.fn();
    const onOpenSettings = vi.fn();
    show({ status: 'failed', error: { key: 'err.unknown', input: null, detail: null } }, {
      onOpenLog,
      onOpenSettings,
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole('button', { name: t('nav.log') }));
    await user.click(screen.getByRole('button', { name: t('nav.settings') }));

    expect(onOpenLog).toHaveBeenCalledTimes(1);
    expect(onOpenSettings).toHaveBeenCalledTimes(1);
  });
});
