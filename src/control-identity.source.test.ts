// @vitest-environment node
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const BASE = readFileSync(
  new URL('./design-system/base.css', import.meta.url).pathname,
  'utf8',
);
const CONTROLS = readFileSync(new URL('./controls.css', import.meta.url).pathname, 'utf8');
const OPERATION = readFileSync(
  new URL('./operation-screen.css', import.meta.url).pathname,
  'utf8',
);
const LOG = readFileSync(new URL('./run-log.css', import.meta.url).pathname, 'utf8');
const RESULT = readFileSync(new URL('./result-view.css', import.meta.url).pathname, 'utf8');
const SETTINGS = readFileSync(
  new URL('./settings-screen.css', import.meta.url).pathname,
  'utf8',
);

describe('production control identity', () => {
  it('select, text, URL, path, and number controls share one radius token', () => {
    expect(BASE).toMatch(/\.field\s*\{[^}]*border-radius:\s*var\(--radius-control\)/s);
    expect(CONTROLS).toMatch(
      /\.select__button\s*\{[^}]*border-radius:\s*var\(--radius-control\)/s,
    );
    expect(OPERATION).toMatch(
      /\.number-field\s*\{[^}]*block-size:\s*var\(--control-h-search\);[^}]*border-radius:\s*var\(--radius-control\)/s,
    );
    // Page 18 uses a dedicated Filter/Chip family rather than search/select.
    expect(LOG).toMatch(
      /\.filter-chip\s*\{[^}]*border-radius:\s*var\(--radius-chip\)/s,
    );
  });

  it('technical number and URL values use the technical face', () => {
    expect(OPERATION).toMatch(
      /\.number-field__input\s*\{[^}]*font-family:\s*var\(--font-mono\)/s,
    );
    expect(OPERATION).toMatch(
      /\.field__technical-value\s*\{[^}]*font-family:\s*var\(--font-mono\)/s,
    );
  });

  it('utility icons start neutral and become Ember on interaction', () => {
    expect(BASE).toMatch(
      /\.btn--quiet svg,[\s\S]*?color:\s*var\(--text-muted\)/,
    );
    expect(BASE).toMatch(
      /\.btn--quiet:hover:not\(:disabled\) svg,[\s\S]*?color:\s*var\(--text-accent\)/,
    );
    expect(CONTROLS).toMatch(
      /\.field__icon\s*\{[^}]*color:\s*var\(--text-muted\)/s,
    );
    expect(CONTROLS).toMatch(
      /\.field:not\(\.field--disabled\):hover \.field__icon,[\s\S]*?color:\s*var\(--text-accent\)/,
    );
    expect(CONTROLS).toMatch(
      /\.select__caret\s*\{[^}]*color:\s*var\(--text-muted\)/s,
    );
    expect(CONTROLS).toMatch(
      /\.select__button:not\(:disabled\):hover \.select__caret,[\s\S]*?color:\s*var\(--text-accent\)/,
    );
  });

  // Page 09 ranks Outline a production `Control/Button` Type beside Primary,
  // Neutral, and Destructive — so it belongs to the shared layer, not to the
  // one screen that happens to use it.
  it('Outline is a design-system button type, not a screen-local override', () => {
    expect(BASE).toMatch(
      /\.btn--outline\s*\{[^}]*background:\s*var\(--bg-surface\);[^}]*color:\s*var\(--text-primary\);[^}]*border-color:\s*var\(--border-default\)/s,
    );
    expect(BASE).toMatch(/\.btn--outline:hover:not\(:disabled\)\s*\{[^}]*var\(--bg-hover\)/s);
    expect(BASE).toMatch(/\.btn--outline:active:not\(:disabled\)\s*\{[^}]*var\(--bg-active\)/s);
    // A screen may consume the type; it may not re-declare its material.
    expect(RESULT).not.toMatch(/\.btn--outline\s*\{/);
  });

  // 14.3 `Search/Field · State=NoResults`: the border alone carries the warning
  // tone. The query is valid, so the field is never an error and never disabled.
  it('an empty search result tones the border without claiming an error', () => {
    expect(CONTROLS).toMatch(
      /\.field--no-results:not\(:focus-within\)\s*\{[^}]*border-color:\s*var\(--status-warning-border\)/s,
    );
    expect(CONTROLS).not.toMatch(/\.field--no-results[^{]*\{[^}]*--status-danger/s);
  });

  it('keeps the interaction layer compatible with the declared macOS 12 floor', () => {
    const productionCss = [BASE, CONTROLS, OPERATION, LOG].join('\n');
    expect(productionCss).not.toMatch(/:has\(/);
    expect(productionCss).not.toMatch(/color-mix\(/);
    expect(BASE).toMatch(/:focus\s*\{[^}]*outline:\s*var\(--focus-ring-width\)/s);
  });

  /**
   * Every `.t-page-title` in production is a `tabIndex={-1}` mount-time
   * `.focus()` target (library, category, run-log, settings) — never a Tab
   * stop. Regression: it inherited the base `:focus`/`:focus-visible` ring
   * meant for real controls, so an orange outline appeared around the
   * heading whenever a screen mounted and vanished the moment focus moved
   * on. `.page:focus` right above it in app-shell.css already sets the
   * precedent for suppressing the ring on a programmatic-only focus target.
   */
  it('suppresses the focus ring on the programmatic-only page title', () => {
    expect(BASE).toMatch(/\.t-page-title:focus\s*\{[^}]*outline:\s*none/s);
  });

  // Settings' own interactive controls (section rail, toggles, the icon-size
  // select) must keep relying on the shared `:focus`/`:focus-visible` ring —
  // none of them may quietly gain an `outline: none` of their own, which
  // (unlike `.select__button`'s or `.number-field__input`'s deliberate
  // outline→box-shadow / outline→wrapper-ring swaps elsewhere) would remove
  // keyboard focus visibility with no replacement ring at all.
  it('keeps focus visibility on every Settings control', () => {
    expect(SETTINGS).not.toMatch(/outline:\s*none/);
    expect(SETTINGS).not.toMatch(/outline:\s*0\b/);
  });
});
