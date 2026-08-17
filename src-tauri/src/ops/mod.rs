//! فهرس العمليات.
//!
//! كل عملية ملفٌّ واحد يعرّف `SPEC`. لا JSON ولا قوالب نصية: خطأ مطبعي في
//! قالب يصبح خطأً وقت التشغيل على ملفات شخص حقيقي، وهنا يصبح خطأ ترجمة.
//! الثمن أن إضافة عملية تحتاج بناءً جديدًا — وهو الثمن الصحيح لأداة تشغّل
//! أوامر على جهاز المستخدم.
//!
//! والبناء المشترك في `common.rs`: `Argv` تبني الأمر وشرحه معًا، فلا يمكن
//! لملفٍّ هنا أن يعرض غير ما ينفّذ.

pub mod common;

pub mod compress_ditto;
pub mod internal_echo;

// ── أدوات المطوّرين ─────────────────────────────────────────────────────
pub mod dev_common;
pub mod dev_npm_dev;
pub mod dev_npm_install;
pub mod dev_npm_lint;
pub mod dev_npm_test;
pub mod dev_npm_typecheck;
pub mod dev_tauri_build;
pub mod dev_tauri_dev;

// ── الملفات والمجلدات ───────────────────────────────────────────────────
pub mod files_copy;
pub mod files_find_large;
pub mod files_find_name;
pub mod files_find_stale;
pub mod files_mkdir;
pub mod files_move;
pub mod files_open;
pub mod files_tree_size;

// ── الضغط وفكّ الضغط ────────────────────────────────────────────────────
pub mod compress_tar_create;
pub mod compress_tar_extract;
pub mod compress_tar_list;
pub mod compress_zip_extract;
pub mod compress_zip_list;
pub mod compress_zip_test;

// ── الصور ───────────────────────────────────────────────────────────────
pub mod image_convert;
pub mod image_info;
pub mod image_resize;
pub mod image_rotate;

// ── النصوص والمستندات ───────────────────────────────────────────────────
pub mod text_diff;
pub mod text_encoding;
pub mod text_merge;
pub mod text_search;
pub mod text_split;

// ── الأقراص ومساحة التخزين ──────────────────────────────────────────────
pub mod disk_compare;
pub mod disk_free;
pub mod disk_hash;
pub mod disk_list;

// ── الشبكة والاتصال ─────────────────────────────────────────────────────
pub mod net_dns;
pub mod net_download;
pub mod net_headers;
pub mod net_ping;
pub mod net_ports;

// ── الأمان والصلاحيات ───────────────────────────────────────────────────
pub mod security_codesign;
pub mod security_gatekeeper;
pub mod security_permissions;
pub mod security_xattr;

// ── Git ومستودعات الشفرة ────────────────────────────────────────────────
pub mod git_archive;
pub mod git_branches;
pub mod git_commit;
pub mod git_diff;
pub mod git_init;
pub mod git_status;

// ── النظام والصيانة الدورية ─────────────────────────────────────────────
pub mod system_dns_flush;
pub mod system_info;
pub mod system_processes;
pub mod system_report;
pub mod system_uptime;
