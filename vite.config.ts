import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'node:url';

// نظام التصميم **منسوخ داخل المشروع** في `src/design-system`.
//
// كان يُقرأ من مشروع الهوية الشقيق عبر مسار نسبي خارج الجذر. صار البناء لا
// يخرج من جذر المشروع أصلًا: لا `fs.allow: ['..']`، ولا اعتماد على وجود مجلد
// الهوية بجوارنا، ولا احتمال أن يغيّر تحديثٌ في الهوية ناتجَ بناء المنتج بلا
// قرار. مشروع الهوية يبقى مرجعًا يُنسخ منه عن قصد، لا تبعيةَ بناء حيّة.
const DESIGN_SYSTEM = fileURLToPath(new URL('./src/design-system', import.meta.url));

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  resolve: { alias: { '@ds': DESIGN_SYSTEM } },
  server: {
    port: 5173,
    strictPort: true,
    // لا قراءة من خارج جذر المشروع: كل ما يحتاجه البناء منسوخ داخله.
    fs: { strict: true },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'safari15',       // WKWebView على macOS 12+
    assetsInlineLimit: 0,     // الخطوط تبقى ملفات
    sourcemap: false,
  },
});
