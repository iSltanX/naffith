/**
 * النصوص العربية.
 *
 * النواة لا تحمل حرفًا عربيًا واحدًا: ترسل مفاتيح ثابتة (`err.path.missing`،
 * `explain.ditto.pkzip`) وهذا الملف يترجمها. السبب أن النواة تُختبر بلا واجهة،
 * وأن النص يتغيّر أكثر مما يتغيّر المنطق — وفصلهما يعني أن تعديل صياغة لا
 * يلمس شيفرةً تشغّل أوامر على ملفات المستخدم.
 *
 * كل مفتاح تصدره النواة يجب أن يكون هنا. اختبار `i18n.test` في `checks.ts`
 * يقارن القائمتين.
 */

export const AR = {
  // ── هوية المنتج ──────────────────────────────────────────────────────
  'app.naffith': 'نَفِّذ',
  'app.satr': 'سَطْر',
  'app.tagline': 'افعلْ ما تريد، وافهمْ ما فعلت.',

  // ── الترحيب — أول تشغيل وحده ─────────────────────────────────────────
  'onboarding.lede':
    'طبقة هادئة بينك وبين النظام. تختار ما تريد، ونتكفّل نحن بالأمر — ويبقى الأمر معروضًا أمامك متى أردت أن تعرف ما الذي جرى.',
  'onboarding.step1.title': 'اختر العملية',
  'onboarding.step1.body': 'تفتح القائمة فترى ما يستطيع التطبيق تنفيذه، وتختار بالاسم لا بالأمر.',
  'onboarding.step2.title': 'راجع ما سيُنفَّذ',
  'onboarding.step2.body':
    'تملأ حقلين أو ثلاثة، فتظهر لك معاينةٌ تقول بالضبط ما الذي سيحدث قبل أن يحدث.',
  'onboarding.step3.title': 'شاهد الأمر في سَطْر',
  'onboarding.step3.body':
    'الأمر نفسه، مشروحًا كلمةً كلمة. تتعلّم منه إن أردت، وتتجاهله إن لم ترد.',
  'onboarding.proof.caption': 'الأمر ظاهر قبل أن يعمل',
  'onboarding.start': 'ابدأ الآن',
  'onboarding.once': 'تظهر هذه الشاشة مرّةً واحدة. يمكنك عرضها مجددًا من الإعدادات.',

  // ── الأقسام ──────────────────────────────────────────────────────────
  // الوصف يقول ما في القسم لا ما يعنيه اسمه: «الملفات والمجلدات» عنوانٌ لا
  // يحتاج شرحًا، والذي يحتاجه هو «ماذا أجد بالضبط تحته؟».
  'cat.files.title': 'الملفات والمجلدات',
  'cat.files.description': 'نسخ ونقل وإنشاء، وبحثٌ عن الكبير والقديم، وقياسُ ما يشغل المساحة.',
  'cat.compress.title': 'الضغط وفكّ الضغط',
  'cat.compress.description': 'أرشفة ZIP وTAR.GZ، وفكُّها في مجلد جديد، وفحصُ محتوياتها قبل ذلك.',
  'cat.images.title': 'الصور',
  'cat.images.description': 'تحويل الصيغة وتغيير الأبعاد والتدوير، وقراءةُ خصائص الصورة.',
  'cat.text.title': 'النصوص والمستندات',
  'cat.text.description': 'دمج وتقسيم، وتحويل الترميز إلى UTF-8، ومقارنةٌ وبحثٌ داخل الملفات.',
  'cat.disk.title': 'الأقراص ومساحة التخزين',
  'cat.disk.description': 'المساحة الحرة، وبصمات SHA-256، ومقارنةُ ملفين، والأقراص المتصلة.',
  'cat.network.title': 'الشبكة والاتصال',
  'cat.network.description': 'اختبار الوصول وفحص DNS والمنافذ المستمعة، وتنزيلٌ من رابط.',
  'cat.security.title': 'الأمان والصلاحيات',
  'cat.security.description': 'قراءة الصلاحيات والسمات الممتدّة والتواقيع. قراءةٌ فقط — لا يُعدَّل شيء.',
  'cat.git.title': 'Git ومستودعات الشفرة',
  'cat.git.description': 'إنشاء مستودع، وحالته، وتسجيل commit، ومقارنةُ التغييرات وأرشفتها.',
  'cat.system.title': 'النظام والصيانة الدورية',
  'cat.system.description': 'العمليات الأعلى استهلاكًا، ومعلومات النظام، وتفريغ ذاكرة DNS.',
  'cat.developer.title': 'أدوات المطوّرين',
  'cat.developer.description': 'فحص الأنواع والأسلوب والاختبارات، وتشغيل مشاريع Node.js وTauri.',
  'cat.history.title': 'سجلّ العمليات السابقة',
  'cat.history.description': 'ما شُغِّل ومتى وبأي نتيجة، مع إعادة التشغيل بالقيم نفسها.',
  'cat.internal.title': 'داخلي',
  'cat.internal.description': 'لا يظهر في الإصدار.',

  // ── شاشة الفئات ──────────────────────────────────────────────────────
  'lib.heading': 'مكتبة',
  'lib.subheading': 'اختر فئة لاستعراض العمليات المتاحة فيها.',
  'lib.loading': 'يجري تحميل المكتبة…',
  'lib.loading.body': 'نحمّل الفئات والعمليات الآمنة.',
  'lib.failed': 'تعذّر الوصول إلى نواة التطبيق، فلم تُقرأ المكتبة.',
  'lib.failed.title': 'تعذّر الوصول إلى نواة التطبيق',
  'lib.failed.body': 'لم تُقرأ المكتبة. أعد المحاولة.',
  'lib.empty': 'لا توجد فئات في هذا الإصدار',
  'lib.empty.body': 'ستظهر الفئات هنا عند توفرها.',
  'lib.search.label': 'ابحث في المكتبة',
  'lib.search.placeholder': 'ابحث عن فئة أو عملية…',
  'lib.search.title': 'نتائج البحث',
  'lib.search.summary': 'نتائج لـ',
  'lib.search.empty': 'لا شيء يطابق ما كتبت. جرّب اسم العملية أو اسم الأداة.',
  'lib.search.empty.title': 'لا شيء يطابق ما كتبت',
  'lib.search.empty.body': 'جرّب اسم العملية أو اسم الأداة.',
  'lib.search.unavailable.title': 'نتيجة غير متاحة',
  'lib.search.unavailable.body': 'الأداة المطلوبة غير موجودة على هذا الجهاز.',
  'lib.search.unavailable.action': 'تفاصيل الإتاحة',
  'lib.search.categories': 'الفئات',
  'lib.search.operations': 'العمليات',
  'lib.favourites.title': 'المفضّلة',
  'lib.favourites.hint': 'ما ثبّتّه للوصول السريع.',
  'lib.favourite.add': 'إضافة إلى المفضّلة',
  'lib.favourite.remove': 'إزالة من المفضّلة',
  'lib.recents.title': 'المستخدَمة حديثًا',
  'lib.recents.hint': 'من سجلّ التشغيل الفعلي، لا من ترتيبٍ مفترض.',
  'lib.category.availability.of': 'من',
  'lib.category.availability.operations': 'عمليات متاحة',
  'lib.category.empty.count': 'لا عمليات متاحة',
  'lib.category.journal': 'ما شُغِّل سابقًا',
  'lib.category.empty': 'لا عمليات في هذا القسم بعد.',
  'lib.category.empty.body': 'ارجع إلى كل الفئات.',
  'lib.category.gone': 'لم يعد هذا القسم موجودًا في الفهرس.',
  'lib.category.loading.body': 'سيظهر محتوى الفئة حالًا.',
  'lib.category.gone.title': 'لم يعد هذا القسم موجودًا',
  'lib.category.gone.body': 'حدّث الفهرس أو ارجع إلى المكتبة.',

  // ── قائمة العمليات ───────────────────────────────────────────────────
  'ops.loading': 'يجري تحميل العمليات…',
  'ops.loading.body': 'تحميل تعريف العملية.',
  'ops.empty': 'لا توجد عمليات متاحة في هذا الإصدار.',
  'ops.failed': 'تعذّر الوصول إلى نواة التطبيق، فلم تُقرأ قائمة العمليات.',
  'ops.retry': 'إعادة المحاولة',
  'ops.unavailable': 'غير متاحة',
  'ops.unavailable.tool.label': 'الأداة غير متاحة',
  'ops.unavailable.unsupported.label': 'غير مدعومة',
  'ops.unavailable.why': 'تعلن هذه العملية مدخلًا لا تعرف هذه النسخة كيف ترسمه.',
  // سببٌ يستطيع المستخدم أن يفعل حياله شيئًا، فيُسمّى صراحةً: أداة النظام
  // المطلوبة غائبة عن هذا الجهاز، واسمها في آخر الجملة.
  'ops.unavailable.tool': 'تحتاج هذه العملية أداة نظام غير موجودة على هذا الجهاز:',
  'ops.gone': 'لم تعد هذه العملية موجودة في الفهرس.',
  'ops.gone.title': 'لم تعد هذه العملية موجودة',
  'ops.gone.body': 'العودة إلى المكتبة.',
  'ops.available': 'متاحة',
  'ops.open': 'فتح',
  'ops.count.one': 'عملية واحدة متاحة',
  'ops.count.many': 'عمليات متاحة',

  // ── التنقّل ──────────────────────────────────────────────────────────
  'nav.back': 'رجوع إلى العمليات',
  'nav.back.library': 'رجوع إلى الفئات',
  // الوسم المعروض على زرّ الرجوع في شريط الشاشة. `nav.back` تبقى اسمه المقروء
  // لقارئ الشاشة: كلمةٌ واحدة تكفي العين في شريطٍ مدمج، ولا تكفي صوتًا يُقرأ
  // بلا سياق. والمعروض جزءٌ من المقروء حرفيًا، فلا يتعارض الاسمان.
  'nav.back.short': 'رجوع',
  'nav.breadcrumbs': 'الموقع الحالي',
  'action.return': 'العودة',
  'nav.operations': 'العمليات',
  'nav.log': 'سجلّ التشغيل',
  'nav.settings': 'الإعدادات',
  'nav.leave.busy.title': 'هناك تشغيل جارٍ',
  'nav.leave.busy.body':
    'مغادرة الشاشة الآن لا توقف التشغيل، لكنك لن ترى نتيجته. يمكنك الانتظار حتى ينتهي، أو إلغاؤه أولًا.',
  'nav.leave.busy.stay': 'البقاء هنا',
  'nav.leave.busy.leave': 'المغادرة على أي حال',
  'nav.leave.dirty.title': 'حقول لم تُنفَّذ بعد',
  'nav.leave.dirty.body': 'ملأت حقولًا ولم تنفّذ العملية. المغادرة تمسحها.',
  'nav.leave.dirty.stay': 'العودة إلى الحقول',
  'nav.leave.dirty.leave': 'المغادرة والمسح',
  'dialog.safe_dismiss': 'Escape والنقر على الخلفية = الإجراء الآمن',

  // ── الإعدادات ────────────────────────────────────────────────────────
  'settings.title': 'الإعدادات',
  'settings.subtitle': 'خيارات محدودة وواضحة',
  'settings.onboarding.title': 'شاشة الترحيب',
  'settings.onboarding.body': 'تُعرض مرّةً واحدة عند أول تشغيل. اعرضها الآن متى شئت.',
  'settings.onboarding.replay': 'عرض شاشة الترحيب',
  'settings.node.title': 'مسار Node.js',
  'settings.node.body':
    'يلزم عمليات أدوات المطوّرين (فحص الأنواع، الاختبارات، Tauri). يُختار مرّةً واحدة.',
  'settings.node.unset': 'لم يُحدَّد بعد',
  'settings.node.choose': 'اختيار…',
  'settings.node.change': 'تغيير…',
  'settings.node.clear': 'مسح',
  'settings.cargo.title': 'مسار Cargo',
  'settings.cargo.body':
    'يلزم عمليات أدوات المطوّرين الخاصّة بـRust (الاختبارات، الفحص، Clippy، التنسيق، البناء). يُختار مرّةً واحدة.',
  'settings.cargo.unset': 'لم يُحدَّد بعد',
  'settings.cargo.choose': 'اختيار…',
  'settings.cargo.change': 'تغيير…',
  'settings.cargo.clear': 'مسح',
  'settings.storage.unavailable.title': 'تعذّر حفظ الإعدادات على هذا الجهاز',
  'settings.storage.unavailable.body': 'لن تُحفظ هذه الخيارات بين التشغيلات.',

  // ── الإعدادات · الألسنة ──────────────────────────────────────────────
  'settings.tab.general': 'عام',
  'settings.tab.appearance': 'المظهر',
  'settings.tab.developer': 'أدوات المطوّرين',
  'settings.tab.about': 'حول',
  'settings.tabs.label': 'أقسام الإعدادات',

  // ── الإعدادات · عام ──────────────────────────────────────────────────
  'settings.general.title': 'الإعدادات العامة',
  'settings.welcome.title': 'شاشة الترحيب',
  'settings.welcome.body': 'عرض شاشة الترحيب وتلميحات البداية عند تشغيل التطبيق',
  'settings.sound.title': 'صوت الإشعارات',
  'settings.sound.body': 'تشغيل نغمة تنبيه لطيفة فور اكتمال معالجة العمليات بنجاح',
  'settings.confirm.title': 'تأكيد قبل التنفيذ',
  'settings.confirm.body':
    'طلب تأكيد صريح قبل تنفيذ الأوامر والعمليات الحساسة أو غير القابلة للتراجع',
  'settings.workpath.title': 'مسار العمل الافتراضي',
  'settings.workpath.active': 'المجلد النشط:',
  'settings.workpath.unset': 'لم يُحدَّد بعد',
  'settings.workpath.choose': 'اختيار…',
  'settings.workpath.change': 'تغيير…',
  'settings.workpath.clear': 'مسح',

  // ── الإعدادات · المظهر ───────────────────────────────────────────────
  'settings.appearance.title': 'مظهر التطبيق',
  'settings.theme.title': 'السمة العامة للواجهة',
  'settings.theme.system': 'تلقائي',
  'settings.theme.light': 'فاتح',
  'settings.theme.dark': 'داكن',
  'settings.iconsize.title': 'حجم الأيقونات في الشريط الجانبي',
  'settings.iconsize.small': 'صغير',
  'settings.iconsize.medium': 'متوسط',
  'settings.iconsize.large': 'كبير',

  // ── الإعدادات · أدوات المطوّرين ──────────────────────────────────────
  'settings.developer.title': 'أدوات المطوّرين',
  'settings.toolpath.selected': 'المسار المحدَّد:',

  // ── الإعدادات · حول ──────────────────────────────────────────────────
  'settings.about.title': 'حول نَفِّذ',
  'settings.about.version': 'الإصدار {version}',
  'settings.about.description': 'أداة تنفيذ عمليات macOS شاملة — ملفات، صور، أقراص، شبكة، وأكثر.',
  'settings.about.credit': 'تطوير وتصميم سلطان',
  'settings.update.auto': 'التحديث التلقائي',
  'settings.update.check': 'فحص وجود تحديث',
  'settings.update.retry': 'إعادة المحاولة',
  'settings.update.download': 'تحميل التحديث',
  'settings.update.idle': 'لم يُفحص بعد',
  'settings.update.checking': 'جارٍ البحث عن تحديثات…',
  'settings.update.uptodate': 'محدّث — لا توجد تحديثات جديدة',
  'settings.update.available': 'تحديث متوفر — الإصدار {version}',
  'settings.update.failed': 'تعذّر التحقق من التحديثات',
  // حالتان لا واحدة، ولكلٍّ عنوانها ونبرتها:
  //
  // ‏«غير مهيأة» ليست فشلًا. لم تُضبط وجهةُ التحديث في هذا البناء بعد، فلا
  // شيء انكسر ولا شيء يُعيد المستخدم محاولته — وعرضُها بلون الخطر ونصّ
  // «تعذّر التحقق» كان يتّهم الشبكة بما لم تفعله ويدفع المستخدم إلى فحص
  // اتصالٍ سليم. و«تعذّر التحقق» تبقى لما هو فشلٌ فعلًا: وجهةٌ مضبوطة لم
  // تُبلَغ.
  'settings.update.unconfigured': 'التحديثات غير مهيأة بعد',
  'settings.update.unconfigured.hint': 'ستعمل تلقائيًا فور ضبط وجهة التحديث ومفتاح التوقيع.',
  'settings.update.failed.network': 'تحقق من اتصالك بالإنترنت وحاول مرة أخرى.',
  'settings.update.installing': 'جارٍ تنزيل التحديث…',
  // H-7: كانت الشاشة تبقى على «جارٍ تنزيل التحديث…» إلى الأبد بعد نجاح
  // التثبيت فعليًا — لا حالة نهائية كانت تعقب `installing`. هذه هي تلك
  // الحالة: تُعرض بعد أن ينجح `downloadAndInstallUpdate` فعلًا، وتقول ما
  // يفعله المستخدم بعدها — إذ لا وسيلة في هذا الإصدار لإعادة التشغيل تلقائيًا.
  'settings.update.installed': 'ثُبِّت التحديث',
  'settings.update.installed.hint': 'أعد تشغيل نَفِّذ لتبدأ النسخة الجديدة.',

  // ── سجلّ التشغيل ─────────────────────────────────────────────────────
  'log.title': 'سجلّ التشغيل',
  'log.subtitle': 'آخر ٢٠٠ قيد كحد أقصى',
  'log.loading': 'يجري تحميل…',
  'log.loading.body': 'انتظار هادئ.',
  'log.empty': 'لا توجد تشغيلات بعد.',
  'log.empty.body': 'ستظهر هنا العمليات بعد تنفيذها، سواء نجحت أو فشلت أو أُلغيت.',
  'log.empty.filtered': 'لا قيود تطابق المرشّحات المختارة.',
  'log.empty.filtered.body': 'غيّر البحث أو المرشّحات لرؤية قيود أخرى.',
  'log.search.label': 'البحث في سجلّ التشغيل',
  'log.search.placeholder': 'ابحث في العملية أو المسار…',
  'log.start': 'ابدأ أول عملية',
  'log.failed': 'تعذّر تحميل سجلّ التشغيل.',
  'log.failed.body': 'تعذّر الوصول إلى سجلّ التشغيل الآن. حاول مرة أخرى.',
  'log.state.planned': 'خُطِّطت',
  'log.state.running': 'جارية',
  'log.state.succeeded': 'نجحت',
  'log.state.failed': 'فشلت',
  'log.state.cancelled': 'أُلغيت',
  'log.state.unknown': 'عملية أو حالة غير معروفة',
  'log.cap.title': 'تم عرض أحدث ٢٠٠ قيد',
  'log.cap.body': 'القيود الأقدم لا تُعرض في الواجهة.',
  'log.filters': 'تصفية السجل',
  'log.filter.state': 'الحالة',
  'log.filter.category': 'القسم',
  'log.filter.period': 'المدّة',
  'log.filter.any': 'الكل',
  'log.filter.succeeded': 'ناجح',
  'log.filter.failed': 'فاشل',
  'log.filter.cancelled': 'ملغى',
  'log.filter.today': 'آخر ٢٤ ساعة',
  'log.filter.week': 'آخر أسبوع',
  'log.filter.month': 'آخر ٣٠ يومًا',
  'log.filter.reset': 'إلغاء التصفية',
  'log.tail': 'آخر ما طبعته الأداة',
  'log.command': 'الأمر المنفّذ',
  'log.details.show': 'إظهار التفاصيل',
  'log.details.hide': 'إخفاء التفاصيل',
  'log.action.cancel': 'إلغاء',
  'log.action.stop': 'إيقاف',
  'log.action.details': 'عرض التفاصيل',
  'log.entry.status.planned': 'مخطط · بانتظار التنفيذ',
  'log.entry.status.running': 'قيد التشغيل',
  'log.entry.status.succeeded': 'نجح',
  'log.entry.status.failed': 'فشل',
  'log.entry.status.cancelled': 'أُلغي · أوقفه المستخدم',
  'log.entry.status.unknown': 'حالة غير معروفة · تعذّر تفسير النتيجة',
  'log.unknown.title': 'عملية أو حالة غير معروفة',
  'log.unknown.body': 'يبقى الأمر والمعرّف متاحين للمراجعة.',
  'log.unknown.continue': 'متابعة',
  'log.entry.since': 'منذ',
  'log.entry.completed_in': 'اكتمل خلال',
  'log.entry.output.retained': 'تم تسجيل آخر خرج موثوق للعملية.',
  'log.preview.cancel.failed': 'تعذّر إلغاء المعاينة. بقي قيدها في السجل.',
  'log.stop.failed': 'تعذّر إيقاف التشغيل. قد يكون قد انتهى بالفعل.',
  // النبرة إرشاد لا وعد: الزرّ يملأ النموذج ويقف، ولا ينفّذ شيئًا.
  'log.rerun': 'تشغيل مجددًا',
  'log.delete': 'حذف القيد',
  'log.delete.failed': 'تعذّر حذف القيد. بقي سجلّ التشغيل كما هو.',
  'log.clear': 'مسح السجل',
  'log.clear.title': 'مسح سجلّ التشغيل كله؟',
  // الفرق الذي يجب أن يُقال صراحةً: هذا يمحو الأثر لا ما أنتجه.
  'log.clear.body':
    'يُحذف تاريخ ما شُغِّل من هذا الجهاز، ولا يُتراجع عنه. الملفات التي أنشأتها العمليات تبقى في أماكنها كما هي — هذا مسحٌ للأثر لا للنتائج.',
  'log.clear.cancel': 'الإبقاء على السجل',
  'log.clear.confirm': 'مسح السجل',
  'log.clear.failed': 'تعذّر مسح السجل. بقيت القيود كما هي.',

  // ── العملية ──────────────────────────────────────────────────────────
  // الضغط وفكّ الضغط
  'op.compress.zip.list.title': 'فحص محتويات أرشيف ZIP',
  'op.compress.zip.list.description': 'يعرض فهرس أرشيف ZIP قبل استخراجه.',
  'op.compress.zip.list.execution':
    'قراءةٌ فقط: لا يُكتب شيء على القرص. الأرشيف يبقى كما هو، ويُنبَّه إن كان فيه ما يخرج من جذره.',
  'op.compress.zip.extract.title': 'فكّ أرشيف ZIP في مجلد جديد',
  'op.compress.zip.extract.description': 'يستخرج محتوى أرشيف ZIP في مجلد جديد.',
  'op.compress.zip.extract.execution':
    'يُنشأ مجلد جديد باسمٍ تختاره في الوجهة. الأرشيف يبقى كما هو. ويُرفض ما يخرج من جذر الأرشيف، ولا يظهر المجلد باسمه النهائي إلا بعد نجاحٍ كامل.',
  'op.compress.zip.extract.result': 'تم فكّ الأرشيف',
  'op.compress.zip.test.title': 'اختبار سلامة أرشيف ZIP',
  'op.compress.zip.test.description': 'يتحقّق من سلامة أرشيف ZIP ويكشف تلفه.',
  'op.compress.zip.test.execution':
    'تُفكّ كل مدخلة في الذاكرة وحدها: لا يُستخرج شيء ولا يُكتب على القرص. الأرشيف يبقى كما هو. والجواب أنه وصل كما غادر، لا أن محتواه صحيح.',
  'op.compress.tar.create.title': 'إنشاء أرشيف TAR.GZ من مجلد',
  'op.compress.tar.create.description': 'يجمع مجلدًا ويضغطه في أرشيف TAR.GZ واحد.',
  'op.compress.tar.create.execution':
    'يُنشأ ملف ⁦.tar.gz⁩ جديد في الوجهة التي تختارها. المجلد الأصلي لا يُمسّ. وبيانات macOS الوصفية لا تُحفظ في هذه الصيغة.',
  'op.compress.tar.create.result': 'تم إنشاء الأرشيف',
  'op.compress.tar.extract.title': 'فكّ أرشيف TAR أو TAR.GZ',
  'op.compress.tar.extract.description': 'يستخرج محتوى أرشيف TAR في مجلد جديد.',
  'op.compress.tar.extract.execution':
    'يُنشأ مجلد جديد باسمٍ تختاره في الوجهة. الأرشيف يبقى كما هو. والاستخراج يقع في مجلد مؤقّت لا يظهر باسمه النهائي إلا بعد نجاحٍ كامل.',
  'op.compress.tar.extract.result': 'تم فكّ الأرشيف',
  'op.compress.tar.list.title': 'فحص محتويات أرشيف TAR',
  'op.compress.tar.list.description': 'يعرض فهرس أرشيف TAR قبل استخراجه.',
  'op.compress.tar.list.execution':
    'قراءةٌ فقط: لا يُكتب شيء على القرص ولا يُستخرج شيء. الأرشيف يبقى كما هو.',

  'op.compress.folder.zip.title': 'ضغط مجلد في أرشيف ZIP',
  'op.compress.folder.zip.description': 'يجمع مجلدًا كاملًا في ملف ZIP واحد.',
  'op.compress.folder.zip.execution':
    'يُنشأ ملف ZIP جديد في الوجهة التي تختارها، ببيانات macOS الوصفية محفوظة. المجلد الأصلي يبقى كما هو ولا يُحذف. وإن كان الاسم مأخوذًا تتوقّف العملية ولا يُستبدل شيء.',
  'op.compress.folder.zip.result': 'تم إنشاء الأرشيف',
  'op.internal.echo.title': 'عملية اختبار داخلية',
  'op.internal.echo.description': 'لا تظهر في الإصدار.',

  // ── الحقول ───────────────────────────────────────────────────────────
  'field.source.label': 'المجلد المراد ضغطه',
  'field.source.help': 'اختر مجلدًا قائمًا.',
  'field.source.placeholder': 'لم يُختَر مجلد بعد',
  'field.destination.label': 'مكان حفظ الأرشيف',
  'field.destination.help': 'مجلد قائم تملك صلاحية الكتابة فيه.',
  'field.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.archive_name.label': 'اسم الأرشيف',
  'field.archive_name.help': 'يُضاف ⁦.zip⁩ تلقائيًا مرة واحدة.',
  'field.archive_name.placeholder': 'مثال: نسخة احتياطية ٢٠٢٦',
  // نصوص الحقول العامّة. `tFirst` تفضّل `field.<op>.<input>.*` حين يعلنها
  // ملفُّ عملية، وما لم تفعل يُستعمل هذا — فحقلٌ مألوف لا يحتاج نصًّا مكرّرًا
  // في كل عملية.
  'field.target.label': 'الملف أو المجلد',
  'field.target.help': 'اختر الملف المقصود.',
  'field.target.placeholder': 'لم يُختَر شيء بعد',
  'field.folder.label': 'المجلد',
  'field.folder.help': 'مجلد قائم داخل المنزل أو قرص مركَّب.',
  'field.folder.placeholder': 'لم يُختَر مجلد بعد',
  'field.port.label': 'رقم المنفذ',
  'field.port.help': 'بين ١ و٦٥٥٣٥. والشائع في التطوير ٣٠٠٠ و٥١٧٣ و٨٠٨٠.',
  'field.port.placeholder': '٣٠٠٠',
  'field.pid.label': 'رقم العملية',
  'field.pid.help': 'الرقم كما يظهر في «العمليات الأعلى استهلاكًا» أو «البحث عن عملية».',
  'field.pid.placeholder': '١',
  // تجاوزٌ خاصّ بالإنهاء: الحقل نفسه، والرهان غير الرهان. النصّ العام أعلاه
  // يصف قراءةً، وهذا يصف فعلًا لا رجعة فيه — انظر `fieldKeys` في
  // `operations.ts`: الخاصّ بالعملية يُجرَّب قبل العامّ.
  'field.system.process.kill.pid.help':
    'رقم العملية التي ستُنهى. تأكّد منه: رقمٌ خاطئ يُنهي برنامجًا آخر بلا تراجع.',
  // تجاوزٌ لازم لا تكرار: النصّ العام يقترح ⁧١⁩، وهو رقمٌ **يرفضه مدى هذه
  // العملية** (أدناه ٢، إذ ١ هو launchd). مكانٌ نائبٌ يقترح قيمةً مرفوضة
  // يعلّم المستخدم أن يتجاهل ما تقوله الشاشة.
  'field.system.process.kill.pid.placeholder': '٢',
  'field.minutes.label': 'المدّة بالدقائق',
  'field.minutes.help': 'كم دقيقةً إلى الوراء من الآن. والحدّ الأقصى ١٥ لإبقاء الجواب كاملًا.',
  'field.minutes.placeholder': '٥',
  'field.system.process.find.name.label': 'اسم العملية',
  'field.system.process.find.name.help':
    'جزءٌ من الاسم يكفي: «‏Find» تجد «‏Finder». ويُقرأ تعبيرًا نمطيًّا، فالأقواس لها معنًى.',
  'field.system.process.find.name.placeholder': 'مثال: Finder',
  'field.parent.label': 'المجلد الحاوي',
  'field.parent.help': 'المجلد الذي يُنشأ بداخله.',
  'field.parent.placeholder': 'لم يُختَر مجلد بعد',
  'field.folder_name.label': 'اسم المجلد الجديد',
  'field.folder_name.help': 'اسمٌ جديد: لا يُستبدل مجلد قائم بالاسم نفسه.',
  'field.folder_name.placeholder': 'مثال: المستخرَج',
  'field.out_name.label': 'اسم الملف الناتج',
  'field.out_name.help': 'اسمٌ جديد: لا يُستبدل ملف قائم بالاسم نفسه.',
  'field.out_name.placeholder': 'اكتب اسمًا',
  'field.new_name.label': 'الاسم في الوجهة',
  'field.new_name.help': 'اسمٌ غير مستعمَل في الوجهة.',
  'field.new_name.placeholder': 'اكتب اسمًا',
  'field.archive.label': 'ملف الأرشيف',
  'field.archive.help': 'أرشيف قائم على قرصك.',
  'field.archive.placeholder': 'لم يُختَر أرشيف بعد',
  'field.left.label': 'الملف الأول',
  'field.left.help': 'الملف الأول في المقارنة.',
  'field.left.placeholder': 'لم يُختَر ملف بعد',
  'field.right.label': 'الملف الثاني',
  'field.right.help': 'الملف الثاني في المقارنة.',
  'field.right.placeholder': 'لم يُختَر ملف بعد',
  'field.first.label': 'الملف الأول',
  'field.first.help': 'يأتي أولًا في الناتج.',
  'field.first.placeholder': 'لم يُختَر ملف بعد',
  'field.second.label': 'الملف الثاني',
  'field.second.help': 'يُلحق بالأول كما هو، بلا فاصل بينهما.',
  'field.second.placeholder': 'لم يُختَر ملف بعد',
  'field.repo.label': 'مجلد المستودع',
  'field.repo.help': 'مجلد يحوي ⁦.git⁩.',
  'field.repo.placeholder': 'لم يُختَر مستودع بعد',
  'field.pattern.label': 'ما تبحث عنه',
  'field.pattern.help': 'نصّ حرفي لا تعبير نمطي: ما تكتبه هو ما يُبحث عنه.',
  'field.pattern.placeholder': 'اكتب نصًّا',
  'field.message.label': 'رسالة الـcommit',
  'field.message.help': 'سطر واحد يصف ما تغيّر.',
  'field.message.placeholder': 'مثال: إصلاح حساب المساحة',
  'field.url.label': 'الرابط',
  'field.url.help': 'يبدأ بـ ⁦http://⁩ أو ⁦https://⁩ فقط.',
  'field.url.placeholder': 'https://example.com/file',
  'field.host.label': 'اسم المضيف أو عنوانه',
  'field.host.help': 'مثال: example.com أو 1.1.1.1',
  'field.host.placeholder': 'example.com',
  'field.domain.label': 'اسم النطاق',
  'field.domain.help': 'اسم النطاق وحده، بلا ⁦https://⁩.',
  'field.domain.placeholder': 'example.com',
  'field.count.label': 'عدد المحاولات',
  'field.count.help': 'كم رزمة تُرسل قبل التوقّف.',
  'field.record.label': 'نوع السجل',
  'field.record.help': 'A للعناوين، MX للبريد، TXT للتحقّق.',
  'field.format.label': 'الصيغة الناتجة',
  'field.format.help': 'الصيغة التي تُكتب بها النسخة الجديدة.',
  'field.degrees.label': 'زاوية التدوير',
  'field.degrees.help': 'باتجاه عقارب الساعة.',
  'field.max_pixels.label': 'أطول ضلع بالبكسل',
  'field.max_pixels.help': 'تُحفظ النسبة، ولا تُكبَّر الصورة فوق حجمها الأصلي.',
  'field.min_megabytes.label': 'أصغر حجم بالميغابايت',
  'field.min_megabytes.help': 'يُعرض ما حجمه أكبر من هذا الرقم.',
  'field.days.label': 'عدد الأيام',
  'field.days.help': 'يُعرض ما لم يُفتح منذ هذه المدّة.',
  'field.depth.label': 'عمق التحليل',
  'field.depth.help': 'صفر للمجموع وحده، وواحد للمجلدات المباشرة.',
  'field.input_encoding.label': 'ترميز الملف الحالي',
  'field.input_encoding.help': 'الترميز الذي كُتب به الملف قبل التحويل.',
  'field.input_encoding.placeholder': 'اختر الترميز الحالي',
  'field.ignore_case.label': 'تجاهل حالة الأحرف',
  'field.staged.label': 'المُدرَج للـcommit فقط',
  'field.staged.help': 'فعّله لعرض التغييرات المدرجة فقط.',
  'field.git.archive.revision.label': 'النسخة أو المرجع',
  'field.git.archive.revision.help': 'اسم فرع أو وسم أو بصمة commit، مثل HEAD.',
  'field.git.archive.revision.placeholder': 'مثال: HEAD أو v1.0.0',
  'field.git.log.limit.label': 'عدد التسجيلات',
  'field.git.log.limit.help': 'أقصى عدد تسجيلات يُعرض، من الأحدث.',
  'field.git.log.limit.placeholder': '٥٠',
  'field.git.diff.commits.from.label': 'من',
  'field.git.diff.commits.from.help': 'فرعٌ أو وسمٌ أو بصمة commit — نقطة البداية في المقارنة.',
  'field.git.diff.commits.from.placeholder': 'مثال: main',
  'field.git.diff.commits.to.label': 'إلى',
  'field.git.diff.commits.to.help': 'فرعٌ أو وسمٌ أو بصمة commit — نقطة النهاية في المقارنة.',
  'field.git.diff.commits.to.placeholder': 'مثال: HEAD',
  'field.git.show.file.ref.label': 'المرجع',
  'field.git.show.file.ref.help': 'اسم فرع، أو وسم، أو بصمة commit، أو HEAD.',
  'field.git.show.file.ref.placeholder': 'مثال: HEAD',
  'field.git.show.file.path.label': 'مسار الملفّ داخل المستودع',
  'field.git.show.file.path.help':
    'مسارٌ نسبي كما يظهر في شجرة المستودع، حتى لو لم يعد موجودًا الآن.',
  'field.git.show.file.path.placeholder': 'مثال: src/main.rs',
  'field.git.blame.path.label': 'الملفّ',
  'field.git.blame.path.help': 'الملفّ الذي تريد معرفة من كتب كل سطرٍ فيه.',
  'field.git.blame.path.placeholder': 'لم يُختَر ملفّ بعد',
  'field.system.report.detail.label': 'قسم التقرير',
  'field.system.report.detail.help': 'اختر نوع المعلومات التي تريد قراءتها.',
  'field.system.report.detail.placeholder': 'اختر قسمًا',
  'field.choice.placeholder': 'اختر قيمة',
  'field.number.range': 'أدخل رقمًا ضمن النطاق المعلن.',
  'field.number.range.label': 'النطاق:',
  'field.number.default.label': 'الافتراضي:',
  'field.choose': 'اختيار…',
  'field.chosen': 'تغيير…',
  'field.choose.file': 'اختيار ملف',
  'field.choose.folder': 'اختيار مجلد',

  // تخصيصات تمنع نصوص عملية ZIP العامة من الظهور في عمليات أخرى.
  'field.files.copy.source.label': 'الملف أو المجلد المراد نسخه',
  'field.files.copy.source.help': 'اختر ملفًا أو مجلدًا قائمًا.',
  'field.files.copy.source.placeholder': 'لم يُختَر شيء بعد',
  'field.files.copy.destination.label': 'المجلد الذي تُحفظ فيه النسخة',
  'field.files.copy.destination.help': 'مجلد قائم تملك صلاحية الكتابة فيه، وليس داخل المصدر.',
  'field.files.copy.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.files.find.name.pattern.label': 'نمط الاسم',
  'field.files.find.name.pattern.help': 'تعمل فيه ⁦*⁩ و⁦?⁩ والأقواس المربعة كنمط glob، ولا يُقبل ⁦/⁩.',
  'field.files.find.name.pattern.placeholder': 'مثال: *.pdf',
  'field.compress.zip.extract.destination.label': 'مكان حفظ المجلد المستخرَج',
  'field.compress.zip.extract.destination.help': 'مجلد قائم تملك صلاحية الكتابة فيه.',
  'field.compress.zip.extract.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.compress.tar.extract.destination.label': 'مكان حفظ المجلد المستخرَج',
  'field.compress.tar.extract.destination.help': 'مجلد قائم تملك صلاحية الكتابة فيه.',
  'field.compress.tar.extract.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.compress.tar.create.archive_name.help': 'تُضاف ⁦.tar.gz⁩ تلقائيًا مرّةً واحدة.',

  // ── الملخّص ──────────────────────────────────────────────────────────
  'summary.title': 'ما الذي سيحدث',
  'summary.plan.creates-file.title': 'سيُنشأ ملف جديد',
  'summary.plan.creates-file.chip': 'ينشئ عنصرًا جديدًا',
  'summary.plan.creates-file.body': 'يبقى المصدر كما هو، ويُثبت الملف بعد نجاح التنفيذ.',
  'summary.plan.creates-directory.title': 'سيُنشأ مجلد جديد',
  'summary.plan.creates-directory.chip': 'ينشئ عنصرًا جديدًا',
  'summary.plan.creates-directory.body': 'يُجهّز المجلد مؤقتًا ثم يُثبت في الموقع النهائي.',
  'summary.plan.safe.title': 'عملية قراءة فقط',
  'summary.plan.safe.chip': 'قراءة فقط',
  'summary.plan.safe.body': 'لن تغيّر هذه العملية الملفات أو إعدادات النظام.',
  'summary.plan.modifies.title': 'سيُعدّل محتوى قائمًا',
  'summary.plan.modifies.chip': 'يعدّل محتوى قائمًا',
  'summary.plan.modifies.body': 'راجع الهدف بعناية؛ قد يتعذر التراجع عن النتيجة.',
  'summary.plan.unavailable.title': 'لا يمكن تجهيز الخطة',
  'summary.plan.unavailable.chip': 'غير متاحة',
  'summary.plan.unavailable.body': 'أداة النظام المطلوبة غير متاحة على هذا الجهاز:',
  'summary.output': 'الناتج المتوقع',
  'summary.working_directory': 'مجلد العمل',
  'summary.facts.heading': 'حدود التنفيذ',
  'summary.effect.safe': 'قراءة فقط؛ لا تكتب هذه العملية تغييرًا على القرص.',
  'summary.effect.creates': 'تنشئ هذه العملية عنصرًا أو حالة جديدة وفق الحقول أعلاه.',
  'summary.effect.creates_artifact': 'تنشئ هذه العملية الناتج المبين أعلاه.',
  'summary.effect.modifies': 'تعدّل هذه العملية محتوى أو حالة قائمة وفق الحقول أعلاه.',
  'summary.effect.destructive': 'تُحدث هذه العملية تغييرًا لا يمكن للتطبيق التراجع عنه.',
  'summary.conflict.refuse': 'لا يُستبدل ملف قائم. إن وُجد الاسم، تتوقّف العملية وتُخبرك.',
  'summary.conflict.no_artifact': 'هذه العملية لا تُنشئ ملفًا، فلا اسم يتضارب.',
  'summary.tool': 'الأداة المنفِّذة',
  'summary.estimate': 'حجم المصدر تقديريًا',
  'summary.estimate.note': 'تقدير أولي من معلومات المصدر قبل التنفيذ.',
  'summary.estimate.partial': 'المسح توقّف عند حدّه، فالرقم حدّ أدنى لا مجموعًا.',
  // عنوان عدّاد لا جملة: الصفّ يظهر مع كل تقدير، تامًّا كان أو ناقصًا، فلا
  // يصحّ أن يعِد بأحد الحالين. أمّا خبر التوقّف عند الحدّ فمكانه
  // `summary.estimate.partial` بجانب الحجم — تكرارُه هنا يجعل المستخدم يبحث
  // عن فرقٍ بين جملتين تقولان الشيء نفسه.
  'summary.estimate.entries': 'العناصر التي مُسحت',
  'unit.bytes': 'بايت',
  'unit.kb': 'ك.ب',
  'unit.mb': 'م.ب',
  'unit.gb': 'ج.ب',
  'summary.danger.safe': 'قراءة فقط',
  'summary.danger.creates': 'ينشئ عنصرًا جديدًا',
  'summary.danger.modifies': 'يعدّل محتوى قائمًا',
  'summary.danger.destructive': 'تغيير لا يمكن التراجع عنه',

  // ── تأكيد التنفيذ (إعداد «تأكيد قبل التنفيذ») ──────────────────────────
  // ثلاثةٌ لا أربعة: «قراءة فقط» لا تصل هذا الحوار أصلًا — انظر توثيق
  // `confirmBeforeExecute` في settings.ts. الأزرار تستعير `action.execute`
  // و`action.cancel` نفسيهما، فلا صياغة ثانية لفعلٍ واحد.
  'run.confirm.creates.title': 'تأكيد الإنشاء',
  'run.confirm.creates.body': 'العملية المعروضة تنشئ عنصرًا جديدًا على القرص. التنفيذ يبدأ الآن.',
  'run.confirm.modifies.title': 'تأكيد التعديل',
  'run.confirm.modifies.body':
    'العملية المعروضة تعدّل محتوًى قائمًا على القرص. التنفيذ يبدأ الآن.',
  'run.confirm.destructive.title': 'تأكيد تغييرٍ لا رجعة فيه',
  'run.confirm.destructive.body':
    'العملية المعروضة تغييرٌ لا يمكن التراجع عنه بعد تنفيذه. التنفيذ يبدأ الآن.',

  // ── الأزرار والحالات ────────────────────────────────────────────────
  'action.execute': 'نفِّذ',
  'action.execute.incomplete': 'أكمل الحقول',
  // سبب تعطّل «نفِّذ». القرار المعتمد: الزر ظاهر دائمًا ومعطّل بسببٍ مكتوب
  // تحته — لا يظهر ويختفي. زرٌّ يظهر عند صلاحية الخطة يجعل الفعل الأساسي
  // يرقص مع كل ضغطة مفتاح، ويحرم من لم تكتمل حقوله من معرفة أنّ ثمّة زرًّا
  // أصلًا. والسبب المكتوب حالةٌ لا خطأ: نبرته إرشاد لا لوم.
  'action.execute.why.incomplete': 'أكمل الحقول أعلاه ليصبح التنفيذ ممكنًا.',
  'action.execute.why.planning': 'يجري التحقّق من المسارات…',
  'action.execute.why.invalid': 'صحّح ما ذُكر أعلاه ليصبح التنفيذ ممكنًا.',
  'action.cancel': 'إلغاء',
  'action.reveal': 'إظهار في Finder',
  'action.again': 'تشغيل مجددًا',
  'action.return.library': 'العودة إلى المكتبة',
  'action.copy.path': 'نسخ المسار',
  'action.copy.result': 'نسخ النتيجة',
  'action.copy.diagnostic': 'نسخ التشخيص',
  'action.show.raw': 'عرض الخرج الخام',
  'action.hide.raw': 'إخفاء الخرج الخام',
  'action.copy': 'نسخ الأمر',
  'action.copy.short': 'نسخ',
  'action.copied': 'نُسخ',
  'state.checking': 'يجري التحقّق…',
  'state.checking.note': 'تُبنى الخطة الآمنة الآن، ويمكنك مراجعة الحقول أثناء ذلك.',
  'state.running': 'جارٍ تنفيذ العملية…',
  'state.running.note': 'لا تعرض الأداة نسبة تقدّم موثوقة. يمكنك متابعة خرجها في سَطْر.',
  'state.succeeded': 'اكتملت العملية بنجاح',
  'state.failed': 'لم تكتمل العملية',
  'state.cancelled': 'أُلغيت العملية',
  'state.cancelled.note': 'أُوقفت العملية ونُظّفت نواتجها المؤقتة. لم يُسجّل ناتج نهائي.',
  'state.failed.note': 'لم تكتمل العملية. راجع السبب وخرج الأداة أدناه.',
  'state.cancelling': 'يجري الإلغاء…',
  // تفصيل الفشل: النواة تعلن رمز الخروج أو الإشارة، وبدونهما تبقى الرسالة
  // «لم تكتمل العملية» بلا ما يُبحث عنه. الرقم يُركَّب في الشاشة لا في النصّ.
  'state.failed.code': 'رمز الخروج',
  'state.failed.signal': 'أنهتها إشارة',

  // ── مجرى التشغيل ─────────────────────────────────────────────────────
  // ما تطبعه الأداة نفسها. النواة تبثّه سطرًا سطرًا وتعلن قصَّه عند سقفها.
  'stream.heading': 'ما طبعته الأداة',
  'stream.waiting': 'لم تطبع الأداة شيئًا بعد.',
  // بلا اسم أداةٍ بعينها: الشاشة واحدة لكل العمليات، ونصٌّ يقول «هذا متوقَّع من
  // ⁦ditto⁩» يصير كذبًا في أوّل عمليةٍ تستعمل أداةً مسهبة.
  'stream.silent': 'لم تطبع الأداة شيئًا. وكثيرٌ من الأدوات تصمت عند النجاح.',
  'stream.stdout': 'خرج',
  'stream.stderr': 'خطأ',
  'stream.lines': 'سطرًا',
  // قصٌّ في النواة: بُلغ سقف الأسطر فتوقّف البثّ.
  'stream.truncated': 'بُلغ سقف الأسطر في النواة، فتوقّف البثّ. لم يُبَثّ:',
  'stream.omitted': 'يُعرض الذيل المحدود للنتيجة. أُسقط من بدايته:',
  // قصٌّ في الواجهة: يُحفظ آخر ما وصل لأن الذيل هو ما يُشخَّص به.
  'stream.dropped': 'يُعرض آخر ما وصل. أُسقط من أوّل المجرى:',

  // ── النتيجة المهيكلة ────────────────────────────────────────────────
  'result.heading': 'النتيجة',
  'result.ready': 'أصبحت النتيجة جاهزة.',
  'result.collecting': 'جارٍ جمع النتيجة…',
  'result.empty': 'لا توجد نتائج.',
  'result.silent': 'اكتملت العملية ولم تُرجع الأداة بيانات.',
  'result.partial': 'هذه نتيجة جزئية — لم يصل كامل خرج الأداة.',
  'result.fallback': 'تعذّر تنظيم النتيجة. يمكنك مراجعة الخرج الخام.',
  'result.artifact.title': 'ملف أو مجلد جاهز',
  'result.artifact.body': 'ناتج قابل للكشف في Finder.',
  'result.acknowledgement.title': 'اكتمل الإجراء',
  'result.acknowledgement.body': 'تأكيدٌ مهيكل بأن الإجراء المطلوب اكتمل.',
  'result.acknowledgement.empty': 'لا يحتاج هذا الإجراء إلى عرض بيانات إضافية.',
  'result.collection.title': 'قائمة النتائج',
  'result.collection.body': 'العناصر التي أعادتها العملية في قائمة قابلة للمراجعة.',
  'result.properties.title': 'الخصائص والتقرير',
  'result.properties.body': 'قيم وخصائص أعادتها أداة النظام عن العنصر المطلوب.',
  'result.metrics.title': 'القياسات',
  'result.metrics.body': 'قياسات رقمية أعادتها العملية من النظام.',
  'result.digest.title': 'البصمة',
  'result.digest.body': 'قيمة تحقق ثابتة يمكن نسخها أو مقارنتها.',
  'result.comparison.title': 'المقارنة',
  'result.comparison.body': 'الطرفان ونتيجة المقارنة التي حددتها النواة.',
  'result.comparison.reference': 'المرجع',
  'result.comparison.compared': 'القيمة المقارَنة',
  'result.comparison.git.reference': 'حالة Git المرجعية',
  'result.comparison.git.current': 'الحالة الحالية قيد المقارنة',
  'result.report.image.row': 'تفاصيل الصورة',
  'result.report.http_headers.row': 'ترويسة HTTP',
  'result.report.permissions.row': 'تفاصيل الصلاحيات',
  'result.report.extended_attributes.row': 'سمة ممتدة',
  'result.report.system_version.row': 'معلومة النظام',
  'result.report.system_profile.row': 'بند تقرير النظام',
  'result.metric.network_latency.row': 'قياس زمن الشبكة',
  'result.metric.system_uptime.row': 'قياس مدة التشغيل',
  'result.verdict.title': 'الحكم',
  'result.verdict.body': 'حكم دلالي من أداة النظام، لا خطأ تنفيذ عامًا.',
  'result.diff_search.title': 'المطابقات والفروق',
  'result.diff_search.body': 'المطابقات أو الفروق كما صنفتها النواة.',
  'result.raw_output.title': 'الخرج الخام',
  'result.raw_output.body': 'خرج الأداة كما وصل، حين لا توجد بنية أدق وآمنة.',
  'result.diagnostic.title': 'تفاصيل التعذّر',
  'result.diagnostic.body': 'خرج تشخيصي يساعد على فهم سبب عدم اكتمال العملية.',
  'result.contract.structured': 'نتيجة مهيكلة',
  'result.contract.raw': 'خرج خام',
  'result.status.success': 'Success',
  'result.status.info': 'Information',
  'result.status.warning': 'Warning',
  'result.status.failure': 'Failure',
  'result.status.cancelled': 'Cancelled',
  'result.status.signalled': 'Signalled',
  'result.status.core_error': 'CoreError',
  'result.partial.eyebrow': 'تحذير — نتيجة جزئية',
  'result.partial.title': 'اكتملت مع تنبيه',
  'result.partial.body': 'النتيجة قابلة للاستخدام، مع تفاصيل تحتاج مراجعة.',
  'result.actions.artifact': 'إجراءات الملف',
  'result.actions.followup': 'إجراءات المتابعة',
  'result.actions.result': 'إجراءات النتيجة',
  'result.actions.recovery': 'إجراءات الاسترداد',
  'result.copy.failed': 'تعذّر النسخ إلى الحافظة. يمكنك المحاولة مجددًا.',
  'result.equal': 'متطابقان',
  'result.unavailable': 'غير متاح',
  'result.artifact.size': 'الحجم',
  'result.artifact.entries': 'العناصر',
  'result.artifact.destination': 'الوجهة',
  'unit.byte': 'بايت',
  'unit.kib': 'كيلوبايت',
  'unit.mib': 'ميغابايت',
  'unit.gib': 'غيغابايت',
  'unit.tib': 'تيرابايت',
  'result.artifact.location.unavailable': 'الموقع غير متاح حاليًا',
  'result.execution.unavailable': 'لا تتوفر بيانات تنفيذ إضافية لهذا القيد.',
  'result.technical.redacted': 'لا تُعرض الوسائط الحساسة في التفاصيل التقنية.',
  'result.technical.run_id': 'معرّف التشغيل',
  'result.technical.status': 'حالة التنفيذ',
  'result.technical.exit_code': 'رمز الخروج',
  'result.technical.signal': 'الإشارة',
  'result.technical.executable': 'الأداة المنفّذة',
  'result.technical.arguments': 'الوسائط',
  'result.technical.arguments.redacted': 'محجوبة لحماية القيم الحساسة',
  'result.column.value': 'القيمة',
  'result.column.path': 'المسار',
  'result.column.size': 'الحجم',
  'result.column.filesystem': 'نظام الملفات',
  'result.column.used': 'المستخدم',
  'result.column.available': 'المتاح',
  'result.column.capacity': 'نسبة الاستخدام',
  'result.column.files_used': 'العُقد المستخدمة',
  'result.column.files_free': 'العُقد المتاحة',
  'result.column.files_capacity': 'نسبة العُقد المستخدمة',
  'result.column.mount': 'نقطة التركيب',
  'result.column.dns.name': 'الاسم',
  'result.column.dns.ttl': 'مدة التخزين',
  'result.column.dns.class': 'الفئة',
  'result.column.dns.type': 'نوع السجل',
  'result.column.dns.value': 'قيمة السجل',
  'result.column.git.status': 'الحالة',
  'result.column.git.current': 'الفرع الحالي',
  'result.column.git.branch': 'الفرع',
  'result.column.git.hash': 'البصمة',
  'result.column.git.date': 'التاريخ',
  'result.column.git.author': 'المؤلف',
  'result.column.git.subject': 'الرسالة',
  'result.column.content': 'المحتوى',
  'result.column.name': 'الاسم',
  'result.column.process.pid': 'معرّف العملية',
  'result.column.process.name': 'اسم العملية',
  'result.column.process.ppid': 'معرّف العملية الأب',
  'result.column.process.cpu': 'المعالج %',
  'result.column.process.memory': 'الذاكرة %',
  'result.column.process.command': 'الأمر',
  'result.property.source': 'المصدر',
  'result.property.image.pixel_width': 'العرض بالبكسل',
  'result.property.image.pixel_height': 'الارتفاع بالبكسل',
  'result.property.image.type_identifier': 'معرّف النوع',
  'result.property.image.format': 'الصيغة',
  'result.property.image.format_options': 'خيارات الصيغة',
  'result.property.image.dpi_width': 'الدقة الأفقية',
  'result.property.image.dpi_height': 'الدقة الرأسية',
  'result.property.image.samples_per_pixel': 'العينات لكل بكسل',
  'result.property.image.bits_per_sample': 'البتات لكل عينة',
  'result.property.image.has_alpha': 'قناة الشفافية',
  'result.property.image.color_space': 'فضاء اللون',
  'result.property.image.profile': 'ملف اللون',
  'result.property.http.status': 'حالة HTTP',
  'result.property.system.product_name': 'اسم النظام',
  'result.property.system.product_version': 'إصدار النظام',
  'result.property.system.build_version': 'رقم البناء',
  'result.property.git.version': 'إصدار Git',
  'result.property.file_type': 'نوع الملفّ',
  'result.property.system.architecture': 'معمارية المعالج',
  'result.metric.ping.transmitted': 'الحزم المرسلة',
  'result.metric.ping.received': 'الحزم المستلمة',
  'result.metric.ping.packet_loss': 'فقد الحزم',
  'result.metric.ping.minimum': 'أقل زمن',
  'result.metric.ping.average': 'متوسط الزمن',
  'result.metric.ping.maximum': 'أعلى زمن',
  'result.metric.ping.stddev': 'الانحراف المعياري',
  'result.ack.opened': 'أُرسل العنصر إلى التطبيق الافتراضي بنجاح.',
  'result.ack.repository_initialized': 'أُنشئ مستودع Git في المسار المحدد.',
  'result.ack.commit_created': 'أُنشئ تسجيل Git بالقيم التي راجعتها.',
  'result.ack.dns_flushed': 'أُفرغت ذاكرة DNS للنظام.',
  // «أُرسلت» لا «أُنهيت»: الأمر ينجح حين تُرسَل الإشارة لا حين تموت العملية.
  'result.ack.signal_sent': 'أُرسلت إشارة الإنهاء إلى العملية.',
  'result.ack.typecheck_passed': 'اجتاز فحص الأنواع بلا أخطاء.',
  'result.ack.lint_passed': 'اجتاز فحص الأسلوب بلا أخطاء.',
  'result.ack.tests_passed': 'اكتملت الاختبارات بنجاح.',
  'result.ack.packages_installed': 'ثُبّتت حزم المشروع.',
  'result.ack.dev_server_stopped': 'توقّف خادم التطوير.',
  'result.ack.tauri_build_completed': 'اكتمل بناء تطبيق Tauri.',
  'result.ack.cargo_tests_passed': 'اكتملت اختبارات Rust بنجاح.',
  'result.ack.cargo_check_passed': 'اجتاز فحص البناء بلا أخطاء.',
  'result.ack.cargo_clippy_passed': 'اجتاز فحص Clippy بلا تحذيرات.',
  'result.ack.cargo_fmt_check_passed': 'التنسيق مطابقٌ للمعيار.',
  'result.ack.cargo_fmt_applied': 'أُعيد تنسيق المشروع.',
  'result.ack.cargo_build_completed': 'اكتمل بناء الإصدار.',
  'result.ack.cargo_cleaned': 'حُذفت نواتج البناء.',
  'result.technical': 'عرض التفاصيل التقنية',
  'result.technical.hide': 'إخفاء التفاصيل التقنية',
  'result.execution.data': 'بيانات التنفيذ',
  'result.execution.signalled': 'توقفت العملية بإشارة',
  'result.execution.signalled.body': 'أنهى النظام العملية قبل اكتمالها.',
  'result.execution.core_error': 'تعذر بدء التنفيذ الآمن',
  'result.execution.core_error.body': 'رفضت طبقة التشغيل الطلب قبل تشغيل الأمر.',
  'result.semantic.completed': 'اكتملت العملية بنجاح',
  'result.semantic.completed.body': 'أصبحت النتيجة جاهزة.',
  'result.semantic.matches': 'وُجدت نتائج مطابقة',
  'result.semantic.matches.body': 'أعادت الأداة عناصر تطابق الطلب.',
  'result.semantic.no_matches': 'لا توجد نتائج مطابقة',
  'result.semantic.no_matches.body': 'اكتمل البحث ولم يعثر على عنصر يطابق الطلب.',
  'result.semantic.differences': 'توجد فروق',
  'result.semantic.differences.body': 'اكتملت المقارنة وحددت الأداة فروقًا بين الطرفين.',
  'result.semantic.no_differences': 'لا توجد فروق',
  'result.semantic.no_differences.body': 'اكتملت المقارنة ولم تجد الأداة فرقًا بين الطرفين.',
  // نصوص الأحكام (`Verdict`) مصنَّفةٌ بمعرّف نوعها (`VerdictKind`) لا بقيمتها
  // وحدها: نفس القيمة `accepted`/`rejected` تعني قرار سياسة macOS من
  // `security.gatekeeper`، وتعني تحقّقًا تشفيريًا من `security.codesign.verify`،
  // وتعني سلامة أرشيفٍ من `compress.zip.test` — ثلاثة معانٍ لا معنًى واحد.
  // نصٌّ عامٌّ واحد لكل القيم كان يعرض «مقبول وفق سياسة macOS» عن أرشيفٍ سليم
  // وعن توقيعٍ صالحٍ تشفيريًا معًا — راجع H-1 في تدقيق الفرع.
  'result.semantic.gatekeeper.accepted': 'مقبول وفق سياسة macOS',
  'result.semantic.gatekeeper.accepted.body': 'سمحت سياسة النظام بالعنصر الذي جرى تقييمه.',
  'result.semantic.gatekeeper.rejected': 'مرفوض وفق سياسة macOS',
  'result.semantic.gatekeeper.rejected.body': 'رفضت سياسة النظام العنصر الذي جرى تقييمه.',
  'result.semantic.code_signature.signed': 'التوقيع موجود وصالح للقراءة',
  'result.semantic.code_signature.signed.body': 'أعادت أداة النظام معلومات توقيع العنصر.',
  'result.semantic.code_signature.unsigned': 'لا يوجد توقيع صالح',
  'result.semantic.code_signature.unsigned.body':
    'اكتمل الفحص ولم تجد أداة النظام توقيعًا صالحًا.',
  // `security.codesign.verify` فحصٌ تشفيري — هل يطابق التوقيع محتوى العنصر
  // فعلًا؟ — لا تقييم سياسة، ولذلك نصّه مختلفٌ عمدًا عن نصّ Gatekeeper أعلاه
  // رغم مشاركتهما القيمتين `accepted`/`rejected` نفسيهما.
  'result.semantic.code_integrity.accepted': 'التوقيع صالحٌ وسليم',
  'result.semantic.code_integrity.accepted.body':
    'تحقّقت أداة النظام من مطابقة التوقيع لمحتوى العنصر، ولم تجد فيه تلاعبًا.',
  'result.semantic.code_integrity.rejected': 'التوقيع غير سليم',
  'result.semantic.code_integrity.rejected.body':
    'لم تتحقّق أداة النظام من مطابقة التوقيع لمحتوى العنصر — قد يكون معطوبًا، أو عُدِّل العنصر بعد توقيعه.',
  'result.semantic.archive_integrity.accepted': 'الأرشيف سليم',
  'result.semantic.archive_integrity.accepted.body':
    'اختبرت الأداة كل عنصرٍ داخل الأرشيف ولم تجد فيه عطبًا.',
  'result.semantic.archive_integrity.rejected': 'الأرشيف تالف',
  'result.semantic.archive_integrity.rejected.body':
    'وجدت الأداة عطبًا في الأرشيف أثناء اختباره — قد يكون غير مكتمل أو معطوب البيانات.',
  'result.semantic.failed': 'تعذر إكمال العملية',
  'result.semantic.failed.body': 'راجع التشخيص ثم صحّح المدخلات أو أعد التشغيل.',
  'result.semantic.cancelled': 'أُلغيت العملية',
  'result.semantic.cancelled.body': 'توقفت العملية بطلب منك ولم تُعرض كنتيجة ناجحة.',

  'naffith.subtitle': 'املأ الحقول، وراجع ما سيحدث قبل أن يحدث.',

  // ── سَطْر ────────────────────────────────────────────────────────────
  'satr.title': 'الأمر الذي سيُنفَّذ',
  'satr.subtitle': 'هذا هو الأمر نفسه، لا تمثيلٌ له.',
  'satr.no_shell':
    'لا يشغّل التطبيق صدفة. البرنامج ووسائطه تُمرَّر إلى النظام مصفوفةً منفصلة، فالمسافات وعلامات الاقتباس و⁦$⁩ و⁦;⁩ داخل الأسماء محارف عادية لا يفسّرها شيء.',
  'satr.copy_note':
    'نسخة Terminal تُضاف إليها علامات اقتباس لأن الصدفة تحتاجها. التطبيق لا يستعمل هذه النسخة، ولا يمكن تعديل الأمر داخله.',
  'satr.promotion':
    'بعد خروج الأمر بنجاح، يرقّي التطبيق الملف المؤقّت إلى اسمه النهائي داخل المجلد نفسه — بربط صلب ثم حذف المؤقّت. وعلى أنظمة ملفات لا تعرف الروابط الصلبة، مثل ⁦exFAT⁩ في ذاكرات USB، يُحجز الاسم النهائي أولًا ثم يُنقل المؤقّت فوق الحجز. وفي الحالتين لا يُستبدل ملف قائم.',
  'satr.arg': 'وسيط',
  'satr.legend.tool': 'الأداة',
  'satr.legend.flag': 'راية',
  'satr.legend.path': 'مسار',
  // وسوم أقسام اللوحة. الأمر يُعرض مفتوحًا دائمًا، وما دونه يُطوى: قائمةُ
  // الوسائط شرحُ كل وسيط فيها مطويّ، والملاحظات قسمٌ مطويّ كلّه.
  'satr.args.heading': 'الوسائط، وسيطًا وسيطًا',
  'satr.notes.heading': 'ملاحظات تقنية',
  // حالة اللوحة المطوية: الاسم وهذا السطر، لا أكثر. شريطٌ بعرض ٩٦ بكسلًا لا
  // يحمل تعليمات، وحضورُ الحالة فيه هو ما يقول إن اللوحة تنتظر شيئًا لم يأتِ.
  'satr.idle.title': 'لا أمر بعد',
  'satr.state.planned': 'الخطة جاهزة',
  'satr.state.running': 'قيد التشغيل',
  'satr.state.succeeded': 'اكتمل التنفيذ',
  'satr.state.failed': 'تعذّر التنفيذ',
  'satr.action.cancel': 'إلغاء التشغيل',
  'satr.command.empty': 'لا يوجد أمر لعرضه بعد.',
  'satr.command.suspicious': 'تحتوي قيمة على مسافة؛ ستُمرَّر إلى الأداة وسيطًا واحدًا آمنًا.',
  'stream.waiting.title': 'بانتظار المخرجات',
  'stream.waiting.meta': 'لم يبدأ البث بعد',
  'stream.waiting.body': 'سيظهر خرج العملية هنا عند بدء التنفيذ.',
  'stream.waiting.state': 'قيد الانتظار',
  'stream.state.waiting.title': 'بانتظار المخرجات',
  'stream.state.waiting.meta': 'لم يبدأ البث بعد',
  'stream.state.waiting.body': 'سيظهر خرج العملية هنا عند بدء التنفيذ.',
  'stream.state.waiting.footer': 'قيد الانتظار',
  'stream.state.stdout.title': 'المخرجات',
  'stream.state.stdout.meta': 'stdout · مباشر',
  'stream.state.stderr.title': 'رسالة تشخيص',
  'stream.state.stderr.meta': 'stderr · مباشر',
  'stream.state.silent.title': 'اكتمل بلا مخرجات نصية',
  'stream.state.silent.meta': 'لا يوجد stdout أو stderr',
  'stream.state.silent.body': 'لا تحتاج هذه العملية إلى عرض سجل نصي.',
  'stream.state.truncated.title': 'مخرجات مقتطعة',
  'stream.state.truncated.meta': 'تم حفظ آخر 64 KB',
  'stream.state.truncated.footer': 'عرض آخر جزء متاح — انسخ السجل عند الحاجة',
  'stream.state.dropped.title': 'تعذّر الاحتفاظ بكامل البث',
  'stream.state.dropped.meta': 'فُقدت أحداث أثناء التدفق',
  'stream.state.dropped.body': 'استمرت العملية، لكن بعض المقاطع لم تصل إلى الواجهة.',
  'stream.state.dropped.footer': 'النتيجة النهائية تظل المصدر الأساسي للحالة',

  // ── شرح الأمر ────────────────────────────────────────────────────────
  'explain.ditto.tool':
    'أداة النسخ والأرشفة الأصلية في macOS. اختيرت على ⁦zip⁩ لأنها تحفظ resource forks والسمات الممتدة التي يُسقطها ⁦zip⁩ صامتًا، وهي نفسها ما يستعمله Finder عند «ضغط».',
  'explain.ditto.create': 'أنشئ أرشيفًا في المسار الأخير بدل أن تنسخ الملفات.',
  'explain.ditto.pkzip': 'اجعل الأرشيف بصيغة ZIP لا CPIO. بدونها ينتج ملف CPIO باسم ينتهي بـ ⁦.zip⁩.',
  'explain.ditto.sequester':
    'احفظ بيانات macOS الوصفية داخل مجلد ⁦__MACOSX⁩ في الأرشيف، وهي الطريقة القياسية التي تفهمها أدوات فكّ ZIP على كل الأنظمة.',
  'explain.ditto.keep_parent':
    'ضع اسم المجلد المصدر مجلدًا أعلى داخل الأرشيف، فلا ينثر الفكُّ عشرات الملفات في المجلد الحالي.',
  'explain.echo.cmd': 'يطبع نصًا. للاختبار الداخلي.',
  'explain.role.source': 'المجلد المصدر، بمساره المطلق بعد حلّ الروابط الرمزية.',
  'explain.role.temp': 'الملف المؤقّت. يُرقّى إلى اسمه النهائي بعد النجاح وحده.',
  'explain.role.temp_dir':
    'المجلد المؤقّت. يُنشأ حصريًا قبل التشغيل، ويُنقل إلى اسمه النهائي بعد النجاح وحده — فاستخراجٌ انقطع في المنتصف لا يترك شجرةً ناقصة تبدو تامّة.',
  'explain.end_of_flags':
    'نهاية الرايات. كل ما بعده يُقرأ بياناتٍ مهما بدا شكله، فاسمٌ يبدأ بشرطة لا يُقرأ خيارًا.',

  // ── الضغط وفكّ الضغط ─────────────────────────────────────────────────
  'explain.ditto.rsrc': 'انسخ resource forks مع الملفات بدل أن تُسقطها.',
  'explain.ditto.extattr': 'انسخ السمات الممتدّة كذلك — الوسوم والألوان وبيانات التطبيقات.',
  'explain.ditto.extract': 'استخرج من الأرشيف بدل أن تنشئه.',
  'explain.ditto.pkzip_read':
    'اقرأ المصدر أرشيفَ ZIP لا شجرةَ ملفات. وهي التي تفهم ⁦__MACOSX⁩ فتعيد البيانات الوصفية إلى مكانها بدل أن تنثر مجلدًا غريبًا.',
  'explain.unzip.tool':
    'أداة ZIP الأصلية في macOS. تُستعمل هنا للقراءة وحدها — عرض الفهرس واختبار السلامة — أمّا الاستخراج فبـ⁦ditto⁩ لأنها تفهم بيانات macOS الوصفية وتقرأ Zip64 صحيحًا.',
  'explain.unzip.list': 'اعرض فهرس الأرشيف: الأسماء والأحجام والتواريخ. لا يُكتب شيء على القرص.',
  'explain.unzip.test':
    'فكّ كل مدخلة في الذاكرة وقارن CRC-32 المحسوب بالمخزَّن. يكشف التلف في النقل أو على القرص، ولا يقول شيئًا عن صحّة المحتوى.',
  'explain.tar.tool':
    'أداة الأرشفة الأصلية (‏bsdtar). تجمع الشجرة ثم تضغط الكتلة كلها، فتخرج أصغر من ZIP على شجرة شيفرة أو نصوص، وهي الصيغة التي تتوقّعها الخوادم.',
  'explain.tar.create': 'أنشئ أرشيفًا جديدًا.',
  'explain.tar.extract':
    'استخرج من الأرشيف. بلا راية المسارات المطلقة تُسقط الأداة الشرطة الأولى وترفض أي مدخلة فيها ⁦..⁩.',
  'explain.tar.list': 'اعرض فهرس الأرشيف ولا تكتب شيئًا.',
  'explain.tar.verbose': 'أضف الصلاحيات والمالك والحجم والتاريخ إلى كل سطر بدل الاسم وحده.',
  'explain.tar.gzip': 'اضغط الأرشيف بـ⁦gzip⁩.',
  'explain.tar.file': 'الملف الذي يُقرأ منه أو يُكتب إليه، بدل المجرى القياسي.',
  'explain.tar.directory':
    'انتقل إلى هذا المجلد قبل الأرشفة، فيدخل المصدر باسمه المجرّد ولا يحمل الأرشيف مسارات جهازك في داخله.',
  'explain.tar.into': 'استخرج داخل هذا المجلد.',

  // ── التحذيرات ────────────────────────────────────────────────────────
  // صياغةٌ عامّة لا صياغةُ الضغط: كانت تقول «سيُضغط الموضع…»، وهي جملةٌ صارت
  // كذبًا في أول عمليةِ نسخٍ أو بحثٍ تستعمل المفتاح نفسه. والمعنى الذي يجب أن
  // يبلغ المستخدم واحدٌ في كل عملية: ما اخترته ليس ما سيُعمل عليه.
  'warn.source.resolved':
    'ما اخترته رابط رمزي. ستعمل العملية على الموضع الذي يشير إليه، وهو المسار الظاهر في الأمر أدناه.',
  'warn.destination.resolved':
    'مجلد الوجهة رابط رمزي. سيُكتب الناتج في الموضع الذي يشير إليه، لا في مسار الرابط.',
  'warn.source.empty': 'المجلد المصدر فارغ. سيُنشأ أرشيف لا يحوي ملفات.',
  'warn.source.appledouble':
    'في المجلد ملفات تبدأ أسماؤها بـ ⁦._⁩. تعاملها ditto سِجلاتٍ مصاحبة لا ملفات، فلن تدخل الأرشيف ولن يمكن استعادتها منه.',
  'warn.size.partial':
    'المجلد كبير، فقُدِّر حجمه تقديرًا ناقصًا حتى لا تتجمّد الواجهة. اعتبر تقدير المساحة إرشاديًا.',
  'warn.size.zip_limit':
    'المجلد يتجاوز ما تسعه صيغة ZIP التي تنتجها ditto: ٤ غيغابايت من الحجم، أو نحو ٣٢ ألف عنصر. وقد يُقرأ الأرشيف على macOS ويظهر معطوبًا على غيرها.',
  'warn.zip.name_encoding':
    'تُكتب الأسماء العربية في الأرشيف بلا راية الترميز التي تنتظرها أدوات فكّ الضغط خارج macOS، فقد تظهر لديها مشوّهة. المحتوى نفسه سليم.',
  'warn.space.low':
    'حجم المجلد يتجاوز المساحة الحرة في الوجهة. قد ينجح الضغط لأن ZIP يقلّص الحجم — وقد لا ينجح.',
  'warn.space.unknown': 'تعذّر قياس المساحة الحرة في الوجهة.',
  'warn.archive.symlinks':
    'يحوي الأرشيف روابط رمزية. تُستخرَج داخل مجلدٍ جديد يخصّه وحده، فلا تكتب خارجه — لكن راجع إلى أين تشير قبل أن تنقل الشجرة.',
  'warn.archive.empty': 'الأرشيف لا يحوي مدخلة واحدة.',
  'warn.archive.escapes':
    'في الأرشيف مدخلة مسارها يخرج من جذره — مسار مطلق أو ⁦..⁩. العرض آمن، والاستخراج مرفوض.',
  'warn.archive.partial_scan':
    'الأرشيف أكبر مما يفحصه التطبيق قبل التشغيل، فالفحص ناقص. العرض يعمل، والاستخراج مرفوض حتى لا يمرّ ما لم يُفحص.',
  'warn.archive.slow_test':
    'الاختبار يفكّ كل بايت في الذاكرة، فزمنه زمن استخراج كامل. الأرشيف كبير، والانتظار مقصود.',
  'warn.tar.no_macos_metadata':
    'لا يحفظ ⁦tar.gz⁩ بيانات macOS الوصفية — resource forks والسمات الممتدّة والوسوم. إن كانت تعنيك فاستعمل ZIP.',
  'warn.tar.no_pre_scan':
    'لا يقرأ التطبيق فهرس أرشيفات TAR قبل التشغيل كما يفعل مع ZIP. الحماية هنا من الأداة نفسها ومن الاستخراج داخل مجلد معزول يُنقل بعد النجاح وحده.',

  // ── الأخطاء ──────────────────────────────────────────────────────────
  'err.op.unknown': 'عملية غير معروفة.',
  'err.op.unavailable': 'هذه العملية غير متاحة في هذا الإصدار.',
  'err.input.missing': 'هذا الحقل مطلوب.',
  'err.input.type': 'قيمة هذا الحقل من نوع غير متوقّع.',
  'err.input.unexpected': 'أُرسل مدخل لا تعرفه هذه العملية.',
  'err.name.invalid': 'اسم غير صالح.',
  'err.name.invalid.empty': 'الاسم فارغ.',
  'err.name.invalid.too_long':
    'الاسم أطول مما يقبله نظام الملفات: الحدّ ٢٥٥ حرفًا، ويُحتسب الرمز التعبيري حرفين، ويدخل الامتداد في العدّ. اختصره قليلًا.',
  'err.name.invalid.contains_separator': 'لا يجوز أن يحتوي الاسم على ⁦/⁩.',
  'err.name.invalid.contains_nul': 'الاسم يحتوي محرفًا غير مسموح.',
  'err.name.invalid.contains_control': 'الاسم يحتوي محارف تحكّم.',
  'err.name.invalid.dot_or_dot_dot': 'لا يصلح ⁦.⁩ ولا ⁦..⁩ اسمًا لملف.',
  'err.name.invalid.leading_dot': 'اسم يبدأ بنقطة يُنشئ ملفًا مخفيًا لن تجده.',
  'err.name.invalid.trailing_space_or_dot': 'لا يجوز أن ينتهي الاسم بمسافة أو نقطة.',
  'err.path.relative': 'المسار يجب أن يكون مطلقًا.',
  'err.path.traversal': 'المسار يحتوي ⁦..⁩ ولا يُقبل.',
  'err.path.outside': 'هذا الموضع خارج ما يعمل عليه التطبيق: مجلد المنزل والأقراص المركّبة فقط.',
  'err.path.protected': 'هذا موضع محميّ (مفاتيح أو بيانات اعتماد) ولا يُقرأ.',
  'err.path.missing': 'لا يوجد شيء في هذا المسار.',
  'err.path.not_dir': 'هذا ليس مجلدًا.',
  'err.dest.exists': 'يوجد ملف بهذا الاسم في الوجهة. اختر اسمًا آخر — لا نستبدل ولا ننحّي جانبًا.',
  'err.dest.readonly': 'لا يمكن الكتابة في هذا المجلد.',
  'err.dest.inside_source': 'الوجهة داخل المجلد المصدر، فسيدخل الأرشيف في نفسه. اختر وجهة خارجه.',
  'err.source.inside_dest': 'المصدر داخل الوجهة، فالعملية تبتلع نفسها. اختر أحدهما خارج الآخر.',
  'err.path.same': 'المصدر والوجهة الموضع نفسه بعد حلّ الروابط الرمزية.',
  'err.input.range': 'القيمة خارج المدى المسموح.',
  'err.input.url': 'أدخل عنوانًا يبدأ بـ ⁦http://⁩ أو ⁦https://⁩ بلا فراغات.',
  'err.archive.escapes':
    'في الأرشيف مدخلة تخرج من مجلد الاستخراج — مسار مطلق أو ⁦..⁩. رُفض قبل تشغيل أي شيء. استعرض محتوياته أولًا لتعرف ما فيه.',
  'err.archive.unreadable': 'تعذّرت قراءة فهرس الأرشيف: إمّا أنه تالف، أو ليس بالصيغة المتوقّعة.',
  'err.journal.not_found': 'لا يوجد في السجل قيد بهذا المعرّف.',
  'err.redirect': 'خللٌ داخلي: حاولت العملية توجيه خرجها خارج خطتها. لم يُنفَّذ شيء.',
  'err.arg.flag': 'قيمةٌ تبدأ بشرطة كانت ستُقرأ خيارًا للأداة. غيّرها ولا تبدأها بـ ⁦-⁩.',
  'err.reveal.nothing': 'لا يوجد ناتج لهذا التشغيل يمكن إظهاره.',
  'err.reveal.failed': 'تعذّر إظهار الناتج في Finder. ربما نُقل أو حُذف منذ انتهاء التشغيل.',
  'err.picker.failed': 'تعذّر فتح نافذة الاختيار. يمكنك كتابة المسار أو لصقه في الحقل.',
  'err.tool.missing': 'أداة النظام المطلوبة غير موجودة.',
  'err.tool.not_exec': 'أداة النظام المطلوبة غير قابلة للتنفيذ.',
  'err.plan.not_found': 'انتهت صلاحية الخطة أو استُهلكت. أعد المحاولة.',
  'err.plan.stale': 'تغيّر شيء منذ إعداد الخطة، فأُوقفت قبل التنفيذ.',
  'err.plan.stale.source_gone': 'لم يعد المجلد المصدر موجودًا.',
  'err.plan.stale.source_replaced': 'استُبدل المجلد المصدر بغيره.',
  'err.plan.stale.destination_gone': 'لم يعد مجلد الوجهة موجودًا.',
  'err.plan.stale.destination_not_writable': 'لم تعد الكتابة ممكنة في مجلد الوجهة.',
  'err.plan.stale.final_path_appeared': 'ظهر ملف بالاسم النهائي بعد إعداد الخطة.',
  'err.plan.stale.tool_gone': 'لم تعد أداة النظام متاحة.',
  'err.plan.stale.temp_path_taken':
    'شُغل الموضع المؤقّت الذي حجزته الخطة. أعد المحاولة — سيُختار اسمٌ مؤقّت آخر.',
  'err.plan.limit': 'عدد الخطط المفتوحة بلغ حدّه. أعد المحاولة بعد قليل.',
  'err.run.not_found': 'لا يوجد تشغيل بهذا المعرّف.',
  'err.io': 'تعذّرت العملية على نظام الملفات.',
  'err.spawn': 'تعذّر تشغيل الأداة.',
  'err.wait': 'انقطع انتظار الأداة.',
  'err.output.empty': 'خرجت الأداة بنجاح دون أن تُنتج ملفًا.',
  'err.commit': 'تعذّرت ترقية الملف المؤقّت إلى اسمه النهائي، فلم تُسجَّل العملية ناجحة.',
  'err.unknown': 'حدث خطأ غير متوقّع.',
  'err.dialog': 'تعذّر فتح نافذة اختيار المجلد.',

  // ── الملفات: النسخ ───────────────────────────────────────────────────
  'op.files.copy.title': 'نسخ ملف أو مجلد',
  'op.files.copy.description': 'ينسخ ملفًا أو مجلدًا إلى مكان آخر.',
  'op.files.copy.execution':
    'تُنشأ نسخة جديدة في الوجهة باسمٍ تختاره، ببيانات macOS الوصفية كاملة. الأصل لا يُمسّ. وإن كان الاسم مأخوذًا تتوقّف العملية ولا يُستبدل شيء.',
  'op.files.copy.result': 'تمّ النسخ',
  'explain.ditto.rsrc.copy': 'انسخ resource forks مع الملفات بدل أن تُسقطها.',

  // ── Git ──────────────────────────────────────────────────────────────
  'op.git.init.title': 'إنشاء مستودع Git',
  'op.git.init.description': 'يُهيّئ مجلدًا قائمًا ليصير مستودع Git.',
  'op.git.init.execution':
    'يُنشأ مجلد ⁦.git⁩ داخل المجلد الذي تختاره. ملفاتك لا تُمسّ ولا تُسجَّل. ويُرفض إن كان المجلد مستودعًا بالفعل، فلا يُعاد ضبط شيء بالخطأ.',
  'op.git.init.result': 'أُنشئ المستودع',
  'op.git.status.title': 'عرض حالة المستودع',
  'op.git.status.description': 'يعرض الفرع الحالي والملفات المعدَّلة وغير المتتبَّعة.',
  'op.git.status.execution': 'قراءةٌ فقط: لا يُدرَج شيء ولا يُسجَّل ولا يُغيَّر.',
  'op.git.commit.title': 'تسجيل commit برسالة',
  'op.git.commit.description': 'يسجّل تعديلات الملفات المتتبَّعة في commit جديد.',
  'op.git.commit.execution':
    'يُضاف commit إلى تاريخ الفرع الحالي. الملفات نفسها لا تتغيّر. والملفات الجديدة لا تدخل: تحتاج إدراجًا لا تفعله هذه العملية.',
  'op.git.commit.result': 'سُجّل التغيير',
  'op.git.diff.title': 'مقارنة التغييرات الحالية',
  'op.git.diff.description': 'يعرض ملخّص ما تغيّر في المستودع.',
  'op.git.diff.execution':
    'قراءةٌ فقط: لا يُدرَج شيء ولا يُسجَّل ولا يُعدَّل ملف. ووجود الفروق وعدمها جوابان سليمان.',
  'op.git.branches.merged.title': 'الفروع المحلية المدمجة',
  'op.git.branches.merged.description': 'يعرض الفروع التي دُمجت في الفرع الحالي.',
  'op.git.branches.merged.execution':
    'قراءةٌ فقط: لا يُحذف فرع ولا يتغيّر شيء في المستودع. والحذف قرارك وحدك بعد أن ترى القائمة.',
  'op.git.archive.title': 'إنشاء أرشيف من نسخة محدّدة',
  'op.git.archive.description': 'يُصدّر محتوى المستودع عند نسخة تحدّدها في أرشيف ZIP.',
  'op.git.archive.execution':
    'يُنشأ ملف ZIP جديد في الوجهة. المستودع لا يُمسّ ولا تتغيّر نسخته الحالية. ويحوي الأرشيف ما هو مسجَّل في تلك النسخة وحدها دون تعديلاتك غير المسجَّلة.',
  'op.git.archive.result': 'تم إنشاء الأرشيف',
  'op.git.log.title': 'سجلّ تسجيلات مستودع Git',
  'op.git.log.description': 'يسرد آخر تسجيلات المستودع: البصمة والتاريخ والمؤلف وعنوان الرسالة.',
  'op.git.log.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المستودع. يعرض العدد المحدَّد من آخر التسجيلات على الفرع الحالي.',
  'op.git.diff.commits.title': 'مقارنة بين مرجعين في تاريخ المستودع',
  'op.git.diff.commits.description':
    'يقارن التعديلات بين مرجعين اختارهما المستخدم في تاريخ المستودع.',
  'op.git.diff.commits.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المستودع. ملخّصٌ للملفّات التي تغيّرت بين المرجعين وعدد أسطرها، لا الفرق الكامل.',
  'op.git.show.file.title': 'عرض ملفّ من تاريخ المستودع',
  'op.git.show.file.description':
    'يعرض محتوى ملفٍّ كما كان عند مرجعٍ معيّن، حتى لو حُذف أو أُعيدت تسميته لاحقًا.',
  'op.git.show.file.execution':
    'قراءةٌ فقط: لا يُنشأ ملفّ ولا يتغيّر شيء على القرص. المحتوى المعروض نسخةٌ تاريخية، لا الملف كما هو في مساحة العمل الآن.',
  'op.git.blame.title': 'من كتب كل سطر في ملفّ',
  'op.git.blame.description': 'يُسنِد كل سطرٍ في ملفٍّ إلى آخر commit عدّله ومن كتبه.',
  'op.git.blame.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المستودع. قد يستغرق لحظاتٍ في ملفٍّ طويل التاريخ.',
  'op.git.grep.title': 'بحث داخل ملفّات مستودع Git',
  'op.git.grep.description': 'يبحث عن نصٍّ حرفي داخل الملفّات المتتبَّعة في المستودع فقط.',
  'op.git.grep.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المستودع. يتجاهل ما يتجاهله ⁦.gitignore⁩ تلقائيًا.',
  'op.git.version.title': 'إصدار Git',
  'op.git.version.description': 'يعرض رقم النسخة المثبَّتة من Git على هذا الجهاز.',
  'op.git.version.execution':
    'قراءةٌ فقط: بلا اتصال بالشبكة وبلا مقارنة بأحدث إصدار منشور.',
  'explain.git.tool':
    'أداة Git الرسمية. تأتي مع أدوات سطر الأوامر في Xcode، وقد تغيب عن نظام لم تُثبَّت فيه — ولذلك تظهر عمليات هذا القسم معطّلة حينها بسببٍ مسمّى.',
  'explain.git.dash_c':
    'اعمل داخل هذا المستودع. التطبيق لا يغيّر مجلد عمله أبدًا، فالمستودع يُمرَّر بمساره المطلق بدل الانتقال إليه.',
  'explain.git.init': 'أنشئ مستودعًا جديدًا في المجلد المحدّد.',
  'explain.git.status': 'اعرض حالة الملفات في مساحة العمل.',
  'explain.git.status.short': 'صيغة مختصرة: سطر لكل ملف برمزين يقولان حالته.',
  'explain.git.status.branch': 'أضف سطر الفرع الحالي وعلاقته بالفرع البعيد.',
  'explain.git.commit': 'سجّل لقطة من الحالة الحالية في تاريخ المستودع.',
  'explain.git.commit.all':
    'أدرِج التعديلات والحذوفات في الملفات المتتبَّعة تلقائيًا. لا تشمل الملفات الجديدة — تلك تحتاج إدراجًا صريحًا.',
  'explain.git.commit.message': 'نصّ الرسالة كما كتبته، وسيطًا واحدًا مهما حوى من رموز.',
  'explain.git.diff': 'قارن مساحة العمل بآخر لقطة مسجَّلة.',
  'explain.git.diff.stat': 'ملخّص بالأرقام بدل الفروق سطرًا سطرًا: ما الملفات وكم تغيّر فيها.',
  'explain.git.diff.exit_code':
    'اطلب من Git تمييز «لا فروق» عن «توجد فروق» برمز الخروج؛ كلاهما جواب ناجح في التطبيق.',
  'explain.git.diff.staged': 'قارن ما أُدرج للتسجيل بدل ما في مساحة العمل.',
  'explain.git.branch': 'تعامل مع الفروع المحلية.',
  'explain.git.branch.merged': 'اعرض ما دُمج في الفرع الحالي وحده — أي ما يمكن حذفه بأمان.',
  'explain.git.archive': 'أنشئ أرشيفًا من محتوى نسخة مسجَّلة.',
  'explain.git.archive.format': 'الصيغة: ZIP.',
  'explain.git.archive.output': 'اكتب الأرشيف في هذا المسار بدل الخرج القياسي.',
  'explain.git.revision':
    'المرجع المطلوب: اسم فرع، أو وسم، أو بصمة commit، أو HEAD لآخر لقطة على الفرع الحالي.',
  'explain.git.log': 'اسرد التسجيلات.',
  'explain.git.log.format':
    'الحقول المعروضة لكل تسجيل: البصمة، والتاريخ، والمؤلف، وعنوان الرسالة.',
  'explain.git.log.date': 'صيغة التاريخ: سنة-شهر-يوم.',
  'explain.git.log.limit': 'أقصى عدد التسجيلات المعروضة.',
  'explain.git.diff.from': 'نقطة البداية في المقارنة.',
  'explain.git.diff.to': 'نقطة النهاية في المقارنة.',
  'explain.git.show': 'اعرض محتوى ملفٍّ كما كان عند مرجعٍ في تاريخ المستودع.',
  'explain.git.show.target':
    'المرجع والمسار معًا: النقطتان الفاصلتان بينهما جزءٌ من صيغة Git نفسها، لا رمزٌ يضيفه التطبيق.',
  'explain.git.blame': 'أسنِد كل سطرٍ في الملفّ إلى آخر commit عدّله.',
  'explain.git.blame.porcelain':
    'أخرِج كل حقلٍ في سطره الخاص بعنوانٍ صريح، بدل صيغةٍ مضغوطة يصعب تحليلها بثقة.',
  'explain.git.grep': 'ابحث داخل الملفّات المتتبَّعة في المستودع.',
  'explain.git.grep.line_numbers': 'أظهر رقم السطر أمام كل نتيجة.',
  'explain.git.grep.fixed_string': 'عامِل النمط نصًّا حرفيًّا لا تعبيرًا نمطيًّا.',
  'explain.git.grep.ignore_case': 'تجاهل الفرق بين الأحرف الكبيرة والصغيرة.',
  'explain.git.grep.pattern_flag': 'ما يلي هذه الراية نمط البحث، لا رايةً أخرى.',
  'explain.git.grep.pattern': 'النصّ الذي يبحث عنه المستخدم.',
  'explain.git.version': 'اطلب رقم إصدار Git المثبَّت.',
  'warn.git.repo.resolved': 'مجلد المستودع رابط رمزي. سيعمل الأمر على الموضع الذي يشير إليه.',
  'warn.git.folder.resolved': 'المجلد المختار رابط رمزي. سيُنشأ المستودع في الموضع الذي يشير إليه.',
  'warn.git.untracked':
    'لا تدخل الملفات الجديدة في هذا الـcommit: الراية المستعملة تُدرج التعديلات والحذوفات في الملفات المتتبَّعة فقط. أدرِج الجديد أولًا إن أردته.',
  'warn.git.merged.head':
    'المقارنة بالفرع الحالي. الفرع الذي تقف عليه الآن يظهر في القائمة وهو ليس مرشّحًا للحذف.',
  'warn.git.archive.committed_only':
    'يحوي الأرشيف ما هو مسجَّل في النسخة المطلوبة فقط. التعديلات غير المسجَّلة والملفات غير المتتبَّعة لن تدخل فيه.',

  // ── النظام والصيانة ──────────────────────────────────────────────────
  'op.system.process.find.title': 'البحث عن عملية بالاسم',
  'op.system.process.find.description': 'يعرض العمليات الجارية التي يطابق اسمُها ما تكتبه.',
  'op.system.process.find.execution':
    'قراءةٌ فقط: لا تُنهى عملية ولا تتغيّر أولويّتها. والمطابقة على اسم العملية وحده لا على سطر أوامرها، فلا تظهر مسارات ما تفتحه. وعدمُ العثور جوابٌ مكتمل لا فشل.',
  'op.system.process.open_files.title': 'الملفّات التي تفتحها عملية',
  'op.system.process.open_files.description':
    'يعرض الملفّات والمقابس التي تفتحها عمليةٌ برقمها.',
  'op.system.process.open_files.execution':
    'قراءةٌ فقط: لا يُغلق مقبض ولا تُمسّ العملية. و«لا نتيجة» جوابٌ لا خطأ، لكنه لا يفرّق بين ثلاث حالات: رقمٌ لا وجود له، وعمليةٌ قائمة لا تملكها، وعمليةٌ لا تفتح شيئًا.',
  'op.system.process.kill.title': 'إنهاء عملية برقمها',
  'op.system.process.kill.description': 'يرسل إشارة الإنهاء اللطيفة إلى عمليةٍ واحدة برقمها.',
  'op.system.process.kill.execution':
    'تُرسَل إشارة إنهاءٍ إلى عمليةٍ واحدة بعينها. ما لم يُحفظ فيها يضيع ولا تراجع. والإرسال ليس الموت: برنامجٌ يتجاهل الإشارة يبقى يعمل. ولا تُمسّ عمليةٌ لا تملكها.',
  'op.system.log.recent.title': 'أخطاء النظام الأخيرة',
  'op.system.log.recent.description': 'يعرض أخطاء النظام وأعطاله خلال مدّةٍ قصيرة تختارها.',
  'op.system.log.recent.execution':
    'قراءةٌ فقط: لا يُمحى أرشيف ولا يتغيّر إعداد تسجيل. والجواب مقصورٌ على الأخطاء والأعطال في المدّة المختارة — لا سجلّ النظام كاملًا، إذ إنّ دقيقتين منه بلا ترشيح تتجاوزان ما يمكن عرضه.',
  'op.system.processes.title': 'العمليات الأعلى استهلاكًا',
  'op.system.processes.description': 'يعرض العمليات الجارية مرتّبةً باستهلاك المعالج.',
  'op.system.processes.execution':
    'قراءةٌ فقط: لا تُنهى عملية ولا تتغيّر أولويّتها. وهي لقطة لحظية لا مراقبة مستمرة.',
  'op.system.info.title': 'معلومات النظام الأساسية',
  'op.system.info.description': 'يعرض اسم النظام ورقم إصداره ورقم بنائه.',
  'op.system.info.execution': 'قراءةٌ فقط: لا يُكتب شيء ولا يُنتظر تقرير كامل.',
  'op.system.architecture.title': 'معمارية المعالج',
  'op.system.architecture.description': 'يعرض معمارية المعالج: arm64 أو x86_64.',
  'op.system.architecture.execution': 'قراءةٌ فقط: سطرٌ واحد لا يتصل بشبكة ولا يكتب شيئًا.',
  'op.system.uptime.title': 'مدّة التشغيل ومتوسّط الحمل',
  'op.system.uptime.description': 'يعرض منذ متى يعمل الجهاز ومتوسّط حمله.',
  'op.system.uptime.execution':
    'قراءةٌ فقط: لا يُكتب شيء ولا يتغيّر إعداد. والحمل طول طابور لا نسبة مئوية.',
  'op.system.dns.flush.title': 'تفريغ ذاكرة DNS المؤقتة',
  'op.system.dns.flush.description': 'يفرّغ ذاكرة أسماء النطاقات المؤقتة.',
  'op.system.dns.flush.execution':
    'يُفرَّغ مخزن directory service وحده. لا يُحذف ملف ولا يتغيّر إعداد شبكة. وهو تفريغ جزئي: الوصفة الكاملة تحتاج صلاحيات مدير لا يطلبها هذا التطبيق.',
  'op.system.dns.flush.result': 'أُفرغت الذاكرة المؤقتة',
  'op.system.report.title': 'تقرير معلومات النظام',
  'op.system.report.description': 'يستخرج قسمًا واحدًا من تقرير النظام.',
  'op.system.report.execution':
    'قراءةٌ فقط: لا يُكتب شيء على القرص. وقسمٌ واحد لا التقرير كله، فالكامل يستغرق عشرات الثواني.',

  // ── أدوات المطوّرين ──────────────────────────────────────────────────
  'op.dev.npm.typecheck.title': 'فحص أنواع مشروع Node.js',
  'op.dev.npm.typecheck.description': 'يشغّل سكربت فحص الأنواع المعرَّف في المشروع.',
  'op.dev.npm.typecheck.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المشروع. والنتيجة نجاحٌ أو فشل بحسب رمز الخروج، والمخرَجات كما طبعتها الأداة.',

  'op.dev.npm.lint.title': 'فحص أسلوب مشروع Node.js',
  'op.dev.npm.lint.description': 'يشغّل سكربت فحص الأسلوب المعرَّف في المشروع.',
  'op.dev.npm.lint.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المشروع. والنتيجة نجاحٌ أو فشل بحسب رمز الخروج.',

  'op.dev.npm.test.title': 'تشغيل اختبارات مشروع Node.js',
  'op.dev.npm.test.description': 'يشغّل سكربت الاختبارات المعرَّف في المشروع.',
  'op.dev.npm.test.execution':
    'يشغّل الاختبارات كما عرّفها المشروع. لا يُعدَّل شيء من هذه العملية نفسها، وإن كان إطار الاختبار يكتب تقارير تغطية فذلك قرار المشروع لا هذه العملية.',

  'op.dev.npm.install.title': 'تثبيت حزم مشروع Node.js',
  'op.dev.npm.install.description': 'يثبّت الحزم المعلَنة في package.json.',
  'op.dev.npm.install.execution':
    'يكتب مجلد node_modules أو يعدّله، وقد يعدّل package-lock.json إن لم يطابق package.json تمامًا. المصدر لا يتغيّر.',

  'op.dev.npm.dev.title': 'تشغيل خادم تطوير Node.js',
  'op.dev.npm.dev.description': 'يشغّل خادم التطوير المعرَّف في المشروع.',
  'op.dev.npm.dev.execution':
    'يبقى شغّالًا حتى تُلغيه أنت — لا ينتهي من تلقاء نفسه. الإلغاء هو الطريق الطبيعي لإيقافه، لا خطأً.',

  'op.dev.tauri.dev.title': 'تشغيل تطبيق Tauri للتطوير',
  'op.dev.tauri.dev.description': 'يبني تطبيق Tauri ويشغّله، ويعيد البناء عند كل تعديل.',
  'op.dev.tauri.dev.execution':
    'يبقى شغّالًا حتى تُلغيه أنت. يكتب نواتج بناءٍ في مجلد target أثناء عمله — هذا متوقَّع، لا عطلٌ.',

  'op.dev.tauri.build.title': 'بناء تطبيق Tauri للإصدار',
  'op.dev.tauri.build.description': 'يبني حزمة تطبيق Tauri جاهزةً للتوزيع.',
  'op.dev.tauri.build.execution':
    'يكتب حزمة تطبيقٍ جديدة في target/release. المصدر لا يتغيّر، وقد يستغرق دقائق.',

  'op.dev.cargo.test.title': 'تشغيل اختبارات مشروع Rust',
  'op.dev.cargo.test.description': 'يشغّل اختبارات المشروع عبر Cargo.',
  'op.dev.cargo.test.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المشروع. والنتيجة نجاحٌ أو فشل بحسب رمز الخروج، والمخرَجات كما طبعتها الأداة.',

  'op.dev.cargo.check.title': 'فحص بناء مشروع Rust',
  'op.dev.cargo.check.description': 'يتحقّق من صحّة الكود دون توليد ملفٍّ تنفيذي كامل.',
  'op.dev.cargo.check.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المشروع. أسرع من بناءٍ كامل، لأنه يتحقّق من الأنواع والاستدلال فقط.',

  'op.dev.cargo.clippy.title': 'فحص Clippy لمشروع Rust',
  'op.dev.cargo.clippy.description': 'يفحص الكود بأسلوب صارم يحوّل كل تحذيرٍ إلى خطأ.',
  'op.dev.cargo.clippy.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المشروع. يشمل أهداف الاختبار والمقاييس أيضًا، لا الكود الأساسي وحده.',

  'op.dev.cargo.fmt.check.title': 'التحقّق من تنسيق مشروع Rust',
  'op.dev.cargo.fmt.check.description': 'يتحقّق من مطابقة الكود لتنسيق Rust القياسي دون تعديله.',
  'op.dev.cargo.fmt.check.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء في المشروع. ينجح إن كان التنسيق مطابقًا، ويفشل إن وجد ما يحتاج إعادة صياغة.',

  'op.dev.cargo.fmt.title': 'إعادة تنسيق مشروع Rust',
  'op.dev.cargo.fmt.description': 'يعيد صياغة كل ملفّ Rust في المشروع وفق التنسيق القياسي.',
  'op.dev.cargo.fmt.execution':
    'يكتب في ملفّات الكود مباشرةً — لا نسخة جديدة، بل تعديلٌ على الأصل. لا تراجع تلقائي بعد التنفيذ.',

  'op.dev.cargo.build.release.title': 'بناء مشروع Rust للإصدار',
  'op.dev.cargo.build.release.description': 'يبني حزمة نواتج مُحسَّنة في target/release.',
  'op.dev.cargo.build.release.execution':
    'يكتب حزمة نواتج جديدة في target/release. المصدر لا يتغيّر، وقد يستغرق دقائق لمشروعٍ كبير.',

  'op.dev.cargo.clean.title': 'حذف نواتج بناء مشروع Rust',
  'op.dev.cargo.clean.description': 'يحذف مجلد target كاملًا، وكل ما فيه من نواتج بناء.',
  'op.dev.cargo.clean.execution':
    'حذفٌ لا رجعة فيه، لكن كل ما يُحذف قابلٌ للتوليد من جديد بإعادة البناء. لا يمسّ الكود المصدري.',

  'field.project.label': 'مجلد المشروع',
  'field.project.help': 'مجلد المشروع — يحوي package.json.',
  'field.project.placeholder': 'لم يُختَر مجلد بعد',
  'field.dev.cargo.test.project.help': 'مجلد المشروع — يحوي Cargo.toml.',
  'field.dev.cargo.check.project.help': 'مجلد المشروع — يحوي Cargo.toml.',
  'field.dev.cargo.clippy.project.help': 'مجلد المشروع — يحوي Cargo.toml.',
  'field.dev.cargo.fmt.check.project.help': 'مجلد المشروع — يحوي Cargo.toml.',
  'field.dev.cargo.fmt.project.help': 'مجلد المشروع — يحوي Cargo.toml.',
  'field.dev.cargo.build.release.project.help': 'مجلد المشروع — يحوي Cargo.toml.',
  'field.dev.cargo.clean.project.help': 'مجلد المشروع — يحوي Cargo.toml.',
  'field.node_path.label': 'ملفّ Node.js',
  'field.node_path.help':
    'اختره مرّة واحدة: ملفّ node التنفيذي. يُتذكَّر تلقائيًا في المرّات التالية.',
  'field.node_path.placeholder': 'لم يُختَر ملفّ بعد',
  'field.cargo_path.label': 'ملفّ Cargo',
  'field.cargo_path.help':
    'اختره مرّة واحدة: ملفّ cargo التنفيذي. يُتذكَّر تلقائيًا في المرّات التالية.',
  'field.cargo_path.placeholder': 'لم يُختَر ملفّ بعد',

  'explain.npm.tool': 'أداة npm — تُشتقّ من مسار Node.js الذي اخترته.',
  'explain.npm.run': 'شغّل سكربتًا معرَّفًا في package.json.',
  'explain.npm.install': 'ثبّت الحزم المعلَنة في package.json.',
  'explain.tauri.tool': 'واجهة Tauri CLI المحلّية لهذا المشروع.',

  'explain.cargo.tool': 'أداة cargo — المسار الذي اخترته.',
  'explain.cargo.manifest_path': 'ملفّ Cargo.toml الذي يحدّد المشروع.',
  'explain.cargo.test': 'شغّل اختبارات المشروع.',
  'explain.cargo.check': 'تحقّق من صحّة الكود دون بناء ملفٍّ تنفيذي كامل.',
  'explain.cargo.clippy': 'شغّل فحص Clippy على المشروع.',
  'explain.cargo.all_targets': 'افحص أهداف الاختبار والمقاييس أيضًا، لا الكود الأساسي وحده.',
  'explain.cargo.deny': 'حوّل كل تحذيرٍ يليه إلى خطأ.',
  'explain.cargo.fmt': 'أداة تنسيق Rust القياسية.',
  'explain.cargo.fmt.check': 'تحقّق من التنسيق دون كتابة أي تعديل.',
  'explain.cargo.build': 'ابنِ المشروع.',
  'explain.cargo.release': 'بناءٌ مُحسَّن للإصدار، لا للتطوير.',
  'explain.cargo.clean': 'احذف مجلد target كاملًا.',

  'explain.ps.tool': 'أداة عرض العمليات الجارية. لقطة لحظية لا مراقبة.',
  'explain.ps.all_and_format': 'اعرض عمليات كل المستخدمين، وبالأعمدة المحدّدة بعدها.',
  'explain.ps.columns':
    'الأعمدة المطلوبة بالضبط: المعرّف، ومعرّف الأب، ونسبة المعالج، ونسبة الذاكرة، واسم الأمر. اختيارها يجعل الخرج مقروءًا بدل الأعمدة الافتراضية.',
  'explain.ps.sort_by_cpu': 'رتّب تنازليًا بحسب استهلاك المعالج، فيظهر الأثقل أولًا.',
  'explain.sw_vers.tool': 'أداة النظام التي تعلن اسم الإصدار ورقمه ورقم البناء.',
  'explain.uptime.tool': 'مدّة عمل الجهاز ومتوسّطات الحمل لدقيقة وخمس وخمس عشرة.',
  'explain.uname.tool': 'أداة استعلام معلومات النظام الأساسية في يونكس.',
  'explain.uname.machine': 'اطبع معمارية المعالج وحدها: arm64 أو x86_64.',
  'explain.dscacheutil.tool': 'أداة إدارة ذاكرة directory service المؤقتة.',
  'explain.dscacheutil.flushcache':
    'أفرغ الذاكرة المؤقتة. لا تحتاج صلاحيات مدير، وأثرها جزئي — انظر التحذير.',
  'explain.profiler.tool': 'أداة تقرير النظام الرسمية. تقرأ العتاد والبرمجيات من مصدرها.',
  'explain.profiler.detail_level': 'مستوى التفصيل.',
  'explain.profiler.mini': 'المستوى الأدنى: يتخطّى المسح المطوّل ويحذف ما يعرّف الجهاز شخصيًا.',
  'explain.profiler.data_type': 'قسم التقرير المطلوب. قسمٌ واحد لا التقرير كله.',
  'warn.ps.snapshot':
    'لقطة لحظة واحدة. نسب المعالج محسوبة منذ إقلاع كل عملية لا في هذه الثانية، فقد تختلف عمّا يعرضه مراقب النشاط.',
  'warn.dns.partial_flush':
    'تفريغ جزئي: ذاكرة mDNSResponder لا تُفرَّغ لأن ذلك يحتاج صلاحيات مدير، وهذا التطبيق لا يطلبها. إن بقي الاسم القديم يُحلّ، فذاك السبب.',
  'warn.profiler.slow':
    'قد يستغرق هذا الأمر عدّة ثوانٍ. لا تُعرض نسبة تقدّم لأن الأداة لا تعطي واحدة.',

  // ══════════════════════════════════════════════════════════════════
  //  مكتبة العمليات — نصوص الأقسام التسعة
  //
  //  كل مفتاح هنا يقابل شيئًا في النواة: عمليةً، أو رايةً في أمرها، أو
  //  تحذيرًا تعلنه قبل التنفيذ. اختبار `i18n.test.ts` يقرأ مصدر Rust نفسه
  //  ويطالب بترجمةٍ لكل مفتاح يصدر عنه، فلا يمكن أن يتقادم هذا القسم صامتًا.
  // ══════════════════════════════════════════════════════════════════

  // ── عناوين العمليات وأوصافها ──────────────────────────────────────────
  'op.disk.directory.open_handles.description':
    'يعرض العمليات التي تُبقي ملفًّا مفتوحًا داخل مجلدٍ وما تحته.',
  'op.disk.directory.open_handles.execution':
    'قراءةٌ فقط: لا يُغلق مقبض ولا يُنهى برنامج. والبحث ينزل الشجرة كاملةً فقد يتمهّل على مجلدٍ كبير. وبلا صلاحيات مدير تظهر عمليات المستخدم الحالي وحدها.',
  'op.disk.directory.open_handles.title': 'ما الذي يمسك ملفًّا في هذا المجلد؟',
  'op.disk.compare.bytes.description': 'يقارن ملفّين بايتًا بايت، ويتوقّف عند أول اختلاف.',
  'op.disk.compare.bytes.execution':
    'قراءةٌ فقط: لا يُعدَّل أيٌّ من الملفّين. أسرع من مقارنة البصمات حين يختلف الملفّان مبكرًا، لكنها تحتاج كليهما حاضرين معًا الآن.',
  'op.disk.compare.bytes.title': 'مقارنة ملفّين بايتيًا',
  'op.disk.compare.hash.description': 'يطبع بصمة SHA-256 لكلٍّ من ملفين لتقارن بينهما.',
  'op.disk.compare.hash.execution':
    'قراءةٌ فقط: لا يُعدَّل أيٌّ من الملفين ولا يُنقل. وتُحسب البصمتان كاملتين، فالملفات الكبيرة تأخذ وقتًا.',
  'op.disk.compare.hash.title': 'مقارنة ملفين ببصمتيهما',
  'op.disk.free.description': 'يعرض المساحة المستخدَمة والمتاحة في كل قرص مركَّب.',
  'op.disk.free.execution':
    'قراءةٌ فقط: لا يُمسّ ملف ولا يُكتب شيء. والأرقام بأساس ⁦1024⁩ فتقلّ قليلًا عمّا يعرضه Finder.',
  'op.disk.free.title': 'المساحة الحرة في الأقراص',
  'op.disk.hash.sha256.description': 'يحسب بصمة SHA-256 لملف لتتحقّق من صحّته.',
  'op.disk.hash.sha256.execution':
    'قراءةٌ فقط: لا يُعدَّل الملف ولا يُنسخ ولا يُنشأ شيء بجانبه. والبصمة على بيانات الملف وحدها دون وسومه وسماته.',
  'op.disk.hash.sha256.title': 'بصمة SHA-256 لملف',
  'op.disk.list.description': 'يعرض أقراص الجهاز وأقسامها ووحداتها.',
  'op.disk.list.execution':
    'قراءةٌ فقط: لا يمحو ولا يقسّم ولا يركّب ولا يفصل. وبلا صلاحيات مدير قد يمتنع بعض التفصيل عن وحدات لا تملكها.',
  'op.disk.list.title': 'قائمة الأقراص والأقسام',
  'op.files.find.large.description': 'يعرض الملفات التي يتجاوز حجمها حدًّا تختاره.',
  'op.files.find.large.execution':
    'قراءةٌ فقط: تُعرض المسارات ولا يُحذف شيء ولا يُعدَّل. والحجم يُقاس بالميغابايت مقرَّبًا إلى أعلى، فملفٌ على الحدّ تمامًا لا يظهر.',
  'op.files.find.large.title': 'العثور على الملفات الكبيرة',
  'op.files.find.name.description': 'يبحث عن ملفات يطابق اسمها نمطًا تكتبه.',
  'op.files.find.name.execution':
    'قراءةٌ فقط: تُعرض المسارات ولا يُفتح ملف ولا يُعدَّل. والمطابقة على الاسم وحده دون محتوى الملف.',
  'op.files.find.name.title': 'البحث عن ملف بالاسم',
  'op.files.find.stale.description':
    'يعرض الملفات التي لم يُسجَّل وصولٌ إليها منذ مدّة تختارها.',
  'op.files.find.stale.execution':
    'قراءةٌ فقط: تُعرض المسارات ولا يُحذف شيء. والمقياس زمنُ آخر وصول وهو غير موثوق على macOS، فالنتيجة مؤشّرٌ يُراجَع لا حكمٌ يُبنى عليه.',
  'op.files.find.stale.title': 'العثور على الملفات التي لم تُفتح منذ مدّة',
  'op.files.identify.description': 'يتعرّف نوع ملفٍّ أو مجلدٍ من محتواه، لا من امتداد اسمه.',
  'op.files.identify.execution':
    'قراءةٌ فقط: لا يُعدَّل الهدف ولا يُنسخ. والوصف من محتوى الملفّ الفعلي، لا من الاسم.',
  'op.files.identify.title': 'تعرّف نوع ملفّ',
  'op.files.list.description': 'يسرد أسماء ما في مجلدٍ واحد، بلا نزولٍ إلى ما تحته.',
  'op.files.list.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء. القائمة الفارغة إجابةٌ صحيحة عن مجلدٍ فارغ لا خطأً.',
  'op.files.list.title': 'محتويات مجلد',
  'op.files.mkdir.description': 'يُنشئ مجلدًا فارغًا جديدًا.',
  'op.files.mkdir.execution':
    'يُنشأ مجلد واحد داخل المجلد الأب الذي تختاره. ما في المجلد الأب لا يُمسّ. وإن كان الاسم مأخوذًا تتوقّف العملية وتُخبرك.',
  'op.files.mkdir.result': 'أُنشئ المجلد',
  'op.files.mkdir.title': 'إنشاء مجلد جديد',
  'op.files.move.description': 'ينقل ملفًا أو مجلدًا إلى مجلد آخر.',
  'op.files.move.execution':
    'يختفي من موضعه الحالي ويظهر في الوجهة باسمٍ تختاره. لا يُستبدل شيء: إن كان الاسم مأخوذًا تتوقّف العملية. وبين قرصين يصير النقل نسخًا ثم حذفًا، فيبطئ.',
  'op.files.move.result': 'تمّ النقل',
  'op.files.move.title': 'نقل ملف أو مجلد',
  'op.files.open.description': 'يفتح مجلدًا في نافذة Finder.',
  'op.files.open.execution':
    'كلّ أثره ظهور نافذة: لا يُكتب شيء ولا يُقرأ محتوى ملف. والمواضع المحميّة مثل مجلد المفاتيح تُرفض.',
  'op.files.open.result': 'فُتحت النافذة في Finder',
  'op.files.open.title': 'فتح مجلد في Finder',
  'op.files.tree.size.description': 'يقيس المساحة التي يشغلها مجلد وما تحته.',
  'op.files.tree.size.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء ولا يُحذف. والرقم ما يحجزه القرص فعلًا لا مجموع أحجام الملفات.',
  'op.files.tree.size.title': 'قياس حجم مجلد',
  'op.image.convert.description': 'يحوّل صورة إلى صيغة أخرى.',
  'op.image.convert.execution':
    'تُكتب نسخة جديدة بالصيغة المختارة في الوجهة. الصورة الأصلية لا تُمسّ ولا تُحذف، ولا يُستبدل ملفٌ قائم.',
  'op.image.convert.result': 'تم تحويل الصورة',
  'op.image.convert.title': 'تحويل صيغة صورة',
  'op.image.info.description': 'يعرض أبعاد صورة ودقّتها وفضاء ألوانها.',
  'op.image.info.execution':
    'قراءةٌ فقط: لا يُكتب شيء ولا يُعدَّل الملف. وعلى ملفٍ ليس صورة يعرض سجلًّا شبه فارغ بدل أن يفشل.',
  'op.image.info.title': 'قراءة بيانات صورة',
  'op.image.resize.description': 'يصغّر صورة إلى حدٍّ تختاره لضلعها الأطول.',
  'op.image.resize.execution':
    'تُكتب نسخة جديدة في الوجهة بنسبة أبعادٍ محفوظة. الأصل لا يُمسّ ولا يُستبدل ملفٌ قائم. وعددٌ أكبر من ضلع الصورة يكبّرها بلا تفصيل جديد.',
  'op.image.resize.result': 'تم تصغير الصورة',
  'op.image.resize.title': 'تصغير صورة بحدٍّ على ضلعها الأطول',
  'op.image.rotate.description': 'يدوّر صورة ⁦90⁩ أو ⁦180⁩ أو ⁦270⁩ درجة.',
  'op.image.rotate.execution':
    'تُكتب نسخة جديدة مُدارة في الوجهة. الأصل لا يُمسّ ولا تتغيّر الصيغة. وتُدار البكسلات نفسها مع إعادة ترميز، فتخسر ملفات JPEG جيلًا من الجودة.',
  'op.image.rotate.result': 'تم تدوير الصورة',
  'op.image.rotate.title': 'تدوير صورة بزاوية قائمة',
  'op.net.dns.description': 'يسأل خوادم الأسماء عن سجلٍّ لنطاق.',
  'op.net.dns.execution':
    'قراءةٌ فقط: لا يُحفظ شيء على القرص. ولا يمرّ بذاكرة macOS المؤقّتة، فقد يختلف الجواب عمّا يستعمله متصفّحك الآن.',
  'op.net.dns.title': 'استعلام سجلّ DNS',
  'op.net.download.description': 'ينزّل ملفًا من عنوان إلى مجلد تختاره.',
  'op.net.download.execution':
    'يُنشأ ملف جديد في الوجهة باسمٍ تكتبه، ولا يُستبدل ملفٌ قائم. ويُكتب باسم مؤقّت فلا يظهر باسمه النهائي إلا بعد تنزيلٍ كامل. ولا يُتحقَّق من بصمة الملف ولا من توقيعه.',
  'op.net.download.result': 'تم تنزيل الملف',
  'op.net.download.title': 'تنزيل ملف من عنوان',
  'op.net.headers.description': 'يعرض ترويسات ردّ الخادم على عنوان.',
  'op.net.headers.execution':
    'قراءةٌ فقط: لا يُنزَّل جسد الردّ ولا يُحفظ شيء على القرص. ويُتتبَّع إعادة التوجيه حتى خمس مرات.',
  'op.net.headers.title': 'قراءة ترويسات عنوان',
  'op.net.ping.description': 'يقيس وصول الرزم إلى مضيف وزمن رحلتها.',
  'op.net.ping.execution':
    'قراءةٌ فقط: لا يُفتح اتصال ولا تُقاس سرعة تنزيل. وصمتُ المضيف لا يعني توقّفه: كثير من الشبكات تحجب هذه الرزم بينما تعمل خدماته.',
  'op.net.ping.title': 'فحص وصول مضيف',
  'op.net.port.owner.description':
    'يعرض ما على منفذٍ بعينه من مقابس والبرامج التي تملكها، مستمعةً كانت أو متّصلة.',
  'op.net.port.owner.execution':
    'قراءةٌ فقط: لا يُفتح منفذ ولا يُغلق ولا تُمسّ عمليةٌ تملكه. وبلا صلاحيات مدير تظهر عمليات المستخدم الحالي وحدها، فمخرجٌ خالٍ لا يعني أن المنفذ حرّ.',
  'op.net.port.owner.title': 'من يشغل هذا المنفذ؟',
  'op.net.ports.description': 'يعرض منافذ TCP التي تنتظر اتصالًا والبرامج التي تملكها.',
  'op.net.ports.execution':
    'قراءةٌ فقط: لا يُفتح منفذ ولا يُغلق ولا يتغيّر إعداد. وبلا صلاحيات مدير تظهر عمليات المستخدم الحالي وحدها.',
  'op.net.ports.title': 'المنافذ المستمعة على هذا الجهاز',
  'op.security.codesign.description': 'يعرض توقيع تطبيق والجهة التي وقّعته.',
  'op.security.codesign.execution':
    'قراءةٌ فقط: لا يوقّع ولا يستبدل توقيعًا ولا يزيله. وتقرير الأداة يُكتب على قناة الخطأ، فيظهر تقريرٌ سليم موسومًا «خطأ».',
  'op.security.codesign.title': 'قراءة توقيع تطبيق',
  'op.security.codesign.verify.description':
    'يتحقّق أن توقيع ملفٍّ لا يزال سليمًا، لا من عرض من وقّعه.',
  'op.security.codesign.verify.execution':
    'قراءةٌ فقط: لا يوقّع ولا يستبدل شيئًا. ملفٌّ موقَّعٌ ثم عُبِث به يفشل هنا حتى لو ظهر «موقَّع» في العملية الأخرى.',
  'op.security.codesign.verify.title': 'التحقّق من سلامة توقيع',
  'op.security.gatekeeper.description': 'يسأل النظام أيسمح بتشغيل ملف أم يرفضه.',
  'op.security.gatekeeper.execution':
    'قراءةٌ فقط: لا يُثبَّت شيء ولا تُغيَّر سياسة ولا يُرفع حجر. وما لا تفهمه السياسة يعود «مرفوضًا» لأنه ليس ممّا يُقيَّم، لا لعيبٍ فيه.',
  'op.security.gatekeeper.title': 'تقييم Gatekeeper',
  'op.security.permissions.description': 'يعرض صلاحيات ملف ومالكه وقوائم التحكّم بالوصول.',
  'op.security.permissions.execution':
    'قراءةٌ فقط: لا يُكتب شيء ولا تتغيّر صلاحية. وحين يكون الهدف مجلدًا يوصَف المجلد نفسه لا ما بداخله.',
  'op.security.permissions.title': 'صلاحيات ملف أو مجلد',
  'op.security.xattr.description': 'يعرض البيانات المرفقة بملف خارج محتواه.',
  'op.security.xattr.execution':
    'قراءةٌ فقط: لا تُحذف سمة ولا تُكتب. ولا يشمل العرض ما داخل المجلد، والقيم الثنائية تظهر بايتاتٍ خامًا.',
  'op.security.xattr.title': 'السمات الممتدّة لملف',
  'op.text.diff.description': 'يعرض ما اختلف بين ملفين نصّيين سطرًا سطرًا.',
  'op.text.diff.execution':
    'قراءةٌ فقط: لا يُعدَّل الملفان ولا يُدمج أحدهما في الآخر. ووجود الفروق وعدمها جوابان سليمان لا فشلٌ في التنفيذ.',
  'op.text.diff.title': 'مقارنة ملفين نصّيين',
  'op.text.encoding.utf8.description': 'يحوّل ملفًا نصّيًا من ترميز عربي قديم إلى UTF-8.',
  'op.text.encoding.utf8.execution':
    'تُكتب نسخة جديدة في الوجهة التي تختارها. الأصل لا يُمسّ. واختيار ترميزٍ غير ترميز الملف يُنتج نصًّا مشوّهًا بلا فشل، فراجع الناتج قبل حذف الأصل.',
  'op.text.encoding.utf8.result': 'تم تحويل الترميز',
  'op.text.encoding.utf8.title': 'تحويل ترميز ملف نصّي إلى UTF-8',
  'op.text.merge.description': 'يضمّ ملفين نصّيين في ملفٍ ثالثٍ جديد.',
  'op.text.merge.execution':
    'يُنشأ ملف ثالث في الوجهة بالترتيب الذي تحدّده. الملفان الأصليان لا يُعدَّلان ولا يُحذفان. ولا فاصل بينهما: يبدأ الثاني حيث انتهى الأول بايتًا ببايت.',
  'op.text.merge.result': 'تم دمج الملفين',
  'op.text.merge.title': 'دمج ملفين نصّيين',
  'op.text.search.description': 'يبحث عن نصٍّ داخل ملفات مجلد وما تحته.',
  'op.text.search.execution':
    'قراءةٌ فقط: لا يُعدَّل شيء ولا يُستبدل. والبحث حرفيّ لا بتعبيرٍ نمطي، وعدم وجود مطابقات جوابٌ سليم.',
  'op.text.search.title': 'البحث عن نصّ داخل مجلد',
  'op.text.split.description': 'يقسّم ملفًا نصّيًا إلى أجزاء متساوية في عدد الأسطر.',
  'op.text.split.execution':
    'يُنشأ مجلد جديد في الوجهة يضمّ الأجزاء. الملف الأصلي يبقى كما هو، والقطع عند حدود الأسطر فلا ينقسم سطر. والحدّ ⁦676⁩ جزءًا، وما زاد يوقف العملية قبل إنشاء المجلد.',
  'op.text.split.result': 'تم تقسيم الملف',
  'op.text.split.title': 'تقسيم ملف نصّي إلى أجزاء',

  // ── نصوص الحقول ───────────────────────────────────────────────────────
  'field.days.placeholder': '١٨٠',
  'field.degrees.placeholder': 'اختر زاوية',
  'field.depth.placeholder': '١',
  'field.disk.compare.hash.left.help': 'الملف الأول. بصمته تظهر في السطر الأول.',
  'field.disk.compare.hash.left.label': 'الملف الأول',
  'field.disk.compare.hash.left.placeholder': 'لم يُختَر ملف بعد',
  'field.disk.compare.hash.right.help': 'ملفٌ غير الأول — مقارنة ملف بنفسه لا تقول شيئًا.',
  'field.disk.compare.hash.right.label': 'الملف الثاني',
  'field.disk.compare.hash.right.placeholder': 'لم يُختَر ملف بعد',
  'field.disk.hash.sha256.source.help': 'اختر ملفًا قائمًا.',
  'field.disk.hash.sha256.source.label': 'الملف المراد بصمُه',
  'field.disk.hash.sha256.source.placeholder': 'لم يُختَر ملف بعد',
  'field.files.move.destination.help': 'مجلد قائم تملك صلاحية الكتابة فيه، وليس داخل المصدر.',
  'field.files.move.destination.label': 'المجلد الذي يُنقل إليه',
  'field.files.move.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.files.move.source.help': 'اختر ملفًا أو مجلدًا قائمًا.',
  'field.files.move.source.label': 'الملف أو المجلد المراد نقله',
  'field.files.move.source.placeholder': 'لم يُختَر شيء بعد',
  'field.files.open.folder.help': 'اختر المجلد الذي تريد فتحه.',
  'field.files.open.folder.label': 'المجلد المراد فتحه',
  'field.format.placeholder': 'اختر صيغة',
  'field.ignore_case.help': 'يفيد في الإنجليزية وأسماء الشيفرة، ولا أثر له في العربية.',
  'field.image.convert.destination.help': 'مجلدٌ قائم تملك صلاحية الكتابة فيه.',
  'field.image.convert.destination.label': 'مكان حفظ الصورة الجديدة',
  'field.image.convert.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.image.convert.out_name.help':
    'اكتب الامتداد بنفسك ليطابق الصيغة: ⁦.jpg⁩ أو ⁦.png⁩ أو ⁦.tif⁩ أو ⁦.heic⁩.',
  'field.image.convert.source.help': 'اختر ملف صورةٍ قائمًا.',
  'field.image.convert.source.label': 'الصورة المراد تحويلها',
  'field.image.convert.source.placeholder': 'لم تُختَر صورة بعد',
  'field.image.info.source.help': 'اختر ملف صورةٍ قائمًا.',
  'field.image.info.source.label': 'الصورة المراد قراءة بياناتها',
  'field.image.info.source.placeholder': 'لم تُختَر صورة بعد',
  'field.image.resize.destination.help': 'مجلدٌ قائم تملك صلاحية الكتابة فيه.',
  'field.image.resize.destination.label': 'مكان حفظ الصورة المصغّرة',
  'field.image.resize.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.image.resize.out_name.help':
    'اجعل الامتداد امتداد الصورة الأصلية — الصيغة لا تتغيّر بالتصغير.',
  'field.image.resize.source.help': 'اختر ملف صورةٍ قائمًا.',
  'field.image.resize.source.label': 'الصورة المراد تصغيرها',
  'field.image.resize.source.placeholder': 'لم تُختَر صورة بعد',
  'field.image.rotate.destination.help': 'مجلدٌ قائم تملك صلاحية الكتابة فيه.',
  'field.image.rotate.destination.label': 'مكان حفظ الصورة المُدارة',
  'field.image.rotate.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.image.rotate.out_name.help':
    'اجعل الامتداد امتداد الصورة الأصلية — الصيغة لا تتغيّر بالتدوير.',
  'field.image.rotate.source.help': 'اختر ملف صورةٍ قائمًا.',
  'field.image.rotate.source.label': 'الصورة المراد تدويرها',
  'field.image.rotate.source.placeholder': 'لم تُختَر صورة بعد',
  'field.lines_per_part.help':
    'بين ١ وعشرة ملايين. القطع عند حدود الأسطر، فلا ينقسم سطرٌ بين جزأين.',
  'field.lines_per_part.label': 'عدد الأسطر في كل جزء',
  'field.lines_per_part.placeholder': '١٠٠٠',
  'field.max_pixels.placeholder': 'مثال: ١٦٠٠',
  'field.min_megabytes.placeholder': '١٠٠',
  'field.net.dns.record.help': 'ما الذي تسأل عنه في هذا النطاق.',
  'field.net.dns.record.label': 'نوع السجلّ',
  'field.net.dns.record.placeholder': 'اختر نوعًا',
  'field.net.download.destination.label': 'مكان حفظ الملف',
  'field.net.download.url.help':
    '‏⁦http⁩ أو ⁦https⁩. ولا تُسجَّل قيمة هذا الحقل: الروابط الموقّعة تحمل رمز وصول.',
  'field.net.ping.count.help':
    'بين ١ و٢٠. تُرسل رزمةً في الثانية، والمهلة عشر ثوانٍ مهما بقي منها.',
  'field.net.ping.count.label': 'عدد الرزم',
  'field.net.ping.count.placeholder': '٥',
  'field.security.codesign.target.help': 'تطبيق أو ملف تنفيذي.',
  'field.security.codesign.target.label': 'التطبيق المراد فحص توقيعه',
  'field.security.gatekeeper.target.help':
    'تطبيق أو مثبِّت أو صورة قرص أو ملف تنفيذي — لا مستند عادي.',
  'field.security.gatekeeper.target.label': 'التطبيق المراد تقييمه',
  'field.text.encoding.utf8.destination.help': 'مجلد قائم تملك صلاحية الكتابة فيه.',
  'field.text.encoding.utf8.destination.label': 'مكان حفظ النسخة الجديدة',
  'field.text.encoding.utf8.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.text.encoding.utf8.out_name.help': 'يُضاف ⁦.txt⁩ تلقائيًا مرّةً واحدة.',
  'field.text.encoding.utf8.out_name.label': 'اسم الملف الناتج',
  'field.text.encoding.utf8.out_name.placeholder': 'مثال: النصّ بعد التحويل',
  'field.text.encoding.utf8.source.help': 'ملفٌ نصّي أو مستندٌ قائم.',
  'field.text.encoding.utf8.source.label': 'الملف المراد تحويله',
  'field.text.encoding.utf8.source.placeholder': 'لم يُختَر ملف بعد',
  'field.text.merge.destination.help': 'مجلد قائم تملك صلاحية الكتابة فيه.',
  'field.text.merge.destination.label': 'مكان حفظ الناتج',
  'field.text.merge.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.text.merge.out_name.help': 'اسمٌ جديد. لا يُضاف امتداد تلقائيًا.',
  'field.text.merge.out_name.label': 'اسم الملف الناتج',
  'field.text.merge.out_name.placeholder': 'مثال: المدموج.txt',
  'field.text.search.folder.help': 'يُبحث فيه وفي كل ما تحته.',
  'field.text.search.folder.label': 'المجلد المبحوث فيه',
  'field.text.search.folder.placeholder': 'لم يُختَر مجلد بعد',
  'field.text.search.pattern.help':
    'نصّ حرفي: النقطة نقطة والقوس قوس. والمسافات في طرفيه تُحسب.',
  'field.text.search.pattern.label': 'النصّ المطلوب',
  'field.text.search.pattern.placeholder': 'مثال: رقم الفاتورة',
  'field.text.split.destination.help': 'مجلد قائم؛ يُنشأ داخله مجلدٌ واحد تنزل فيه الأجزاء.',
  'field.text.split.destination.label': 'مكان حفظ مجلد الأجزاء',
  'field.text.split.destination.placeholder': 'لم تُختَر وجهة بعد',
  'field.text.split.folder_name.help':
    'اسم المجلد الذي تنزل فيه الأجزاء. لا يُستبدل مجلد قائم.',
  'field.text.split.folder_name.label': 'اسم مجلد الأجزاء',
  'field.text.split.folder_name.placeholder': 'مثال: أجزاء السجل',
  'field.text.split.source.help': 'ملف نصّي قائم.',
  'field.text.split.source.label': 'الملف المراد تقسيمه',
  'field.text.split.source.placeholder': 'لم يُختَر ملف بعد',

  // ── خيارات القوائم المغلقة ────────────────────────────────────────────
  'choice.degrees.180': '١٨٠ درجة — رأسًا على عقب',
  'choice.degrees.270': '٢٧٠ درجة مع العقارب، أي ربع دورة عكسها',
  'choice.degrees.90': '٩٠ درجة مع عقارب الساعة',
  'choice.dns.a': '‏A — عنوان IPv4',
  'choice.dns.aaaa': '‏AAAA — عنوان IPv6',
  'choice.dns.cname': '‏CNAME — الاسم البديل',
  'choice.dns.mx': '‏MX — خوادم البريد',
  'choice.dns.ns': '‏NS — خوادم الأسماء المسؤولة',
  'choice.dns.txt': '‏TXT — نصّ حرّ (SPF والتحقّق من الملكية)',
  'choice.encoding.cp1256': 'ويندوز العربية (‏windows-1256‏)',
  'choice.encoding.iso8859_6': 'آيزو العربية (‏ISO-8859-6‏)',
  'choice.encoding.macarabic': 'ماك العربية (‏x-mac-arabic‏)',
  'choice.encoding.utf8':
    'يونيكود (‏UTF-8‏) — للملف السليم أصلًا، لتجريده من صيغته لا لتغيير ترميزه',
  'choice.format.heic': 'HEIC — الأصغر بجودةٍ مساوية، ودعمها خارج أجهزة Apple ناقص',
  'choice.format.jpeg': 'JPEG — الأصغر على الصور، وتفتحها كل أداة',
  'choice.format.png': 'PNG — بلا خسارة، مع شفافية',
  'choice.format.tiff': 'TIFF — بلا خسارة، للأرشفة والطباعة',
  'choice.report.hardware': 'العتاد',
  'choice.report.software': 'البرمجيات',
  'choice.report.storage': 'التخزين',
  'choice.report.displays': 'الشاشات',
  'choice.report.usb': 'أجهزة USB',

  // ── شرح الأوامر — رمزًا رمزًا ─────────────────────────────────────────
  'explain.cat.tool':
    'أداة ضمّ الملفات في يونكس. تقرأ الملفات بالترتيب وتكتب بايتاتها على الخرج القياسي — لا تعرف وجهةً ولا تعدّل شيئًا. اختيرت لأنها لا تفعل غير النسخ: لا تحويل ترميزٍ ولا إعادة تنسيق. والتطبيق هو من يوجّه خرجها إلى ملفه المؤقّت، بلا صدفةٍ ولا علامة ⁦>⁩.',
  'explain.cmp.tool':
    'أداة المقارنة البايتية في يونكس. تتوقّف عند أول بايتٍ مختلف، وتطبع موضعه حين تختلف، وتصمت حين تتطابق.',
  'explain.codesign.display':
    'اعرض التوقيع القائم. هذه راية القراءة؛ ورايات التوقيع والاستبدال القسري (⁦-s⁩ و⁦-f⁩) ليست في هذا الأمر.',
  'explain.codesign.tool':
    'أداة التواقيع الرقمية في macOS. تكتب تقريرها كاملًا على قناة الخطأ لا على الخرج القياسي، فيظهر أدناه موسومًا «خطأ» وهو تقريرٌ سليم لا رسالة عطب.',
  'explain.codesign.verbose':
    'أسهب مرّتين. المستوى الثاني هو ما يطبع سلسلة الجهات الموقِّعة — المطوّر، ثم جهة التصديق، ثم الجذر — ومعرّف فريق المطوّر. وبدونه تبقى كلمة «موقَّع» لا تقول من وقّع.',
  'explain.codesign.verify':
    'أعد حساب التوقيع وقارنه بما هو مثبَّت على الملف. جوابٌ عن سلامة التوقيع، لا عن هويّة صاحبه.',
  'explain.codesign.deep':
    'تحقّق من التوقيعات المتداخلة أيضًا — إطاراتٌ أو ملحقاتٌ موقَّعة كلٌّ على حدة — لا الغلاف الخارجي وحده.',
  'explain.codesign.strict':
    'ارفض حتى المخالفات الصغيرة التي كانت أدوات macOS الأقدم تتساهل معها. الخيار الذي توصي به Apple للتحقّق الجادّ.',
  'explain.curl.fail_and_follow':
    'الحرفان معًا: ⁦f⁩ تجعل ردًّا برمز ٤٠٠ فما فوق خروجًا فاشلًا بدل أن تُكتب صفحة الخطأ في الملف — وهي التي تجعل «لا يظهر الملف باسمه النهائي إلا بعد نجاح» وعدًا عن صحّة الملف لا عن نجاح الأمر وحده. و⁦L⁩ تتبع إعادة التوجيه إلى وجهتها. وحدُّ ⁦f⁩ معلَن: خادمٌ يرسل صفحة خطأ برمز ٢٠٠ لا تميّزه أداة.',
  'explain.curl.follow':
    'اتبع إعادة التوجيه (‏3xx) إلى وجهتها بدل التوقّف عند أوّل ردّ. أكثر روابط التنزيل والروابط المختصرة تحوّل مرّةً أو مرّتين، فبلا هذه الراية يكون الجواب صفحة التحويل لا ما طُلب.',
  'explain.curl.head':
    'اطلب الترويسات وحدها (‏HEAD). لا يُنزَّل جسد الردّ ولا يُكتب شيء على القرص.',
  'explain.curl.max_redirs': 'سقفٌ لعدد مرات تتبّع إعادة التوجيه.',
  'explain.curl.max_redirs.value':
    'خمس مرات. وcurl تتبع حتى خمسين افتراضيًا، وخفضُها قرار: سلسلةٌ أطول من خمسٍ حلقةٌ أو سوء إعداد لا وجهةٌ صحيحة — وحلقةٌ كهذه تستهلك المهلة كاملةً في طلباتٍ صحيحة الشكل.',
  'explain.curl.max_time': 'سقفٌ زمني للعملية كلها بالثواني، لا لمرحلةٍ منها.',
  'explain.curl.max_time.download':
    'تسعمئة ثانية — ربع ساعة — سقفًا للتنزيل كله. تنزيلٌ أبطأ من ذلك يُقطع، ولا يُرقّى الملف المؤقّت، فلا يبقى ملفٌ ناقص باسمه النهائي. والثمن معلَن: ملفٌ ضخم على اتصالٍ بطيء لا يكتمل من هنا.',
  'explain.curl.max_time.headers':
    'عشرون ثانية. تكفي ترويسات أي خادم حيّ، وتمنع خادمًا يقبل الاتصال ثم يصمت من أن يترك الأمر معلّقًا. وهي مثبّتة في التطبيق لا حقلًا في النموذج.',
  'explain.curl.output': 'اكتب جسد الردّ إلى هذا الملف بدل الخرج القياسي.',
  'explain.curl.progress_bar':
    'شريط تقدّم بدل جدول العدّادات. تكتبه curl على مجرى الخطأ، والتطبيق يبثّ هذا المجرى إلى الشاشة سطرًا سطرًا — فالتنزيل يُرى وهو يتقدّم بدل أن يبدو معلّقًا.',
  'explain.curl.silent_but_loud':
    'الحرفان معًا لا أحدهما: ⁦s⁩ تُطفئ عدّاد التقدّم كي لا يمتلئ المجرى بأسطر التحديث، و⁦S⁩ تعيد رسائل الخطأ وحدها — فبلا الثانية يعود أمرٌ فاشل بلا سطرٍ واحد يقول لماذا فشل.',
  'explain.curl.tool':
    'أداة النقل عبر الشبكة في النظام. تُنفَّذ بمسارها المطلق ووسائطها مصفوفةً منفصلة، بلا صدفة تفسّر شيئًا. وتقرأ curl ملف ⁦~/.curlrc⁩ إن وُجد في منزلك، فما فيه يُضاف إلى الرايات أعلاه ولا يظهر في هذا السطر.',
  'explain.curl.url':
    'العنوان كما كتبتَه. تحقّقت النواة قبل بناء الأمر من أنه ⁦http⁩ أو ⁦https⁩ وحدهما — لا ⁦file:⁩ ولا غيرها مما تفهمه curl وكان سيقرأ قرصك متجاوزًا سياسة المسارات — ومن خلوّه من محارف التحكّم والفراغ، ومن أنه لا يبدأ بشرطة فيُقرأ رايةً.',
  'explain.df.human':
    'اعرض الأحجام بوحداتٍ تُقرأ — ⁦Ki⁩ و⁦Mi⁩ و⁦Gi⁩ — بدل كتل الـ ٥١٢ بايت التي تعدّها ⁦df⁩ افتراضيًا. واللاحقة تقول إن العدّ بأساس ١٠٢٤، فالرقم يقلّ عمّا يعرضه Finder لأن أبل تعدّ بالألف. وهي كذلك تتجاوز متغيّر البيئة ⁦BLOCKSIZE⁩، فالأمر يعطي الجدول نفسه هنا وفي صدفتك بعد نسخه.',
  'explain.df.tool':
    'أداة يونكس القياسية للمساحة، وهي جزءٌ من macOS. تسأل النظامَ عن كل نقطة تركيب وتجيب في جزءٍ من ثانية، بينما ⁦system_profiler SPStorageDataType⁩ تطبع كتلةً مسهبة بعد ثوانٍ وتعرض أقلّ ممّا يُركَّب فعلًا.',
  'explain.diff.tool':
    'أداة المقارنة في يونكس. تقرأ الملفين وتطبع ما اختلف بينهما، ولا تكتب على القرص شيئًا ولا تعدّل أيًّا منهما.',
  'explain.diff.unified':
    'اعرض الفرق بالشكل الموحّد: أسطر ⁦-⁩ حُذفت وأسطر ⁦+⁩ أُضيفت، مع ثلاثة أسطر من السياق حولها. اختير على الشكل الافتراضي لأن ذاك يقول «تغيّر السطر الأول» ولا يريك ما كان ولا ما صار.',
  'explain.dig.answer':
    'ثم أعِد قسم الإجابة وحده. الرايتان تعملان معًا: الأولى بلا الثانية تُخرج أمرًا صامتًا، والثانية بلا الأولى لا تُطفئ شيئًا — والبديل عنهما خرجٌ فيه عشرة أسطر حول سطرٍ واحد هو الجواب.',
  'explain.dig.domain':
    'النطاق كما كتبتَه، بعد قصّ الفراغ الطرفيّ وحده. ولا تُضاف إليه لواحق البحث: يُسأل عنه كما هو.',
  'explain.dig.noall': 'أطفئ كل أقسام الخرج: الترويسة، والسؤال، والإحصاءات، والتعليقات.',
  'explain.dig.record':
    'نوع السجلّ المطلوب. القيمة تخرج من قائمةٍ مغلقة في شيفرة التطبيق، ولا تكتبها الواجهة.',
  'explain.dig.tool':
    'أداة استعلام DNS القياسية. اختيرت على ⁦nslookup⁩ المهجورة وعلى ⁦host⁩ المختصِرة لأنها تطبع السجلّات بصيغتها الأصلية — نفس صيغة ملفات المناطق — فما تراه هو ما ردّ به الخادم لا صياغةٌ له. وتسأل الخوادم مباشرة، فلا تمرّ بذاكرة macOS المؤقّتة.',
  'explain.diskutil.list':
    'اعرض الأقراص والأقسام ووحدات APFS جدولًا. فعلٌ قارئ: لا يكتب على قرص، ولا يركّب وحدةً ولا يفصلها، ولا يغيّر شيئًا على الجهاز.',
  'explain.diskutil.tool':
    'أداة إدارة الأقراص في macOS. تحمل أفعالًا تمحو أقراصًا وتعيد تقسيمها، ولا تستطيع هذه العملية بلوغ أيٍّ منها: الفعل مكتوبٌ في شيفرة التطبيق ويدخل الثنائيّة عند الترجمة، والعملية لا تعلن حقلًا واحدًا تصير قيمته وسيطًا.',
  'explain.du.depth':
    'أقصى عمقٍ تُطبع له سطور. صفرٌ يعني المجموع الكلّي في سطرٍ واحد، وواحدٌ يعني سطرًا لكلّ مجلدٍ فرعيّ مباشر. الرقم الكبير يُخرج جدارَ نصٍّ لا جوابًا.',
  'explain.du.human': 'اعرض الأحجام بوحداتٍ يقرؤها الإنسان — ك.ب وم.ب وج.ب — بدل عدد الكتل.',
  'explain.du.tool':
    'أداة قياس المساحة المشغولة. تقيس ما تحجزه الملفات على القرص فعلًا لا مجموع أحجامها المعلَنة، وهو الرقم الصحيح لسؤال «كم يأكل هذا المجلد من قرصي؟».',
  'explain.find.atime':
    'عدد الأيام منذ آخر وصولٍ مسجَّل إلى الملف. علامة ⁦+⁩ تعني «أكثر من». والمقياس نفسه غير موثوق على macOS، انظر التحذير أعلاه.',
  'explain.find.iname':
    'طابق اسم الملف بهذا النمط بلا تفريقٍ بين حالة الأحرف. ⁦*⁩ و⁦?⁩ تفسّرهما ⁦find⁩ نفسها لا صدفة، والنمط يصلها وسيطًا واحدًا كما كتبته حرفًا بحرف — ولذلك لا يحتاج علامات اقتباس هنا وإن احتاجها في Terminal.',
  'explain.find.size':
    'الحدّ الأدنى للحجم بوحدات الميغابايت. علامة ⁦+⁩ تعني «أكبر تمامًا من»، و⁦find⁩ تقرّب حجم كلّ ملف إلى أعلى قبل المقارنة — فالقراءة الصحيحة: كلّ ملفٍ يتجاوز هذا الحدّ، وما كان عليه تمامًا لا يظهر.',
  'explain.find.tool':
    'أداة المسح المُرشَّح للشجرة في يونكس: تمشي المجلد وما تحته وتطبع ما يطابق الشروط. لا تحذف ولا تعدّل ولا تُمرّر النتائج إلى أداةٍ أخرى — هذا الأمر يخبر، والقرار بعده لك.',
  'explain.find.type_f':
    'اقصر النتائج على الملفات العادية: لا مجلدات ولا روابط رمزية. بدون هذا الشرط تظهر المجلدات في القائمة بخصائص لا تعني فيها ما تعنيه في الملف، فتصير الأرقام كلّها موضع شكّ.',
  'explain.file.tool': 'أداة تعرّف نوع الملفّ من محتواه لا من امتداد اسمه.',
  'explain.file.brief': 'اطبع وصف النوع وحده، بلا إعادة كتابة المسار قبله.',
  'explain.kill.tool':
    'أداة إرسال الإشارات إلى العمليات. ليست ⁦pkill⁩ ولا ⁦killall⁩: تلك تُنهي كلّ ما يطابق اسمًا دفعةً واحدة — عددًا لا يظهر في هذا السطر ولا تعرفه قبل الضغط. هذه تأخذ رقمًا واحدًا فتصيب عمليةً واحدة مكتوبةً أمامك.',
  'explain.kill.term':
    'إشارة الإنهاء اللطيفة (‏SIGTERM). تصل إلى البرنامج فيغلق ملفاته ويحفظ ما عنده ثم يخرج. وهي مكتوبةٌ هنا وإن كانت الافتراضية، كي تراها لا كي تستنتجها. ولا سبيل إلى ⁦-KILL⁩ من هذا التطبيق: الإشارة ثابتة في شيفرته.',
  'explain.kill.pid':
    'رقم العملية التي ستصلها الإشارة. رقمٌ واحد لا اسم ولا نمط، فأثر هذا الأمر عمليةٌ واحدة بعينها. تقرأ الرقم من «البحث عن عملية» أو «العمليات الأعلى استهلاكًا».',
  'explain.log.tool':
    'أداة السجلّ الموحَّد في macOS. تحمل أفعالًا تمحو الأرشيف وتغيّر إعداد التسجيل (‏erase وconfig)، ولا سبيل إلى أيٍّ منها من هنا: الفعل مكتوبٌ في شيفرة التطبيق ويدخل الثنائيّة عند الترجمة.',
  'explain.log.show': 'اعرض القيود المسجَّلة. فعلٌ قارئ: لا يمحو أرشيفًا ولا يغيّر إعدادًا.',
  'explain.log.last': 'ارجع إلى الوراء بهذه المدّة من الآن.',
  'explain.log.style': 'شكل الطباعة.',
  'explain.log.compact':
    'سطرٌ واحد لكل قيد بدل الشكل المسهب. ليس تجميلًا: هو جزءٌ من إبقاء الجواب تحت سقف الأسطر الذي يعرضه التطبيق.',
  'explain.log.predicate': 'مُرشِّح القيود.',
  'explain.log.errors_only':
    'الأخطاء والأعطال وحدها. ثابتٌ في الشيفرة لا حقلٌ في الشاشة لسببين: لغة المُرشِّحات لغةٌ ثانية داخل حقل، وبلا ترشيح لا توجد نافذةٌ زمنية مفيدة أصلًا — دقيقتان بلا مُرشِّح تطبعان ما يزيد على خمسة وعشرين ألف سطر.',
  'explain.pgrep.tool':
    'أداة البحث عن العمليات الجارية بالاسم. قارئةٌ فقط: لا تُنهي شيئًا ولا تغيّر أولوية، ولا تملك رايةً تفعل ذلك أصلًا.',
  'explain.pgrep.list_name':
    'اطبع اسم كل عملية بجوار رقمها. والمطابقة على اسم العملية وحده لا على سطر أوامرها — فلا تظهر في الجواب مسارات ما تفتحه، وهو نفس سبب اختيار العمود ⁦comm⁩ في «العمليات الأعلى استهلاكًا». والاسم المطبوع يُقصّ عند خمسة عشر محرفًا، فقد يتشابه اسمان ويختلف رقماهما.',
  'explain.pgrep.name':
    'الاسم المطلوب. يُطابَق كجزءٍ من اسم العملية، فـ⁦Find⁩ تجد ⁦Finder⁩. وتقرؤه الأداة تعبيرًا نمطيًّا، فرموزٌ مثل ⁦(⁩ و⁦[⁩ لها معنًى عندها وقد تُرفض إن لم تكتمل.',
  'explain.ls.list_tool': 'أداة سرد محتويات مجلد في يونكس.',
  'explain.ls.one_per_line': 'سطرٌ واحد لكل عنصر.',
  'explain.ls.almost_all': 'يشمل الملفّات المخفيّة، بلا ⁦.⁩ و⁦..⁩ أنفسهما.',
  'explain.ls.mark_directories': 'يميّز المجلدات بشرطة مائلة ⁦/⁩ في نهاية الاسم.',
  'explain.grep.fixed_string':
    'عامِل ما كتبته نصًّا ثابتًا لا تعبيرًا نمطيًّا. بدونها تصير ⁦.⁩ «أيّ حرف» وتصير ⁦(⁩ خطأ صياغة، وقد يستهلك نمطٌ واحد المعالج زمنًا أسّيًّا. معها تبحث عمّا كتبته حرفًا بحرف.',
  'explain.grep.ignore_case':
    'لا تفرّق بين الحرف الكبير والصغير. لا أثر لها في العربية، وأثرها في الإنجليزية وأسماء الشيفرة.',
  'explain.grep.line_numbers': 'اذكر رقم السطر مع كل نتيجة، فيمكن الذهاب إليه مباشرةً في محرّرك.',
  'explain.grep.pattern_follows': 'ما بعدي هو النمط المطلوب لا راية، حتى لو بدأ بشرطة.',
  'explain.grep.recursive': 'ابحث في المجلد وفي كل ما تحته، لا في مستواه الأول وحده.',
  'explain.grep.tool':
    'أداة البحث داخل الملفات في يونكس. تقرأ وتطبع ولا تكتب شيئًا، ولا تستبدل ما تجده — الاستبدال داخل الملفات ليس من هذه العملية ولا من هذا الإصدار.',
  'explain.ls.acl':
    'اعرض قائمة التحكّم بالوصول (ACL) إن وُجدت. هذه هي الطبقة التي لا يعرضها ⁦chmod⁩ ولا نافذة «عرض المعلومات»، وقاعدةٌ واحدة فيها تكفي لمنع الكتابة بينما يقول وضع الصلاحيات إنها مسموحة.',
  'explain.ls.long':
    'اعرض السطر الطويل: النوع، ووضع الصلاحيات، وعدد الروابط، والمالك والمجموعة، والحجم، وتاريخ آخر تعديل.',
  'explain.ls.self':
    'صِف العنصر نفسه لا محتواه. بدونها يسرد الأمر ما داخل المجلد، فتقرأ صلاحيات أبنائه بدل صلاحياته — والمخرج يبدو معقولًا تمامًا فلا شيء فيه ينبّهك.',
  'explain.ls.tool':
    'أداة سرد الملفات في النظام. اختيرت على ⁦stat⁩ لأنها الوحيدة التي تجمع في مخرجٍ واحد الطبقات الثلاث التي تقرّر من يستطيع فتح الملف: وضع الصلاحيات، وقائمة التحكّم بالوصول، والسمات الممتدّة. و⁦stat⁩ لا ترى الطبقتين الأخيرتين أصلًا.',
  'explain.ls.xattr':
    'اعرض أسماء السمات الممتدّة وأحجامها بالبايت. الأسماء وحدها؛ أمّا قيمها فتُقرأ بعملية «السمات الممتدّة»، لأن قيمة سمةٍ واحدة قد تكون كيلوبايتات من بيانات ثنائية لا مكان لها في سطر صلاحيات.',
  'explain.lsof.listening_only':
    'ومن مقابس TCP، ما حالته «يستمع» وحده — لا الاتصالات القائمة ولا المنصرمة.',
  'explain.lsof.no_lookups':
    'رايتان في رمزٍ واحد كما تكتبهما lsof نفسها: ⁦n⁩ لا تحوّل العناوين إلى أسماء مضيفات، و⁦P⁩ لا تحوّل أرقام المنافذ إلى أسماء خدمات. بدون الأولى يتعلّق الأمر على استعلامات DNS كلّما صادف مقبسًا — وقد لا تعود. وبدون الثانية يُعرض ⁦443⁩ باسم ⁦https⁩، فيبحث القارئ عن رقمٍ لا يجده.',
  'explain.lsof.internet_files':
    'اقتصر على ملفات الإنترنت — أي المقابس — دون سائر ما تفتحه العمليات. وهي هنا مجرّدة من البروتوكول عمدًا: من يسأل عمّن يشغل منفذًا لا يعرف سلفًا أهو TCP أم UDP، وحصرُ السؤال في أحدهما يفترض في السائل معرفةَ ما جاء يسأل عنه.',
  'explain.lsof.port':
    'المنفذ المسؤول عنه. النقطتان بلا مضيفٍ قبلهما تعنيان في صيغة ⁦lsof⁩ «أيّ عنوان، هذا المنفذ». ويُعرض ما عليه بكل حالاته — المستمع والمتّصل معًا — لا المستمعين وحدهم: فما يمنعك من الارتباط بمنفذ قد يكون اتصالًا قائمًا لا مستمعًا.',
  'explain.lsof.process':
    'اقتصر على ما تفتحه هذه العملية بعينها. الرقم الذي يليها هو رقم العملية، ويُقرأ من «العمليات الأعلى استهلاكًا» أو «البحث عن عملية».',
  'explain.lsof.directory_tree':
    'ابحث في هذا المجلد وكل ما تحته لا في مستواه الأول وحده. المِقبض الذي يمنع الحذف أو الإخراج يكون غالبًا في عمقٍ ما، فالمستوى الواحد كان سيقول «لا شيء» وفي الشجرة شيء. وثمنه أن الأداة تمشي الشجرة كاملةً فتتمهّل على الكبيرة.',
  'explain.lsof.tcp_only':
    'اقتصر على مقابس TCP. مقابس UDP خارج هذه العملية عمدًا: «يستمع» ليس لها المعنى نفسه، وجمعُ القائمتين يخلط شيئين مختلفين.',
  'explain.lsof.tool':
    'أداة تعرض الملفات والمقابس المفتوحة والعمليات التي تملكها. اختيرت على ⁦netstat⁩ لأن تلك تعرض المقابس ولا تقول لأيّ برنامج تنتمي، فتجيب نصف السؤال. وهي تربط الطرفين معًا: ما هو مفتوح، ومَن يفتحه.',
  'explain.mkdir.tool':
    'أداة إنشاء المجلدات في النظام. تُستدعى هنا بلا ⁦-p⁩ عمدًا: المجلد الأب مُتحقَّق منه أصلًا فلا آباء ينقصون، و⁦-p⁩ كانت ستخرج بنجاحٍ صامت لو كان المجلد موجودًا — وهو بالضبط التضارب الذي ترفضه هذه العملية.',
  'explain.mv.no_clobber':
    'لا تكتب فوق شيء موجود في الوجهة. هذا حارسٌ في الأداة نفسها يقع تحت حارس التطبيق لا بدلًا منه: التطبيق تحقّق أن الاسم شاغر قبل أن يعرض عليك هذا الأمر، وهذه الراية تسدّ ما بين لحظة الفحص ولحظة التنفيذ.',
  'explain.mv.tool':
    'أداة النقل وإعادة التسمية في النظام. داخل القرص الواحد لا تنسخ بايتًا واحدًا: تغيّر مدخلةً في الدليل، فالنقل فوريّ مهما كبر المجلد. وبين قرصين لا تستطيع ذلك، فتنسخ ثم تحذف.',
  'explain.open.tool':
    'أداة النظام التي تسلّم المسار إلى ⁦LaunchServices⁩ فتفتح Finder عليه. لا تقرأ محتوى ملف ولا تكتب شيئًا؛ كلّ أثرها ظهور نافذة.',
  'explain.ping.count':
    'عدد الرزم التي تُرسل ثم تتوقّف الأداة. بدون هذه الراية تعمل ping على macOS بلا نهاية حتى تُلغى.',
  'explain.ping.deadline': 'سقفٌ زمني للأمر كله بالثواني، مهما بقي من رزم.',
  'explain.ping.deadline_seconds':
    'عشر ثوانٍ، مثبّتة في التطبيق لا حقلًا في النموذج: عدد الرزم وحده لا يحدّ زمن التشغيل، لأن مضيفًا لا يردّ يجعل ping تنتظر كل رزمة حتى تيأس. وping ترسل رزمةً في الثانية، فعددٌ يقارب العشر قد لا يكتمل — وتحذيرٌ يقول ذلك يسبق التنفيذ.',
  'explain.ping.host':
    'المضيف كما كتبتَه، بعد قصّ الفراغ الطرفيّ وحده. لم يُضَف إليه شيء ولم يُحوَّل: ما تراه هنا هو ما يُسأل عنه.',
  'explain.ping.tool':
    'أداة فحص الوصول الأصلية في النظام. تسأل طبقة الشبكة نفسها إن كان الجهاز يردّ (‏ICMP echo)، لا خادمًا أن يجيب طلبًا — وهو الفرق بينها وبين ⁦curl⁩ أو ⁦nc⁩ اللتين تعودان بـ«لا» عن مضيفٍ حيٍّ لا يشغّل ما سألتَ عنه.',
  'explain.role.compare_left': 'الملف الأول، بمساره المطلق. بصمته تُطبع في السطر الأول.',
  'explain.role.compare_right':
    'الملف الثاني، بمساره المطلق. بصمته تُطبع في السطر الثاني، والمقارنة بين السطرين لك.',
  'explain.role.compare_bytes_left': 'الملف الأول، بمساره المطلق.',
  'explain.role.compare_bytes_right':
    'الملف الثاني، بمساره المطلق. لا يُطبع شيء إن تطابقا، وسطرٌ يذكر موضع أول اختلافٍ إن لم يتطابقا.',
  'explain.role.diff_left': 'الملف الأول في المقارنة — «قبل». ما يظهر بعلامة ⁦-⁩ موجودٌ فيه.',
  'explain.role.diff_right': 'الملف الثاني في المقارنة — «بعد». ما يظهر بعلامة ⁦+⁩ موجودٌ فيه.',
  'explain.role.hashed_file':
    'الملف الذي تُحسب بصمته، بمساره المطلق بعد حلّ الروابط الرمزية. يُفتح للقراءة وحدها.',
  'explain.role.image_source':
    'ملف الصورة المصدر، بمساره المطلق بعد حلّ الروابط الرمزية. يُقرأ ولا يُكتب فيه.',
  'explain.role.merge_first':
    'الملف الأول. يُكتب أولًا في الناتج، بمساره المطلق بعد حلّ الروابط الرمزية.',
  'explain.role.merge_second': 'الملف الثاني. يُكتب مباشرةً بعد الأول ولا يُوضع بينهما شيء.',
  'explain.role.moved':
    'المسار النهائي بعد النقل: مجلد الوجهة والاسم الجديد. ‏⁦mv⁩ تكتب هنا مباشرة، فلا ملف مؤقّت ولا ترقية بعد النجاح — إمّا نجحت فصار الاسم في موضعه، وإمّا فشلت فبقي كلّ شيء كما كان.',
  'explain.role.new_dir':
    'المسار الكامل للمجلد الذي سيُنشأ: المجلد الأب والاسم الجديد. ‏⁦mkdir⁩ ذرّيّة في نفسها، فتفشل إن سبق أحدٌ إلى الاسم بين لحظة الفحص ولحظة التنفيذ.',
  'explain.role.search_folder':
    'المجلد المبحوث فيه، بمساره المطلق بعد حلّ الروابط الرمزية. لا يتغيّر فيه شيء.',
  'explain.role.source_file':
    'الملف المصدر، بمساره المطلق بعد حلّ الروابط الرمزية. يُقرأ ولا يُعدَّل ولا يُحذف.',
  'explain.shasum.algorithm':
    'اختر خوارزمية البصمة صراحةً. بدون هذه الراية تحسب ⁦shasum⁩ بصمة SHA-1 افتراضيًا. تُملى الخوارزمية هنا من الشيفرة ولا تُترك لافتراضٍ قد يتغيّر مع إصدار الأداة، فما يُعرض على الشاشة اليوم هو ما سيُحسب غدًا.',
  'explain.shasum.sha256':
    'SHA-256. اختيرت على MD5 وSHA-1 لأن بناء ملفين مختلفين ببصمةٍ واحدة صار عمليًّا في كلتيهما — سنة ٢٠٠٩ في الأولى و٢٠١٧ في الثانية. الاثنتان تكشفان التلف العابر في النقل، ولا تصلحان لقول «هذا هو الملف نفسه الذي نُشر»، والبصمة على الشاشة تُقرأ على أنها الثانية.',
  'explain.shasum.tool':
    'أداة البصمات التي تأتي مع macOS. اختيرت على ⁦openssl dgst⁩ لأن صيغة خرج تلك بادئةٌ وقوسان تغيّرت بين الإصدارات، وعلى ⁦md5⁩ لأنها لا تحسب غير MD5. وتطبع البصمة بنفس صيغة ⁦sha256sum⁩ على لينكس — وهي غير موجودة على macOS — فما يُنسخ من هنا يُقارن هناك.',
  'explain.sips.format.heic':
    'HEIC: نحو نصف حجم JPEG بالجودة نفسها، وهي صيغة كاميرا iPhone. وثمنُها أن دعمها خارج أجهزة Apple ما زال ناقصًا، فليست خيارًا لمرفقٍ يُرسل إلى جهةٍ لا تعرف جهازها.',
  'explain.sips.format.jpeg':
    'JPEG: ضغطٌ بخسارة، وأصغر الملفات على الصور الفوتوغرافية، وتفتحها كل أداةٍ على كل نظام. لا تحمل شفافية، وكل حفظٍ جديد يخسر جيلًا من الجودة.',
  'explain.sips.format.png':
    'PNG: ضغطٌ بلا خسارة مع شفافية. الأنسب للقطات الشاشة والرسوم وما فيه نصٌّ أو حوافّ حادّة، والأكبر حجمًا على الصور الفوتوغرافية.',
  'explain.sips.format.tiff':
    'TIFF: بلا خسارة، وأكبر الأربع حجمًا. صيغة الأرشفة والطباعة والتحرير اللاحق، لا صيغة مشاركة.',
  'explain.sips.format_property':
    'اسم الخاصية المضبوطة: صيغة الترميز نفسها — أي المرمِّز الذي ستُكتب به البايتات، لا امتداد الاسم.',
  'explain.sips.get': 'اقرأ خاصيةً من خصائص الصورة واعرضها. تسأل ولا تكتب شيئًا.',
  'explain.sips.get_all':
    'اطلب الخصائص كلها بدل واحدةٍ بعينها: الأبعاد، والدقّة، والعمق وعدد القنوات، وفضاء الألوان وملفّ تعريفه، والصيغة.',
  'explain.sips.max_pixels':
    'الحدّ بالبكسل. يُطبَّق على أطول ضلعي الصورة وحده، وعددٌ أكبر من ذلك الضلع يكبّر الصورة لا يتركها كما هي.',
  'explain.sips.out':
    'اكتب الناتج في المسار التالي. بدون هذه الراية تكتب sips فوق الملف المصدر مباشرةً، فتضيع الصورة الأصلية بلا رجعة.',
  'explain.sips.resample_max':
    'حدُّ الضلع الأطول (‏resampleHeightWidthMax). يعيد أخذ عيّنات الصورة حتى لا يتجاوز أطولُ ضلعيها العددَ التالي، ويحسب الضلع الآخر بالنسبة نفسها فلا يتشوّه الشكل. والحرف كبيرٌ عمدًا: ⁦-z⁩ الصغيرة تفرض الارتفاع والعرض معًا فتسطّح الصورة.',
  'explain.sips.rotate':
    'أدر الصورة بالزاوية التالية مع عقارب الساعة. يدوّر البكسلات نفسها ويعيد كتابتها، لا وسمَ الاتجاه في الترويسة — فتظهر مدارةً حتى في البرامج التي لا تقرأ الوسم.',
  'explain.sips.rotate.180': 'نصف دورة: الصورة تنقلب رأسًا على عقب. الأبعاد لا تتغيّر.',
  'explain.sips.rotate.270':
    'ثلاثة أرباع دورة مع العقارب، أي ربع دورة عكسها: أعلى الصورة يصير يسارها. يتبادل ضلعا الصورة.',
  'explain.sips.rotate.90':
    'ربع دورة مع عقارب الساعة: أعلى الصورة يصير يمينها. يتبادل ضلعا الصورة.',
  'explain.sips.set': 'اضبط خاصيةً من خصائص الصورة. تأخذ رمزين بعدها: اسم الخاصية، ثم قيمتها.',
  'explain.sips.tool':
    'أداة الصور الأصلية في macOS. تمرّ عبر ImageIO — الفكّاك نفسه الذي يستعمله Preview وQuick Look — فما تراه في المعاينة هو ما تكتبه الأداة. اختيرت على ImageMagick لأنها جزءٌ من النظام: أداةٌ تُثبَّت من الخارج يستطيع أيُّ برنامج استبدالها بلا صلاحيات مدير، ويصير ناتجُ العملية رهنَ نسخةٍ لا نعرفها ولا نستطيع وصفها هنا.',
  'explain.spctl.assess': 'قيّم الهدف بسياسة التنفيذ: أيسمح النظام بفتحه أم يرفضه.',
  'explain.spctl.tool':
    'أداة تقييم سياسة الأمان في النظام. تسأل المحرّك نفسه الذي يسأله النظام عند النقر المزدوج على ما نُزّل، لا فحصًا يشبهه — وأي فحصٍ نكتبه بأنفسنا كان سيتقادم صامتًا مع أول تحديثٍ يغيّر السياسة.',
  'explain.spctl.verbose':
    'أسهب مرّتين. المستوى الأول يعطي الحكم وحده؛ والثاني يضيف الجهة الموقِّعة — و«مرفوض» بلا مصدرٍ لا يفرّق بين ملفٍ مزوَّر وملفٍ لم يُوثَّق بعد.',
  'explain.split.lines':
    'عدد الأسطر في كل جزء. القطع يقع عند حدود الأسطر لا عند حدود البايتات، فلا ينقسم سطرٌ واحد بين جزأين.',
  'explain.split.prefix':
    'بادئة أسماء الأجزاء، لا مجلدًا: تُلحق بها ⁦split⁩ حرفين (‏aa ثم ab…) فتصير الأجزاء ⁦part-aa⁩ و⁦part-ab⁩. وهي داخل مجلدٍ مؤقّت يملكه التطبيق، فلا ينزل في مجلدك جزءٌ واحد إلا بعد نجاحٍ كامل.',
  'explain.split.tool':
    'أداة تقسيم الملفات في يونكس. تقرأ الملف وتكتب أجزاءه ملفاتٍ مستقلّة، وتترك الأصل كما هو.',
  'explain.textutil.convert':
    'حوّل المستند إلى الصيغة التالية. ⁦txt⁩ تعني نصًّا مجرّدًا بلا خطوطٍ ولا تنسيق.',
  'explain.textutil.input_encoding':
    'اقرأ بايتات الملف بهذا الترميز بدل أن تُخمّنه. هذا هو الجدول الذي تُفسَّر به البايتات القديمة، وهو جوهر هذه العملية.',
  'explain.textutil.output':
    'اكتب الناتج في المسار التالي. وهو الملف المؤقّت لا الاسم النهائي، فالترقية لا تقع إلا بعد نجاحٍ كامل.',
  'explain.textutil.output_encoding':
    'اكتب الناتج بترميز ⁦UTF-8⁩، وهو ما تقرؤه أنظمة اليوم كلها بلا إعداد.',
  'explain.textutil.tool':
    'أداة النصوص الأصلية في macOS. تمرّ بنظام النصّ في Cocoa فتفهم ⁦rtf⁩ و⁦doc⁩ و⁦html⁩ لا البايتات وحدها. اختيرت على ⁦iconv⁩ لأن ⁦iconv⁩ تعيد ترميز الملف بايتًا ببايت فتُفسد ما ليس نصًّا مجرّدًا، ولأن ⁦textutil⁩ تكتب الناتج بنفسها عبر ⁦-output⁩.',
  'explain.xattr.list':
    'اسرد كل سمةٍ باسمها وقيمتها. هذه راية قراءة؛ ورايات الحذف (⁦-d⁩ و⁦-c⁩) والكتابة (⁦-w⁩) ليست في هذا الأمر ولا سبيل إلى إدخالها فيه — الأمر يُبنى في شيفرة مترجَمة، ولا حقل في هذه العملية يمكن أن يصير راية.',
  'explain.xattr.tool':
    'أداة السمات الممتدّة في macOS. السمة بياناتٌ مرفقة بالملف خارج محتواه: لا يعرضها Finder، ولا تدخل في حجم الملف، وتنتقل معه حين يُنسخ.',

  // ── التحذيرات — تُعرض قبل التنفيذ ولا تمنعه ───────────────────────────
  'warn.atime.unreliable':
    'زمن آخر وصولٍ إلى الملف غير موثوق على macOS: النظام يؤجّل تحديثه توفيرًا للكتابة، وفهرسةُ Spotlight والنسخُ الاحتياطي وماسحاتُ الفيروسات تلمس ملفاتٍ لم يفتحها أحد. فقد يغيب عن القائمة ملفٌ منسيّ، وقد يظهر فيها ملفٌ تستعمله. راجعها قبل أن تبني عليها قرارًا.',
  'warn.codesign.unsigned':
    'تخرج ⁦codesign⁩ برمزٍ غير صفري حين لا يكون الهدف موقَّعًا، والتطبيق يعتبر أي خروجٍ غير صفري تشغيلًا فاشلًا. فـ«لم تكتمل العملية» هنا قد تعني «لا توقيع على هذا الملف» — وهي إجابةٌ عن السؤال لا فشلٌ في طرحه.',
  'warn.compare.left_resolved':
    'الملف الأول رابط رمزي. ستُحسب بصمة الموضع الذي يشير إليه، وهو المسار الظاهر في الأمر أدناه.',
  'warn.compare.right_resolved':
    'الملف الثاني رابط رمزي. ستُحسب بصمة الموضع الذي يشير إليه، وهو المسار الظاهر في الأمر أدناه.',
  'warn.compare.bytes_left_resolved':
    'الملف الأول رابط رمزي. ستجري المقارنة على الموضع الذي يشير إليه، وهو المسار الظاهر في الأمر أدناه.',
  'warn.compare.bytes_right_resolved':
    'الملف الثاني رابط رمزي. ستجري المقارنة على الموضع الذي يشير إليه، وهو المسار الظاهر في الأمر أدناه.',
  'warn.compare.size_differs':
    'حجما الملفين مختلفان، فبصمتاهما ستختلفان حتمًا. ما زال التنفيذ ممكنًا إن كنت تريد البصمتين نفسيهما، لكن جواب «هل هما ملف واحد؟» معروفٌ قبل أن يبدأ.',
  'warn.curl.head_may_differ':
    'ليس كل خادم يعامل طلب الترويسات كما يعامل التنزيل: بعضها يردّ ⁦405⁩، وبعضها يردّ ترويسات تختلف في الحجم أو في الكعكات. فالمعروض ترويسات هذا الطلب، لا ترويسات تنزيلٍ كامل.',
  'warn.diff.exit_code':
    'تخرج ⁦diff⁩ بالرمز ١ حين تجد فرقًا، وهو المتوقّع لا عطب. لكن التطبيق يعدّ كل خروجٍ غير صفري فشلًا اليوم، فقد تقول الشاشة «لم تكتمل العملية» بينما الفرق معروضٌ أمامك كاملًا.',
  'warn.dns.mdns_not_dns':
    'أسماء ⁦.local⁩ تُحلّ بـ mDNS على الشبكة المحلية لا بـ DNS، ولا تعرفها dig. سيُسأل الخادم ويعود بلا إجابة.',
  'warn.dns.single_label':
    'الاسم بلا نقطة، فليس اسمًا كاملًا. وdig لا تضيف لواحق البحث افتراضيًا، بل تسأل عنه كما هو — ويعود الجواب فارغًا في الغالب.',
  'warn.download.destination_resolved':
    'مجلد الوجهة رابط رمزي. سيُنزَّل الملف إلى الموضع الذي يشير إليه، وهو المسار الظاهر في الأمر أدناه.',
  'warn.download.unverified':
    'لا يتحقّق التطبيق من بصمة الملف ولا من توقيعه: ينزّل ما يرسله الخادم كما هو. تحقّق مما نزّلته قبل فتحه، وخاصةً إن كان برنامجًا.',
  'warn.encoding.silent_mojibake':
    'الترميزات القديمة أحادية البايت تقبل أي بايت، فاختيار ترميزٍ غير ترميز الملف لا يُفشل العملية بل يُنتج نصًّا مشوّهًا برمز خروجٍ ناجح. افتح الناتج وتحقّق منه قبل أن تحذف الأصل.',
  'warn.grep.exit_code':
    'تخرج ⁦grep⁩ بالرمز ١ حين لا تجد أي نتيجة، وهو جوابٌ لا عطب. لكن التطبيق يعدّ كل خروجٍ غير صفري فشلًا اليوم، فقد تقول الشاشة «لم تكتمل العملية» ومعناها «لا نتائج».',
  'warn.hash.slow':
    'الملف كبير. ⁦shasum⁩ تقرؤه كاملًا ولا تطبع شيئًا حتى تنتهي، فقد يطول الانتظار بلا مؤشّر تقدّم. الإلغاء ممكن في أي لحظة، ولا يترك أثرًا لأن شيئًا لا يُكتب.',
  'warn.hash.source_resolved':
    'ما اخترته رابط رمزي. ستُحسب بصمة الموضع الذي يشير إليه، وهو المسار الظاهر في الأمر أدناه.',
  'warn.image.format_extension':
    'امتداد الاسم الذي اخترته لا يوافق الصيغة المطلوبة، وsips لا تستنتج الصيغة من اسم الناتج. فستُكتب بيانات الصيغة المختارة داخل ملفٍ يحمل امتدادًا آخر، وتقرؤه مشوّهًا كلُّ أداةٍ تثق بالامتداد.',
  'warn.image.metadata':
    'إعادة الترميز تمرّ بالبيانات الوصفية، وما يبقى منها يقرّره زوج الصيغتين لا هذه العملية: قيس أن الوصف وحقوق النسخ نجيا من JPEG إلى PNG. فلا تعتمد على هذه العملية لإزالة الموقع الجغرافي أو بيانات الكاميرا قبل المشاركة — تلك عمليةٌ أخرى لم تُضف بعد.',
  'warn.image.no_extension':
    'الاسم الذي اخترته بلا امتداد. الملف سيُكتب صحيحًا، لكن Finder وغيره لن تعرف صيغته من اسمه.',
  'warn.image.recompress':
    'التدوير بزاوية قائمة يمكن أن يقع بلا خسارة، وsips لا تفعله كذلك: تفكّ الصورة وتعيد ترميزها. فتخسر ملفات JPEG جيلًا من الجودة على صورةٍ لم يتغيّر محتواها في الحقيقة.',
  'warn.image.source_extension':
    'امتداد الاسم الذي اخترته يخالف امتداد الملف المصدر، وهذه العملية لا تغيّر الصيغة. فستُكتب بيانات صيغة المصدر داخل ملفٍ يحمل امتدادًا آخر. ولتغيير الصيغة فعلًا استعمل «تحويل صيغة صورة».',
  'warn.image.suffix_notice':
    'ستطبع sips ملاحظةً عن امتداد الملف الناتج. تخصّ الاسم المؤقّت الذي تكتب فيه العملية قبل الترقية، لا الاسم الذي اخترته، والملف يُكتب صحيحًا.',
  'warn.image.upscale':
    'الرقم حدٌّ على الضلع الأطول لا تصغيرٌ مضمون: إن كان أكبر من ذلك الضلع كبّرت sips الصورة إليه. والتكبير لا يضيف تفصيلًا لم يكن في الأصل، بل يخترع بكسلاتٍ بينية فتبدو الصورة أنعم لا أوضح.',
  'warn.kill.signal_not_guarantee':
    'إرسال الإشارة ليس موت العملية: الأمر ينجح حين تُرسَل لا حين تنتهي. وبرنامجٌ يلتقط الإشارة ويتجاهلها يبقى يعمل ورمزُ الخروج صفر، وآخرُ يحفظ عمله قبل أن يخرج يستغرق لحظات. راجع قائمة العمليات بعدها لتعرف.',
  'warn.kill.no_undo':
    'لا تراجع بعد هذا. ما لم يُحفظ في البرنامج المستهدَف يضيع، ولا تستعيده هذه العملية ولا غيرها. تأكّد من الرقم قبل التنفيذ: رقمٌ خاطئ يُنهي برنامجًا آخر.',
  'warn.log.bounded_window':
    'الجواب مقصورٌ على الأخطاء والأعطال داخل المدّة التي اخترتها، وليس سجلّ النظام كاملًا. والأخطاء تأتي نوبًا لا بمعدّلٍ ثابت، فقد يفوق خرجُ نوبةٍ كثيفة ما تعرضه الشاشة فيُقصّ — وإن قُصّ قيل لك كم سقط. وقد تُحجب قيمٌ حسّاسة داخل السطور بكلمة ⁦private⁩، وذاك حجبُ النظام لا حجبُنا: اقرأ قبل أن تلصق.',
  'warn.lsof.deep_scan':
    'تمشي الأداة المجلد وكل ما تحته، فقد تتمهّل على شجرةٍ كبيرة. لا تُعرض نسبة تقدّم لأن الأداة لا تعطي واحدة، والإلغاء في أي لحظة بلا أثر: لا شيء يُكتب.',
  'warn.lsof.user_scope':
    'بلا صلاحيات مدير لا ترى lsof إلا عمليات المستخدم الحالي: ما يملكه النظام أو مستخدمٌ آخر لا يظهر إطلاقًا. فالجواب هنا قد يكون ناقصًا، وقد يكون فارغًا تمامًا عن شيءٍ قائمٍ فعلًا. وهذا التطبيق لا يطلب صلاحيات المدير ولا يعرض عليك منحها.',
  'warn.move.cross_device':
    'المصدر والوجهة على قرصين مختلفين. ‏⁦mv⁩ لا تستطيع إعادة التسمية عبر الأقراص، فتنسخ ثم تحذف: العملية أبطأ بكثير على المجلدات الكبيرة، وليست ذرّيّة — فانقطاعها في المنتصف يترك نسخةً ناقصة في الوجهة ويُبقي الأصل في مكانه.',
  'warn.ping.deadline_clips_count':
    'المهلة عشر ثوانٍ، وping ترسل رزمةً في الثانية، فالعدد الذي اخترته قد لا يكتمل. تخرج الأداة عند المهلة بما جمعته، وتحسب النسبة على ما أُرسل فعلًا.',
  'warn.spctl.exit_code':
    'تخرج ⁦spctl⁩ برمزٍ غير صفري حين ترفض الهدف، والتطبيق يعتبر أي خروجٍ غير صفري تشغيلًا فاشلًا. فإن قالت الشاشة «لم تكتمل العملية» فاقرأ الحكم في مجرى الخرج قبل أن تستنتج: قد يكون رفضًا صحيحًا لهدفٍ لا يقبله النظام، أو لهدفٍ ليس تطبيقًا أصلًا.',
  'warn.split.empty_source': 'الملف فارغ، فلن تُنتج ⁦split⁩ جزءًا واحدًا وستنتهي العملية بلا ناتج.',
  'warn.split.suffix_limit':
    'قد يتجاوز عدد الأجزاء ٦٧٦ جزءًا، وهو أقصى ما تسعه لواحق ⁦split⁩ الافتراضية. هذا احتمالٌ لا يقين — فعدد الأسطر لا يُعرف دون قراءة الملف كاملًا — وإن وقع توقّفت الأداة ولم يُنشأ المجلد في وجهتك أصلًا. زد عدد الأسطر في كل جزء إن أردت تفاديه.',
  'warn.target.resolved':
    'ما اخترته رابط رمزي. سيُفحص الموضع الذي يشير إليه، وهو المسار الظاهر في الأمر أدناه — لا الرابط نفسه.',
  'warn.text.glued_lines':
    'الملف الأول لا ينتهي بسطرٍ جديد، فسيلتصق آخر سطرٍ فيه بأول سطرٍ في الثاني ويصيران سطرًا واحدًا في الناتج.',
  'warn.text.no_separator':
    'لا تضع ⁦cat⁩ فاصلًا بين الملفين: يبدأ الثاني حيث انتهى الأول بايتًا ببايت. إن أردت سطرًا فاصلًا فأضفه إلى آخر الملف الأول قبل الدمج.',
  'warn.url.plaintext':
    'العنوان بلا تشفير (⁦http⁩)، فما يُرسل ويُستقبل يمرّ نصًّا ظاهرًا على الشبكة، ويستطيع من كان بينك وبين الخادم أن يقرأه أو يبدّله.',
} as const;

