<div align="center">

<img src="docs/brand/github/readme-hero.png" alt="نَفِّذ — Naffith · Execute with precision. Arabic-first macOS utility." width="100%">

[![version](https://img.shields.io/badge/version-v0.2.0-d0470d?style=flat-square&labelColor=272831)](https://github.com/iSltanX/naffith)
[![platform](https://img.shields.io/badge/platform-macOS%2012%2B-4f505e?style=flat-square&labelColor=272831)](https://github.com/iSltanX/naffith)
[![framework](https://img.shields.io/badge/framework-Tauri%202.0-4f505e?style=flat-square&labelColor=272831)](https://github.com/iSltanX/naffith)
[![backend](https://img.shields.io/badge/backend-Rust-4f505e?style=flat-square&labelColor=272831)](https://github.com/iSltanX/naffith)
[![frontend](https://img.shields.io/badge/frontend-React%2018-4f505e?style=flat-square&labelColor=272831)](https://github.com/iSltanX/naffith)

</div>

---

## Overview / نظرة عامة

**نَفِّذ** (*Naffith* — "execute") is an Arabic-first macOS utility that turns everyday
system work into reviewable commands. It shows you the **command**, not a black box:
every operation is planned, displayed, and only then run.

> أداة تنفيذ متقدمة لنظام ماك — واجهة عربية أولًا، تعرض الأمر قبل تنفيذه.

The interface is right-to-left Arabic by default with a full English surface alongside
it. The core is Rust; the shell is React inside Tauri 2.

---

## Features / الميزات

| | English / العربية | |
|---|---|---|
| **Operations** | العمليات | Execute system commands with structured parameters and real-time output streaming. |
| **Result View** | ResultContract | Rich result rendering with formatted output, data tables and status indicators. |
| **RTL Arabic** | واجهة عربية | Full right-to-left interface with self-hosted bilingual Cairo / Almarai typography. |
| **Security** | الأمان | Sandboxed execution with restrictive permission controls and an audited run journal. |
| **Developer Tools** | أدوات المطورين | Built-in debugging, process inspection and development utilities. |
| **Git Operations** | عمليات Git | Repository management, branching, atomic commits and diff review. |
| **System Utilities** | أدوات النظام | Disk layout, network ports and host process diagnosis. |
| **Appearance** | المظهر | Dark and light modes with system-adaptive theme switching. |

---

## Operations / العمليات

**48 operations across 10 categories.** The count is read from the catalogue, never
hand-maintained — `cargo run --example dump_catalogue` prints the authoritative list.

| Category | القسم | Ops |
|---|---|---|
| `files` | الملفات | 8 |
| `compress` | الضغط | 7 |
| `git` | Git | 6 |
| `text` | النصوص | 5 |
| `network` | الشبكة | 5 |
| `system` | النظام | 5 |
| `images` | الصور | 4 |
| `disk` | القرص | 4 |
| `security` | الأمان | 4 |
| `history` | السجل | sourced from the run log |

Remaining gaps — batch plans, multi-path inputs and tool-output parsing — are named
and costed in [`docs/roadmap.md`](docs/roadmap.md).

---

## Installation / التثبيت

**Requirements:** macOS 12 or later (Apple Silicon & Intel), Node.js, and a Rust toolchain.

```bash
git clone https://github.com/iSltanX/naffith.git
cd naffith
npm install
```

Build a signed-less local `.dmg` / `.app` bundle:

```bash
npm run build
```

> The macOS icon set is assembled, not derived — never run `tauri icon`.
> See [`src-tauri/icons/README.md`](src-tauri/icons/README.md) for why.

---

## Usage / الاستخدام

```bash
npm run dev          # run the app (Tauri + Vite)
npm run dev:web      # browser-only shell, no Rust core
```

Pick a category, choose an operation, fill its typed inputs — نَفِّذ builds the command,
shows it to you in full, and runs it only when you confirm.

---

## Security / الأمان

- Every run is a single, fully-formed `PlannedCommand` — no shell string interpolation.
- Inputs are typed (`InputKind`) and validated before a command is ever constructed.
- Preconditions are fingerprinted at plan time; outputs are promoted atomically.
- Each run is recorded in the journal so it can be audited after the fact.

---

## Result View / ResultView

Results travel over a typed `ResultContract` rather than raw text, so the UI renders
structured sections — summaries, tables, diffs and status — instead of parsing stdout
in the view layer.

---

## Settings / الإعدادات

Theme (dark / light / system), interface language, Node.js and Cargo tool paths,
notification sound and updater state.

---

## Developer Tools / أدوات المطورين

```bash
npm run typecheck    # tsc, app + tests
npm run lint         # eslint, zero warnings
npm run test:ui      # vitest
npm run test:core    # cargo test
npm run lint:core    # cargo clippy, warnings denied
npm run fmt:core     # cargo fmt --check
```

---

## Screenshots / لقطات الشاشة

Application screenshots are not committed yet. The identity file carries five
presentation frames — single, side-by-side, theme comparison, feature focus and
premium terminal — on page **21 — GitHub & Repository Assets**, so captures can be
dropped straight into a consistent frame.

---

## Contributing / المساهمة

`docs/` is a **read-only** mirror of the identity project — nothing there is edited by
hand. Brand corrections belong upstream and are then re-copied.

The one exception is [`docs/brand/github/`](docs/brand/github/), which is generated:

```bash
./docs/brand/github/_render/build.sh
```

That rebuilds the header, hero and social-preview PNGs from the checked-in HTML
sources using the repository's own self-hosted fonts — no network, no Figma round-trip.

Before opening a pull request, run the full gate: `typecheck`, `lint`, `test:ui`,
`test:core`, `lint:core`, `fmt:core`.

---

## License / الترخيص

**No licence file has been committed yet.** Until `LICENSE` exists, all rights are
reserved by default and the badge is intentionally omitted. Add the licence you intend
and the badge can go back.

---

<div align="center">

<img src="src-tauri/icons/128x128.png" alt="" width="56" height="56">

**نَفِّذ — NAFFITH**

تطوير وتصميم سلطان

`github.com/iSltanX/naffith` · `v0.2.0`

</div>
