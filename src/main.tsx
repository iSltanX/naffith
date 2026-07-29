import { createRoot } from 'react-dom/client';

// نظام التصميم يُستورد من مشروع الهوية قراءةً فقط. Vite يجمعه في dist الخاص
// بالمنتج، ولا يكتب شيئًا داخل مجلد الهوية.
import '@ds/fonts.css';
import '@ds/tokens.css';
import '@ds/base.css';
import './app.css';

// الرموز تُحقن كتعريفات داخل المستند: الإشارة عبر ملف خارجي لا تعمل في
// WebKit/Chromium وتفشل صامتة.
import iconSprite from '@ds/icons.svg?raw';
import logoSprite from '@ds/logo.svg?raw';

import App from './app';

const defs = document.createElement('div');
defs.style.display = 'none';
defs.setAttribute('aria-hidden', 'true');
defs.innerHTML = iconSprite + logoSprite;
document.body.prepend(defs);

const root = document.getElementById('root');
if (!root) throw new Error('#root missing');
createRoot(root).render(<App />);