export type MessageKey = keyof typeof AR;

/** يترجم مفتاحًا. المفتاح غير المعروف يُعرض كما هو بدل أن يختفي النص. */
export function t(key: string): string {
  return (AR as Record<string, string>)[key] ?? key;
}

/**
 * يترجم أوّل مفتاح موجود من قائمة، من الأخصّ إلى الأعمّ.
 *
 * يخدم نصوص الحقول: عمليةٌ تريد صياغةً خاصة تعلن `field.<op>.<input>.label`،
 * وما لم تفعل يُستعمل العام `field.<input>.label`. بدون هذا كان كل حقلٍ مألوف
 * في كل عملية جديدة يحتاج نصًّا مكرّرًا. آخر مفتاح هو ما يُعرض إن لم يوجد شيء،
 * فيبقى الغياب مرئيًا لا صامتًا.
 */
export function tFirst(keys: readonly string[]): string {
  for (const key of keys) {
    if (key in AR) return t(key);
  }
  return keys[keys.length - 1] ?? '';
}

/**
 * النصّ إن وُجد مفتاحه، و`null` إن لم يوجد.
 *
 * لنصٍّ **اختياري** يُطوى حين يغيب بدل أن يظهر مفتاحه. و`t` لا تصلح له لأنها
 * تُرجع المفتاح نفسه عند الغياب — وهو سلوكٌ مقصود يجعل النقص مرئيًا في نصٍّ
 * إلزامي، وكارثةٌ في نصٍّ قد لا يكون له مفتاح أصلًا (‏`internal.echo` مثلًا).
 * و`tFirst` لا تصلح كذلك: آخر مفاتيحها هو ما تعرضه عند الغياب.
 */
