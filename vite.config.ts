import { defineConfig, type Plugin } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'node:url';
import { createRequire } from 'node:module';
import { CRITICAL_FONT_FILES } from './src/critical-fonts';

/**
 * رقم الإصدار المعروض في «حول نَفِّذ».
 *
 * يُقرأ من `package.json` وقت البناء ويُحقن ثابتًا، ولا يُسأل عنه أمر IPC
 * عاشر: سطح الأوامر تسعة وقد أُثبت ذلك باختبارٍ في `security.rs`، وإضافة أمرٍ
 * كي يقول التطبيق رقمه كانت ستوسّع سطح الهجوم من أجل نصٍّ يعرفه البناء أصلًا.
 */
const APP_VERSION = (createRequire(import.meta.url)('./package.json') as { version: string })
  .version;

/**
 * هل ضُبطت وجهة التحديث ومفتاح التوقيع في هذا البناء؟
 *
 * يُقرأ من `tauri.conf.json` وقت البناء لا يُستنتج من نصّ خطأ وقت التشغيل.
 * كان الاستنتاج بمطابقة رسالة النواة (`…does not have any endpoints set`)،
 * وهي رسالةٌ إنجليزية من مكتبةٍ خارجية قد تتغيّر صياغتها في أي ترقية فتنقلب
 * «غير مهيأة» إلى «تعذّر التحقق» بلا أن يتغيّر شيء عندنا. والبناء يعرف
 * الجواب يقينًا، فيُسأل هو.
 *
 * والفائدة الثانية أصدق: بناءٌ بلا وجهة لا يسأل الشبكة أصلًا، فلا انتظار ولا
 * فشلٌ يُعرض ثم يُفسَّر.
 */
const UPDATER = (
  createRequire(import.meta.url)('./src-tauri/tauri.conf.json') as {
    plugins?: { updater?: { endpoints?: string[]; pubkey?: string } };
  }
).plugins?.updater;
const UPDATER_CONFIGURED =
  (UPDATER?.endpoints?.length ?? 0) > 0 && (UPDATER?.pubkey ?? '').trim() !== '';

// نظام التصميم **منسوخ داخل المشروع** في `src/design-system`.
//
// كان يُقرأ من مشروع الهوية الشقيق عبر مسار نسبي خارج الجذر. صار البناء لا
// يخرج من جذر المشروع أصلًا: لا `fs.allow: ['..']`، ولا اعتماد على وجود مجلد
// الهوية بجوارنا، ولا احتمال أن يغيّر تحديثٌ في الهوية ناتجَ بناء المنتج بلا
// قرار. مشروع الهوية يبقى مرجعًا يُنسخ منه عن قصد، لا تبعيةَ بناء حيّة.
const DESIGN_SYSTEM = fileURLToPath(new URL('./src/design-system', import.meta.url));

/**
 * حقنُ `<link rel="preload">` للوجوه الحرجة في رأس المستند.
 *
 * لماذا إضافة بناء لا سطورٌ مكتوبة في `index.html`: البناء يبصم كل ملفّ خطٍّ
 * بمحتواه (`cairo-000-BW4_w4rE.woff2`)، فأي عنوانٍ يُكتب باليد يبطل عند أول
 * تحديثٍ للخطوط ويصير تحميلًا مسبقًا لملفٍّ غير موجود — أسوأ من غيابه، لأنه
 * يفشل صامتًا. الإضافة تقرأ الأسماء المُخرَجة من الحزمة نفسها، فلا تكذب أبدًا.
 *
 * ولماذا استبدالُ علامةٍ في `index.html` لا `injectTo: 'head-prepend'`: الحقن
 * في مقدّمة `<head>` يدفع `<meta charset>` إلى ما بعد ستّة وسوم، والمواصفة
 * تشترط بلوغه في أول 1024 بايت. العلامة تضع الوسوم حيث يجب: بعد `charset`
 * مباشرةً وقبل `<link>` الأنماط — فيبدأ طلب الخطّ قبل تحليل CSS لا بعده.
 *
 * وغياب الملف أو غياب العلامة يُسقط البناء عمدًا: مشروع الهوية يُنسخ منه
 * يدويًا، وتغيّرُ أسماء المقاطع فيه يجب أن يُكتشف هنا لا في أول تشغيلٍ عند
 * المستخدم — تحميلٌ مسبق لعنوانٍ خاطئ يفشل صامتًا ولا أثر له إلا القفز عائدًا.
 */
const PRELOAD_SLOT = '<!--@preload-critical-fonts-->';

function preloadCriticalFonts(): Plugin {
  return {
    name: 'naffith:preload-critical-fonts',
    transformIndexHtml: {
      order: 'post',
      handler(html, ctx) {
        if (!html.includes(PRELOAD_SLOT)) {
          throw new Error(`preload-critical-fonts: ‏${PRELOAD_SLOT} ليست في index.html`);
        }
        const links = CRITICAL_FONT_FILES.map((file) => {
          // في التطوير لا حزمة ولا بصمة: المسار المصدري هو ما يخدمه الخادم،
          // وهو نفسه ما تشير إليه `fonts.css`، فيتشاركان المخبأ.
          let href = `/src/design-system/fonts/${file}`;
          if (ctx.bundle) {
            const stem = file.replace(/\.woff2$/, '');
            const emitted = Object.keys(ctx.bundle).find((name) =>
              new RegExp(`(?:^|/)${stem}-[^/]*\\.woff2$`).test(name),
            );
            if (emitted === undefined) {
              throw new Error(`preload-critical-fonts: ‏${file} ليس في ناتج البناء`);
            }
            href = `/${emitted}`;
          }
          // ‏`crossorigin` مطلوبة حتى لخطٍّ من الأصل نفسه: الخطوط تُطلب دائمًا في
          // وضع CORS مجهّلًا، وبدونها يصير الطلب طلبين لا طلبًا.
          return `<link rel="preload" as="font" type="font/woff2" href="${href}" crossorigin>`;
        });
        return html.replace(PRELOAD_SLOT, links.join('\n  '));
      },
    },
  };
}

export default defineConfig({
  plugins: [react(), preloadCriticalFonts()],
  clearScreen: false,
  define: {
    __APP_VERSION__: JSON.stringify(APP_VERSION),
    __UPDATER_CONFIGURED__: JSON.stringify(UPDATER_CONFIGURED),
  },
  resolve: { alias: { '@ds': DESIGN_SYSTEM } },
  server: {
    port: 5173,
    strictPort: true,
    // لا قراءة من خارج جذر المشروع: كل ما يحتاجه البناء منسوخ داخله.
    fs: { strict: true },
  },
  // البيئة الافتراضية `node`، ويطلب اختبارُ المكوّنات jsdom بسطرٍ في رأسه:
  //
  //     // @vitest-environment jsdom
  //
  // جعلُ jsdom افتراضًا عامًّا يكسر `i18n.test.ts`: هو يقرأ مصدر Rust عبر
  // `import.meta.url`، وjsdom يعيد كتابة الأصل إلى عنوان http فيصير المسار
  // ‏`/src-tauri/src` من جذر القرص. الاختباران يحتاجان بيئتين، فلتكن لكلٍّ بيئته.
  test: {
    environment: 'node',
    globals: false,
    include: ['src/**/*.test.{ts,tsx}'],
    restoreMocks: true,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'safari15',       // WKWebView على macOS 12+
    assetsInlineLimit: 0,     // الخطوط تبقى ملفات
    sourcemap: false,
  },
});
