/**
 * إعداد معاينة بصرية مؤقّت — أداةُ عملٍ لا جزءٌ من المنتج.
 *
 * يحقن جسرَ Tauri وهميًّا في `index.html` أثناء التطوير وحده، فتعمل الشاشة
 * الحقيقية في المتصفّح ببيانات مطابقة لما تعيده النواة. لا سطر من `src/` يعرف
 * بوجود هذا الملف.
 *
 *   npx vite --config vite.preview.config.ts
 *
 * ## الفهرس حقيقي لا مخترع
 *
 * `preview-catalogue.json` يُولَّد من النواة نفسها:
 *
 *   cargo run --manifest-path src-tauri/Cargo.toml --example dump_catalogue \
 *     > preview-catalogue.json
 *
 * كانت القائمة مكتوبةً بيدٍ داخل هذا الملف. لقطةُ شاشةٍ لقائمةٍ مخترعة تُري ما
 * نتمنّاه لا ما بنيناه، وتبقى «صحيحة» بعد أن تتغيّر النواة — وهو أسوأ ما يمكن
 * أن تفعله أداةُ معاينة. الآن تنكسر المعاينة إن انكسر الفهرس.
 *
 * ## سائق الحالات
 *
 * ما بعد `#` في العنوان يقود الشاشة إلى حالةٍ بعينها، للقطات وحدها:
 * `#category`، `#search`، `#op`، `#done`، `#failed`، `#log`، `#light`.
 */
import { readFileSync } from 'node:fs';
import { defineConfig, mergeConfig, type Plugin } from 'vite';
import base from './vite.config';

const CATALOGUE = readFileSync(new URL('./preview-catalogue.json', import.meta.url), 'utf8');

