// @vitest-environment node
/**
 * أيقونة التطبيق — عقدٌ مع الهوية، لا ملفاتٌ في مجلد.
 *
 * الأيقونة المعتمدة ذات شكلين مقصودين: العلامة مجرّدةً حتى ‎64px‎، وداخل
 * البلاطة الداكنة من ‎128px‎ فما فوق. السبب أن البلاطة عند ‎16px‎ تبتلع العلامة.
 *
 * وهذا بالضبط ما لا يستطيع `tauri icon` إنتاجه: يقرأ صورةً واحدة ويصغّرها، فلا
 * يعرف أن المقاسات الصغيرة تتخلّى عن حاويتها. تشغيلُه — من أي مصدر — يستبدل
 * الأيقونة المعتمدة بأخرى، ويفعل ذلك صامتًا: الملفات تُكتب، والبناء ينجح،
 * والعطل لا يظهر إلا في الـDock.
 *
 * وقد وقع فعلًا: كانت ملفات PNG هنا كلها «علامة مجرّدة على شفافية» بينما
 * `icon.icns` بلاطة — أي أن المجلد كان يصف أيقونتين مختلفتين. لم يظهر أثره لأن
 * macOS يقرأ `.icns` وحده، وكان سيظهر عند أول تشغيلٍ لأداة التوليد.
 *
 * فالبصمات هنا هي الحارس. تغيير الأيقونة يبقى ممكنًا، لكنه يصير فعلًا مقصودًا
 * يُحدَّث معه هذا الملف — لا انزلاقًا صامتًا.
 */
import { createHash } from 'node:crypto';
import { readFileSync, existsSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { inflateSync } from 'node:zlib';
import { describe, expect, it } from 'vitest';

const ICONS = new URL('../src-tauri/icons/', import.meta.url).pathname;

/** بصمات الأيقونة المعتمدة، منسوخةً من `naffith-satr-brand-v2/icons/app/`. */
const APPROVED: Record<string, string> = {
  '32x32.png': '344c332a9be2d219feb9bbdd6db5ce8e717697623671c22ff2e1c51af5a9460c',
  '128x128.png': '643d3d74ff28401b409034ac6e3c58b37c5acf22601298eb6468532eb91f7dd2',
  '128x128@2x.png': '5ea09455ecfc4b364b31acd14f9e663310d9d4b5ac398eff655d99160a8139e5',
  '256x256.png': '5ea09455ecfc4b364b31acd14f9e663310d9d4b5ac398eff655d99160a8139e5',
  '512x512.png': 'd4529efeb5f36857b0fd0901a02b6dbf3390c83ac9f7ed8b6907487649272780',
  'icon.png': '52719e2d51711bbe9b733d8f1a4c7c88e4d53caec186687d726bd9f7ab94f81e',
  'icon.icns': '4569d407c3cdc394a06403e47081a69d156f658e0f9d23351db230f2b747537e',
};

const sha = (p: string) => createHash('sha256').update(readFileSync(p)).digest('hex');

/**
 * نسبة البكسلات الشفّافة في PNG من نوع RGBA8 غير متشابك.
 *
 * فكُّ الضغط ثم رفعُ مرشّحات الأسطر الخمسة — لا مكتبة صور في التبعيات، وإضافةُ
 * واحدة من أجل فحصٍ واحد ثمنٌ أكبر من ثلاثين سطرًا مقروءة. وكل أيقونات المشروع
 * من هذا النوع، والدالّة ترمي صراحةً إن لم تكن، فلا تُقاس صورةٌ بغير أداتها.
 */
function transparentRatio(path: string): number {
  const buf = readFileSync(path);
  const width = buf.readUInt32BE(16);
  const height = buf.readUInt32BE(20);
  const [depth, colorType, interlace] = [buf[24], buf[25], buf[28]];
  if (depth !== 8 || colorType !== 6 || interlace !== 0) {
    throw new Error(`${path}: ليس RGBA8 غير متشابك (depth=${depth} type=${colorType})`);
  }

  // تجميع كل كتل IDAT — قد تُقسَّم على أكثر من كتلة.
  const parts: Buffer[] = [];
  for (let at = 8; at + 8 <= buf.length; ) {
    const len = buf.readUInt32BE(at);
    const tag = buf.toString('ascii', at + 4, at + 8);
    if (tag === 'IDAT') parts.push(buf.subarray(at + 8, at + 8 + len));
    if (tag === 'IEND') break;
    at += 12 + len; // طول + وسم + بيانات + CRC
  }
  const raw = inflateSync(Buffer.concat(parts));

  const bpp = 4;
  const stride = width * bpp;
  const out = Buffer.alloc(height * stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)];
    const line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let x = 0; x < stride; x++) {
      // القراءة خارج الحدّ صفرٌ بتعريف المواصفة، و`?? 0` يقولها للمترجم أيضًا.
      const a = x >= bpp ? (out[y * stride + x - bpp] ?? 0) : 0; // يسار
      const b = y > 0 ? (out[(y - 1) * stride + x] ?? 0) : 0; // فوق
      const c = x >= bpp && y > 0 ? (out[(y - 1) * stride + x - bpp] ?? 0) : 0; // قطري
      let v = line[x] ?? 0;
      if (filter === 1) v += a;
      else if (filter === 2) v += b;
      else if (filter === 3) v += (a + b) >> 1;
      else if (filter === 4) {
        // Paeth
        const p = a + b - c;
        const pa = Math.abs(p - a);
        const pb = Math.abs(p - b);
        const pc = Math.abs(p - c);
        v += pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      }
      out[y * stride + x] = v & 0xff;
    }
  }

  let clear = 0;
  for (let i = 3; i < out.length; i += bpp) if ((out[i] ?? 0) < 16) clear++;
  return clear / (width * height);
}

