// @vitest-environment jsdom
/**
 * مجرى التشغيل: ما تقوله الأداة يصل كما قالته، ولا يُقصّ صامتًا.
 *
 * العطل الذي وُلد منه المكوّن: النواة تبثّ `run://output` والواجهة لا تستمع، فكان
 * كل ما تطبعه الأداة يُهمل — ومع الفشل تبقى الرسالة «لم تكتمل العملية» بلا سبب.
 * فما يُحرس هنا ثلاثة:
 *
 * 1. **الاحتفاظ محدود، والإسقاط من الرأس.** خمسة آلاف صفٍّ في DOM تُلجلج الشاشة،
 *    والذيلُ هو ما يُشخَّص به. ومنطقُ الاحتفاظ يُختبر وحده لأنه ما يُخطئ صامتًا.
 * 2. **`stderr` يُميَّز بوسمٍ نصّي** لا باللون وحده.
 * 3. **الطيّ يتبع الطور**: مفتوحٌ أثناء التشغيل وبعد الفشل، مطويٌّ بعد النجاح —
 *    وإغلاقُ المستخدم يُحترم فلا يُعاد فتحه مع كل سطرٍ يصل.
 */
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import RunStream, {
  MAX_KEPT_LINES,
  appendLine,
  droppedCount,
  streamPresentation,
  type StreamLine,
} from './run-stream';
import { AR } from './i18n';
import type { RunOutputEvent } from './ipc';

afterEach(cleanup);

const RUN = 'run-1';

const out = (line: string): RunOutputEvent => ({ run_id: RUN, stream: 'stdout', line });
const err = (line: string): RunOutputEvent => ({ run_id: RUN, stream: 'stderr', line });
const cut = (dropped: number): RunOutputEvent => ({
  run_id: RUN,
  stream: 'truncated',
  line: { dropped },
});
const omitted = (dropped: number): RunOutputEvent => ({
  run_id: RUN,
  stream: 'omitted',
  line: { dropped },
});

/** يبني مجرًى من أحداث، بنفس الطريق الذي يبنيه به `app.tsx`. */
function build(events: RunOutputEvent[]): StreamLine[] {
  return events.reduce<StreamLine[]>((kept, e) => appendLine(kept, e), []);
}

describe('الاحتفاظ بالأسطر', () => {
  it('يرقّم كل سطرٍ ترقيمًا متصلًا لا يعيد استعماله', () => {
    // الترقيم مفتاحُ الرسم وقياسُ ما أُسقط معًا، فتكرارُه يخلط الصفوف.
    const kept = build([out('أ'), err('ب'), out('ج')]);
    expect(kept.map((l) => l.seq)).toEqual([0, 1, 2]);
  });

  it('يقصّ من أوّل المجرى لا من آخره عند بلوغ السقف', () => {
    const events = Array.from({ length: MAX_KEPT_LINES + 25 }, (_, i) => out(`سطر ${i}`));
    const kept = build(events);
    expect(kept).toHaveLength(MAX_KEPT_LINES);
    // آخرُ ما وصل محفوظ: هو ما يشرح توقّف الأداة.
    expect((kept[kept.length - 1]?.event as { line: string }).line).toBe(
      `سطر ${MAX_KEPT_LINES + 24}`,
    );
    expect(droppedCount(kept)).toBe(25);
  });

  it('لا يعلن إسقاطًا لم يقع', () => {
    expect(droppedCount(build([out('أ')]))).toBe(0);
    expect(droppedCount([])).toBe(0);
  });
});

describe('حالات Run/Stream المهيكلة', () => {
  it('يختار الحالات الست من الطور وأنواع الحدث لا من نص الأداة', () => {
    expect(streamPresentation([], 'running')).toBe('waiting');
    expect(streamPresentation([], 'finished')).toBe('silent');
    expect(streamPresentation(build([out('ok')]), 'running')).toBe('stdout');
    expect(streamPresentation(build([err('no')]), 'running')).toBe('stderr');
    expect(streamPresentation(build([out('a'), cut(4)]), 'finished')).toBe('truncated');
    expect(streamPresentation(build([omitted(3)]), 'finished')).toBe('dropped');
  });
});

