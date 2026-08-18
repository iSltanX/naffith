// @vitest-environment node
/**
 * أيقونة التطبيق — عقدٌ مع الهوية، لا ملفاتٌ في مجلد.
 *
 * المعتمد اليوم سيّدٌ واحد: بلاطةُ `>_` من صفحة «‏03 — App Icon»، بالشكل نفسه
 * من ‎1024‎ إلى ‎16‎. وقد سبقتها ثنائيةٌ مختلفة — علامةٌ مجرّدة حتى ‎64px‎ وبلاطةٌ
 * من ‎128px‎ — ولم تعد الشكل المعتمد.
 *
 * ومن لوح التصدير في تلك الصفحة تُقرأ المعالجة، لا من المكوّن وحده:
 * نُسَخ التصدير الخمس تحمل `exportSettings` صريحة. وحدُّ الأيقونة الخارجي
 * صار **بلون الأرضية نفسها** (`#0F0F1E`) بعد أن كان أبيض، فتُقرأ الحافة
 * امتدادًا لكتلة الأيقونة لا إطارًا حولها. أمّا **الظلّ المُسقَط** فيخرج عن
 * المربّع (‏‎1024×1058‎ لا ‎1024×1024‎) ويضاعف ظلّ الـDock، فلا يُشحن.
 *
 * وهذا ما لا يستطيع `tauri icon` أن يحفظه: يقرأ صورةً واحدة ويصغّرها. لذلك
 * يبقى `naffith.iconset/` منسوخًا كاملًا، ويُجمَع `.icns` منه بـ`iconutil`.
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

/**
 * بصمات الأيقونة المعتمدة، مولَّدةً من `Brand/App Icon/Master` في صفحة
 * «‏03 — App Icon» من ملف الهوية.
 *
 * المصدر تغيّر: كانت الأيقونة تُنسخ من `naffith-satr-brand-v2/icons/app/`
 * وهي **العلامة (الرسم)**، والهوية تفصل صراحةً بين اثنين:
 * الرسم عَلَمُ الهوية (صفحة ‎02‎)، و`>_` أيقونةُ التطبيق (صفحة ‎03‎ كاملةً).
 */
const APPROVED: Record<string, string> = {
  '32x32.png': '8c92e511854b443d7e8cf35f83f4855ab81fbd4abf7f50bab4da247df0b25c02',
  '128x128.png': 'b8ddfce862fdefb8087335922383e6ccf55e742bc0307d71209019d062f32e6a',
  '128x128@2x.png': '98cc05119397411891e59e45c8f7734db2c817fe7c237c6ca3f1f0177baa0dce',
  '256x256.png': '98cc05119397411891e59e45c8f7734db2c817fe7c237c6ca3f1f0177baa0dce',
  '512x512.png': '1cfcd085f37d13bb0bbe877a1c09dff1d0705b083ca76db0dc27a7fb4acc89d8',
  'icon.png': '41a11210db9307a4fd0966f56a010832656ce1e40f9fabc1f879e1a2d480ca4c',
  'icon.icns': 'd2397ece896e2795e2199cf7f9754aa2e605865417379490ebb856b57825ae97',
};

const sha = (p: string) => createHash('sha256').update(readFileSync(p)).digest('hex');

/**
 * نسبة البكسلات الشفّافة في PNG من نوع RGBA8 غير متشابك.
 *
 * فكُّ الضغط ثم رفعُ مرشّحات الأسطر الخمسة — لا مكتبة صور في التبعيات، وإضافةُ
 * واحدة من أجل فحصٍ واحد ثمنٌ أكبر من ثلاثين سطرًا مقروءة. وكل أيقونات المشروع
 * من هذا النوع، والدالّة ترمي صراحةً إن لم تكن، فلا تُقاس صورةٌ بغير أداتها.
 */
function decodeRgba(path: string): { width: number; height: number; rgba: Buffer } {
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
  return { width, height, rgba: out };
}

/** نسبة البكسلات الشفّافة. */
function transparentRatio(path: string): number {
  const { width, height, rgba } = decodeRgba(path);
  let clear = 0;
  for (let i = 3; i < rgba.length; i += 4) if ((rgba[i] ?? 0) < 16) clear++;
  return clear / (width * height);
}


/** أول بكسل على يسار منتصف الصورة — أي حرف البلاطة عند أعرض نقطة فيها. */
function edgePixel(path: string): { r: number; g: number; b: number; a: number } {
  const { width, height, rgba } = decodeRgba(path);
  const y = height >> 1;
  const i = y * width * 4;
  return {
    r: rgba[i] ?? 0,
    g: rgba[i + 1] ?? 0,
    b: rgba[i + 2] ?? 0,
    a: rgba[i + 3] ?? 0,
  };
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

  it('سيّدٌ واحد في كل المقاس: بلاطةٌ مستديرة، لا علامة مجرّدة ولا مربّع حادّ', () => {
    // صفحة ‎03‎ تعلنها صراحةً: «‏one Master, many sizes — geometry preserved,
    // optics adjusted only when needed»، وسلّمُ المقاسات فيها يعرض البلاطة
    // نفسها من ‎1024‎ إلى ‎16‎. فالثنائيةُ السابقة (علامةٌ مجرّدة حتى ‎64px‎
    // وبلاطةٌ من ‎128px‎) لم تعد الشكل المعتمد — وهي التي كانت تُفحص هنا.
    //
    // والقياس حقيقي: تُفكّ ضغطة PNG وتُقرأ قناة الشفافية. والحدّان يمسكان
    // عطلين متقابلين في طرفَي الرقم:
    //   • فوق ‎10٪‎ → عادت علامةً مجرّدةً على شفافية (المقيس لها: ‎74٪‎).
    //   • تحت ‎1٪‎  → ذهب استدارةُ الزوايا فصارت مربّعًا حادًّا يملأ الإطار.
    // والمقيس اليوم بين ‎2.6٪‎ و‎3.2٪‎ — وهو ما تتركه زوايا المربّع المستدير وحدها.
    for (const file of ['32x32.png', '128x128.png', '512x512.png', 'icon.png'] as const) {
      const transparent = transparentRatio(join(ICONS, file));
      expect(transparent, `${file} لم تعد بلاطة — عادت علامةً مجرّدة؟`).toBeLessThan(0.1);
      expect(transparent, `${file} فقدت استدارة زواياها`).toBeGreaterThan(0.01);
    }
  });

  it('حافة الأيقونة من كتلتها: لونُ الأرضية لا إطارٌ فاتح', () => {
    // الحدّ كان أبيض (‏#FFFFFF‏) فيُقرأ إطارًا مرسومًا حول البلاطة. صار بلون
    // الأرضية، فالحافة امتدادٌ للسطح. ويُقاس عند منتصف الارتفاع حيث تبلغ
    // البلاطة أعرضَ نقطةٍ فيها فتلامس حرف الإطار — أي حيث كان الإطار يظهر.
    const px = edgePixel(join(ICONS, 'icon.png'));
    expect(px.a, 'الحافة صارت شفّافة').toBe(255);
    expect(
      [px.r, px.g, px.b],
      `حافة الأيقونة ${JSON.stringify(px)} ليست لون الأرضية #0F0F1E`,
    ).toEqual([15, 15, 30]);
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
