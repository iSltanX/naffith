//! يطبع الفهرس والأقسام كما يعبران حدّ IPC، بصيغة JSON.
//!
//! **أداةُ عملٍ لا جزءٌ من المنتج**: تُبنى بـ`cargo run --example` ولا تدخل
//! الحزمة الموزَّعة. غرضها واحد — أن تُغذّي معاينة الواجهة (`vite.preview.config.ts`)
//! بالفهرس **الحقيقي** بدل قائمةٍ مكتوبة بيدٍ في ملف المعاينة.
//!
//! الفرق ليس شكليًا: لقطةُ شاشةٍ لقائمةٍ مخترعة تُري ما نتمنّاه لا ما بنيناه،
//! وتبقى صحيحةً بعد أن تتغيّر النواة. هذه تُعاد بنداءٍ واحد فتنكسر إن انكسر
//! الفهرس.

fn main() {
    let policy = naffith_core::policy::Policy::production();
    let out = serde_json::json!({
        "operations": naffith_core::registry::list(policy),
        "categories": naffith_core::registry::categories(policy),
    });
    println!("{}", serde_json::to_string_pretty(&out).expect("the catalogue must serialise"));
}