export function tOptional(key: string): string | null {
  return key in AR ? t(key) : null;
}

/**
 * نصٌّ فيه مواضع تُملأ، مثل «الإصدار {version}».
 *
 * البديل كان تركيب النص بالجمع في الشاشة (`t('...') + version`)، وهو يعمل في
 * العربية اليوم ويكسر أول ما يحتاج النصّ ترتيبًا مختلفًا — والعربية تحديدًا
 * تضع الرقم حيث لا تضعه الإنجليزية. الموضع المسمّى يبقى داخل النصّ المترجَم،
 * فيملك المترجِم ترتيبه.
 *
 * الموضع الذي لا قيمة له يبقى كما هو: ظهور `{version}` على الشاشة عطبٌ مرئي
 * يُصلَح، وحذفه بصمت كان سيخفيه.
 */
export function tFormat(key: string, vars: Readonly<Record<string, string>>): string {
  return t(key).replace(/\{(\w+)\}/g, (whole, name: string) => vars[name] ?? whole);
}

/**
 * يترجم خطأً، مفضّلًا الصياغة الأدقّ حين تصف النواة السبب.
 *
 * `err.name.invalid` وحده يقول «اسم غير صالح»؛ ومع `detail: "leading_dot"`
 * يصير «اسم يبدأ بنقطة يُنشئ ملفًا مخفيًا لن تجده» — وهذه رسالة تُصلح.
 */
export function errorText(key: string, detail?: unknown): string {
  if (typeof detail === 'string') {
    const specific = `${key}.${detail}`;
    if (specific in AR) return t(specific);
  }
  return t(key);
}
