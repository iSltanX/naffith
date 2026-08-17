// @vitest-environment node
/**
 * حارس هندسة سجل Page 18.
 *
 * لا يحمّل jsdom الأنماط، لذلك لا يستطيع اختبار المكوّن أن يميّز بطاقةً بطول
 * 132px من بطاقةٍ قديمة مرنة. نقرأ المصدر هنا ونحلّ الرموز إلى أرقامها كي يبقى
 * الاختبار مربوطًا بالمواصفة وبسلّم الهوية معًا.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const RUN_LOG = readFileSync(new URL('./run-log.css', import.meta.url).pathname, 'utf8');
const SETTINGS = readFileSync(new URL('./settings-screen.css', import.meta.url).pathname, 'utf8');
const TOKENS = readFileSync(
  new URL('./design-system/tokens.css', import.meta.url).pathname,
  'utf8',
);

function tokenPx(name: string): number {
  const value = new RegExp(`${name}:\\s*(\\d+)px`).exec(TOKENS)?.[1];
  expect(value, `missing pixel token ${name}`).toBeTruthy();
  return Number(value);
}

function rule(css: string, selector: string): string {
  const match = new RegExp(
    `${selector.replace(/[.*+?^${}()|[\\]\\]/g, '\\$&')}\\s*\\{([^}]*)\\}`,
  ).exec(css);
  expect(match?.[1], `missing rule ${selector}`).toBeTruthy();
  return match?.[1] ?? '';
}

describe('هندسة History/Entry 109:642', () => {
  it('تشتق 600×132 و600×264 من رموز الهوية', () => {
    expect(tokenPx('--width-base') - tokenPx('--space-64') - tokenPx('--space-56')).toBe(600);
    expect(tokenPx('--space-64') + tokenPx('--space-64') + tokenPx('--space-4')).toBe(132);
    expect(
      tokenPx('--space-96') +
        tokenPx('--space-96') +
        tokenPx('--space-64') +
        tokenPx('--space-8'),
    ).toBe(264);

    expect(rule(RUN_LOG, '.log__list')).toMatch(
      /max-inline-size:\s*calc\(var\(--width-base\) - var\(--space-64\) - var\(--space-56\)\)/,
    );
    expect(rule(RUN_LOG, '.entry')).toMatch(
      /block-size:\s*calc\(var\(--space-64\) \+ var\(--space-64\) \+ var\(--space-4\)\)/,
    );
    expect(rule(RUN_LOG, '.entry.is-expanded')).toMatch(
      /block-size:\s*calc\(var\(--space-96\) \+ var\(--space-96\) \+ var\(--space-64\) \+ var\(--space-8\)\)/,
    );
  });

  it('يستبدل الشريط الجانبي والظل بحدّ كامل ودلالة الحالة', () => {
    const entry = rule(RUN_LOG, '.entry');
    expect(entry).toMatch(/border:\s*var\(--border-hairline\) solid var\(--border-subtle\)/);
    expect(entry).not.toMatch(/box-shadow|border-inline-start/);
    expect(rule(RUN_LOG, '.entry--failed')).toMatch(
      /border-color:\s*var\(--status-danger-border\)/,
    );
    expect(rule(RUN_LOG, '.entry--cancelled')).toMatch(
      /border-color:\s*var\(--status-warning-border\)/,
    );
    expect(RUN_LOG).not.toMatch(/\.entry__(?:chip|delete|chevron)/);
    expect(rule(RUN_LOG, '.entry__action--danger')).toMatch(
      /color:\s*var\(--status-danger-text\)/,
    );
  });

  it('يبني حالتي الفراغ والسقف 380×220 وزر المسح 152×40', () => {
    expect(tokenPx('--width-narrow') - tokenPx('--space-96') - tokenPx('--space-4')).toBe(380);
    expect(tokenPx('--space-96') + tokenPx('--space-96') + tokenPx('--space-28')).toBe(220);
    expect(tokenPx('--space-64') + tokenPx('--space-64') + tokenPx('--space-24')).toBe(152);

    expect(RUN_LOG).toMatch(
      /\.log-empty,\s*\.log-cap,\s*\.log-unknown\s*\{[\s\S]*?inline-size:\s*min\(100%, calc\(var\(--width-narrow\) - var\(--space-96\) - var\(--space-4\)\)\);[\s\S]*?block-size:\s*calc\(var\(--space-96\) \+ var\(--space-96\) \+ var\(--space-28\)\);/,
    );
    expect(RUN_LOG).toMatch(
      /\n\.log-cap,\s*\.log-unknown\s*\{\s*border-color:\s*var\(--status-warning-border\);\s*\}/,
    );
    expect(rule(RUN_LOG, '.log__clear')).toMatch(
      /inline-size:\s*calc\(var\(--space-64\) \+ var\(--space-64\) \+ var\(--space-24\)\)/,
    );
    expect(rule(RUN_LOG, '.log__clear')).toMatch(
      /block-size:\s*var\(--control-h-md\)/,
    );
    expect(rule(RUN_LOG, '.log__clear')).toMatch(
      /background:\s*var\(--status-danger-tint\)/,
    );
  });
});

describe('هندسة إعدادات Page 18', () => {
  it('توسّط بطاقة 688×184 في امتداد الصفحة 776px بلا ظل', () => {
    expect(tokenPx('--width-base') - tokenPx('--space-32')).toBe(688);
    expect(tokenPx('--space-96') + tokenPx('--space-96') - tokenPx('--space-8')).toBe(184);
    expect(tokenPx('--width-base') + tokenPx('--space-56')).toBe(776);

    expect(rule(SETTINGS, '.settings-screen')).toMatch(
      /max-inline-size:\s*calc\(var\(--width-base\) \+ var\(--space-56\)\)/,
    );
    expect(rule(SETTINGS, '.settings-screen')).toMatch(/margin-inline:\s*auto/);

    const card = rule(SETTINGS, '.settings-screen__card');
    expect(card).toMatch(
      /max-inline-size:\s*calc\(var\(--width-base\) - var\(--space-32\)\)/,
    );
    expect(card).toMatch(
      /min-block-size:\s*calc\(var\(--space-96\) \+ var\(--space-96\) - var\(--space-8\)\)/,
    );
    expect(card).toMatch(/margin-inline:\s*auto/);
    expect(card).not.toMatch(/box-shadow/);
  });

  it('يحافظ على صياغة يدعمها Safari 15', () => {
    for (const css of [RUN_LOG, SETTINGS]) {
      expect(css).not.toMatch(/:has\(|color-mix\(|\b(?:cqw|cqi|dvw|svw|lvw)\b/);
      expect(css).not.toMatch(/calc\([^)]*[*/][^)]*\)/);
    }
  });
});
