//! سياسة وقت التشغيل.
//!
//! العمليات الداخلية مُترجَمة دائمًا كي تُختبر النواة كاملةً، لكن *إتاحتها*
//! قرار سياسة لا قرار ترجمة. جعلها `#[cfg]` كان سيعني أن سلوك الإنتاج لا
//! يُختبر إلا ببناء منفصل — وهو بالضبط ما يجعل ثغرة كهذه تمرّ.
//!
//! هنا يمكن لاختبار واحد أن يبني `Policy::production()` ويثبت أن العملية
//! الداخلية مرفوضة، في نفس البناء الذي يثبت أنها تعمل تحت `Policy::dev()`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Policy {
    /// هل يُسمح بتخطيط العمليات المعلَّمة `Visibility::Internal`؟
    pub allow_internal_operations: bool,
}

impl Policy {
    /// ما يعمل به التطبيق المُوزَّع.
    pub const fn production() -> Self {
        Self { allow_internal_operations: false }
    }

    /// بناء التطوير والاختبارات.
    pub const fn dev() -> Self {
        Self { allow_internal_operations: true }
    }

    /// السياسة التي يختارها التطبيق تلقائيًا حسب نوع البناء.
    pub fn for_build() -> Self {
        if cfg!(debug_assertions) {
            Self::dev()
        } else {
            Self::production()
        }
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self::production()
    }
}