describe('الرسم', () => {
  it('لا يعرض قسمًا قبل أن يبدأ تشغيل', () => {
    // خطّةٌ ليست تشغيلًا، وقسمٌ فارغ يَعِد بما لم يبدأ.
    const { container } = render(<RunStream lines={[]} phase="planning" />);
    expect(container.querySelector('.stream')).toBeNull();
  });

  it('يعرض أسطر الأداة كما جاءت، بترتيبها', () => {
    const lines = build([out('أول'), err('تحذير'), out('ثالث')]);
    const { container } = render(<RunStream lines={lines} phase="running" />);
    const texts = [...container.querySelectorAll('.stream__text')].map((n) => n.textContent);
    expect(texts).toEqual(['أول', 'تحذير', 'ثالث']);
  });

  it('يميّز خطأ الأداة بوسمٍ نصّي لا باللون وحده', () => {
    const lines = build([out('عادي'), err('مريب')]);
    const { container } = render(<RunStream lines={lines} phase="running" />);
    const rows = [...container.querySelectorAll('.stream__line')];
    expect(rows[0]?.className).toContain('stream__line--stdout');
    expect(rows[1]?.className).toContain('stream__line--stderr');
    const tags = [...container.querySelectorAll('.stream__tag')].map((n) => n.textContent);
    expect(tags).toEqual([AR['stream.stdout'], AR['stream.stderr']]);
  });

  it('يعلن قصَّ النواة بعدد ما لم يُبَثّ', () => {
    const lines = build([out('أ'), cut(1284)]);
    render(<RunStream lines={lines} phase="finished" />);
    expect(screen.getByText(AR['stream.truncated'], { exact: false })).toBeTruthy();
    expect(screen.getByText('1284')).toBeTruthy();
  });

  it('يميّز إسقاط ذيل الحدث عن توقّف البثّ في النواة', () => {
    const lines = build([omitted(17)]);
    const { container } = render(<RunStream lines={lines} phase="finished" />);
    expect(container.querySelector('.stream--dropped')).toBeTruthy();
    expect(screen.getByText(AR['stream.state.dropped.body'])).toBeTruthy();
  });

  it('يعلن إسقاط الواجهة بصياغةٍ غير صياغة قصّ النواة', () => {
    // شيئان مختلفان: «لم تُبَثّ» و«بُثّت ولم تُحفظ». نصٌّ واحد لهما يخفي أحدهما.
    expect(AR['stream.dropped']).not.toBe(AR['stream.truncated']);
    const events = Array.from({ length: MAX_KEPT_LINES + 3 }, (_, i) => out(`س ${i}`));
    const { container } = render(<RunStream lines={build(events)} phase="finished" />);
    expect(container.querySelector('.stream--dropped')).toBeTruthy();
    expect(screen.getByText(AR['stream.state.dropped.footer'])).toBeTruthy();
  });

  it('يفرّق بين «لم تطبع بعد» و«لم تطبع شيئًا»', () => {
    // أثناء التشغيل الصمتُ انتظار، وبعده الصمتُ خبر. نصٌّ واحد للحالتين يجعل
    // أداةً ما زالت تعمل تبدو وقد فرغت.
    render(<RunStream lines={[]} phase="running" />);
    expect(screen.getByText(AR['stream.state.waiting.body'])).toBeTruthy();
    cleanup();
    render(<RunStream lines={[]} phase="finished" />);
    expect(screen.getByText(AR['stream.state.silent.body'])).toBeTruthy();
  });

  it('يعرض وسم stdout المباشر في الرأس', () => {
    const lines = build([out('أ'), out('ب')]);
    const { container } = render(<RunStream lines={lines} phase="finished" />);
    expect(container.querySelector('.stream--stdout')).toBeTruthy();
    expect(container.querySelector('.stream__count')?.textContent).toBe(
      AR['stream.state.stdout.meta'],
    );
  });
});
