// يمنع نافذة الطرفية الإضافية على ويندوز. لا أثر له على macOS، ويبقى لأن
// إزالته لاحقًا أسهل من تذكّر إضافته.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    naffith_core::run()
}
