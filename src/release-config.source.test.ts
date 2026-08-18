// @vitest-environment node
/**
 * H-8 (تدقيق الفرع): مُلحق التحديث مفعَّلٌ في `tauri.conf.json`
 * (`plugins.updater`)، لكن `tauri build` لم يكن يكتب أرشيف التحديث
 * (`.tar.gz`) ولا توقيعه (`.sig`) — الحقل `bundle.createUpdaterArtifacts`
 * كان غائبًا. فحين تُملأ `endpoints` و`pubkey` يومًا (انظر التعليق المرافق
 * لهما في هذا الملف)، لن يجد بيان التحديث شيئًا يشير إليه: المُلحق مفعَّلٌ
 * من طرف التطبيق، والبناء لا يُنتج ما يحتاجه ذلك الطرف.
 *
 * وبيئته `node`: هذا فحصُ إعدادٍ لا مكوّنًا، والمصدر هو المرجع الوحيد.
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

const CONF_PATH = new URL('../src-tauri/tauri.conf.json', import.meta.url).pathname;
const conf = JSON.parse(readFileSync(CONF_PATH, 'utf8')) as {
  bundle: { createUpdaterArtifacts?: boolean | string; targets: string[] };
  plugins?: { updater?: { endpoints?: string[]; pubkey?: string } };
};

describe('أرشيفات التحديث تُبنى فعلًا حين يُفعَّل المُلحق', () => {
  it('‏`bundle.createUpdaterArtifacts` مفعَّلة', () => {
    expect(conf.bundle.createUpdaterArtifacts).toBe(true);
  });

  /**
   * حارسٌ على الحارس: لو حُذف مُلحق `updater` من `plugins` يومًا،
   * `createUpdaterArtifacts` تصير تفعيلًا بلا فائدة — لا خطأ، لكن الاختبار
   * الأول أعلاه سيبقى يمرّ صامتًا وهو لم يعد يحرس شيئًا. هذا يثبّت أن
   * المُلحق ما زال معلَنًا فعلًا، فيبقى الحارس الأول ذا معنى.
   */
  it('ومُلحق التحديث الذي تخدمه هذه الأرشيفات ما زال معلَنًا', () => {
    expect(conf.plugins?.updater).toBeDefined();
  });
});