const MOCK = `<script>
(function () {
  var CATALOGUE = ${CATALOGUE};
  var OPS = CATALOGUE.operations;
  var CATS = CATALOGUE.categories;

  localStorage.setItem('naffith.settings', JSON.stringify({
    schemaVersion: 2,
    onboardingCompletedAt: '2026-01-01T00:00:00.000Z',
    favourites: ['compress.folder.zip', 'disk.hash.sha256'],
    lastCategoryId: 'compress',
  }));

  var DITTO = '/usr/bin/ditto';
  var SOURCE = '/Users/abutalal/Documents/مشروع الهوية';
  var DEST = '/Users/abutalal/Desktop';

  /** سجلٌّ مصطنع: يغذّي «المستخدَمة حديثًا» وشاشة السجلّ معًا. */
  var NOW = Math.floor(Date.now() / 1000);
  var JOURNAL = [
    {
      id: 'r-1', op_id: 'disk.hash.sha256', category: 'disk', at: NOW - 7400,
      duration_ms: 320, program: '/usr/bin/shasum',
      args: ['-a', '256', '--', '/Users/abutalal/Downloads/installer.dmg'],
      inputs: [{ id: 'source', value: '/Users/abutalal/Downloads/installer.dmg' }],
      tail: ['9f2c…a17b  /Users/abutalal/Downloads/installer.dmg'],
      state: 'succeeded', produced: '/Users/abutalal/Downloads/installer.dmg',
    },
    {
      id: 'r-2', op_id: 'net.dns', category: 'network', at: NOW - 5200,
      duration_ms: 148, program: '/usr/bin/dig',
      args: ['+noall', '+answer', 'example.com', 'MX'],
      inputs: [{ id: 'domain', value: 'example.com' }, { id: 'record', value: 'MX' }],
      tail: ['example.com.  86400  IN  MX  10 mail.example.com.'],
      state: 'succeeded',
    },
    {
      id: 'r-3', op_id: 'compress.zip.extract', category: 'compress', at: NOW - 3100,
      duration_ms: 90, program: '/usr/bin/ditto',
      args: ['-x', '-k', '/Users/abutalal/Downloads/assets.zip', DEST + '/.naffith-a1.part'],
      inputs: [
        { id: 'archive', value: '/Users/abutalal/Downloads/assets.zip' },
        { id: 'destination', value: DEST },
        { id: 'folder_name', value: 'assets' },
      ],
      tail: ['ditto: /Users/abutalal/Downloads/assets.zip: Invalid or incomplete archive'],
      state: 'failed', reason: 'exit', code: 2,
    },
    {
      id: 'r-4', op_id: 'compress.folder.zip', category: 'compress', at: NOW - 900,
      duration_ms: 4120, program: DITTO,
      args: ['-c', '-k', '--sequesterRsrc', '--keepParent', SOURCE, DEST + '/.naffith-b2.part'],
      inputs: [
        { id: 'source', value: SOURCE },
        { id: 'destination', value: DEST },
        { id: 'archive_name', value: 'نسخة احتياطية ٢٠٢٦' },
      ],
      tail: ['copying ' + SOURCE, '  1843 items, 12.4 MB'],
      state: 'succeeded', produced: DEST + '/نسخة احتياطية ٢٠٢٦.zip',
    },
  ];

  function opById(id) {
    for (var i = 0; i < OPS.length; i += 1) if (OPS[i].id === id) return OPS[i];
    return null;
  }

  /**
   * يبني خطةً من مواصفة العملية الحقيقية.
   *
   * ‹argv‹ هنا مصطنع — النواة وحدها تبنيه — لكن **شكل** الاستجابة وحقولها
   * ومفاتيح الشرح مأخوذة من العملية كما أعلنتها. فما تراه الشاشة من بنية هو
   * ما ستراه من النواة.
   */
  function planFor(opId, inputs) {
    var op = opById(opId);
    var vals = {};
    Object.keys(inputs).forEach(function (k) { vals[k] = inputs[k].value; });

    var argv, keys, roles, produces = null, writesTo = null, warnings = [], estimate = null;

    if (opId === 'compress.folder.zip') {
      var name = vals.archive_name || 'أرشيف';
      produces = (vals.destination || DEST).replace(/\\/$/, '') + '/' + name + '.zip';
      writesTo = (vals.destination || DEST).replace(/\\/$/, '') + '/.naffith-7f31a0c4-' + name + '.zip.part';
      argv = [DITTO, '-c', '-k', '--sequesterRsrc', '--keepParent', vals.source, writesTo];
      keys = ['explain.ditto.tool', 'explain.ditto.create', 'explain.ditto.pkzip',
              'explain.ditto.sequester', 'explain.ditto.keep_parent', null, null];
      roles = ['tool', 'flag', 'flag', 'flag', 'flag', 'path', 'path'];
      warnings = ['warn.zip.name_encoding'];
      estimate = { approx_source_bytes: 12400000, scanned_entries: 1843, complete: true };
    } else if (opId === 'files.find.large') {
      argv = ['/usr/bin/find', vals.folder, '-type', 'f', '-size', '+' + (vals.min_megabytes || 100) + 'M'];
      keys = ['explain.find.tool', null, 'explain.find.type', null, 'explain.find.size', null];
      roles = ['tool', 'path', 'flag', 'value', 'flag', 'value'];
    } else {
      argv = ['/usr/bin/true'];
      keys = [null];
      roles = ['tool'];
    }

    return {
      token: 'tok-preview', plan_id: 'p-preview', op_id: opId,
      title_key: op.title_key, description_key: op.description_key,
      category: op.category, danger: op.danger,
      argv_display: argv,
      explain: argv.map(function (tk, i) { return { token: tk, key: keys[i], role: roles[i] }; }),
      warnings: warnings,
      tool: { id: op.tool, path: argv[0] },
      conflict: op.conflict, estimate: estimate,
      produces: produces, writes_to: writesTo, working_directory: null,
    };
  }

  // ── سائق الحالات ───────────────────────────────────────────────────
  var want = String(location.hash || '').slice(1).split(',');
  function wants(n) { return want.indexOf(n) !== -1; }
  if (wants('light')) document.documentElement.setAttribute('data-theme', 'light');
  if (wants('dark')) document.documentElement.setAttribute('data-theme', 'dark');

  function ready(sel, cb, tries) {
    tries = tries === undefined ? 240 : tries;
    var el = document.querySelector(sel);
    if (el) return cb(el);
    if (tries > 0) setTimeout(function () { ready(sel, cb, tries - 1); }, 25);
  }
  function type(el, v) {
    var d = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value');
    d.set.call(el, v);
    el.dispatchEvent(new Event('input', { bubbles: true }));
  }
  function clickWith(sel, text) {
    var all = document.querySelectorAll(sel);
    for (var i = 0; i < all.length; i += 1) {
      if (all[i].textContent.indexOf(text) === 0) { all[i].click(); return true; }
    }
    return false;
  }

  function openOperation(then) {
    ready('.op-card', function () {
      clickWith('button.op-card', 'الضغط وفكّ الضغط');
      ready('.cat__title-row', function () {
        setTimeout(function () {
          clickWith('button.op-card', 'ضغط مجلد');
          ready('.row-field input', function () { setTimeout(then, 40); });
        }, 60);
      });
    });
  }

  function drive() {
    if (wants('category')) {
      return ready('.op-card', function () {
        setTimeout(function () { clickWith('button.op-card', 'الضغط وفكّ الضغط'); }, 40);
      });
    }
    if (wants('search')) {
      return ready('.lib__search-input', function (box) {
        setTimeout(function () { type(box, 'ضغط'); }, 60);
      });
    }
    if (wants('log')) {
      return ready('.ops__quiet button', function () {
        setTimeout(function () { clickWith('.ops__quiet button', 'سجلّ التشغيل'); }, 40);
      });
    }
    if (wants('op') || wants('done') || wants('failed') || wants('running')) {
      return openOperation(function () {
        var boxes = document.querySelectorAll('.row-field input[type=text]');
        type(boxes[0], SOURCE);
        type(boxes[1], DEST);
        type(boxes[2], 'نسخة احتياطية ٢٠٢٦');
        if (wants('op')) return;
        ready('.args', function () {
          setTimeout(function () { document.querySelector('.naffith__go').click(); }, 320);
        });
      });
    }
  }
  addEventListener('DOMContentLoaded', function () { setTimeout(drive, 80); });

  var handlers = {};
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: function () {} };
  window.__TAURI_INTERNALS__ = {
    metadata: {},
    convertFileSrc: function (p) { return p; },
    transformCallback: function (cb) {
      var name = '_cb_' + Math.random().toString(36).slice(2);
      window[name] = cb;
      return name;
    },
    invoke: function (cmd, args) {
      if (cmd === 'list_operations') return Promise.resolve(OPS);
      if (cmd === 'list_categories') return Promise.resolve(CATS);
      if (cmd === 'recent_runs') return Promise.resolve(JOURNAL);
      if (cmd === 'plan') return Promise.resolve(planFor(args.opId, args.inputs));
      if (cmd === 'reveal' || cmd === 'cancel') return Promise.resolve();
      if (cmd === 'journal_delete' || cmd === 'journal_clear') return Promise.resolve();
      if (cmd === 'execute') {
        var id = 'run-preview';
        var fail = wants('failed');
        var stall = wants('running');
        var emit = function (p) {
          var fn = handlers['run://output'];
          if (fn) fn({ event: 'run://output', payload: p });
        };
        var lines = fail
          ? [
              { stream: 'stdout', line: 'ditto: created ' + DEST + '/.naffith-7f31a0c4' },
              { stream: 'stderr', line: 'ditto: ' + SOURCE + '/خاص: Permission denied' },
              { stream: 'stderr', line: 'ditto: exiting with error' },
            ]
          : [
              { stream: 'stdout', line: 'copying ' + SOURCE },
              { stream: 'stdout', line: '  1843 items, 12.4 MB' },
              { stream: 'stderr', line: 'ditto: ._notes: skipping AppleDouble companion' },
            ];
        lines.forEach(function (l, i) {
          setTimeout(function () { emit({ run_id: id, stream: l.stream, line: l.line }); }, 120 + i * 160);
        });
        if (!stall) {
          setTimeout(function () {
            var fn = handlers['run://finished'];
            if (!fn) return;
            fn({
              event: 'run://finished',
              payload: fail
                ? { run_id: id, status: 'failed', code: 1 }
                : { run_id: id, status: 'success', produced: DEST + '/نسخة احتياطية ٢٠٢٦.zip' },
            });
          }, 900);
        }
        return Promise.resolve(id);
      }
      if (cmd === 'plugin:event|listen') {
        handlers[args.event] = window[args.handler];
        return Promise.resolve(1);
      }
      if (cmd === 'plugin:event|unlisten') return Promise.resolve();
      if (cmd === 'plugin:dialog|open') return Promise.resolve(SOURCE);
      return Promise.reject({ key: 'err.unknown', input: null, detail: cmd });
    },
  };
})();
</script>`;

function tauriMock(): Plugin {
  return {
    name: 'preview:tauri-mock',
    transformIndexHtml: {
      order: 'pre',
      handler(html) {
        return html.replace('</head>', `${MOCK}\n</head>`);
      },
    },
  };
}

export default mergeConfig(
  base,
  defineConfig({
    plugins: [tauriMock()],
    server: { port: 5199, strictPort: true },
  }),
);
