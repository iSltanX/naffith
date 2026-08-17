// @vitest-environment jsdom
/**
 * حالتا «سَطْر»: الطيّ والتوسّع.
 *
 * ما يُحرس هنا ثلاثة أشياء:
 *
 * 1. **المطويّة تعرض الاسم والحالة، لا أكثر.** هذا هو العقد البنيوي للوحة:
 *    مساحةٌ محجوزةٌ لمحتوًى غائب تُقرأ عطلًا. وقد سقط ثلاث مرّات — صندوقٌ منقّط،
 *    ثم كتابةٌ مائية موسَّطة، ثم ثلاثة أسطر في أعلى عمودٍ كامل — والثلاثة كانت
 *    محتوًى بديلًا يشغل مكان الغائب. فيُفحص هنا أن الشريط لا يحمل تعليمات ولا
 *    محتوًى تقنيًا، وأن اللوحة تعلن طيّها بصنفٍ يقرؤه ملفّ الأنماط.
 * 2. **المعاينة تعرض الأمر رمزًا رمزًا** كما جاء من الخطة، فلا يتسلّل يومًا عرضٌ
 *    ملخَّص أو مقتطع يخفي وسيطًا.
 * 3. **الشرح مطويّ والأمر مفتوح.** ما يُطوى تفسيرٌ، وما لا يُطوى هو الأمر نفسه
 *    والتحذيرات عليه.
 */
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import Satr from './satr';
import { AR } from './i18n';
import type { PlanResponse } from './ipc';

// `globals: false` في إعداد الاختبار يعني أن `afterEach` ليست عامّة، فلا يعمل
// التنظيف التلقائي في مكتبة الاختبار. بدونه تتراكم الشجرات فيجد `screen` نصًّا
// من اختبارٍ سابق ويمرّ اختبارٌ كان يجب أن يسقط.
afterEach(cleanup);

const PLAN: PlanResponse = {
  token: 'tok',
  plan_id: 'p1',
  op_id: 'compress.folder.zip',
  title_key: 'op.compress.folder.zip.title',
  description_key: 'op.compress.folder.zip.description',
  category: 'compress',
  danger: 'creates',
  argv_display: ['/usr/bin/ditto', '-c', '-k', '/Users/x/src', '/Users/x/dst/.n.part'],
  explain: [
    { token: '/usr/bin/ditto', key: 'explain.ditto.tool', role: 'tool' },
    { token: '-c', key: 'explain.ditto.create', role: 'flag' },
    { token: '-k', key: 'explain.ditto.pkzip', role: 'flag' },
    { token: '/Users/x/src', key: null, role: 'path' },
    { token: '/Users/x/dst/.n.part', key: null, role: 'path' },
  ],
  warnings: [],
  tool: { id: 'ditto', path: '/usr/bin/ditto' },
  conflict: 'refuse',
  estimate: { approx_source_bytes: 12_400_000, scanned_entries: 42, complete: true },
  produces: '/Users/x/dst/n.zip',
  writes_to: '/Users/x/dst/.n.part',
  working_directory: null,
};

describe('سَطْر مطويًّا', () => {
  it('يعرض الاسم والحالة، ولا شيء غيرهما', () => {
    const { container } = render(<Satr plan={null} />);
    const rail = container.querySelector('.satr__rail');
    expect(rail).toBeTruthy();
    expect(rail?.textContent).toContain(AR['app.satr']);
    expect(rail?.textContent).toContain(AR['satr.idle.title']);
    expect(rail?.querySelector('svg')).toBeNull();

    // العقد البنيوي: لا محتوًى تقنيًّا ولا مسرحًا ولا حتى وسمَ قسمٍ في المطويّة.
    // كلّها مساحةٌ محجوزة لما لا يوجد، وهي العطل الذي تكرّر ثلاث مرّات.
    for (const gone of ['.command', '.args', '.satr__stage', '.satr__notes', '.satr__head']) {
      expect(container.querySelector(gone), `${gone} معروض وهو مطويّ`).toBeNull();
    }
  });

  it('يعلن طيّه بصنفٍ على اللوحة نفسها', () => {
    // الصنف هو ما يقرؤه `operation-layout.css` ليقرّر العرض، فغيابُه يعني لوحةً بعرض
    // المطويّة وفيها محتوى المتوسّعة — أو العكس.
    const idle = render(<Satr plan={null} />);
    expect(idle.container.querySelector('.satr')?.className).not.toContain('satr--live');
    cleanup();
    const live = render(<Satr plan={PLAN} />);
    expect(live.container.querySelector('.satr')?.className).toContain('satr--live');
  });

  it('يبقي الاسم مقروءًا لقارئ الشاشة في الحالتين', () => {
    // المنطقة تُسمّى بـ`aria-labelledby="satr-heading"`. مرجعٌ معلَّق يترك
    // المنطقة بلا اسم، ومعرّفٌ مكرّر يجعله يشير إلى غير ما يُرى.
    const idle = render(<Satr plan={null} />);
    expect(idle.container.querySelectorAll('#satr-heading')).toHaveLength(1);
    expect(idle.container.querySelector('.satr__rail')?.getAttribute('aria-hidden')).toBeNull();
    cleanup();
    const live = render(<Satr plan={PLAN} />);
    expect(live.container.querySelectorAll('#satr-heading')).toHaveLength(1);
  });

  it('لا يحيط الشريط ببطاقة ولا صندوق ولا سطح مستقلّ', () => {
    const { container } = render(<Satr plan={null} />);
    const classes = container.querySelector('.satr__rail')?.className.split(/\s+/) ?? [];
    for (const surface of ['card', 'summary', 'field', 'command', 'satr__empty']) {
      expect(classes).not.toContain(surface);
    }
    // ولا يبقى في الشجرة أثرٌ من الصياغتين السابقتين.
    expect(container.querySelector('.satr__empty')).toBeNull();
    expect(container.querySelector('.satr__watermark')).toBeNull();
  });
});

