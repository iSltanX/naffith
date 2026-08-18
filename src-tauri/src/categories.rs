//! الأقسام — بياناتها الوصفية، وعددُ عملياتها محسوبًا لا مكتوبًا.
//!
//! ## لماذا القسم كيانٌ في النواة لا شبكةٌ في الواجهة
//!
//! شاشة الفئات كانت تستطيع أن تُرسم من قائمةٍ في TypeScript: ثمانية عناوين
//! وثمانية أرقام. وهي أسرع كتابةً، وتتقادم في اليوم الذي يليها: عمليةٌ تُضاف
//! في النواة لا تغيّر الرقم، وقسمٌ يفرغ من عملياته يبقى معروضًا بأرقامه.
//!
//! هنا القسم يعلن هويّته ونصوصه وترتيبه، و**العدد يُشتقّ من الفهرس** في
//! `registry::categories()`. لا يوجد في المنتج موضعٌ يمكن أن يقول «ثماني
//! عمليات» بينما الفهرس يقول غير ذلك، لأن لا موضع يقول العدد أصلًا.
//!
//! ## الأقسام الفارغة
//!
//! `visible_categories()` تُسقط القسم الذي لا عملية فيه. قسمٌ يُعلن ولا يُنفَّذ
//! وعدٌ لا يُوفى، والوعد الفارغ أسوأ من الغياب: من يفتحه مرّةً ويجده خاويًا لا
//! يفتحه ثانية حين يمتلئ.
//!
//! ويستثنى من ذلك `Kind::Journal`: قسمٌ مصدره سجلّ التشغيل لا الفهرس، فخلوّه
//! من العمليات وصفُه لا عطبُه.

use crate::spec::Category;
use serde::Serialize;

/// من أين يأتي محتوى القسم.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// عمليات من الفهرس.
    Operations,
    /// قيود من سجلّ التشغيل. لا عمليات فيه، ولا يُسقَط لفراغه.
    Journal,
    /// لا يُعرض في أي بناء.
    Hidden,
}

#[derive(Debug, Clone, Copy)]
pub struct CategoryMeta {
    pub id: Category,
    pub title_key: &'static str,
    pub description_key: &'static str,
    /// معرّفٌ في لوحة الرموز (`design-system/icons.svg`). اختبارٌ في الواجهة
    /// يثبّت أن كل معرّفٍ هنا موجودٌ في اللوحة فعلًا.
    pub icon: &'static str,
    pub sort_order: u16,
    pub kind: Kind,
}

/// كل الأقسام، بترتيب ظهورها. الترتيب معلَنٌ في `sort_order` لا في موضع السطر.
pub static ALL: &[CategoryMeta] = &[
    CategoryMeta {
        id: Category::Files,
        title_key: "cat.files.title",
        description_key: "cat.files.description",
        icon: "#i-folder",
        sort_order: 10,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::Compress,
        title_key: "cat.compress.title",
        description_key: "cat.compress.description",
        icon: "#i-compress",
        sort_order: 20,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::Images,
        title_key: "cat.images.title",
        description_key: "cat.images.description",
        icon: "#i-eye",
        sort_order: 30,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::Text,
        title_key: "cat.text.title",
        description_key: "cat.text.description",
        icon: "#i-file",
        sort_order: 40,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::Disk,
        title_key: "cat.disk.title",
        description_key: "cat.disk.description",
        icon: "#i-disk",
        sort_order: 50,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::Network,
        title_key: "cat.network.title",
        description_key: "cat.network.description",
        icon: "#i-network",
        sort_order: 60,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::Security,
        title_key: "cat.security.title",
        description_key: "cat.security.description",
        icon: "#i-security",
        sort_order: 70,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::Git,
        title_key: "cat.git.title",
        description_key: "cat.git.description",
        icon: "#i-git-branch",
        sort_order: 80,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::System,
        title_key: "cat.system.title",
        description_key: "cat.system.description",
        icon: "#i-system",
        sort_order: 90,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::Developer,
        title_key: "cat.developer.title",
        description_key: "cat.developer.description",
        icon: "#i-terminal",
        sort_order: 95,
        kind: Kind::Operations,
    },
    CategoryMeta {
        id: Category::History,
        title_key: "cat.history.title",
        description_key: "cat.history.description",
        icon: "#i-history",
        sort_order: 100,
        kind: Kind::Journal,
    },
    CategoryMeta {
        id: Category::Internal,
        title_key: "cat.internal.title",
        description_key: "cat.internal.description",
        icon: "#i-admin",
        sort_order: u16::MAX,
        kind: Kind::Hidden,
    },
];

