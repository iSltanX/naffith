// @vitest-environment node
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const RESULT = readFileSync(new URL("./result-view.css", import.meta.url).pathname, "utf8");
const OPERATION = readFileSync(
  new URL("./operation-layout.css", import.meta.url).pathname,
  "utf8",
);
const TOKENS = readFileSync(
  new URL("./design-system/tokens.css", import.meta.url).pathname,
  "utf8",
);

function tokenPx(name: string): number {
  const value = new RegExp(`${name}:\\s*(\\d+)px`).exec(TOKENS)?.[1];
  expect(value, `missing pixel token ${name}`).toBeTruthy();
  return Number(value);
}

describe("Page 19 ResultView measure", () => {
  it("derives the 560px and 608px caps from the shared token scale", () => {
    const narrow = tokenPx("--width-narrow");
    const space80 = tokenPx("--space-80");
    const space64 = tokenPx("--space-64");

    expect(narrow + space80).toBe(560);
    expect(narrow + space64 + space64).toBe(608);
    expect(RESULT).toMatch(
      /--result-column-cap-compact:\s*calc\(var\(--width-narrow\) \+ var\(--space-80\)\)/,
    );
    expect(RESULT).toMatch(
      /--result-column-cap-wide:\s*calc\([\s\S]*?var\(--width-narrow\) \+ var\(--space-64\) \+ var\(--space-64\)[\s\S]*?\)/,
    );
  });

  it("uses 560px by default and promotes the cap only at 1080px", () => {
    expect(RESULT).toMatch(
      /\.result-workspace\s*\{[\s\S]*?--result-column-cap:\s*var\(--result-column-cap-compact\)[\s\S]*?inline-size:\s*100%[\s\S]*?max-inline-size:\s*var\(--result-column-cap\)[\s\S]*?margin-inline:\s*auto/,
    );
    expect(RESULT).toMatch(
      /@media \(min-width:\s*1080px\)\s*\{\s*\.result-workspace\s*\{\s*--result-column-cap:\s*var\(--result-column-cap-wide\);/,
    );
    expect(RESULT).not.toContain("@media (min-width: 1079px)");
  });

  it("fits the native minimum without changing the 899/900 pane boundary", () => {
    const minimumWindow = 680;
    const compactRail = tokenPx("--sidebar-w-collapsed");
    const bodyInsets = tokenPx("--space-20") * 2;
    const compactCap = tokenPx("--width-narrow") + tokenPx("--space-80");

    expect(compactCap).toBeLessThanOrEqual(minimumWindow - compactRail - bodyInsets);
    expect(OPERATION).toMatch(
      /@media \(max-width:\s*899px\)[\s\S]*?\.op__panes\s*\{[^}]*flex-direction:\s*column/,
    );
    expect(OPERATION).not.toContain("@media (max-width: 900px)");
  });

  it("uses Safari 15-compatible sizing syntax", () => {
    expect(RESULT).not.toMatch(/\b(?:cqw|cqi|dvw|svw|lvw)\b/);
    expect(RESULT).not.toMatch(/calc\([^)]*[*/][^)]*\)/);
    expect(RESULT).not.toMatch(/:has\(/);
  });

  it("keeps the approved 128/280/152/72 layer rhythm", () => {
    expect(RESULT).toMatch(/\.result-status\s*\{[^}]*min-block-size:\s*128px/s);
    expect(RESULT).toMatch(
      /\.result-content\s*\{[^}]*min-block-size:\s*calc\(var\(--space-64\) \+ var\(--space-64\) \+ var\(--space-64\) \+ var\(--space-64\) \+ var\(--space-24\)\)/s,
    );
    expect(RESULT).toMatch(
      /\.result-actions\s*\{[^}]*min-block-size:\s*calc\(var\(--space-64\) \+ var\(--space-64\) \+ var\(--space-24\)\)/s,
    );
    expect(RESULT).toMatch(
      /\.result-technical > summary\s*\{[^}]*min-block-size:\s*72px/s,
    );
  });

  it("pins the borderless template actions to opposite RTL edges", () => {
    expect(RESULT).toMatch(
      /\.result-template__actions\s*\{[^}]*justify-content:\s*space-between[^}]*direction:\s*rtl/s,
    );
    expect(RESULT).toMatch(
      /\.result-template__action\s*\{[^}]*border:\s*0[^}]*background:\s*transparent[^}]*color:\s*var\(--text-secondary\)/s,
    );
    expect(RESULT).toMatch(
      /\.result-template__action--accent\s*\{\s*color:\s*var\(--text-accent\)/,
    );
  });
});