describe('سَطْر متوسّعًا', () => {
  it('ينزع الشريط المطويّ تمامًا', () => {
    const { container } = render(<Satr plan={PLAN} />);
    expect(container.querySelector('.satr__rail')).toBeNull();
    expect(screen.queryByText(AR['satr.idle.title'])).toBeNull();
  });

  it('يعرض الأمر رمزًا رمزًا كما جاء من الخطة بلا بادئة طرفية', () => {
    const { container } = render(<Satr plan={PLAN} />);
    const tokens = [...container.querySelectorAll('.command__token')].map((n) =>
      n.textContent?.trim(),
    );
    expect(tokens).toEqual(PLAN.argv_display);
    expect(container.querySelector('.command__body')?.textContent?.trim().startsWith('$')).toBe(false);
  });

  it('يلوّن كل رمز بدوره المعلَن لا بتخمينٍ من نصّه', () => {
    const { container } = render(<Satr plan={PLAN} />);
    const tokens = container.querySelectorAll('.command__token');
    expect(tokens[0]?.className).toContain('command__token--tool');
    expect(tokens[0]?.querySelector('.tok-name')).toBeTruthy();
    expect(tokens[1]?.className).toContain('command__token--flag');
    expect(tokens[3]?.className).toContain('command__token--path');
  });

  it('يعرض أمر Page 16 المدمج ولا يعيد تشريح الوسائط القديم', () => {
    const { container } = render(<Satr plan={PLAN} />);
    expect(container.querySelector('.command__body')?.textContent).toContain(
      PLAN.argv_display[0],
    );
    expect(container.querySelector('.arg')).toBeNull();
    expect(container.querySelector('.args')).toBeNull();
    expect(container.querySelector('.satr__notes')).toBeNull();
    expect(container.querySelector('.command__copy svg')).toBeNull();
  });

  it('يبقي تحذير المسافة ظاهرًا تحت الأمر لا داخل صفٍّ مطوي', () => {
    const flagged: PlanResponse = {
      ...PLAN,
      argv_display: PLAN.argv_display.map((token, i) =>
        i === 3 ? '/Users/x/src ' : token,
      ),
      explain: PLAN.explain.map((tok, i) =>
        i === 3 ? { ...tok, token: '/Users/x/src ' } : tok,
      ),
    };
    const { container } = render(<Satr plan={flagged} />);
    expect(container.querySelector('.command__warning')?.textContent).toBe(
      AR['satr.command.suspicious'],
    );
    expect(container.querySelector('.arg')).toBeNull();
  });

  it('يصل حالات الخطة والتشغيل والإلغاء بالفعل المناسب', () => {
    render(<Satr plan={PLAN} phase="planning" />);
    expect(screen.getByText(AR['satr.state.planned'])).toBeTruthy();
    expect(screen.queryByRole('button', { name: AR['satr.action.cancel'] })).toBeNull();

    cleanup();
    const onCancel = vi.fn();
    render(<Satr plan={PLAN} phase="running" onCancel={onCancel} />);
    screen.getByRole('button', { name: AR['satr.action.cancel'] }).click();
    expect(onCancel).toHaveBeenCalledTimes(1);

    cleanup();
    render(<Satr plan={PLAN} phase="cancelling" onCancel={vi.fn()} />);
    const waiting = screen.getByRole('button', { name: AR['state.cancelling'] });
    expect((waiting as HTMLButtonElement).disabled).toBe(true);
  });
});
