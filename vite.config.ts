import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'node:url';

// نظام التصميم يعيش في مشروع الهوية، ويُستورد **قراءةً فقط**.
// Vite يقرأ منه ويُخرج إلى naffith/dist — لا شيء يُكتب داخل مجلد الهوية.
const DESIGN_SYSTEM = fileURLToPath(
  new URL('../naffith-satr-brand-v2/design-system', import.meta.url),
);

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  resolve: { alias: { '@ds': DESIGN_SYSTEM } },
  server: {
    port: 5173,
    strictPort: true,
    // السماح بالقراءة من خارج جذر المشروع مطلوب لأن نظام التصميم شقيقٌ له.
    fs: { allow: ['..'] },
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'safari15',       // WKWebView على macOS 12+
    assetsInlineLimit: 0,     // الخطوط تبقى ملفات
    sourcemap: false,
  },
});
