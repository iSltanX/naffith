# أصول المستودع — GitHub & README assets

خلافًا لبقية `docs/` — وهي نسخة للقراءة فقط من مشروع الهوية — هذا المجلد
**مُولَّد من داخل المستودع**. مصدره صفحة «‏21 — GitHub & Repository Assets»
في ملف الهوية، لكن الرسم يُعاد بناؤه هنا من خطوط المشروع نفسها.

Unlike the rest of `docs/`, this folder is **generated in-repo**. It mirrors page
*21 — GitHub & Repository Assets* of the identity file, but the artwork is rebuilt
locally so the README never depends on a Figma export round-trip.

## الملفات — files

| File | Size | Used by |
|---|---|---|
| `readme-hero.png` | 1280×640 | the README banner |
| `repo-header.png` | 1280×320 | a slimmer banner variant |
| `social-preview.png` | 1280×640 | GitHub → Settings → Social preview |

The macOS app icon is **not** duplicated here — it lives at
[`src-tauri/icons/`](../../../src-tauri/icons/), which is the only approved source.

## إعادة البناء — rebuilding

```bash
./docs/brand/github/_render/build.sh
```

Renders `_render/*.html` with headless Chromium and crops each shot to the exact
frame size. No network access is used: type comes from
`src/design-system/fonts/` (the same self-hosted Cairo / Almarai / JetBrains Mono
the application ships), and the brand mark is inline SVG whose geometry matches
`src/design-system/logo.svg` exactly.

`_render/pngtool.py` is a stdlib-only PNG crop/probe helper — Chromium clips the
page when the window is sized to the exact artboard, so the build renders into a
larger window and crops back down.

## قواعد ملزمة — invariants

- **علامتان لا واحدة.** الرسم (سـ/نـ) علامةُ الهوية ويظهر في الشعارات المركّبة؛
  و`>_` أيقونةُ تطبيق macOS وحدها. لا يحلّ أحدهما محلّ الآخر.
  The سـ/نـ rasm is the brand mark and appears in composed lockups; `>_` is the
  macOS app icon and nothing else. They are never interchanged.
- **البرتقالي للنقطة وحدها.** `#E85D2C` هو الإعجام — العنصر الدافئ الوحيد.
- Version and platform strings must match `package.json` and
  `src-tauri/tauri.conf.json`. They are currently `0.2.0` and `macOS 12+`.
