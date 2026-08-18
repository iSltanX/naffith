/**
 * الثوابت التي يحقنها البناء.
 *
 * `__APP_VERSION__` يُعرَّف في `vite.config.ts` من `package.json`. تعريفه هنا
 * يجعل TypeScript يعرفه، ويجعل خطأً مطبعيًا في اسمه خطأ ترجمة لا `undefined`
 * يظهر نصًّا في شاشة «حول».
 */
declare const __APP_VERSION__: string;

/**
 * هل ضُبطت وجهة التحديث ومفتاح التوقيع؟ يُحسب في `vite.config.ts` من
 * `tauri.conf.json`. `false` تعني أن شاشة «حول» تعرض «غير مهيأة بعد» بلا أن
 * تسأل الشبكة.
 */
declare const __UPDATER_CONFIGURED__: boolean;
