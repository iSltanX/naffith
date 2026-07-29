(function () {
  'use strict';

  var params = new URLSearchParams(location.search);
  var screens = ['onboarding', 'categories', 'operations', 'zip', 'history', 'settings', 'system'];
  var stateOptions = {
    onboarding: [
      ['default', 'البداية'],
      ['example', 'مثال ZIP مكشوف']
    ],
    categories: [
      ['default', 'كل الفئات'],
      ['results', 'بحث بنتائج'],
      ['empty', 'بحث بلا نتائج']
    ],
    operations: [
      ['default', 'عمليات الضغط'],
      ['future', 'عملية قادمة']
    ],
    zip: [
      ['empty', 'قبل تعبئة الحقول'],
      ['partial', 'تعبئة جزئية'],
      ['valid', 'خطة صالحة'],
      ['running', 'تنفيذ جارٍ'],
      ['cancelling', 'إلغاء جارٍ'],
      ['success', 'نجاح'],
      ['failure', 'فشل'],
      ['conflict', 'ملف نهائي موجود'],
      ['insufficient-space', 'مساحة غير كافية'],
      ['back-confirm', 'تأكيد الرجوع']
    ],
    history: [
      ['default', 'آخر العمليات'],
      ['empty', 'سجل فارغ']
    ],
    settings: [
      ['default', 'كل الإعدادات'],
      ['appearance', 'المظهر والحركة'],
      ['history', 'السجل والمسارات'],
      ['about', 'معلومات التطبيق']
    ],
    system: [
      ['loading', 'تحميل'],
      ['empty', 'قائمة فارغة'],
      ['kernel-failure', 'فشل الاتصال بالنواة'],
      ['unavailable', 'عملية غير متاحة'],
      ['permission', 'صلاحية مفقودة'],
      ['repaired-settings', 'إعداد تالف تم إصلاحه']
    ]
  };

  var fullForm = {
    source: '/Users/sara/Projects/موقع-المجلس',
    destination: '/Volumes/Archive SSD/نسخ احتياطية',
    output: 'موقع-المجلس-2026-07-29.zip',
    conflict: 'ask'
  };

  var appState = {
    screen: screens.indexOf(params.get('screen')) >= 0 ? params.get('screen') : 'categories',
    viewState: params.get('state') || '',
    size: ['small', 'medium', 'large'].indexOf(params.get('size')) >= 0 ? params.get('size') : 'large',
    theme: ['system', 'light', 'dark'].indexOf(params.get('theme')) >= 0 ? params.get('theme') : 'system',
    motion: 'system',
    form: {},
    progress: 64,
    progressTimer: null,
    toastTimer: null,
    lastFocus: null,
    historyFilter: 'all',
    historyQuery: ''
  };

  var els = {};

  function icon(id, className, directional) {
    return '<svg' + (className ? ' class="' + className + '"' : '') +
      (directional ? ' data-directional' : '') +
      ' viewBox="0 0 24 24" aria-hidden="true"><use href="#' + id + '"></use></svg>';
  }

  function esc(value) {
    return String(value == null ? '' : value)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  function filenameMarkup(value, extraClass) {
    var text = String(value == null ? '' : value);
    var classes = 'filename-value' + (extraClass ? ' ' + extraClass : '');
    var mixedParts = /[\u0600-\u06ff]/.test(text)
      ? text.match(/^(.*?)([-._0-9A-Za-z]+)$/)
      : null;

    if (mixedParts && mixedParts[1] && mixedParts[2]) {
      return '<span class="' + classes + '" dir="rtl">' +
        '<bdi class="filename-value__stem" dir="rtl">' + esc(mixedParts[1]) + '</bdi>' +
        '<bdi class="filename-value__suffix" dir="ltr">' + esc(mixedParts[2]) + '</bdi>' +
      '</span>';
    }

    return '<bdi class="' + classes + '" dir="auto">' + esc(text) + '</bdi>';
  }

  function pathMarkup(value, extraClass) {
    var text = String(value == null ? '' : value);
    var classes = 'path-value' + (extraClass ? ' ' + extraClass : '');
    var leadingSlash = text.charAt(0) === '/';
    var segments = text.split('/').filter(Boolean).map(function (segment) {
      if (/\.[A-Za-z0-9]+$/.test(segment)) return filenameMarkup(segment, 'path-value__filename');
      return '<bdi class="path-value__segment" dir="auto">' + esc(segment) + '</bdi>';
    });

    return '<span class="' + classes + '" dir="ltr">' +
      (leadingSlash ? '<span class="path-value__separator">/</span>' : '') +
      segments.join('<span class="path-value__separator">/</span>') +
    '</span>';
  }

  function technicalValueMarkup(value) {
    var text = String(value == null ? '' : value);
    return text.indexOf('/') >= 0
      ? pathMarkup(text, 'path')
      : '<bdi class="path" dir="ltr">' + esc(text) + '</bdi>';
  }

  function chip(kind, iconId, text) {
    return '<span class="chip chip--' + kind + '">' + icon(iconId) + esc(text) + '</span>';
  }

  function defaultStateFor(screen) {
    var options = stateOptions[screen] || [['default', 'افتراضي']];
    return options[0][0];
  }

  function normalizeViewState() {
    var aliases = {
      onboarding: { welcome: 'default' },
      history: { populated: 'default' }
    };
    if (aliases[appState.screen] && aliases[appState.screen][appState.viewState]) {
      appState.viewState = aliases[appState.screen][appState.viewState];
    }
    var options = stateOptions[appState.screen] || [];
    var valid = options.some(function (option) { return option[0] === appState.viewState; });
    if (!valid) appState.viewState = defaultStateFor(appState.screen);
  }

  function seedZipForm(zipState) {
    if (zipState === 'empty') {
      appState.form = { source: '', destination: '', output: '', conflict: 'ask' };
      return;
    }
    if (zipState === 'partial') {
      appState.form = {
        source: fullForm.source,
        destination: '',
        output: fullForm.output,
        conflict: 'ask'
      };
      return;
    }
    appState.form = {
      source: fullForm.source,
      destination: fullForm.destination,
      output: fullForm.output,
      conflict: zipState === 'conflict' ? 'ask' : fullForm.conflict
    };
  }

  function isFormComplete() {
    return Boolean(
      appState.form.source &&
      appState.form.destination &&
      appState.form.output &&
      /\.zip$/i.test(appState.form.output)
    );
  }

  function inferZipState() {
    var count = [appState.form.source, appState.form.destination, appState.form.output].filter(Boolean).length;
    if (!count) return 'empty';
    if (count < 3 || !/\.zip$/i.test(appState.form.output)) return 'partial';
    return 'valid';
  }

  function setTheme(theme, persist) {
    appState.theme = theme;
    if (theme === 'light' || theme === 'dark') {
      document.documentElement.setAttribute('data-theme', theme);
    } else {
      document.documentElement.removeAttribute('data-theme');
    }
    document.querySelectorAll('[data-theme-set]').forEach(function (button) {
      button.setAttribute('aria-pressed', String(button.dataset.themeSet === theme));
    });
    if (persist !== false) {
      try { localStorage.setItem('ns-v3-theme', theme); } catch (error) {}
    }
    syncUrl();
  }

  function setSize(size) {
    appState.size = size;
    els.stage.dataset.size = size;
    document.querySelectorAll('[data-preview-size]').forEach(function (button) {
      button.setAttribute('aria-pressed', String(button.dataset.previewSize === size));
    });
    syncUrl();
  }

  function syncUrl() {
    try {
      var url = new URL(location.href);
      url.searchParams.set('screen', appState.screen);
      url.searchParams.set('state', appState.viewState);
      url.searchParams.set('theme', appState.theme);
      url.searchParams.set('size', appState.size);
      history.replaceState(null, '', url.href);
    } catch (error) {
      /* بقاء التفاعل أهم من تحديث العنوان تحت سياسات file:// المقيدة */
    }
  }

  function syncPickers() {
    els.screenPicker.value = appState.screen;
    var options = stateOptions[appState.screen] || [['default', 'افتراضي']];
    els.statePicker.innerHTML = options.map(function (option) {
      return '<option value="' + option[0] + '">' + option[1] + '</option>';
    }).join('');
    els.statePicker.value = appState.viewState;
    els.statePickerWrap.hidden = options.length < 2;
    document.querySelectorAll('[data-preview-size]').forEach(function (button) {
      button.setAttribute('aria-pressed', String(button.dataset.previewSize === appState.size));
    });
    document.querySelectorAll('[data-theme-set]').forEach(function (button) {
      button.setAttribute('aria-pressed', String(button.dataset.themeSet === appState.theme));
    });
  }

  function setActiveNavigation() {
    var current = appState.screen === 'onboarding' || appState.screen === 'operations' || appState.screen === 'zip'
      ? 'categories'
      : appState.screen;
    document.querySelectorAll('[data-nav]').forEach(function (button) {
      var active = button.dataset.nav === current;
      button.classList.toggle('is-active', active);
      if (active) button.setAttribute('aria-current', 'page');
      else button.removeAttribute('aria-current');
    });
  }

  function setWindowTitle(title) {
    els.windowTitle.textContent = title ? 'نَفِّذ — سَطْر · ' + title : 'نَفِّذ — سَطْر';
  }

  function navigate(screen, viewState, options) {
    options = options || {};
    if (
      appState.screen === 'zip' &&
      (appState.viewState === 'running' || appState.viewState === 'cancelling') &&
      screen !== 'zip' &&
      !options.force
    ) {
      showBackDialog();
      return;
    }
    stopProgress();
    appState.screen = screens.indexOf(screen) >= 0 ? screen : 'categories';
    appState.viewState = viewState || defaultStateFor(appState.screen);
    normalizeViewState();
    if (appState.screen === 'zip') seedZipForm(appState.viewState);
    closeDialog();
    render();
    requestAnimationFrame(function () { els.main.focus({ preventScroll: true }); });
  }

  function render() {
    normalizeViewState();
    els.shell.classList.toggle('is-onboarding', appState.screen === 'onboarding');
    els.main.classList.toggle('app-main--onboarding', appState.screen === 'onboarding');
    els.main.innerHTML = renderScreen();
    setActiveNavigation();
    syncPickers();
    syncUrl();
    if (appState.screen === 'zip' && appState.viewState === 'running') startProgress();
    if (appState.screen === 'zip' && appState.viewState === 'back-confirm') {
      requestAnimationFrame(showBackDialog);
    }
    if (appState.screen === 'categories') {
      requestAnimationFrame(function () {
        var input = document.getElementById('category-search');
        if (input && appState.viewState === 'results') {
          input.value = 'ضغط';
          filterCategories(input.value);
        } else if (input && appState.viewState === 'empty') {
          input.value = 'تشفير';
          filterCategories(input.value);
        }
      });
    }
    if (appState.screen === 'settings' && (appState.viewState === 'history' || appState.viewState === 'about')) {
      requestAnimationFrame(function () {
        var target = document.getElementById('settings-' + appState.viewState);
        if (target) target.scrollIntoView({ block: 'start' });
      });
    }
  }

  function renderScreen() {
    switch (appState.screen) {
      case 'onboarding': return renderOnboarding();
      case 'operations': return renderOperations();
      case 'zip': return renderZip();
      case 'history': return renderHistory();
      case 'settings': return renderSettings();
      case 'system': return renderSystem();
      case 'categories':
      default: return renderCategories();
    }
  }

  function renderOnboarding() {
    setWindowTitle('مرحبًا');
    var exampleOpen = appState.viewState === 'example';
    return '' +
      '<section class="onboarding-screen screen" aria-labelledby="onboarding-title">' +
        '<div class="onboarding-visual">' +
          '<svg class="onboarding-mark" viewBox="0 0 64 64" aria-hidden="true"><use href="#mark"></use></svg>' +
          '<h1 id="onboarding-title">من الاختيار إلى التنفيذ <span>—</span> بوضوح</h1>' +
          '<p>اختر ما تريد فعله، راجع ما سينفّذه التطبيق، ثم نفّذ بثقة. المصدر يبقى في مكانه، والخطة تبقى أمامك.</p>' +
          '<div class="onboarding-line command" id="onboarding-example"' + (exampleOpen ? '' : ' hidden') + '>' +
            '<div class="command__bar"><span class="command__label">مثال · ضغط مجلد إلى ZIP</span></div>' +
            '<div class="command__body"><span class="tok-prompt">$ </span><span class="tok-name">ditto</span> <span class="tok-flag">-c -k --sequesterRsrc --keepParent</span> \\\n' +
            '  <span class="tok-path">"' + pathMarkup('/Users/sara/Projects/موقع-المجلس') + '"</span> \\\n' +
            '  <span class="tok-path">"' + pathMarkup('/Users/sara/Desktop/موقع-المجلس.zip') + '"</span></div>' +
          '</div>' +
        '</div>' +
        '<div class="onboarding-copy">' +
          '<p class="onboarding-copy__kicker">ثلاث خطوات، بلا طرفية</p>' +
          '<h2>هكذا يعمل نَفِّذ — سَطْر</h2>' +
          '<ol class="onboarding-steps">' +
            '<li class="onboarding-step is-current">' +
              '<span class="onboarding-step__number num">١</span>' +
              '<div><h3>اختر العملية</h3><p>تصفّح فئات واضحة، واعرف فورًا ما هو متاح الآن وما هو قادم.</p></div>' +
            '</li>' +
            '<li class="onboarding-step">' +
              '<span class="onboarding-step__number num">٢</span>' +
              '<div><h3>أكمل نَفِّذ</h3><p>حدّد المصدر والوجهة واسم الناتج من حقول هادئة ومباشرة.</p></div>' +
            '</li>' +
            '<li class="onboarding-step">' +
              '<span class="onboarding-step__number num">٣</span>' +
              '<div><h3>راجع سَطْر</h3><p>شاهد الخطة والأمر قبل التنفيذ، ثم تابع النتيجة في المكان نفسه.</p></div>' +
            '</li>' +
          '</ol>' +
          '<div class="onboarding-actions">' +
            '<button type="button" class="btn btn--primary btn--lg" data-action="start-onboarding">' +
              icon('i-execute', '', true) + 'ابدأ باختيار عملية</button>' +
            '<button type="button" class="btn btn--quiet" data-action="toggle-example" aria-expanded="' + String(exampleOpen) + '" aria-controls="onboarding-example">' +
              icon('i-eye') + (exampleOpen ? 'إخفاء المثال' : 'عرض مثال ZIP') + '</button>' +
            '<button type="button" class="btn btn--ghost" data-action="skip-onboarding">تخطّي</button>' +
          '</div>' +
        '</div>' +
      '</section>';
  }

  var categories = [
    {
      id: 'compress',
      icon: 'i-compress',
      name: 'الضغط وفك الضغط',
      description: 'أنشئ ملف ZIP من ملف أو مجلد، مع إبقاء المصدر كما هو.',
      status: 'available',
      statusText: 'متاح الآن',
      count: 'عملية واحدة',
      terms: 'ضغط zip أرشيف فك ملف مجلد'
    },
    {
      id: 'files',
      icon: 'i-folder',
      name: 'الملفات والمجلدات',
      description: 'نقل ونسخ وإعادة تسمية بقرارات واضحة قبل التنفيذ.',
      status: 'future',
      statusText: 'قريبًا',
      count: 'نماذج تصميمية',
      terms: 'ملفات مجلدات نقل نسخ اسم'
    },
    {
      id: 'git',
      icon: 'i-git-branch',
      name: 'Git والمستودعات',
      description: 'عمليات المستودع اليومية بشرح عربي للأمر والرايات.',
      status: 'future',
      statusText: 'قريبًا',
      count: 'نماذج تصميمية',
      terms: 'git مستودع شيفرة فرع commit'
    },
    {
      id: 'network',
      icon: 'i-network',
      name: 'الشبكة والاتصال',
      description: 'تشخيص الاتصال والمنافذ والعناوين دون غموض تقني.',
      status: 'future',
      statusText: 'قريبًا',
      count: 'نماذج تصميمية',
      terms: 'شبكة اتصال ping منافذ عنوان'
    },
    {
      id: 'security',
      icon: 'i-security',
      name: 'الأمان والصلاحيات',
      description: 'فهم الصلاحيات وتعديلها مع توضيح الأثر قبل التغيير.',
      status: 'future',
      statusText: 'قريبًا',
      count: 'نماذج تصميمية',
      terms: 'أمان صلاحية chmod وصول'
    },
    {
      id: 'disk',
      icon: 'i-disk',
      name: 'الأقراص والتخزين',
      description: 'قراءة المساحة وتنظيم وحدات التخزين والنسخ الخارجية.',
      status: 'future',
      statusText: 'قريبًا',
      count: 'نماذج تصميمية',
      terms: 'قرص مساحة تخزين volume'
    },
    {
      id: 'system',
      icon: 'i-system',
      name: 'النظام والصيانة الدورية',
      description: 'مهام صيانة متكررة مع سجل واضح لما تغيّر.',
      status: 'future',
      statusText: 'قريبًا',
      count: 'نماذج تصميمية',
      terms: 'نظام صيانة cache تحديث'
    },
    {
      id: 'history',
      icon: 'i-history',
      name: 'سجل العمليات السابقة',
      description: 'النتائج والمدد والمسارات، مع كشف الناتج وإعادة الخطة.',
      status: 'support',
      statusText: 'واجهة مساندة',
      count: 'نموذج تفاعلي',
      terms: 'سجل تاريخ عمليات نتائج'
    }
  ];

  function renderCategories() {
    setWindowTitle('الفئات');
    var cards = categories.map(function (category) {
      var kind = category.status === 'available' ? 'success' : category.status === 'support' ? 'info' : 'neutral';
      var statusIcon = category.status === 'available' ? 'i-success' : category.status === 'support' ? 'i-info' : 'i-pending';
      return '' +
        '<button type="button" class="category-card category-card--' + category.status + '" ' +
          'data-category="' + category.id + '" data-search="' + esc(category.name + ' ' + category.description + ' ' + category.terms) + '">' +
          '<span class="category-card__icon">' + icon(category.icon) + '</span>' +
          '<h2>' + esc(category.name) + '</h2>' +
          '<p>' + esc(category.description) + '</p>' +
          '<span class="category-card__meta">' +
            chip(kind, statusIcon, category.statusText) +
            '<span class="category-card__count">' + esc(category.count) + '</span>' +
          '</span>' +
        '</button>';
    }).join('');
    return '' +
      '<section class="screen" aria-labelledby="categories-title">' +
        '<header class="page-head">' +
          '<div class="page-head__copy">' +
            '<p class="page-head__eyebrow">' + icon('i-categories') + 'التشغيلات اللاحقة تبدأ من هنا</p>' +
            '<h1 id="categories-title">ماذا تريد أن تنفّذ؟</h1>' +
            '<p class="page-head__desc">الفئة المتاحة فعليًا تظهر أولًا. الفئات الأخرى نماذج واضحة للمستقبل، ولا تُقدَّم كوظائف جاهزة.</p>' +
          '</div>' +
          '<div class="page-head__actions">' +
            '<button type="button" class="btn btn--quiet" data-nav="history">' + icon('i-history') + 'آخر العمليات</button>' +
          '</div>' +
        '</header>' +
        '<div class="screen-toolbar">' +
          '<label class="field" for="category-search">' +
            icon('i-search') +
            '<input id="category-search" type="search" aria-label="البحث في الفئات والعمليات" placeholder="ابحث عن فئة أو عملية…" autocomplete="off">' +
          '</label>' +
          '<span class="toolbar-note"><span class="num">١</span> متاحة الآن · <span class="num">٦</span> قادمة</span>' +
        '</div>' +
        '<div class="category-grid" id="category-grid">' + cards +
          '<div class="empty-search" id="category-empty" hidden>' +
            icon('i-search') +
            '<h2 class="t-section-title">لا توجد نتيجة مطابقة</h2>' +
            '<p class="t-body-sec" id="category-empty-copy">جرّب كلمة أخرى أو امسح البحث.</p>' +
            '<button type="button" class="btn btn--quiet btn--sm" data-action="clear-category-search">مسح البحث</button>' +
          '</div>' +
        '</div>' +
      '</section>';
  }

  function filterCategories(query) {
    query = String(query || '').trim().toLocaleLowerCase('ar');
    var count = 0;
    document.querySelectorAll('.category-card').forEach(function (card) {
      var matches = !query || card.dataset.search.toLocaleLowerCase('ar').indexOf(query) >= 0;
      card.hidden = !matches;
      if (matches) count += 1;
    });
    var empty = document.getElementById('category-empty');
    var copy = document.getElementById('category-empty-copy');
    if (empty) empty.hidden = count > 0;
    if (copy && query) copy.innerHTML = 'لم نجد فئة أو عملية تطابق «<bdi>' + esc(query) + '</bdi>».';
  }

  function renderOperations() {
    setWindowTitle('الضغط وفك الضغط');
    return '' +
      '<section class="screen" aria-labelledby="operations-title">' +
        '<button type="button" class="btn btn--ghost btn--sm back-link" data-nav="categories">' +
          icon('i-chevron', '', true) + 'العودة إلى الفئات</button>' +
        '<header class="page-head">' +
          '<div class="page-head__copy">' +
            '<p class="page-head__eyebrow">' + icon('i-compress') + 'فئة متاحة جزئيًا</p>' +
            '<h1 id="operations-title">الضغط وفك الضغط</h1>' +
            '<p class="page-head__desc">اختر العملية بحسب النتيجة التي تريدها. عملية ZIP وحدها جاهزة للتجربة الفعلية في هذه النسخة.</p>' +
          '</div>' +
        '</header>' +
        '<div class="category-summary">' +
          '<span class="category-summary__icon">' + icon('i-compress') + '</span>' +
          '<div><h2>ملخص الإتاحة</h2><p><span class="num">١</span> متاحة الآن · <span class="num">٣</span> قادمة · <span class="num">١</span> غير متاحة في البيئة</p></div>' +
          chip('success', 'i-success', 'ZIP متاح') +
        '</div>' +
        '<div class="operation-list">' +
          '<article class="operation-row operation-row--available">' +
            '<span class="operation-row__icon">' + icon('i-compress') + '</span>' +
            '<div><h3>ضغط ملف أو مجلد إلى ZIP</h3><p>أنشئ أرشيفًا واحدًا مع إبقاء المصدر كما هو، وراجع أمر macOS قبل التنفيذ.</p></div>' +
            '<div class="operation-row__meta">' +
              '<span class="operation-impact">' + icon('i-info') + 'أثر منخفض · ينشئ ملفًا جديدًا</span>' +
              '<button type="button" class="btn btn--primary btn--sm" data-action="open-zip">فتح العملية</button>' +
            '</div>' +
          '</article>' +
          '<article class="operation-row">' +
            '<span class="operation-row__icon">' + icon('i-folder-open') + '</span>' +
            '<div><h3>فك ملف ZIP</h3><p>استخرج المحتويات إلى مجلد تختاره، مع فحص التعارضات أولًا.</p></div>' +
            '<div class="operation-row__meta">' + chip('neutral', 'i-pending', 'قريبًا') +
              '<button type="button" class="btn btn--quiet btn--sm" data-action="preview-future">استعراض النموذج</button></div>' +
          '</article>' +
          '<article class="operation-row">' +
            '<span class="operation-row__icon">' + icon('i-export') + '</span>' +
            '<div><h3>ضغط إلى TAR.GZ</h3><p>أرشيف مناسب للمشروعات ونقلها بين الأنظمة مع حفظ البنية.</p></div>' +
            '<div class="operation-row__meta">' + chip('neutral', 'i-pending', 'قريبًا') +
              '<span class="operation-impact">' + icon('i-info') + 'أثر منخفض</span></div>' +
          '</article>' +
          '<article class="operation-row">' +
            '<span class="operation-row__icon">' + icon('i-eye') + '</span>' +
            '<div><h3>فحص محتوى أرشيف</h3><p>اعرض الملفات والمسارات داخل الأرشيف قبل الاستخراج.</p></div>' +
            '<div class="operation-row__meta">' + chip('neutral', 'i-pending', 'قريبًا') +
              '<span class="operation-impact">' + icon('i-eye') + 'قراءة فقط</span></div>' +
          '</article>' +
          '<article class="operation-row">' +
            '<span class="operation-row__icon">' + icon('i-warning') + '</span>' +
            '<div><h3>فك ملف 7z</h3><p>يتطلب أداة <code>7zz</code> غير المثبتة في بيئة النموذج الحالية.</p></div>' +
            '<div class="operation-row__meta">' + chip('warning', 'i-warning', 'غير متاح في البيئة') +
              '<button type="button" class="btn btn--quiet btn--sm" data-action="open-unavailable">معرفة المتطلبات</button></div>' +
          '</article>' +
        '</div>' +
      '</section>';
  }

  function zipStateMeta(zipState) {
    var map = {
      empty: ['neutral', 'i-pending', 'بانتظار المعطيات'],
      partial: ['info', 'i-info', 'خطة غير مكتملة'],
      valid: ['success', 'i-success', 'جاهز للتنفيذ'],
      running: ['accent', 'i-pending', 'جارٍ الضغط'],
      cancelling: ['warning', 'i-warning', 'جارٍ الإلغاء'],
      success: ['success', 'i-success', 'اكتمل بنجاح'],
      failure: ['danger', 'i-error', 'تعذّر التنفيذ'],
      conflict: ['warning', 'i-warning', 'الناتج موجود'],
      'insufficient-space': ['danger', 'i-error', 'المساحة غير كافية'],
      'back-confirm': ['accent', 'i-pending', 'تنفيذ نشط']
    };
    return map[zipState] || map.empty;
  }

  function zipCommand() {
    return 'ditto -c -k --sequesterRsrc --keepParent "' +
      appState.form.source + '" "' +
      appState.form.destination + '/' +
      appState.form.output + '"';
  }

  function renderZip() {
    setWindowTitle('ضغط إلى ZIP');
    var zipState = appState.viewState === 'back-confirm' ? 'running' : appState.viewState;
    var meta = zipStateMeta(appState.viewState);
    var locked = zipState === 'running' || zipState === 'cancelling' || zipState === 'success';
    var source = esc(appState.form.source);
    var destination = esc(appState.form.destination);
    var outputValue = String(appState.form.output || '');
    var output = esc(outputValue);
    var outputPreview = outputValue
      ? '<span class="filename-field__preview" aria-hidden="true">' + filenameMarkup(outputValue) + '</span>'
      : '';
    var destinationMessage = zipState === 'insufficient-space'
      ? '<p class="form-helper form-helper--danger" id="destination-help">المتاح <span class="num">620</span> م.ب. فقط؛ اختر وجهة أخرى.</p>'
      : '<p class="form-helper" id="destination-help">يُنشأ ملف ZIP داخل هذا المجلد.</p>';
    var outputMessage = zipState === 'conflict'
      ? '<p class="form-helper warning-text" id="output-help">يوجد ملف بهذا الاسم؛ اختر سياسة التعارض.</p>'
      : '<p class="form-helper" id="output-help">ينبغي أن ينتهي الاسم بالامتداد <span class="lat">.zip</span>.</p>';
    return '' +
      '<section class="screen" aria-labelledby="zip-title">' +
        '<div class="zip-head">' +
          '<button type="button" class="btn btn--ghost btn--sm back-link" data-action="back-from-zip">' +
            icon('i-chevron', '', true) + 'عمليات الضغط</button>' +
          '<header class="page-head">' +
            '<div class="page-head__copy">' +
              '<p class="page-head__eyebrow">' + icon('i-compress') + 'عملية متاحة فعليًا</p>' +
              '<h1 id="zip-title">ضغط ملف أو مجلد إلى ZIP</h1>' +
              '<p class="page-head__desc">أكمل نَفِّذ في اليمين، وراجع في سَطْر ما سيحدث حرفيًا. لن يتغيّر المصدر أو يُحذف.</p>' +
              '<div class="zip-head__meta">' +
                chip(meta[0], meta[1], meta[2]) +
                '<span class="chip chip--neutral">' + icon('i-info') + 'الأداة: <span class="lat">ditto</span></span>' +
                '<span class="chip chip--neutral">' + icon('i-system') + 'مضمّنة في macOS</span>' +
              '</div>' +
            '</div>' +
          '</header>' +
        '</div>' +
        '<div class="zip-workspace">' +
          '<section class="naffith-panel" aria-labelledby="naffith-heading">' +
            '<header class="mode-heading">' +
              icon('mode-naffith') +
              '<h2 id="naffith-heading">نَفِّذ</h2>' +
              '<p>معطيات العملية</p>' +
            '</header>' +
            '<div class="form-stack">' +
              '<div class="form-group">' +
                '<label class="form-label" for="zip-source">المصدر</label>' +
                '<div class="field field--path field--with-action">' +
                  icon('i-folder') +
                  '<input id="zip-source" data-zip-field="source" value="' + source + '" placeholder="اختر ملفًا أو مجلدًا…" spellcheck="false" ' +
                    (locked ? 'disabled' : '') + '>' +
                  '<button type="button" class="btn btn--quiet btn--sm" aria-label="اختيار المصدر" data-action="choose-source" ' + (locked ? 'disabled' : '') + '>اختيار…</button>' +
                '</div>' +
                '<p class="form-helper">المصدر يبقى في مكانه دون تعديل.</p>' +
              '</div>' +
              '<div class="form-group">' +
                '<label class="form-label" for="zip-destination">الوجهة</label>' +
                '<div class="field field--path field--with-action">' +
                  icon('i-folder-open') +
                  '<input id="zip-destination" data-zip-field="destination" value="' + destination + '" placeholder="اختر مجلد الوجهة…" spellcheck="false" aria-describedby="destination-help" ' +
                    (locked ? 'disabled' : '') + '>' +
                  '<button type="button" class="btn btn--quiet btn--sm" aria-label="اختيار الوجهة" data-action="choose-destination" ' + (locked ? 'disabled' : '') + '>اختيار…</button>' +
                '</div>' +
                destinationMessage +
              '</div>' +
              '<div class="form-group">' +
                '<label class="form-label" for="zip-output">اسم ملف ZIP <span class="form-label__optional">يُقترح تلقائيًا</span></label>' +
                '<div class="field field--path field--filename">' +
                  icon('i-file') +
                  '<input id="zip-output" class="' + (outputValue ? 'has-visual-value' : '') + '" dir="rtl" data-zip-field="output" value="' + output + '" placeholder="مثال: أرشيف-المشروع.zip" spellcheck="false" aria-describedby="output-help" ' +
                    (locked ? 'disabled' : '') + '>' +
                  outputPreview +
                '</div>' +
                outputMessage +
              '</div>' +
              '<div class="form-group">' +
                '<label class="form-label" for="zip-conflict">عند وجود ملف بالاسم نفسه</label>' +
                '<div class="field">' +
                  icon('i-copy') +
                  '<select id="zip-conflict" data-zip-field="conflict" ' + (locked ? 'disabled' : '') + '>' +
                    '<option value="ask"' + (appState.form.conflict === 'ask' ? ' selected' : '') + '>اسألني قبل الاستبدال</option>' +
                    '<option value="copy"' + (appState.form.conflict === 'copy' ? ' selected' : '') + '>احتفظ بالنسختين</option>' +
                    '<option value="replace"' + (appState.form.conflict === 'replace' ? ' selected' : '') + '>استبدل الملف الموجود</option>' +
                  '</select>' +
                  icon('i-chevron-down', 'select-chevron') +
                '</div>' +
              '</div>' +
              '<div class="inline-message inline-message--info">' +
                icon('i-info') +
                '<span>ينشئ الضغط ملفًا جديدًا فقط. لن يُعدَّل المصدر أو يُحذف، حتى إذا فشل التنفيذ أو أُلغي.</span>' +
              '</div>' +
            '</div>' +
            renderZipActions(zipState) +
          '</section>' +
          '<section class="satr-panel" aria-labelledby="satr-heading">' +
            '<header class="mode-heading">' +
              icon('mode-satr') +
              '<h2 id="satr-heading">سَطْر</h2>' +
              '<p>الخطة والتنفيذ والنتيجة</p>' +
            '</header>' +
            renderSatr(zipState) +
          '</section>' +
        '</div>' +
      '</section>';
  }

  function renderZipActions(zipState) {
    var summary = '<span class="form-actions__summary">الحجم التقديري: <span class="num">1.21–1.45</span> غ.ب.</span>';
    if (zipState === 'empty' || zipState === 'partial') {
      return '<div class="form-actions">' + summary +
        '<button type="button" class="btn btn--primary" disabled>' + icon('i-execute', '', true) + 'أكمل الحقول</button></div>';
    }
    if (zipState === 'valid') {
      return '<div class="form-actions">' + summary +
        '<button type="button" class="btn btn--primary" data-action="run-zip">' + icon('i-execute', '', true) + 'تنفيذ الضغط</button></div>';
    }
    if (zipState === 'running') {
      return '<div class="form-actions"><span class="form-actions__summary">المصدر مقفول أثناء التنفيذ</span>' +
        '<button type="button" class="btn btn--danger" data-action="cancel-zip">' + icon('i-close') + 'إلغاء العملية</button></div>';
    }
    if (zipState === 'cancelling') {
      return '<div class="form-actions"><span class="form-actions__summary">جارٍ حذف الملف الجزئي بأمان</span>' +
        '<button type="button" class="btn btn--quiet" disabled>' + icon('i-pending') + 'جارٍ الإلغاء…</button></div>';
    }
    if (zipState === 'success') {
      return '<div class="form-actions">' + summary +
        '<button type="button" class="btn btn--primary" data-action="new-zip">' + icon('i-plus') + 'عملية جديدة</button></div>';
    }
    if (zipState === 'failure') {
      return '<div class="form-actions">' + summary +
        '<button type="button" class="btn btn--primary" data-action="retry-zip">' + icon('i-execute', '', true) + 'إعادة المحاولة</button></div>';
    }
    if (zipState === 'conflict') {
      return '<div class="form-actions">' + summary +
        '<button type="button" class="btn btn--primary" data-action="keep-both">' + icon('i-copy') + 'الاحتفاظ بالنسختين</button></div>';
    }
    if (zipState === 'insufficient-space') {
      return '<div class="form-actions"><span class="form-actions__summary">المطلوب: نحو <span class="num">1.45</span> غ.ب.</span>' +
        '<button type="button" class="btn btn--primary" data-action="choose-other-destination">' + icon('i-folder-open') + 'اختيار وجهة أخرى</button></div>';
    }
    return '';
  }

  function renderSatr(zipState) {
    if (zipState === 'empty') {
      return '' +
        '<div class="satr-watermark" aria-label="أكمل حقول نَفِّذ لتظهر الخطة">' +
          '<span class="satr-watermark__word">سَطْر</span>' +
          '<span class="satr-watermark__prompt">أكمل حقول نَفِّذ</span>' +
          '<span class="satr-watermark__desc">ستظهر هنا الخطوات التي سينفذها التطبيق</span>' +
        '</div>';
    }
    if (zipState === 'partial') return renderPartialPlan();
    return renderFullPlan(zipState);
  }

  function renderPartialPlan() {
    return '' +
      '<div class="plan-preview">' +
        '<p class="plan-preview__status">' + icon('i-info') + '<strong>خطة غير مكتملة</strong> · أكمل الوجهة ليظهر الأمر كاملًا</p>' +
        '<dl class="plan-grid">' +
          '<div class="plan-item plan-item--wide"><dt>المصدر</dt><dd class="path">' + pathMarkup(appState.form.source || 'لم يُحدّد بعد') + '</dd></div>' +
          '<div class="plan-item plan-item--missing"><dt>الوجهة</dt><dd>لم تُحدّد بعد</dd></div>' +
          '<div class="plan-item"><dt>اسم الناتج</dt><dd class="filename-cell">' + filenameMarkup(appState.form.output || 'سيُقترح بعد المصدر') + '</dd></div>' +
          '<div class="plan-item"><dt>الأداة</dt><dd><span class="lat">ditto</span> · مضمّنة في macOS</dd></div>' +
          '<div class="plan-item plan-item--missing"><dt>الحجم التقديري</dt><dd>بانتظار الوجهة</dd></div>' +
        '</dl>' +
        '<div class="human-plan">' +
          icon('i-info') +
          '<h3>ما نعرفه حتى الآن</h3>' +
          '<p>سيُضغط المصدر المحدد إلى ملف ZIP. نحتاج مجلد الوجهة قبل حساب المساحة وبناء الأمر القابل للتنفيذ.</p>' +
        '</div>' +
      '</div>';
  }

  function renderFullPlan(zipState) {
    var command = zipCommand();
    var stateBlock = '';
    if (zipState === 'running') stateBlock = renderRunning();
    if (zipState === 'cancelling') stateBlock = renderCancelling();
    if (zipState === 'success') stateBlock = renderSuccess();
    if (zipState === 'failure') stateBlock = renderFailure();
    if (zipState === 'conflict') stateBlock = renderConflict();
    if (zipState === 'insufficient-space') stateBlock = renderInsufficientSpace();
    return '' +
      '<div class="plan-preview">' +
        '<p class="plan-preview__status">' + icon(zipState === 'valid' ? 'i-success' : 'i-info') +
          '<strong>' + (zipState === 'valid' ? 'خطة صالحة' : 'خطة العملية') + '</strong> · لن يتغيّر المصدر</p>' +
        '<dl class="plan-grid">' +
          '<div class="plan-item plan-item--wide"><dt>المصدر</dt><dd class="path">' + pathMarkup(appState.form.source) + '</dd></div>' +
          '<div class="plan-item"><dt>الوجهة</dt><dd class="path">' + pathMarkup(appState.form.destination) + '</dd></div>' +
          '<div class="plan-item"><dt>اسم الناتج</dt><dd class="filename-cell">' + filenameMarkup(appState.form.output) + '</dd></div>' +
          '<div class="plan-item"><dt>الأداة</dt><dd><span class="lat">ditto</span> · مضمّنة في macOS</dd></div>' +
          '<div class="plan-item"><dt>الحجم التقديري</dt><dd><span class="num">1.21–1.45</span> غيغابايت</dd></div>' +
          '<div class="plan-item plan-item--wide"><dt>سياسة التعارض</dt><dd>' + conflictLabel(appState.form.conflict) + '</dd></div>' +
        '</dl>' +
        '<div class="human-plan">' +
          icon('i-eye') +
          '<h3>التفسير البشري</h3>' +
          '<p>سيجمع التطبيق محتويات «موقع-المجلس» في ملف ZIP واحد داخل «نسخ احتياطية». سيبقى المجلد الأصلي في مكانه دون تعديل.</p>' +
        '</div>' +
        '<div class="plan-command command">' +
          '<div class="command__bar">' +
            '<span class="command__label">الأمر الذي سيُسلَّم إلى macOS</span>' +
            '<button type="button" class="command-copy" data-action="copy-command" data-copy-value="' + esc(command) + '">' +
              icon('i-copy') + '<span>نسخ الأمر</span></button>' +
          '</div>' +
          '<div class="command__body"><span class="tok-prompt">$ </span><span class="tok-name">ditto</span> ' +
            '<span class="tok-flag">-c -k --sequesterRsrc --keepParent</span> \\\n' +
            '  <span class="tok-path">"' + pathMarkup(appState.form.source) + '"</span> \\\n' +
            '  <span class="tok-path">"' + pathMarkup(appState.form.destination + '/' + appState.form.output) + '"</span></div>' +
        '</div>' +
        stateBlock +
      '</div>';
  }

  function conflictLabel(policy) {
    if (policy === 'copy') return 'الاحتفاظ بالنسختين باسم مرقّم';
    if (policy === 'replace') return 'استبدال الملف الموجود بعد التأكيد';
    return 'اسألني قبل الاستبدال';
  }

  function renderRunning() {
    return '' +
      '<section class="execution-card" aria-labelledby="running-title">' +
        '<div class="execution-card__top">' +
          '<span class="spinner" role="status" aria-label="جارٍ ضغط الملفات"></span>' +
          '<h3 id="running-title">جارٍ إنشاء ملف ZIP</h3>' +
          '<time dir="ltr" class="num">00:18</time>' +
        '</div>' +
        '<div class="progress" role="progressbar" aria-label="تقدم الضغط" aria-valuemin="0" aria-valuemax="100" aria-valuenow="' + appState.progress + '">' +
          '<div class="progress__fill" style="width:' + appState.progress + '%"></div>' +
        '</div>' +
        '<div class="execution-card__meta">' +
          '<strong class="num">' + appState.progress + '٪</strong>' +
          '<span><span class="num">786</span> م.ب. من نحو <span class="num">1.24</span> غ.ب.</span>' +
          pathMarkup('Assets/صور/الشعار-1024.png', 'path') +
        '</div>' +
      '</section>';
  }

  function renderCancelling() {
    return '' +
      '<section class="execution-card" aria-labelledby="cancelling-title">' +
        '<div class="execution-card__top">' +
          '<span class="spinner" role="status" aria-label="جارٍ إلغاء العملية"></span>' +
          '<h3 id="cancelling-title">جارٍ إنهاء العملية بأمان</h3>' +
        '</div>' +
        '<div class="progress" role="progressbar" aria-label="تنظيف الملف الجزئي">' +
          '<div class="progress__fill" style="width:72%"></div>' +
        '</div>' +
        '<p class="t-caption" style="color:var(--command-prompt)">يُوقَف <span class="lat">ditto</span> الآن، ثم يُحذف ملف ZIP الجزئي. المصدر لن يتغيّر.</p>' +
      '</section>';
  }

  function renderSuccess() {
    var outputPath = appState.form.destination + '/' + appState.form.output;
    return '' +
      '<section class="status-panel status-panel--success" role="status">' +
        icon('i-success') +
        '<div><h3>تم إنشاء ملف ZIP</h3><p>اكتمل الضغط خلال <span class="num">24</span> ثانية. حجم الناتج <span class="num">1.32</span> غيغابايت.</p></div>' +
        '<div class="status-panel__actions">' +
          '<button type="button" class="btn btn--quiet btn--sm" data-action="reveal-file">' + icon('i-eye') + 'كشف في Finder</button>' +
          '<button type="button" class="btn btn--quiet btn--sm" data-action="copy-path" data-copy-value="' + esc(outputPath) + '">' + icon('i-copy') + 'نسخ المسار</button>' +
        '</div>' +
      '</section>';
  }

  function renderFailure() {
    return '' +
      '<section class="status-panel status-panel--danger" role="alert">' +
        icon('i-error') +
        '<div><h3>تعذّر إنشاء الملف</h3><p>مجلد الوجهة لا يسمح بالكتابة. لم يتغيّر المصدر، وحُذف ملف ZIP الجزئي.</p></div>' +
        '<div class="status-panel__actions">' +
          '<button type="button" class="btn btn--quiet btn--sm" data-action="choose-other-destination">' + icon('i-folder-open') + 'اختيار وجهة أخرى</button>' +
          '<button type="button" class="btn btn--quiet btn--sm" data-action="copy-details">' + icon('i-copy') + 'نسخ التفاصيل</button>' +
        '</div>' +
        '<div class="command plan-item--wide" style="grid-column:1/-1">' +
          '<div class="command__body t-terminal"><span class="tok-comment">ditto: ' +
            pathMarkup('/Volumes/Archive SSD/نسخ احتياطية') + ': Permission denied</span></div>' +
        '</div>' +
      '</section>';
  }

  function renderConflict() {
    return '' +
      '<section class="status-panel status-panel--warning" role="status">' +
        icon('i-warning') +
        '<div><h3>يوجد ملف نهائي بالاسم نفسه</h3><p>حجمه <span class="num">1.28</span> غيغابايت، وعُدّل اليوم عند <bdi dir="ltr" class="num">09:16</bdi> ص. الاقتراح الآمن هو الاحتفاظ بالنسختين.</p></div>' +
        '<div class="status-panel__actions">' +
          '<button type="button" class="btn btn--quiet btn--sm" data-action="keep-both">' + icon('i-copy') + 'الاحتفاظ بالنسختين</button>' +
          '<button type="button" class="btn btn--quiet btn--sm" data-action="change-output-name">' + icon('i-rename') + 'تغيير الاسم</button>' +
          '<button type="button" class="btn btn--danger btn--sm" data-action="replace-existing">' + icon('i-warning') + 'استبدال</button>' +
        '</div>' +
      '</section>';
  }

  function renderInsufficientSpace() {
    return '' +
      '<section class="status-panel status-panel--danger" role="alert">' +
        icon('i-disk') +
        '<div><h3>المساحة غير كافية</h3><p>يحتاج الضغط إلى نحو <span class="num">1.45</span> غيغابايت، والمتاح <span class="num">620</span> ميغابايت فقط. لم يبدأ أمر <span class="lat">ditto</span>.</p></div>' +
        '<div class="status-panel__actions">' +
          '<button type="button" class="btn btn--quiet btn--sm" data-action="choose-other-destination">' + icon('i-folder-open') + 'اختيار وجهة أخرى</button>' +
          '<button type="button" class="btn btn--quiet btn--sm" data-action="open-storage">' + icon('i-disk') + 'فتح إدارة التخزين</button>' +
        '</div>' +
      '</section>';
  }

  var historyItems = [
    {
      id: 'h1',
      status: 'success',
      icon: 'i-success',
      chip: 'success',
      label: 'نجاح',
      title: 'ضغط «عقود الربع الثاني»',
      source: '/Users/sara/Documents/عقود الربع الثاني',
      output: '/Users/sara/Documents/أرشيف/عقود-Q2.zip',
      time: 'اليوم · 10:42 ص',
      duration: '00:24'
    },
    {
      id: 'h2',
      status: 'failure',
      icon: 'i-error',
      chip: 'danger',
      label: 'فشل',
      title: 'ضغط «صور RAW»',
      source: '/Volumes/Camera/صور RAW',
      output: 'لم يُنشأ ملف',
      time: 'أمس · 6:18 م',
      duration: '00:03'
    },
    {
      id: 'h3',
      status: 'cancelled',
      icon: 'i-cancelled',
      chip: 'neutral',
      label: 'ملغاة',
      title: 'ضغط «نسخة موقع المجلس»',
      source: '/Users/sara/Projects/موقع-المجلس',
      output: 'حُذف الملف الجزئي',
      time: '27 يوليو · 11:03 ص',
      duration: '00:07'
    },
    {
      id: 'h4',
      status: 'success',
      icon: 'i-success',
      chip: 'success',
      label: 'نجاح',
      title: 'ضغط «مرفقات المناقصة»',
      source: '/Users/sara/Desktop/مرفقات المناقصة',
      output: '/Volumes/Archive SSD/مناقصات/مرفقات-0726.zip',
      time: '26 يوليو · 3:34 م',
      duration: '01:12'
    }
  ];

  function renderHistory() {
    setWindowTitle('السجل');
    if (appState.viewState === 'empty') {
      return '' +
        '<section class="screen" aria-labelledby="history-title">' +
          historyHeader() +
          '<div class="system-state-feature"><div class="system-state-content">' +
            icon('i-history') +
            '<h2>لا توجد عمليات بعد</h2>' +
            '<p>ستظهر هنا نتائج الضغط ومدتها ومساراتها، سواء نجحت أو فشلت أو أُلغيت.</p>' +
            '<div class="system-state-content__actions"><button type="button" class="btn btn--primary" data-nav="operations">' + icon('i-plus') + 'ابدأ أول عملية</button></div>' +
          '</div></div>' +
        '</section>';
    }
    var rows = historyItems.map(function (item) {
      return '' +
        '<article class="history-row" data-history-status="' + item.status + '" data-history-search="' + esc(item.title + ' ' + item.source + ' ' + item.output) + '">' +
          '<div class="history-row__title">' +
            '<span class="history-row__icon ' + item.status + '-text">' + icon(item.icon) + '</span>' +
            '<div><h3>' + esc(item.title) + '</h3><span class="history-row__path">' + pathMarkup(item.source, 'path') + '</span></div>' +
          '</div>' +
          '<span class="history-row__output">' +
            (item.output.charAt(0) === '/' ? pathMarkup(item.output, 'path') : esc(item.output)) +
          '</span>' +
          '<time class="history-row__time">' + esc(item.time) + '</time>' +
          '<span class="history-row__duration num" dir="ltr">' + esc(item.duration) + '</span>' +
          '<div>' + chip(item.chip, item.icon, item.label) +
            (item.status === 'success'
              ? '<button type="button" class="btn btn--ghost btn--sm" data-action="reveal-history">كشف الناتج</button>'
              : '<button type="button" class="btn btn--ghost btn--sm" data-action="history-details">التفاصيل</button>') +
          '</div>' +
        '</article>';
    }).join('');
    return '' +
      '<section class="screen" aria-labelledby="history-title">' +
        historyHeader() +
        '<div class="history-summary">' +
          '<div class="summary-tile"><strong class="num">٤</strong><span>عمليات محفوظة</span></div>' +
          '<div class="summary-tile"><strong class="num success-text">٢</strong><span>نجاح</span></div>' +
          '<div class="summary-tile"><strong class="num danger-text">١</strong><span>فشل</span></div>' +
          '<div class="summary-tile"><strong class="num muted">١</strong><span>إلغاء</span></div>' +
        '</div>' +
        '<div class="screen-toolbar">' +
          '<label class="field" for="history-search">' + icon('i-search') +
            '<input id="history-search" type="search" aria-label="البحث في سجل العمليات" placeholder="ابحث في العملية أو المسار…" autocomplete="off"></label>' +
          '<div class="segment" role="group" aria-label="تصفية حسب الحالة">' +
            '<button type="button" data-history-filter="all" aria-pressed="true">الكل</button>' +
            '<button type="button" data-history-filter="success" aria-pressed="false">نجاح</button>' +
            '<button type="button" data-history-filter="failure" aria-pressed="false">فشل</button>' +
            '<button type="button" data-history-filter="cancelled" aria-pressed="false">ملغاة</button>' +
          '</div>' +
        '</div>' +
        '<div class="history-table" id="history-table">' + rows +
          '<div class="history-empty" id="history-empty">' +
            icon('i-search') + '<h2 class="t-section-title">لا توجد نتيجة مطابقة</h2>' +
            '<p class="t-body-sec">غيّر الفلاتر أو امسح البحث.</p>' +
            '<button type="button" class="btn btn--quiet btn--sm" data-action="clear-history">مسح الفلاتر</button>' +
          '</div>' +
        '</div>' +
      '</section>';
  }

  function historyHeader() {
    return '' +
      '<header class="page-head">' +
        '<div class="page-head__copy">' +
          '<p class="page-head__eyebrow">' + icon('i-history') + 'النتيجة محفوظة بوضوح</p>' +
          '<h1 id="history-title">سجل العمليات</h1>' +
          '<p class="page-head__desc">آخر ما نُفِّذ، حالته، مدته، والناتج الذي أنشأه. السجل لا يحذف ملفاتك الناتجة.</p>' +
        '</div>' +
        '<div class="page-head__actions"><button type="button" class="btn btn--primary btn--sm" data-nav="operations">' + icon('i-plus') + 'عملية جديدة</button></div>' +
      '</header>';
  }

  function filterHistory() {
    var query = appState.historyQuery.trim().toLocaleLowerCase('ar');
    var visible = 0;
    document.querySelectorAll('.history-row').forEach(function (row) {
      var statusMatch = appState.historyFilter === 'all' || row.dataset.historyStatus === appState.historyFilter;
      var queryMatch = !query || row.dataset.historySearch.toLocaleLowerCase('ar').indexOf(query) >= 0;
      row.hidden = !(statusMatch && queryMatch);
      if (!row.hidden) visible += 1;
    });
    var empty = document.getElementById('history-empty');
    if (empty) empty.classList.toggle('is-visible', visible === 0);
  }

  function renderSettings() {
    setWindowTitle('الإعدادات');
    var activeSettingsSection = ['appearance', 'history', 'about'].indexOf(appState.viewState) >= 0
      ? appState.viewState
      : 'appearance';
    return '' +
      '<section class="screen" aria-labelledby="settings-title">' +
        '<header class="page-head">' +
          '<div class="page-head__copy">' +
            '<p class="page-head__eyebrow">' + icon('i-settings') + 'تُحفظ التغييرات تلقائيًا</p>' +
            '<h1 id="settings-title">الإعدادات</h1>' +
            '<p class="page-head__desc">خيارات محدودة تخص طريقة العرض والسجل والمسارات. لا يوجد زر حفظ عام، مثل إعدادات macOS.</p>' +
          '</div>' +
        '</header>' +
        '<div class="settings-layout">' +
          '<nav class="settings-nav" aria-label="أقسام الإعدادات">' +
            '<button type="button" class="' + (activeSettingsSection === 'appearance' ? 'is-active' : '') + '" data-settings-section="appearance">المظهر والحركة</button>' +
            '<button type="button" class="' + (activeSettingsSection === 'history' ? 'is-active' : '') + '" data-settings-section="history">السجل والمسارات</button>' +
            '<button type="button" class="' + (activeSettingsSection === 'about' ? 'is-active' : '') + '" data-settings-section="about">معلومات التطبيق</button>' +
          '</nav>' +
          '<div class="settings-pane">' +
            '<section class="settings-section" id="settings-appearance">' +
              '<h2>المظهر والحركة</h2><p>يتبع التطبيق macOS افتراضيًا، مع بقاء سطح سَطْر داكنًا في كل الحالات.</p>' +
              '<div class="setting-row">' +
                '<div><h3>المظهر</h3><p>يُطبَّق التغيير فورًا على كل الشاشات.</p></div>' +
                '<div class="segment" role="group" aria-label="مظهر التطبيق">' +
                  '<button type="button" data-theme-set="system" aria-pressed="' + String(appState.theme === 'system') + '">حسب النظام</button>' +
                  '<button type="button" data-theme-set="light" aria-pressed="' + String(appState.theme === 'light') + '">فاتح</button>' +
                  '<button type="button" data-theme-set="dark" aria-pressed="' + String(appState.theme === 'dark') + '">داكن</button>' +
                '</div>' +
              '</div>' +
              '<div class="setting-row">' +
                '<div><h3>تقليل الحركة</h3><p>تظل مؤشرات الحالة ظاهرة، لكن الانتقالات تصبح فورية تقريبًا.</p></div>' +
                '<div class="segment" role="group" aria-label="الحركة">' +
                  '<button type="button" data-motion-set="system" aria-pressed="' + String(appState.motion === 'system') + '">حسب macOS</button>' +
                  '<button type="button" data-motion-set="reduce" aria-pressed="' + String(appState.motion === 'reduce') + '">تقليل دائمًا</button>' +
                '</div>' +
              '</div>' +
              '<div class="setting-row">' +
                '<div><h3>إظهار الترحيب في التشغيل القادم</h3><p>لا يمسح أي إعداد آخر أو سجل.</p></div>' +
                '<button type="button" class="switch" role="switch" aria-checked="false" aria-label="إظهار الترحيب في التشغيل القادم" data-action="toggle-switch"></button>' +
              '</div>' +
              '<div class="setting-row">' +
                '<div><h3>الترحيب</h3><p>راجع شرح نَفِّذ وسَطْر متى شئت.</p></div>' +
                '<button type="button" class="btn btn--quiet btn--sm" data-nav="onboarding">' + icon('i-eye') + 'عرض الترحيب الآن</button>' +
              '</div>' +
            '</section>' +
            '<section class="settings-section" id="settings-history">' +
              '<h2>السجل والمسارات المفضلة</h2><p>تُحفظ بيانات النموذج محليًا في هذه الحزمة التجريبية.</p>' +
              '<div class="setting-row">' +
                '<div><h3>الاحتفاظ بالسجل</h3><p>بعد المدة تُحذف السجلات فقط، لا ملفات ZIP.</p></div>' +
                '<div class="field"><select aria-label="مدة الاحتفاظ بالسجل" data-setting-select>' +
                  '<option>٣٠ يومًا</option><option selected>٩٠ يومًا</option><option>سنة</option><option>دائمًا</option>' +
                '</select>' + icon('i-chevron-down') + '</div>' +
              '</div>' +
              '<div class="setting-row">' +
                '<div><h3>مسح السجل الآن</h3><p>يوجد <span class="num">٤</span> سجلات. لن يُحذف أي ملف ناتج.</p></div>' +
                '<button type="button" class="btn btn--danger btn--sm" data-action="clear-history-settings">' + icon('i-delete') + 'مسح السجل…</button>' +
              '</div>' +
              '<div class="setting-row preferred-path">' +
                '<div><h3>أرشيف المستندات</h3><p>' + pathMarkup('/Users/sara/Documents/أرشيف', 'path') + '</p></div>' +
                '<button type="button" class="btn btn--ghost btn--sm" data-action="remove-path">' + icon('i-minus') + 'إزالة</button>' +
              '</div>' +
              '<div class="setting-row preferred-path">' +
                '<div><h3>نسخ القرص الخارجي</h3><p>' + pathMarkup('/Volumes/Archive SSD/نسخ احتياطية', 'path') + '</p></div>' +
                '<div>' + chip('warning', 'i-disk', 'غير متصل الآن') +
                  '<button type="button" class="btn btn--ghost btn--sm" data-action="remove-path">إزالة</button></div>' +
              '</div>' +
              '<div class="setting-row">' +
                '<div><h3>إضافة مسار مفضل</h3><p>يظهر أولًا عند اختيار وجهة ZIP.</p></div>' +
                '<button type="button" class="btn btn--quiet btn--sm" data-action="add-path">' + icon('i-plus') + 'إضافة مسار…</button>' +
              '</div>' +
            '</section>' +
            '<section class="settings-section" id="settings-about">' +
              '<h2>معلومات التطبيق</h2><p>معلومات هذه النسخة التصميمية، من دون أرقام إنتاج أو روابط تنزيل غير مؤكدة.</p>' +
              '<div class="about-mark">' +
                '<svg viewBox="0 0 64 64" aria-hidden="true"><use href="#mark"></use></svg>' +
                '<div><h3>نَفِّذ — سَطْر</h3><p>نسخة النماذج التفاعلية v3 · macOS · عربية أولًا</p></div>' +
              '</div>' +
              '<div class="setting-row">' +
                '<div><h3>محرك التنفيذ</h3><p>محاكاة محلية لا تنفذ أوامر فعلية.</p></div>' +
                chip('success', 'i-success', 'متصل في النموذج') +
              '</div>' +
              '<div class="setting-row">' +
                '<div><h3>الأصول</h3><p>الخطوط والشعار والأيقونات كلها من نسخة الهوية المحلية.</p></div>' +
                '<button type="button" class="btn btn--quiet btn--sm" data-action="open-design-system">' + icon('i-external', '', true) + 'نظام التصميم</button>' +
              '</div>' +
            '</section>' +
          '</div>' +
        '</div>' +
      '</section>';
  }

  var systemStateData = {
    loading: {
      icon: 'i-pending',
      tone: 'info-text',
      title: 'جارٍ قراءة العمليات المتاحة…',
      description: 'يتحقق التطبيق من الأدوات المضمّنة في macOS ومن صلاحية المسارات. لم تبدأ أي عملية.',
      actions: ''
    },
    empty: {
      icon: 'i-categories',
      tone: 'muted',
      title: 'لا توجد عمليات في هذه القائمة',
      description: 'هذه حالة قائمة أول استخدام، وتختلف عن نتيجة بحث بلا تطابق.',
      actions: '<button type="button" class="btn btn--primary" data-nav="operations">' + icon('i-plus') + 'ابدأ أول عملية</button>'
    },
    'kernel-failure': {
      icon: 'i-error',
      tone: 'danger-text',
      title: 'تعذّر الاتصال بمحرك التنفيذ',
      description: 'لم تبدأ أي عملية ولم يتغيّر أي ملف. يمكنك إعادة الاتصال أو العودة إلى الفئات.',
      path: 'NSCoreError · connection refused',
      actions: '<button type="button" class="btn btn--primary" data-action="reconnect-kernel">' + icon('i-execute', '', true) + 'إعادة الاتصال</button>' +
        '<button type="button" class="btn btn--quiet" data-action="copy-kernel-details">' + icon('i-copy') + 'نسخ التفاصيل</button>'
    },
    unavailable: {
      icon: 'i-warning',
      tone: 'warning-text',
      title: 'العملية غير متاحة في هذه البيئة',
      description: 'فك ملفات 7z يحتاج الأداة 7zz، وهي غير مثبّتة. يمكنك استخدام ZIP الآن أو العودة إلى العمليات.',
      path: '/usr/local/bin/7zz · not found',
      actions: '<button type="button" class="btn btn--primary" data-action="open-zip">' + icon('i-compress') + 'استخدام ZIP</button>' +
        '<button type="button" class="btn btn--quiet" data-nav="operations">العودة إلى العمليات</button>'
    },
    permission: {
      icon: 'i-security',
      tone: 'info-text',
      title: 'تحتاج صلاحية الوصول إلى هذا المجلد',
      description: 'يحتاج التطبيق قراءة المصدر المحدد فقط. لن يطلب وصولًا إلى بقية ملفاتك.',
      path: '/Users/sara/Documents/العقود',
      actions: '<button type="button" class="btn btn--primary" data-action="open-system-settings">' + icon('i-settings') + 'فتح إعدادات النظام</button>' +
        '<button type="button" class="btn btn--quiet" data-action="check-permission">تحقق مجددًا</button>'
    },
    'repaired-settings': {
      icon: 'i-success',
      tone: 'success-text',
      title: 'أُصلح إعداد المظهر',
      description: 'تعذّرت قراءة القيمة السابقة، فأعيد المظهر إلى «حسب النظام». بقية إعداداتك وسجلك سليمة.',
      actions: '<button type="button" class="btn btn--primary" data-nav="settings">' + icon('i-settings') + 'فتح الإعدادات</button>' +
        '<button type="button" class="btn btn--quiet" data-action="dismiss-system-state">إخفاء</button>'
    }
  };

  function renderSystem() {
    setWindowTitle('حالات النظام');
    var selected = systemStateData[appState.viewState] || systemStateData.loading;
    var featureRole = appState.viewState === 'kernel-failure' || appState.viewState === 'permission'
      ? 'alert'
      : 'status';
    var cards = Object.keys(systemStateData).map(function (key) {
      var item = systemStateData[key];
      return '<button type="button" class="system-state-card' + (key === appState.viewState ? ' is-active' : '') + '" data-system-state="' + key + '">' +
        icon(item.icon, item.tone) +
        '<div><h3>' + esc(item.title) + '</h3><p>' + systemLabel(key) + '</p></div>' +
      '</button>';
    }).join('');
    var loadingExtra = appState.viewState === 'loading'
      ? '<div class="skeleton-stack" aria-hidden="true"><span class="skeleton-line"></span><span class="skeleton-line"></span><span class="skeleton-line"></span></div>'
      : '';
    return '' +
      '<section class="screen" aria-labelledby="system-title">' +
        '<header class="page-head">' +
          '<div class="page-head__copy">' +
            '<p class="page-head__eyebrow">' + icon('i-info') + 'رسالة + أثر + فعل تعافٍ</p>' +
            '<h1 id="system-title">حالات النظام</h1>' +
            '<p class="page-head__desc">نماذج لحالات لا تخص عملية ZIP وحدها. كل حالة تقول هل بدأت عملية وما الذي يمكن فعله الآن.</p>' +
          '</div>' +
        '</header>' +
        '<div class="system-state-feature" role="' + featureRole + '" aria-live="' +
          (featureRole === 'alert' ? 'assertive' : 'polite') + '">' +
          '<div class="system-state-content">' +
            (appState.viewState === 'loading' ? '<span class="spinner" role="status" aria-label="جارٍ التحميل"></span>' : icon(selected.icon, selected.tone)) +
            '<h2>' + esc(selected.title) + '</h2>' +
            '<p>' + esc(selected.description) + '</p>' +
            (selected.path ? technicalValueMarkup(selected.path) : '') +
            loadingExtra +
            '<div class="system-state-content__actions">' + selected.actions + '</div>' +
          '</div>' +
        '</div>' +
        '<div class="system-state-grid" role="group" aria-label="اختيار حالة أخرى">' + cards + '</div>' +
      '</section>';
  }

  function systemLabel(key) {
    var labels = {
      loading: 'تحميل',
      empty: 'قائمة فارغة',
      'kernel-failure': 'اتصال النواة',
      unavailable: 'عدم الإتاحة',
      permission: 'صلاحية',
      'repaired-settings': 'إصلاح إعداد'
    };
    return labels[key] || key;
  }

  function startProgress() {
    stopProgress();
    appState.progressTimer = setInterval(function () {
      if (appState.screen !== 'zip' || appState.viewState !== 'running') return stopProgress();
      appState.progress = Math.min(100, appState.progress + 4);
      var fill = document.querySelector('.execution-card .progress__fill');
      var progress = document.querySelector('.execution-card .progress');
      var label = document.querySelector('.execution-card__meta strong');
      if (fill) fill.style.width = appState.progress + '%';
      if (progress) progress.setAttribute('aria-valuenow', String(appState.progress));
      if (label) label.textContent = appState.progress + '٪';
      if (appState.progress >= 100) {
        stopProgress();
        appState.viewState = 'success';
        render();
        showToast('تم إنشاء ملف ZIP وإضافته إلى السجل.', 'success');
      }
    }, 900);
  }

  function stopProgress() {
    if (appState.progressTimer) {
      clearInterval(appState.progressTimer);
      appState.progressTimer = null;
    }
  }

  function showBackDialog() {
    appState.lastFocus = document.activeElement;
    var cancelling = appState.viewState === 'cancelling';
    els.modal.innerHTML = '' +
      '<div class="dialog" role="dialog" aria-modal="true" aria-labelledby="dialog-title" aria-describedby="dialog-desc">' +
        '<div class="dialog__icon">' + icon('i-warning') + '</div>' +
        '<h2 id="dialog-title">' + (cancelling ? 'جارٍ إنهاء العملية' : 'الضغط ما زال جاريًا') + '</h2>' +
        '<p id="dialog-desc">' +
          (cancelling
            ? 'انتظر لحظات حتى يُحذف الملف الجزئي بأمان. سنرجع تلقائيًا بعد اكتمال الإلغاء.'
            : 'الرجوع الآن يتطلب إلغاء العملية أولًا. سنوقف الضغط ونحذف ملف ZIP الجزئي، وسيبقى المصدر كما هو.') +
        '</p>' +
        '<div class="dialog__progress">' + (cancelling ? '<span class="spinner" aria-hidden="true"></span>' : icon('i-info')) +
          '<span>' + (cancelling ? 'تنظيف الملف الجزئي…' : 'المصدر لن يتغيّر في الحالتين.') + '</span></div>' +
        '<div class="dialog__actions">' +
          (!cancelling
            ? '<button type="button" class="btn btn--danger" data-action="cancel-and-back">إلغاء العملية والرجوع</button>'
            : '') +
          '<button type="button" class="btn btn--primary" data-action="stay-running" autofocus>البقاء' +
            (cancelling ? '' : ' ومتابعة التنفيذ') + '</button>' +
        '</div>' +
      '</div>';
    requestAnimationFrame(function () {
      var safeButton = els.modal.querySelector('[data-action="stay-running"]');
      if (safeButton) safeButton.focus();
    });
  }

  function closeDialog() {
    if (!els.modal) return;
    els.modal.innerHTML = '';
    if (appState.lastFocus && document.contains(appState.lastFocus)) {
      try { appState.lastFocus.focus(); } catch (error) {}
    }
  }

  function showToast(message, tone) {
    clearTimeout(appState.toastTimer);
    var iconId = tone === 'danger' ? 'i-error' : tone === 'warning' ? 'i-warning' : 'i-success';
    els.toast.innerHTML = '<div class="toast">' + icon(iconId) + '<span>' + esc(message) + '</span></div>';
    appState.toastTimer = setTimeout(function () { els.toast.innerHTML = ''; }, 2600);
  }

  function copyValue(value, successMessage) {
    var done = function () { showToast(successMessage || 'نُسخ المحتوى.', 'success'); };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(value).then(done, function () { fallbackCopy(value, done); });
    } else {
      fallbackCopy(value, done);
    }
  }

  function fallbackCopy(value, done) {
    var area = document.createElement('textarea');
    area.value = value;
    area.setAttribute('readonly', '');
    area.style.position = 'fixed';
    area.style.opacity = '0';
    document.body.appendChild(area);
    area.select();
    try {
      document.execCommand('copy');
      done();
    } catch (error) {
      showToast('تعذّر النسخ في هذا المتصفح.', 'danger');
    }
    area.remove();
  }

  function handleClick(event) {
    var target = event.target.closest('button, a');
    if (!target) return;

    if (target.dataset.nav) {
      event.preventDefault();
      navigate(target.dataset.nav);
      return;
    }
    if (target.dataset.previewSize) {
      setSize(target.dataset.previewSize);
      return;
    }
    if (target.dataset.themeSet) {
      setTheme(target.dataset.themeSet);
      if (appState.screen === 'settings') render();
      return;
    }
    if (target.dataset.motionSet) {
      appState.motion = target.dataset.motionSet;
      document.documentElement.toggleAttribute('data-motion', appState.motion === 'reduce');
      if (appState.motion === 'reduce') document.documentElement.setAttribute('data-motion', 'reduce');
      else document.documentElement.removeAttribute('data-motion');
      render();
      showToast(appState.motion === 'reduce' ? 'فُعّل تقليل الحركة.' : 'الحركة تتبع إعداد macOS.', 'success');
      return;
    }
    if (target.dataset.category) {
      if (target.dataset.category === 'compress') navigate('operations');
      else if (target.dataset.category === 'history') navigate('history');
      else showToast('هذه الفئة نموذج تصميمي قادم؛ لا توجد عمليات قابلة للتنفيذ فيها بعد.', 'warning');
      return;
    }
    if (target.dataset.historyFilter) {
      appState.historyFilter = target.dataset.historyFilter;
      document.querySelectorAll('[data-history-filter]').forEach(function (button) {
        button.setAttribute('aria-pressed', String(button === target));
      });
      filterHistory();
      return;
    }
    if (target.dataset.systemState) {
      appState.viewState = target.dataset.systemState;
      render();
      return;
    }
    if (target.dataset.settingsSection) {
      appState.viewState = target.dataset.settingsSection;
      document.querySelectorAll('[data-settings-section]').forEach(function (button) {
        button.classList.toggle('is-active', button === target);
      });
      var section = document.getElementById('settings-' + target.dataset.settingsSection);
      if (target.dataset.settingsSection === 'appearance') {
        els.main.scrollTo({ top: 0, behavior: 'smooth' });
      } else if (section) {
        section.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
      syncPickers();
      syncUrl();
      return;
    }

    var action = target.dataset.action;
    if (!action) return;

    switch (action) {
      case 'start-onboarding':
      case 'skip-onboarding':
        try { localStorage.setItem('ns-onboarding-seen', 'true'); } catch (error) {}
        navigate('categories');
        break;
      case 'toggle-example':
        appState.viewState = appState.viewState === 'example' ? 'default' : 'example';
        render();
        break;
      case 'clear-category-search': {
        var categoryInput = document.getElementById('category-search');
        if (categoryInput) {
          categoryInput.value = '';
          filterCategories('');
          categoryInput.focus();
        }
        break;
      }
      case 'preview-future':
        appState.viewState = 'future';
        syncPickers();
        showToast('هذه معاينة بنيوية فقط؛ فعل التنفيذ غير موجود حتى إطلاق العملية.', 'warning');
        break;
      case 'open-unavailable':
        navigate('system', 'unavailable');
        break;
      case 'open-zip':
        navigate('zip', 'empty');
        break;
      case 'back-from-zip':
        if (appState.viewState === 'running' || appState.viewState === 'cancelling' || appState.viewState === 'back-confirm') showBackDialog();
        else navigate('operations');
        break;
      case 'choose-source':
        appState.form.source = fullForm.source;
        if (!appState.form.output) appState.form.output = fullForm.output;
        appState.viewState = inferZipState();
        render();
        showToast('اختير مجلد «موقع-المجلس» كمصدر.', 'success');
        break;
      case 'choose-destination':
        appState.form.destination = fullForm.destination;
        appState.viewState = inferZipState();
        render();
        showToast('اختير مجلد «نسخ احتياطية» كوجهة.', 'success');
        break;
      case 'run-zip':
        if (!isFormComplete()) {
          showToast('أكمل الحقول قبل التنفيذ.', 'warning');
          return;
        }
        appState.viewState = 'running';
        appState.progress = 18;
        render();
        break;
      case 'cancel-zip':
        stopProgress();
        appState.viewState = 'cancelling';
        render();
        setTimeout(function () {
          if (appState.screen === 'zip' && appState.viewState === 'cancelling') {
            appState.viewState = 'valid';
            render();
            showToast('أُلغيت العملية وحُذف ملف ZIP الجزئي.', 'success');
          }
        }, 1800);
        break;
      case 'new-zip':
        navigate('zip', 'empty');
        break;
      case 'retry-zip':
        appState.viewState = 'running';
        appState.progress = 12;
        render();
        break;
      case 'keep-both':
        appState.form.output = 'موقع-المجلس-2026-07-29-2.zip';
        appState.form.conflict = 'copy';
        appState.viewState = 'valid';
        render();
        showToast('عُدّل الاسم للاحتفاظ بالنسختين.', 'success');
        break;
      case 'change-output-name':
        appState.viewState = 'valid';
        render();
        requestAnimationFrame(function () {
          var nameInput = document.getElementById('zip-output');
          if (nameInput) { nameInput.focus(); nameInput.select(); }
        });
        break;
      case 'replace-existing':
        appState.form.conflict = 'replace';
        appState.viewState = 'valid';
        render();
        showToast('ستطلب الخطة تأكيد الاستبدال قبل التنفيذ.', 'warning');
        break;
      case 'choose-other-destination':
        appState.form.destination = '/Users/sara/Documents/أرشيف';
        appState.viewState = 'valid';
        render();
        showToast('اختيرت وجهة فيها مساحة كافية.', 'success');
        break;
      case 'open-storage':
        showToast('محاكاة: سيفتح macOS إعدادات التخزين.', 'success');
        break;
      case 'copy-command':
      case 'copy-path':
        copyValue(target.dataset.copyValue || zipCommand(), action === 'copy-command' ? 'نُسخ الأمر.' : 'نُسخ مسار الناتج.');
        break;
      case 'copy-details':
        copyValue('ditto: /Volumes/Archive SSD/نسخ احتياطية: Permission denied', 'نُسخت تفاصيل الفشل.');
        break;
      case 'reveal-file':
      case 'reveal-history':
        showToast('محاكاة: سيُكشف الملف الناتج في Finder.', 'success');
        break;
      case 'history-details':
        showToast('تتضمن التفاصيل المصدر والأداة والأمر والمدة وما لم يتغيّر.', 'success');
        break;
      case 'clear-history':
        appState.historyFilter = 'all';
        appState.historyQuery = '';
        var historyInput = document.getElementById('history-search');
        if (historyInput) historyInput.value = '';
        document.querySelectorAll('[data-history-filter]').forEach(function (button) {
          button.setAttribute('aria-pressed', String(button.dataset.historyFilter === 'all'));
        });
        filterHistory();
        break;
      case 'toggle-switch': {
        var checked = target.getAttribute('aria-checked') === 'true';
        target.setAttribute('aria-checked', String(!checked));
        showToast(!checked ? 'سيظهر الترحيب في التشغيل القادم.' : 'لن يظهر الترحيب تلقائيًا.', 'success');
        break;
      }
      case 'clear-history-settings':
        showToast('هذه محاكاة: سيظهر تأكيد قبل مسح ٤ سجلات، دون حذف الملفات الناتجة.', 'warning');
        break;
      case 'remove-path':
        var pathRow = target.closest('.preferred-path');
        if (pathRow) pathRow.remove();
        showToast('أُزيل المسار من المفضلة فقط.', 'success');
        break;
      case 'add-path':
        showToast('محاكاة: سيفتح منتقي مجلدات macOS.', 'success');
        break;
      case 'open-design-system':
        location.href = 'design-system.html';
        break;
      case 'reconnect-kernel':
        appState.viewState = 'loading';
        render();
        setTimeout(function () {
          if (appState.screen === 'system' && appState.viewState === 'loading') {
            navigate('categories', 'default', { force: true });
            showToast('أُعيد الاتصال بمحرك التنفيذ.', 'success');
          }
        }, 1000);
        break;
      case 'copy-kernel-details':
        copyValue('NSCoreError · connection refused', 'نُسخت تفاصيل الاتصال.');
        break;
      case 'open-system-settings':
        showToast('محاكاة: سيفتح macOS قسم «الملفات والمجلدات».', 'success');
        break;
      case 'check-permission':
        showToast('تم التحقق: الصلاحية متاحة في نموذج العرض.', 'success');
        break;
      case 'dismiss-system-state':
        navigate('categories');
        break;
      case 'stay-running':
        if (appState.viewState === 'back-confirm') appState.viewState = 'running';
        closeDialog();
        syncPickers();
        syncUrl();
        if (appState.viewState === 'running') startProgress();
        break;
      case 'cancel-and-back':
        stopProgress();
        closeDialog();
        appState.viewState = 'cancelling';
        render();
        setTimeout(function () {
          navigate('operations', 'default', { force: true });
          showToast('أُلغيت العملية وحُذف الملف الجزئي قبل الرجوع.', 'success');
        }, 1500);
        break;
      default:
        break;
    }
  }

  function handleInput(event) {
    var target = event.target;
    if (target.id === 'category-search') {
      filterCategories(target.value);
      return;
    }
    if (target.id === 'history-search') {
      appState.historyQuery = target.value;
      filterHistory();
      return;
    }
    if (target.dataset.zipField) {
      appState.form[target.dataset.zipField] = target.value;
    }
  }

  function handleChange(event) {
    var target = event.target;
    if (target === els.screenPicker) {
      appState.screen = target.value;
      appState.viewState = defaultStateFor(appState.screen);
      if (appState.screen === 'zip') seedZipForm(appState.viewState);
      render();
      return;
    }
    if (target === els.statePicker) {
      appState.viewState = target.value;
      if (appState.screen === 'zip') seedZipForm(appState.viewState);
      render();
      return;
    }
    if (target.dataset.zipField) {
      appState.form[target.dataset.zipField] = target.value;
      appState.viewState = inferZipState();
      render();
      return;
    }
    if (target.hasAttribute('data-setting-select')) {
      showToast('حُفظت مدة الاحتفاظ تلقائيًا.', 'success');
    }
  }

  function handleKeydown(event) {
    if (event.key === 'Escape') {
      if (els.modal.innerHTML) {
        if (appState.viewState === 'back-confirm') appState.viewState = 'running';
        closeDialog();
        syncPickers();
        return;
      }
      var active = document.activeElement;
      if (active && (active.id === 'category-search' || active.id === 'history-search') && active.value) {
        active.value = '';
        if (active.id === 'category-search') filterCategories('');
        else {
          appState.historyQuery = '';
          filterHistory();
        }
        return;
      }
    }
    if (event.metaKey && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      if (appState.screen !== 'categories') navigate('categories');
      requestAnimationFrame(function () {
        var search = document.getElementById('category-search');
        if (search) search.focus();
      });
    }
    if (event.metaKey && event.key === ',') {
      event.preventDefault();
      navigate('settings');
    }
    if (event.metaKey && event.key.toLowerCase() === 'n') {
      event.preventDefault();
      navigate('zip', 'empty');
    }
    if (event.metaKey && event.altKey && event.key.toLowerCase() === 'm' && appState.screen === 'zip') {
      event.preventDefault();
      var satr = document.querySelector('.satr-panel');
      if (satr) satr.scrollIntoView({ block: 'start', behavior: 'smooth' });
    }
    if (event.key === 'Tab' && els.modal.innerHTML) {
      var focusables = Array.prototype.slice.call(els.modal.querySelectorAll('button:not([disabled])'));
      if (!focusables.length) return;
      var first = focusables[0];
      var last = focusables[focusables.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
  }

  function init() {
    els.stage = document.getElementById('prototype-stage');
    els.shell = document.getElementById('app-shell');
    els.main = document.getElementById('app-main');
    els.modal = document.getElementById('modal-root');
    els.toast = document.getElementById('toast-root');
    els.windowTitle = document.getElementById('window-title');
    els.screenPicker = document.getElementById('screen-picker');
    els.statePicker = document.getElementById('state-picker');
    els.statePickerWrap = document.getElementById('state-picker-wrap');

    normalizeViewState();
    if (appState.screen === 'zip') seedZipForm(appState.viewState);
    setSize(appState.size);
    setTheme(appState.theme, false);

    document.addEventListener('click', handleClick);
    document.addEventListener('input', handleInput);
    document.addEventListener('change', handleChange);
    document.addEventListener('keydown', handleKeydown);

    render();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
