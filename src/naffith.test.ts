/**
 * ما تعرضه المعاينة قبل التنفيذ.
 *
 * الرقم التقديري تحديدًا: عرضُه بصيغة خاطئة يجعل المستخدم يقارنه بمساحة قرصه
 * فيخطئ بمعامل ١٠٢٤ أو بمنزلة عشرية.
 */
import { describe, expect, it } from 'vitest';
import { readableSize } from './naffith';

describe('عرض الحجم التقديري', () => {
  it('يستعمل القاعدة العشرية كما يعرض النظام مساحة القرص', () => {
    // ١٠٠٠ ليست ١٠٢٤: الرقم يُقارَن بما يقوله Finder عن القرص.
    expect(readableSize(1_000)).toBe('1.0 ك.ب');
    expect(readableSize(1_000_000)).toBe('1.0 م.ب');
    expect(readableSize(2_500_000_000)).toBe('2.5 ج.ب');
  });

  it('يعرض البايتات كما هي تحت الكيلوبايت', () => {
    expect(readableSize(0)).toBe('0 بايت');
    expect(readableSize(999)).toBe('999 بايت');
  });

  it('يُسقط المنزلة العشرية فوق العشرة فلا يعرض دقّة لا يملكها', () => {
    expect(readableSize(124_700_000)).toBe('125 م.ب');
    expect(readableSize(9_400_000)).toBe('9.4 م.ب');
  });
});