describe('أيقونة التطبيق هي المعتمدة في الهوية', () => {
  it('كل ملف أيقونة مطابقٌ لبصمته المعتمدة', () => {
    const drifted = Object.entries(APPROVED)
      .filter(([name, hash]) => sha(join(ICONS, name)) !== hash)
      .map(([name]) => name);
    expect(
      drifted,
      `أيقونةٌ تغيّرت عن المعتمد: ${drifted.join(', ')} — إن كان التغيير مقصودًا فانسخ من الهوية وحدّث البصمات هنا`,
    ).toEqual([]);
  });

  it('مصدر التوليد `naffith.iconset` منسوخٌ كاملًا', () => {
    // بلا هذا المجلد لا سبيل إلى إعادة بناء `.icns` إلا بأداةٍ تشتقّ المقاسات
    // من صورةٍ واحدة، فتهدم الشكلين. وجودُه هو ما يجعل `iconutil` ممكنًا.
    const want = [16, 32, 128, 256, 512].flatMap((n) => [
      `icon_${n}x${n}.png`,
      `icon_${n}x${n}@2x.png`,
    ]);
    const have = readdirSync(join(ICONS, 'naffith.iconset'));
    expect(want.filter((f) => !have.includes(f)), 'مقاسٌ ناقص من مصدر التوليد').toEqual([]);
  });

  it('الشكلان محفوظان: مجرّدة حتى ‎64px‎، وبلاطة من ‎128px‎', () => {
    // فحصٌ على البكسلات لا على البصمة وحدها: البصمة تمسك أي تغيير، وهذا يشرح
    // *أيّ* تغييرٍ وقع — أن الأيقونة صارت شكلًا واحدًا في كل المقاسات، وهو
    // الأثر الحتميّ لتشغيل `tauri icon`.
    //
    // والقياس حقيقي: تُفكّ ضغطة PNG وتُقرأ قناة الشفافية. المحاولة الأولى هنا
    // قدّرت الشكل من «حجم الملف لكل بكسل» — وهو تخمينٌ يبدو قياسًا: ‎0.11‎
    // للبلاطة و‎0.33‎ للمجرّدة، رقمان لا يفصل بينهما حدٌّ له معنى.
    for (const [file, wantTile] of [
      ['32x32.png', false],
      ['128x128.png', true],
      ['512x512.png', true],
      ['icon.png', true],
    ] as const) {
      const transparent = transparentRatio(join(ICONS, file));
      // البلاطة تملأ المربّع فلا يبقى شفافًا إلا ما استدار من زواياها؛ والعلامة
      // المجرّدة شفافةٌ فيما حولها. المقيس: ‎3–4٪‎ للبلاطة، و‎74٪‎ للمجرّدة.
      if (wantTile) expect(transparent, `${file} لم تعد بلاطة`).toBeLessThan(0.2);
      else expect(transparent, `${file} لم تعد علامةً مجرّدة`).toBeGreaterThan(0.5);
    }
  });

  it('لا تُترك أدوات توليد أيقونات في المشروع', () => {
    // `tauri icon` يكتب `icon.ico` و`Square*Logo.png` لويندوز، وهذه حزمة macOS
    // وحدها. وجودُها دليلٌ على أن الأداة شُغِّلت — أي أن الأيقونة أُعيد توليدها.
    const generated = readdirSync(ICONS).filter(
      (f) => /^Square.*Logo\.png$/.test(f) || f === 'icon.ico' || f === 'StoreLogo.png',
    );
    expect(generated, `آثار توليدٍ آلي: ${generated.join(', ')}`).toEqual([]);
  });

  it('كل أيقونة يشير إليها الإعداد موجودة فعلًا', () => {
    const conf = JSON.parse(readFileSync(join(ICONS, '../tauri.conf.json'), 'utf8')) as {
      bundle: { icon: string[] };
    };
    const missing = conf.bundle.icon.filter(
      (rel) => !existsSync(join(ICONS, '..', rel)),
    );
    expect(missing, `مسارٌ في الإعداد بلا ملف: ${missing.join(', ')}`).toEqual([]);
    // وكلّها من المجموعة المعتمدة، فلا يتسرّب ملفٌ غير مثبَّت ببصمة.
    const unpinned = conf.bundle.icon
      .map((rel) => rel.replace(/^icons\//, ''))
      .filter((name) => !(name in APPROVED));
    expect(unpinned, `أيقونةٌ في الحزمة بلا بصمة: ${unpinned.join(', ')}`).toEqual([]);
  });
});
