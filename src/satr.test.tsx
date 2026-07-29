// @vitest-environment jsdom
/**
 * حالتا «سَطْر»: الانتظار والمعاينة.
 *
 * ما يُحرس هنا شيئان. الأول أن نصّ الانتظار **كتابة مائية لا صندوق**: صندوقٌ
 * منقّط بحدٍّ وأرضية يعلن «هنا عنصر فارغ»، وهذه المساحة ليست فارغة بل منتظِرة.
 * والثاني أن المعاينة تعرض الأمر رمزًا رمزًا كما جاء من الخطة، فلا يتسلّل
 * يومًا عرضٌ ملخَّص أو مقتطع يخفي وسيطًا.
 */
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
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

describe('سَطْر في الانتظار', () => {
  it('يكتب الأسطر الثلاثة حين لا توجد خطة', () => {
    const { container } = render(<Satr plan={null} />);
    const mark = container.querySelector('.satr__watermark');
    expect(mark?.textContent).toContain(AR['satr.idle.title']);
    expect(screen.getByText(AR['satr.idle.line1'])).toBeTruthy();
    expect(screen.getByText(AR['satr.idle.line2'])).toBeTruthy();
  });

  it('يترك الكتابة مقروءةً لقارئ الشاشة: تعليمة لا زينة', () => {
    const { container } = render(<Satr plan={null} />);
    const mark = container.querySelector('.satr__watermark');
    expect(mark).toBeTruthy();
    expect(mark?.getAttribute('aria-hidden')).toBeNull();
    // ولا تُعلَن كخبرٍ عاجل: النصّ ساكن منذ أول رسم، فالإعلان مقاطعة لا خبر.
    expect(mark?.getAttribute('aria-live')).toBeNull();
    expect(mark?.getAttribute('role')).toBeNull();
  });

  it('لا يحيط الكتابة ببطاقة ولا صندوق ولا سطح مستقلّ', () => {
    const { container } = render(<Satr plan={null} />);
    const classes = container.querySelector('.satr__watermark')?.className.split(/\s+/) ?? [];
    for (const surface of ['card', 'summary', 'field', 'command', 'satr__empty']) {
      expect(classes).not.toContain(surface);
    }
    // ولا يبقى في الشجرة أثرٌ من الصندوق المنقّط القديم.
    expect(container.querySelector('.satr__empty')).toBeNull();
  });

  it('يحفظ للمساحة ارتفاعها في الحالتين فلا يقفز العمودان', () => {
    const idle = render(<Satr plan={null} />);
    expect(idle.container.querySelector('.satr__stage')).toBeTruthy();
    cleanup();
    const preview = render(<Satr plan={PLAN} />);
    expect(preview.container.querySelector('.satr__stage')).toBeTruthy();
  });
});

describe('سَطْر مع خطة', () => {
  it('ينزع الكتابة المائية تمامًا', () => {
    const { container } = render(<Satr plan={PLAN} />);
    expect(container.querySelector('.satr__watermark')).toBeNull();
    expect(screen.queryByText(AR['satr.idle.line1'])).toBeNull();
    expect(screen.queryByText(AR['satr.idle.line2'])).toBeNull();
  });

  it('يعرض الأمر رمزًا رمزًا كما جاء من الخطة', () => {
    const { container } = render(<Satr plan={PLAN} />);
    const tokens = [...container.querySelectorAll('.arg__token')].map((n) => n.textContent);
    expect(tokens).toEqual(PLAN.argv_display);
  });

  it('يلوّن كل رمز بدوره المعلَن لا بتخمينٍ من نصّه', () => {
    const { container } = render(<Satr plan={PLAN} />);
    const tokens = container.querySelectorAll('.arg__token');
    expect(tokens[0]?.className).toContain('tok-name');
    expect(tokens[1]?.className).toContain('tok-flag');
    expect(tokens[3]?.className).toContain('tok-path');
  });
});
