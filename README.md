<div dir="rtl">

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/header-ar-dark.png">
  <img alt="نفّذ — Naffith: افعلْ ما تريد، وافهمْ ما فعلت. تختار «قياس حجم مجلد» فيُعرض الأمر du -h -d 2 قبل أن يعمل" src="docs/assets/header-ar-light.png" width="100%">
</picture>

[![Release](https://img.shields.io/github/v/release/iSltanX/naffith?label=release&color=C53F01&style=flat-square)](https://github.com/iSltanX/naffith/releases/latest)
[![macOS 12+](https://img.shields.io/badge/macOS-12%2B%20%C2%B7%20Apple%20Silicon-272831?style=flat-square)](#المتطلبات)
[![No shell](https://img.shields.io/badge/no%20shell-fixed%20commands-4F505E?style=flat-square)](#الأمان)
[![License: MIT](https://img.shields.io/badge/license-MIT-7C7D8E?style=flat-square)](LICENSE)

### [⬇︎ تنزيل أحدث إصدار](https://github.com/iSltanX/naffith/releases/latest)

<sub>مجاني ومفتوح المصدر · macOS 12 أو أحدث · Apple Silicon</sub>

[الفكرة](#الفكرة) · [طريقة العمل](#طريقة-العمل) · [الميزات](#الميزات) · [التثبيت](#التثبيت) · [الاستخدام](#الاستخدام) · [لقطات الشاشة](#لقطات-الشاشة) · [الأمان](#الأمان) · [الأسئلة المتكررة](#الأسئلة-المتكررة)

</div>

---

## الفكرة

كثير من مهام macOS اليومية لا طريق إليها إلا الطرفية: صيغة أمر تحفظها، وحروف لا تتذكرها، وواجهة لا تتحدث العربية أصلًا.

**نفّذ** يضع بينك وبين النظام طبقة هادئة: تختار ما تريد بالاسم، لا بالأمر. نقل ملف، ضغط مجلد، فحص مستودع Git، معرفة المساحة الحرة. ويبقى الأمر نفسه معروضًا أمامك قبل أن يعمل، فلا شيء يحدث في الخفاء.

---

## طريقة العمل

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/steps-ar-dark.png">
  <img alt="تختار العملية بالاسم، ثم ترى الأمر في سطر قبل أن يعمل، ثم تقرأ النتيجة جدولًا منظمًا" src="docs/assets/steps-ar-light.png" width="100%">
</picture>

1. **تختار العملية** من قائمة مقسّمة، أو تبحث عنها بالعربية.
2. **تملأ حقلًا أو حقلين،** فتظهر معاينة تقول بالضبط ما سيحدث.
3. **ترى الأمر في «سَطْر»،** مكتوبًا ومشروحًا كلمةً كلمة، إن أردت أن تفهمه.
4. **تؤكّد فيُنفَّذ،** وتظهر النتيجة جداول وملخّصات، لا نصًّا خامًا.

---

## الميزات

**79 عملية في عشرة أقسام:**

| القسم | أمثلة |
| --- | --- |
| **الملفات والمجلدات** | نسخ ونقل وإنشاء، والعثور على الملفات الكبيرة والقديمة، وقياس حجم مجلد |
| **الضغط** | أرشفة ZIP وTAR.GZ وفكّها، وفحص محتواها قبل ذلك |
| **الصور** | تحويل الصيغة، وتغيير الأبعاد، والتدوير، وقراءة الخصائص |
| **النصوص** | دمج وتقسيم، وتحويل الترميز إلى UTF-8، والمقارنة والبحث داخل الملفات |
| **الأقراص** | المساحة الحرة، وبصمة SHA-256، ومقارنة ملفين، والأقراص المتصلة |
| **الشبكة** | فحص الوصول وDNS والمنافذ المستمعة، والتنزيل من رابط |
| **الأمان** | قراءة الصلاحيات والسمات والتواقيع، دون تعديل أي شيء |
| **Git** | إنشاء مستودع، وحالته، وتسجيل commit، ومقارنة التغييرات |
| **النظام** | العمليات الأعلى استهلاكًا، ومعلومات النظام، وتفريغ ذاكرة DNS |
| **أدوات المطوّرين** | فحص الأنواع والأسلوب والاختبارات، وتشغيل مشاريع Node.js وTauri |

- **الأمر ظاهر قبل التنفيذ.** كل عملية تُعرض أمرًا صريحًا مشروحًا، بلا أسطر مخفية.
- **نتائج منظّمة.** جداول وملخّصات وحالة، بدل قراءة الخرج سطرًا سطرًا.
- **سجل العمليات.** كل تشغيل محفوظ بوقته ونتيجته، ويُعاد بالقيم نفسها بضغطة.
- **عربي أولًا.** واجهة من اليمين إلى اليسار بخطَّي Cairo وAlmarai، ووضعان فاتح وداكن.

---

## التثبيت

1. نزّل ملف <span dir="ltr">`.dmg`</span> من [صفحة الإصدارات](https://github.com/iSltanX/naffith/releases/latest).
2. افتحه واسحب **نفّذ** إلى مجلد **التطبيقات**.
3. شغّله، وتظهر شاشة ترحيب قصيرة تعرّفك بالأقسام وبلوحة «سَطْر».

</div>

> [!IMPORTANT]
> **تنبيه Gatekeeper:** نفّذ موقَّع ذاتيًا ولم يمرّ بتوثيق Apple (Notarization)، لأنه مشروع شخصي. لذلك يرفض macOS فتحه أول مرة.
>
> - **في macOS 15 فما بعد:** حاول فتحه مرة، ثم افتح **إعدادات النظام ← الخصوصية والأمن** واضغط **افتح على أي حال**.
> - **في macOS 12 إلى 14:** في Finder انقر على التطبيق بالزر الأيمن ← **فتح** ← **فتح**.
>
> تكفي مرة واحدة.

<div dir="rtl">

### المتطلبات

- نظام macOS 12 (Monterey) أو أحدث.
- معالج **Apple Silicon** للإصدار الجاهز. على Intel، ابنِه من المصدر ([للمطوّرين](#للمطوّرين)).
- أدوات المطوّرين وحدها تحتاج **Node.js** و**Cargo**، ويمكن تحديد مساريهما من الإعدادات.

---

## الاستخدام

اختر قسمًا من الشريط الجانبي، ثم عملية، ثم املأ حقولها. يبني نفّذ الأمر ويعرضه عليك، ولا ينفّذه إلا بعد موافقتك.

| من الإعدادات | ما يفعله |
| --- | --- |
| **تأكيد قبل التنفيذ** | يطلب موافقتك قبل كل عملية، وهو مفعّل افتراضيًا |
| **مسار العمل الافتراضي** | المجلد الذي تبدأ منه العمليات |
| **مظهر التطبيق** | داكن، أو فاتح، أو حسب النظام، وحجم أيقونات الشريط الجانبي |
| **صوت الإشعارات** | تنبيه عند انتهاء عملية طويلة |
| **أدوات المطوّرين** | مسارا Node.js وCargo إن لم يُكتشفا تلقائيًا |
| **شاشة الترحيب** | تعرضها من جديد متى شئت |

---

## لقطات الشاشة

<table>
  <tr>
    <td width="50%" align="center" valign="top">
      <img alt="قسم من أقسام العمليات في نفّذ" src="docs/assets/screen-category.png" width="100%"><br>
      <b>الأقسام</b><br>
      كل عملية باسمها ووصفها، في قسمها.
    </td>
    <td width="50%" align="center" valign="top">
      <img alt="عملية مفتوحة ولوحة سطر تعرض الأمر" src="docs/assets/screen-operation.png" width="100%"><br>
      <b>العملية وسَطْر</b><br>
      الحقول والمعاينة، والأمر مشروحًا قبل أن يعمل.
    </td>
  </tr>
  <tr>
    <td width="50%" align="center" valign="top">
      <img alt="نافذة تأكيد قبل تنفيذ عملية تنشئ ملفًا جديدًا" src="docs/assets/screen-confirm.png" width="100%"><br>
      <b>التأكيد قبل التنفيذ</b><br>
      يقول ما سيحدث، والخيار الآمن هو الافتراضي.
    </td>
    <td width="50%" align="center" valign="top">
      <img alt="مكتبة الأقسام في نفّذ بالوضع الفاتح" src="docs/assets/screen-light.png" width="100%"><br>
      <b>المكتبة</b><br>
      عشرة أقسام وسجل التشغيل، بالوضع الفاتح.
    </td>
  </tr>
</table>

---

## الأمان

نفّذ لا يشغّل نصًّا حرًّا في صدفة خفية (shell). كل عملية برنامج واحد محدَّد، ومدخلاتها من حقول مفحوصة، ويُعرض الأمر عليك كاملًا قبل أن يعمل.

- **حدود واضحة:** يعمل داخل مجلد المنزل و<span dir="ltr">`/Volumes`</span> فقط، ويرفض مجلدات حساسة مثل <span dir="ltr">`~/.ssh`</span> وسلسلة المفاتيح وملفات تعريف الارتباط.
- **لا صلاحيات مرتفعة:** لا يستخدم <span dir="ltr">`sudo`</span> أبدًا.
- **عمليات الأمان قراءة فقط:** فحص الصلاحيات والتواقيع وGatekeeper لا يعدّل شيئًا.
- **لا خادم ولا تتبّع:** يعمل على جهازك مباشرة، ولا يتصل بالشبكة إلا حين تشغّل أنت عملية شبكة، كفحص عنوان أو تنزيل ملف.

---

## الأسئلة المتكررة

<details>
<summary><strong>لماذا يحذّرني macOS حين أفتحه أول مرة؟</strong></summary><br>

لأن نفّذ موقَّع ذاتيًا لا بشهادة Apple Developer ID، ولم يمرّ بتوثيق Apple. طريقة الفتح في [التثبيت](#التثبيت)، وتكفي مرة واحدة.
</details>

<details>
<summary><strong>هل يمكن أن يفسد نفّذ ملفاتي؟</strong></summary><br>

كل عملية تُعرض عليك أمرًا صريحًا قبل تنفيذها، والتأكيد مفعّل افتراضيًا. ولا يعمل خارج مجلد المنزل والأقراص المتصلة، ولا يلمس المجلدات الحساسة، ولا يستخدم <span dir="ltr">`sudo`</span>.
</details>

<details>
<summary><strong>هل يتصل بالإنترنت؟</strong></summary><br>

فقط حين تشغّل عملية تحتاجه، كفحص وصول مضيف أو تنزيل ملف. لا حساب، ولا تحليلات، ولا تحقق تلقائي من التحديثات.
</details>

<details>
<summary><strong>لماذا لا تعمل بعض عمليات النظام كاملة؟</strong></summary><br>

لأن نفّذ لا يطلب صلاحيات المدير. تفريغ ذاكرة DNS مثلًا يتم جزئيًا، لأن جزءه الآخر يحتاج <span dir="ltr">`sudo`</span>.
</details>

<details>
<summary><strong>هل الواجهة متاحة بالإنجليزية؟</strong></summary><br>

ليس بعد. الواجهة عربية بالكامل حاليًا.
</details>

<details>
<summary><strong>هل يعمل على معالجات Intel؟</strong></summary><br>

الإصدار الجاهز لـ Apple Silicon. على Intel يمكنك بناؤه من المصدر، والخطوات في [للمطوّرين](#للمطوّرين).
</details>

<details>
<summary><strong>كيف أحدّثه، وكيف أزيله؟</strong></summary><br>

**التحديث** يدوي: نزّل الإصدار الجديد من صفحة الإصدارات واستبدل التطبيق.

**الإزالة:** احذف التطبيق من مجلد التطبيقات، ثم احذف مجلد بياناته <span dir="ltr">`~/Library/Application Support/com.naffith.satr/`</span> وفيه سجل آخر 500 عملية.
</details>

---

## للمطوّرين

<details>
<summary><b>البناء من المصدر</b></summary><br>

**المتطلبات:** macOS 12 أو أحدث، وأدوات سطر أوامر Xcode، وRust، وNode.js 22.22 أو 24.15 أو أحدث.

| الأمر | ما يفعله |
| --- | --- |
| <span dir="ltr">`npm install`</span> | يثبّت الاعتماديات |
| <span dir="ltr">`npm run dev`</span> | التطبيق كاملًا (Tauri وVite) |
| <span dir="ltr">`npm run build`</span> | يبني <span dir="ltr">`.app`</span> و<span dir="ltr">`.dmg`</span> محليًا، بلا توقيع |
| <span dir="ltr">`npm run typecheck`</span> · <span dir="ltr">`npm run lint`</span> | فحص الأنواع والأسلوب |
| <span dir="ltr">`npm run test:ui`</span> | اختبارات الواجهة (Vitest) |
| <span dir="ltr">`npm run test:core`</span> · <span dir="ltr">`npm run lint:core`</span> | اختبارات النواة وclippy |

النواة Rust والواجهة React داخل Tauri 2. مجموعة أيقونات macOS مجمَّعة يدويًا، فلا تشغّل <span dir="ltr">`tauri icon`</span>؛ التفصيل في [src-tauri/icons/README.md](src-tauri/icons/README.md).
</details>

## الرخصة

مرخَّص بـ[MIT](LICENSE).

---

<div align="center">

<img src="src-tauri/icons/128x128@2x.png" alt="أيقونة نفّذ" width="96">

**تصميم وتطوير: سلطان** · Designed & developed by Sultan

الموقع: [bysltan.com](https://www.bysltan.com)

للتواصل: [S@BySltan.com](mailto:S@BySltan.com)

من الصانع نفسه<br>
تطبيقات macOS: [بدّل](https://github.com/iSltanX/Baddel) · [رفّ](https://github.com/iSltanX/Raff) · [Luma](https://github.com/iSltanX/Luma)<br>
إضافات المتصفح: [SnRead](https://github.com/iSltanX/SnRead) · [صَوْب](https://github.com/iSltanX/SAWB) · [جسور](https://github.com/iSltanX/Jusoor)

<sub>[الإصدارات](https://github.com/iSltanX/naffith/releases) · [الرخصة](LICENSE)</sub>

</div>

</div>