pub fn find(id: Category) -> Option<&'static CategoryMeta> {
    ALL.iter().find(|c| c.id == id)
}

/// ما يُرسل إلى الواجهة: البيانات الوصفية، والعددان محسوبين من الفهرس.
#[derive(Debug, Clone, Serialize)]
pub struct CategorySummary {
    pub id: Category,
    pub title_key: &'static str,
    pub description_key: &'static str,
    pub icon: &'static str,
    pub sort_order: u16,
    pub kind: Kind,
    /// كل عمليات القسم في هذا البناء.
    pub operation_count: usize,
    /// ما يستطيع هذا الجهاز تشغيله منها.
    ///
    /// عددان لا واحد: قسمٌ فيه ستّ عمليات تعمل منها أربع يقول ذلك صراحةً، بدل
    /// أن يَعِد بستّ ثم يعرض اثنتين معطّلتين بلا تفسير في الشاشة التي قبله.
    pub available_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn category_ids_are_unique() {
        let mut seen = HashSet::new();
        for c in ALL {
            assert!(seen.insert(c.id), "duplicate category: {:?}", c.id);
        }
    }

    #[test]
    fn sort_orders_are_unique_so_the_screen_order_is_deterministic() {
        let mut seen = HashSet::new();
        for c in ALL {
            assert!(seen.insert(c.sort_order), "duplicate sort_order on {:?}", c.id);
        }
    }

    #[test]
    fn every_category_declares_distinct_title_and_description_keys() {
        for c in ALL {
            assert!(!c.title_key.is_empty(), "{:?} has no title key", c.id);
            assert!(!c.description_key.is_empty(), "{:?} has no description key", c.id);
            assert_ne!(c.title_key, c.description_key, "{:?} reuses one key twice", c.id);
        }
    }

    #[test]
    fn every_category_declares_a_sprite_icon() {
        for c in ALL {
            assert!(c.icon.starts_with('#'), "{:?} icon must be a sprite id: {}", c.id, c.icon);
        }
    }

    #[test]
    fn the_hidden_category_sorts_last_so_it_can_never_lead_the_screen() {
        let hidden: Vec<_> = ALL.iter().filter(|c| c.kind == Kind::Hidden).collect();
        assert_eq!(hidden.len(), 1, "only the internal category is hidden");
        assert_eq!(hidden[0].id, Category::Internal);
        for c in ALL.iter().filter(|c| c.kind != Kind::Hidden) {
            assert!(c.sort_order < hidden[0].sort_order);
        }
    }

    #[test]
    fn every_declared_category_has_metadata() {
        // القائمة أعلاه هي المصدر، وهذا يثبّت أن كل نوعٍ في `Category` مذكور
        // فيها: نوعٌ يُضاف ويُنسى هنا يعني قسمًا بلا اسم على الشاشة.
        for id in [
            Category::Files,
            Category::Compress,
            Category::Images,
            Category::Text,
            Category::Disk,
            Category::Network,
            Category::Security,
            Category::Git,
            Category::System,
            Category::Developer,
            Category::History,
            Category::Internal,
        ] {
            assert!(find(id).is_some(), "{id:?} has no metadata");
        }
        assert_eq!(ALL.len(), 12, "adding a category is a deliberate act — update this test");
    }
}
