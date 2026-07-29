// @vitest-environment node
/**
 * عقد السلك: أن يبقى ما تعلنه `ipc.ts` هو ما يكتبه serde فعلًا.
 *
 * الحدّ بين النواة والواجهة موصوفٌ مرّتين: مرّة في Rust بأنواعٍ تحمل
 * `#[derive(Serialize)]`، ومرّة في `ipc.ts` بواجهاتٍ مكتوبة باليد. لا شيء
 * يربطهما — لا توليد، ولا مخطَّط مشترك — فهما وصفان يدويان لصيغةٍ واحدة،
 * ووصفان يدويان يفترقان حتمًا مع الوقت.
 *
 * وافتراقهما لا يُسقط بناءً ولا اختبارًا: `tsc` يصدّق ما في `ipc.ts` لأنه لا
 * يعرف غيره، والنواة تُختبر بلا واجهة. الثمن يظهر عند المستخدم وحده — حقلٌ
 * أُعيدت تسميته في Rust يصل `undefined`، ومتغيّرٌ أُضيف إلى `enum` ونُسي في
 * الواجهة يصل شكلًا لا تعرفه أي فرع `if`، فلا تُرسم الشاشة أو تنهار.
 *
 * لذلك يقرأ هذا الملف **المصدرين معًا** — مصدر Rust ومصدر `ipc.ts` — ويثبت
 * أنهما يقولان الشيء نفسه: أسماء الحقول بعد إعادة تسمية serde، ومتغيّرات كل
 * `enum` بعد `snake_case`، والاتجاه العكسي أيضًا (ما تستطيع الواجهة إرساله
 * يجب أن تستطيع النواة قراءته).
 *
 * الأسلوب مأخوذ عن `i18n.test.ts`: المصدر هو المرجع، لا قائمةٌ يدوية تتقادم
 * بصمت. ومنه أيضًا انضباط «حراسة الحارس»: كل مُحلِّل هنا يُختبر أنه وجد شيئًا
 * فعلًا، كي لا يمرّ تعبيرٌ نمطيّ مكسورٌ فارغًا فيبدو العقد سليمًا وهو غير محروس.
 */
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

const CORE_SRC = new URL('../src-tauri/src', import.meta.url).pathname;
const IPC = readFileSync(new URL('./ipc.ts', import.meta.url).pathname, 'utf8');

/** ملفات النواة التي تعبر أنواعُها الحدّ. */
const RUST_FILES = [
  'lib.rs',
  'spec.rs',
  'planner.rs',
  'journal.rs',
  'executor.rs',
  'error.rs',
  'value.rs',
] as const;

const RUST: Map<string, string> = new Map(
  RUST_FILES.map((f) => [f, readFileSync(join(CORE_SRC, f), 'utf8')]),
);

// ── قراءة مصدر Rust ────────────────────────────────────────────────────

/**
 * يحذف تعليقات Rust قبل أي تقطيع.
 *
 * تعليقات التوثيق هنا عربية وفيها فواصل بغزارة، وتركُها يجعل «اقسم عند كل
 * فاصلة في المستوى الأعلى» يخترع حقولًا لا وجود لها.
 */
function stripRustComments(source: string): string {
  return source.replace(/\/\/.*$/gm, '');
}

type RustItem = { file: string; attrs: string[]; body: string };

/**
 * جسم `struct` أو `enum` معلَن في المستوى الأعلى، مع سماته.
 *
 * البحث مقيّد ببداية السطر بلا مسافة بادئة: كل نوع يعبر الحدّ معلَنٌ في
 * المستوى الأعلى، وذكرُ اسمه داخل تعليقٍ أو دالة لا يُلتقط.
 */
function findItem(kind: 'struct' | 'enum', name: string): RustItem | null {
  for (const file of RUST_FILES) {
    const source = RUST.get(file);
    if (source === undefined) continue;
    const found = new RegExp(String.raw`^pub ${kind} ${name}\b`, 'm').exec(source);
    if (found === null) continue;

    // السمات تعلو الإعلان مباشرة، وقد يتخلّلها تعليق توثيق.
    const before = source.slice(0, found.index).split('\n');
    const attrs: string[] = [];
    for (let i = before.length - 2; i >= 0; i -= 1) {
      const line = (before[i] ?? '').trim();
      if (line.startsWith('#[')) {
        attrs.unshift(line);
        continue;
      }
      if (line.startsWith('//')) continue;
      break;
    }

    const open = source.indexOf('{', found.index);
    if (open === -1) continue;
    let depth = 0;
    for (let i = open; i < source.length; i += 1) {
      if (source[i] === '{') depth += 1;
      else if (source[i] === '}') {
        depth -= 1;
        if (depth === 0) return { file, attrs, body: source.slice(open + 1, i) };
      }
    }
  }
  return null;
}

function requireItem(kind: 'struct' | 'enum', name: string): RustItem {
  const item = findItem(kind, name);
  // حارسٌ على الحارس: اسمٌ أُعيدت تسميته في Rust يجب أن يُسقط الاختبار هنا،
  // لا أن يجعله يفحص مجموعةً فارغة فيمرّ.
  expect(item, `لم يُعثر على ${kind} ${name} في مصدر النواة`).not.toBeNull();
  return item as RustItem;
}

type SerdeAttrs = { renameAll: string | null; tag: string | null; content: string | null };

function serdeAttrs(attrs: string[]): SerdeAttrs {
  const text = attrs.filter((a) => a.startsWith('#[serde')).join(' ');
  return {
    renameAll: /rename_all\s*=\s*"([a-z_]+)"/.exec(text)?.[1] ?? null,
    tag: /\btag\s*=\s*"(\w+)"/.exec(text)?.[1] ?? null,
    content: /\bcontent\s*=\s*"(\w+)"/.exec(text)?.[1] ?? null,
  };
}

/** يقسم جسم نوع عند فواصل المستوى الأعلى وحدها. */
function topLevelParts(body: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < body.length; i += 1) {
    const c = body[i];
    if (c === '{' || c === '(' || c === '[') depth += 1;
    else if (c === '}' || c === ')' || c === ']') depth -= 1;
    else if (c === ',' && depth === 0) {
      parts.push(body.slice(start, i));
      start = i + 1;
    }
  }
  parts.push(body.slice(start));
  return parts.map((p) => p.trim()).filter((p) => p !== '');
}

/** ما يكتبه serde لاسمٍ بصيغة `CamelCase` تحت `rename_all = "snake_case"`. */
function applyRename(name: string, renameAll: string | null): string {
  if (renameAll !== 'snake_case') return name;
  return name.replace(/(?<!^)([A-Z])/g, '_$1').toLowerCase();
}

type RustVariant = { name: string; wire: string; fields: string[] };

/**
 * متغيّرات `enum` بأسمائها على السلك وأسماء حقول حمولتها **على السلك**.
 *
 * والتمييز بين صيغتَي الوسم هو كل الفرق:
 *
 * — وسمٌ داخلي (`tag` وحده): تُكتب حقول المتغيّر إلى جانب الوسم بأسمائها من
 *   Rust، فحمولة `Signalled { signal }` هي `signal`.
 * — وسمٌ مجاور (`tag` مع `content`): تُكتب الحمولة كلها — بنيةً كانت أو قيمة
 *   واحدة — تحت اسم `content` واحد، فحمولة `Truncated { dropped }` على السلك
 *   هي `line` و`dropped` داخلها. قراءتها `dropped` تجعل الواجهة تنتظر الحقل
 *   في المستوى الأعلى وهو ليس هناك.
 */
function enumVariants(item: RustItem): RustVariant[] {
  const { renameAll, content } = serdeAttrs(item.attrs);
  return topLevelParts(stripRustComments(item.body)).map((chunk) => {
    const text = chunk
      .split('\n')
      .map((l) => l.trim())
      .filter((l) => l !== '' && !l.startsWith('#['))
      .join(' ');
    const name = /^(\w+)/.exec(text)?.[1] ?? '';
    const brace = text.indexOf('{');
    const carries = brace !== -1 || text.includes('(');
    let fields: string[] = [];
    if (content !== null) {
      fields = carries ? [content] : [];
    } else if (brace !== -1) {
      fields = fieldNames(text.slice(brace + 1, text.lastIndexOf('}')));
    }
    return { name, wire: applyRename(name, renameAll), fields };
  });
}

function fieldNames(body: string): string[] {
  return topLevelParts(body)
    .map((part) => {
      const decl = part
        .split('\n')
        .map((l) => l.trim())
        .filter((l) => l !== '' && !l.startsWith('#['))
        .join(' ');
      return /^(?:pub\s+)?(\w+)\s*:/.exec(decl)?.[1] ?? '';
    })
    .filter((n) => n !== '');
}

type RustField = { name: string; type: string; flatten: boolean; skipped: boolean };

function structFields(item: RustItem): RustField[] {
  const { renameAll } = serdeAttrs(item.attrs);
  return topLevelParts(stripRustComments(item.body))
    .map((part) => {
      const lines = part.split('\n').map((l) => l.trim());
      const attrs = lines.filter((l) => l.startsWith('#['));
      const decl = lines.filter((l) => l !== '' && !l.startsWith('#[')).join(' ');
      const matched = /^(?:pub\s+)?(\w+)\s*:\s*(.+)$/.exec(decl);
      if (matched === null) return null;
      const attrText = attrs.join(' ');
      return {
        name: applyRename(matched[1] as string, renameAll),
        type: (matched[2] as string).trim(),
        flatten: /\bflatten\b/.test(attrText),
        skipped: /\bskip_serializing_if\b/.test(attrText),
      };
    })
    .filter((f): f is RustField => f !== null);
}

/**
 * أسماء الحقول التي يكتبها `enum` مسطَّحًا داخل بنية.
 *
 * ‏`tag` وحده يعني وسمًا داخليًا: يُكتب الوسم ثم حقول المتغيّر مباشرة، فاتحاد
 * حقول كل المتغيّرات هو ما يمكن أن يظهر. و`tag` مع `content` يعني وسمًا
 * مجاورًا: حقلان لا غير مهما كانت الحمولة.
 */
function enumWireFields(typeName: string): string[] {
  const bare = (typeName.split('::').pop() ?? typeName).trim();
  const item = requireItem('enum', bare);
  const { tag, content } = serdeAttrs(item.attrs);
  const out = new Set<string>();
  if (tag !== null) out.add(tag);
  if (content !== null) {
    out.add(content);
  } else {
    for (const variant of enumVariants(item)) for (const f of variant.fields) out.add(f);
  }
  return [...out];
}

/**
 * حقولٌ يكتبها `enum` مسطَّح لبعض متغيّراته دون بعض، فقد تغيب عن السلك.
 *
 * الوسم نفسه يصل دائمًا، وما عداه مشروط بالمتغيّر: `produced` مع `succeeded`
 * وحدها، و`code` مع `failed` وحدها. أما `line` في وسمٍ مجاورٍ كل متغيّراته
 * تحمل حمولة فلا يغيب أبدًا — والفرق يُحسب من المتغيّرات لا يُفترض.
 */
function enumAbsentableFields(typeName: string): string[] {
  const bare = (typeName.split('::').pop() ?? typeName).trim();
  const variants = enumVariants(requireItem('enum', bare));
  const out = new Set<string>();
  for (const variant of variants) {
    for (const other of variants) {
      for (const f of other.fields) if (!variant.fields.includes(f)) out.add(f);
    }
  }
  return [...out];
}

/**
 * أسماء الحقول كما تصل السلك، مع فتح كل `flatten`.
 *
 * ‏`skipped` ما يحذفه `skip_serializing_if` عند الغياب، و`conditional` ما
 * يكتبه متغيّرٌ دون آخر من `enum` مسطَّح. الطريقان مختلفان في Rust ومتطابقان
 * عند الواجهة: كلاهما مفتاحٌ قد لا يصل، وكلاهما يستوجب `?:`.
 */
function structWireFields(name: string): {
  names: string[];
  skipped: string[];
  conditional: string[];
} {
  const item = requireItem('struct', name);
  const names: string[] = [];
  const skipped: string[] = [];
  const conditional: string[] = [];
  for (const field of structFields(item)) {
    if (field.flatten) {
      names.push(...enumWireFields(field.type));
      conditional.push(...enumAbsentableFields(field.type));
      continue;
    }
    names.push(field.name);
    if (field.skipped) skipped.push(field.name);
  }
  return { names, skipped, conditional };
}

// ── قراءة مصدر الواجهة ─────────────────────────────────────────────────

function stripTsComments(source: string): string {
  return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
}

function balancedFrom(source: string, open: number): string {
  let depth = 0;
  for (let i = open; i < source.length; i += 1) {
    if (source[i] === '{') depth += 1;
    else if (source[i] === '}') {
      depth -= 1;
      if (depth === 0) return source.slice(open + 1, i);
    }
  }
  return '';
}

/** يقسم جسم شكلٍ عند فواصل المستوى الأعلى وحدها. */
function tsTopLevelParts(body: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < body.length; i += 1) {
    const c = body[i];
    if (c === '{' || c === '(' || c === '[') depth += 1;
    else if (c === '}' || c === ')' || c === ']') depth -= 1;
    else if ((c === ';' || c === ',') && depth === 0) {
      parts.push(body.slice(start, i));
      start = i + 1;
    }
  }
  parts.push(body.slice(start));
  return parts.map((p) => p.trim()).filter((p) => p !== '');
}

type TsField = { name: string; optional: boolean; type: string };

/**
 * حقول المستوى الأعلى في جسم شكلٍ واحد.
 *
 * القسمة على العمق لا على الأسطر: `line: { dropped: number }` حقلٌ واحد اسمه
 * ‏`line`، ومُحلِّلٌ يلتقط كل `اسم:` في النصّ يخترع حقلًا اسمه `dropped` في
 * المستوى الأعلى فيقارن عقدًا لا وجود له.
 */
function objectFields(body: string): TsField[] {
  const out: TsField[] = [];
  for (const part of tsTopLevelParts(body)) {
    const matched = /^(\w+)(\?)?\s*:([\s\S]*)$/.exec(part);
    if (matched === null) continue;
    out.push({
      name: matched[1] as string,
      optional: matched[2] !== undefined,
      type: (matched[3] as string).trim(),
    });
  }
  return out;
}

/** أجسام الأشكال `{…}` في المستوى الأعلى من اتحاد. */
function unionMembers(body: string): string[] {
  const members: string[] = [];
  let depth = 0;
  let start = -1;
  for (let i = 0; i < body.length; i += 1) {
    if (body[i] === '{') {
      if (depth === 0) start = i;
      depth += 1;
    } else if (body[i] === '}') {
      depth -= 1;
      if (depth === 0 && start !== -1) members.push(body.slice(start + 1, i));
    }
  }
  return members;
}

/** نصّ الطرف الأيمن لـ `export type X = …;` كاملًا. */
function tsUnionBody(name: string): string {
  const at = IPC.indexOf(`export type ${name} =`);
  expect(at, `لم يُعثر على النوع ${name} في ipc.ts`).toBeGreaterThan(-1);
  let depth = 0;
  for (let i = at; i < IPC.length; i += 1) {
    if (IPC[i] === '{') depth += 1;
    else if (IPC[i] === '}') depth -= 1;
    else if (IPC[i] === ';' && depth === 0) return stripTsComments(IPC.slice(at, i));
  }
  return '';
}

type TsShape = { open: boolean; members: string[] };

/**
 * الشكل كما تعلنه `ipc.ts`: واجهةً واحدة أو اتحادًا من أشكال.
 *
 * الاتحاد ليس صياغة بديلة للواجهة بل معنًى أوسع — حقلٌ حاضرٌ في عضو وغائبٌ عن
 * آخر، ونوعٌ يتغيّر بتغيّر الوسم — ولذلك يعيد المُحلِّل الأعضاء لا جسمًا
 * مدموجًا: الدمج يُخفي بالضبط ما يجعل الاتحاد اتحادًا.
 */
function tsShape(name: string): TsShape {
  const header = new RegExp(String.raw`^export interface ${name}\b([^{]*)\{`, 'm').exec(IPC);
  if (header !== null) {
    return {
      // ‏`extends Record<string, unknown>` إعلانٌ صريح بأن الشكل مفتوح: تصل
      // حقولٌ لا تسمّيها الواجهة. يغيّر ما نطالب به، ولا يلغيه.
      open: /extends\s+Record<string,\s*unknown>/.test(header[1] ?? ''),
      members: [stripTsComments(balancedFrom(IPC, IPC.indexOf('{', header.index)))],
    };
  }
  const members = unionMembers(tsUnionBody(name));
  expect(members.length, `لم يُعثر على الشكل ${name} في ipc.ts`).toBeGreaterThan(0);
  return { open: false, members };
}

function tsShapeFields(name: string): string[] {
  return tsShape(name).members.flatMap((m) => objectFields(m).map((f) => f.name));
}

/**
 * هل يجوز أن يصل الشكل بلا هذا الحقل؟
 *
 * في واجهة: علامة `?` وحدها. وفي اتحاد: العلامة، أو غياب الحقل عن عضوٍ منه —
 * فكلاهما يُلزم القارئ بالفحص قبل الاستعمال، وهو المطلوب.
 */
function tsMayBeAbsent(shape: TsShape, field: string): boolean {
  return shape.members.some((member) => {
    const found = objectFields(member).find((f) => f.name === field);
    return found === undefined || found.optional;
  });
}

/** الحرفيّات النصّية المعلَنة لحقلٍ داخل شكل (اتحادٌ مضمَّن لا مُسمّى). */
function tsFieldLiterals(shapeName: string, field: string): string[] {
  const declarations = tsShape(shapeName)
    .members.flatMap((m) => objectFields(m))
    .filter((f) => f.name === field);
  expect(declarations.length, `لم يُعثر على الحقل ${field} في ${shapeName}`).toBeGreaterThan(0);
  return declarations.flatMap((f) =>
    [...f.type.matchAll(/'([a-z0-9_]+)'/g)].map((m) => m[1] as string),
  );
}

function tsStringUnion(name: string): string[] {
  return [...tsUnionBody(name).matchAll(/'([a-z0-9_]+)'/g)].map((m) => m[1] as string);
}

type TsVariant = { tag: string; fields: string[] };

/**
 * أعضاء اتحادٍ موسوم، بوسم كلٍّ وحقوله.
 *
 * يمرّ عبر `tsShape` لا عبر `tsUnionBody` مباشرة كي يبقى شكلٌ أُعيد إلى واجهة
 * واحدة مقروءًا: عضوٌ واحد بوسمٍ واحد. فرقٌ يُقرأ يُسقط الاختبار باسم
 * المتغيّر الناقص، وشكلٌ لا يُقرأ أصلًا يُسقطه باسم الملف — والأول أنفع.
 */
function tsTaggedUnion(name: string, tag: string): TsVariant[] {
  return tsShape(name).members.map((member) => {
    const fields = objectFields(member);
    return {
      tag:
        /'([a-z0-9_]+)'/.exec(fields.find((f) => f.name === tag)?.type ?? '')?.[1] ?? '',
      fields: fields.map((f) => f.name).filter((f) => f !== tag),
    };
  });
}

const sorted = (xs: string[]): string[] => [...new Set(xs)].sort();

// ── حراسة الحارس ───────────────────────────────────────────────────────

describe('محلّلات العقد نفسها', () => {
  it('يقرأ متغيّرات enum من مصدر Rust فعلًا', () => {
    const danger = enumVariants(requireItem('enum', 'Danger')).map((v) => v.wire);
    expect(danger.length, 'لم تُقرأ متغيّرات Danger').toBeGreaterThan(1);
    // شاهدٌ محدَّد: تعبيرٌ نمطيّ مكسور قد يعيد قائمةً غير فارغة لكنها خاطئة.
    expect(danger).toContain('destructive');

    const outcome = enumVariants(requireItem('enum', 'Outcome'));
    expect(outcome.length).toBeGreaterThan(1);
    expect(outcome.find((v) => v.wire === 'success')?.fields).toEqual(['produced']);
  });

  it('يقرأ حقول struct من مصدر Rust فعلًا', () => {
    const plan = structWireFields('PlanResponse').names;
    expect(plan.length, 'لم تُقرأ حقول PlanResponse').toBeGreaterThan(10);
    expect(plan).toContain('argv_display');
    expect(plan).toContain('working_directory');
  });

  it('يفتح flatten إلى حقول فعلية لا إلى اسم الحقل الحامل', () => {
    const finished = structWireFields('RunFinished').names;
    expect(finished).not.toContain('outcome');
    expect(finished).toContain('status');
    expect(finished).toContain('produced');
  });

  it('يميّز الوسم المجاور من الوسم الداخلي في قراءة الحمولة', () => {
    const output = enumVariants(requireItem('enum', 'OutputLine'));
    expect(output.length, 'لم تُقرأ متغيّرات OutputLine').toBe(3);
    // شاهدٌ محدَّد: تحت `content = "line"` تُكتب كل حمولة باسم `line`، سواء
    // كانت قيمة واحدة (`Stdout(String)`) أو بنية (`Truncated { dropped }`).
    expect(output.find((v) => v.wire === 'stdout')?.fields).toEqual(['line']);
    expect(output.find((v) => v.wire === 'truncated')?.fields).toEqual(['line']);
    // ولا يمتدّ هذا إلى الوسم الداخلي: `Outcome` بلا `content` تبقى بأسمائها.
    const outcome = enumVariants(requireItem('enum', 'Outcome'));
    expect(outcome.find((v) => v.wire === 'signalled')?.fields).toEqual(['signal']);
  });

  it('يرصد ما قد يغيب عن السلك: حذفَ serde وشرطَ المتغيّر معًا', () => {
    // حارسٌ على الحارس: لو كفّ المحلّل عن رؤية `skip_serializing_if` أو عن فتح
    // ‏`flatten` لصار فحص الاختيارية كلّه يقارن قوائم فارغة فيمرّ دائمًا.
    const entry = structWireFields('Entry');
    expect(entry.skipped, 'لم يُرصد ما يحذفه serde في Entry').toEqual(['duration_ms', 'cwd']);
    expect(sorted(entry.conditional)).toEqual(['code', 'produced', 'reason']);

    expect(structWireFields('PlanResponse').skipped, 'PlanResponse لا تحذف حقلًا').toEqual([]);
    expect(sorted(structWireFields('RunFinished').conditional)).toEqual([
      'code',
      'key',
      'produced',
      'signal',
    ]);
    // ‏`line` حاضرة مع كل متغيّرات `OutputLine`، فلا تُعدّ مما قد يغيب.
    expect(structWireFields('RunOutput').conditional).toEqual([]);
  });

  it('يقرأ الواجهات والاتحادات من ipc.ts فعلًا', () => {
    expect(tsShapeFields('PlanResponse').length).toBeGreaterThan(10);
    expect(tsShapeFields('PlanResponse')).toContain('writes_to');
    expect(tsStringUnion('Danger')).toContain('destructive');
    expect(tsTaggedUnion('Outcome', 'status').map((v) => v.tag)).toContain('signalled');
    expect(tsShape('RunFinishedEvent').open, 'RunFinishedEvent شكلٌ مفتوح').toBe(true);
    expect(tsShape('PlanResponse').open, 'PlanResponse شكلٌ مغلق').toBe(false);
  });

  it('يقرأ حقول المستوى الأعلى وحدها من أعضاء الاتحاد', () => {
    const members = tsTaggedUnion('RunOutputEvent', 'stream');
    expect(members.length, 'لم تُقرأ أعضاء RunOutputEvent').toBe(3);
    // ‏`dropped` داخل `line` لا إلى جانبه: مُحلِّلٌ لا يرى العمق يخترعه حقلًا
    // في المستوى الأعلى، فيبدو العقد مطابقًا لشيء لا يُبَثّ.
    expect(members.flatMap((m) => m.fields)).not.toContain('dropped');
    expect(members.every((m) => m.fields.includes('line'))).toBe(true);
  });

  it('يميّز الحقل الاختياري من الإلزامي في ipc.ts', () => {
    const entry = tsShape('JournalEntry');
    expect(tsMayBeAbsent(entry, 'duration_ms'), 'duration_ms معلَنة اختيارية').toBe(true);
    expect(tsMayBeAbsent(entry, 'program'), 'program معلَنة إلزامية').toBe(false);
    // وفي الاتحاد: غياب الحقل عن عضوٍ كعلامة `?` تمامًا.
    const outcome = tsShape('Outcome');
    expect(tsMayBeAbsent(outcome, 'produced'), 'produced تغيب عن أعضاء أخرى').toBe(true);
    expect(tsMayBeAbsent(outcome, 'status'), 'status وسمٌ في كل عضو').toBe(false);
  });
});

// ── المتغيّرات: أخطر ما يفترق ──────────────────────────────────────────

/**
 * متغيّرٌ يُضاف في Rust ويُنسى في الواجهة ثقبٌ صامت: تصل حالةٌ لا يعرفها أي
 * فرع، فلا يُرسم شيء أو تنهار الشاشة. ولا يوجد `tsc` ولا `cargo` يراه، لأن
 * كلًّا منهما يرى نصف العقد.
 */
describe('متغيّرات كل enum يعبر الحدّ', () => {
  const cases: Array<{ rust: string; label: string; ts: () => string[] }> = [
    { rust: 'Danger', label: 'Danger', ts: () => tsStringUnion('Danger') },
    { rust: 'Conflict', label: 'Conflict', ts: () => tsStringUnion('Conflict') },
    { rust: 'TokenRole', label: 'TokenRole', ts: () => tsStringUnion('TokenRole') },
    {
      rust: 'Category',
      label: 'Category → OperationSummary.category',
      ts: () => tsFieldLiterals('OperationSummary', 'category'),
    },
    {
      rust: 'State',
      label: 'journal::State → JournalState',
      ts: () => tsStringUnion('JournalState'),
    },
  ];

  for (const c of cases) {
    it(`يطابق ${c.label}`, () => {
      const rust = sorted(enumVariants(requireItem('enum', c.rust)).map((v) => v.wire));
      const ts = sorted(c.ts());
      expect(rust.length, `لم تُقرأ متغيّرات ${c.rust}`).toBeGreaterThan(1);
      expect(ts, `متغيّرات ${c.label} تفترق بين النواة والواجهة`).toEqual(rust);
    });
  }

  /**
   * كل `enum` موسوم يعبر الحدّ، والاتحاد الذي يصفه في `ipc.ts`.
   *
   * المطابقة على مستويين لا مستوًى واحد: أسماء المتغيّرات، **وحمولة كلٍّ منها
   * باسمها على السلك**. الاسم وحده لا يكفي — اتحادٌ يذكر `signalled` ولا يذكر
   * `signal` يمرّ فحص الأسماء ويترك الرقم غير مقروء.
   *
   * ‏`carrier` حقول البنية الحاملة حين يُسطَّح الاتحاد داخلها: `RunOutput`
   * تضيف `run_id` إلى كل صيغة خرج، وهي ليست من `OutputLine` في شيء.
   */
  const TAGGED: Array<{
    rust: string;
    ts: string;
    tag: string;
    count: number;
    carrier: string[];
  }> = [
    { rust: 'Outcome', ts: 'Outcome', tag: 'status', count: 5, carrier: [] },
    { rust: 'InputKind', ts: 'InputKind', tag: 'kind', count: 6, carrier: [] },
    { rust: 'RawValue', ts: 'RawValue', tag: 'kind', count: 3, carrier: [] },
    { rust: 'OutputLine', ts: 'RunOutputEvent', tag: 'stream', count: 3, carrier: ['run_id'] },
  ];

  for (const c of TAGGED) {
    it(`يطابق ${c.rust} ↔ ${c.ts} متغيّرًا متغيّرًا وحمولةً حمولة`, () => {
      const rust = enumVariants(requireItem('enum', c.rust));
      const ts = tsTaggedUnion(c.ts, c.tag);
      expect(rust.length, `لم تُقرأ متغيّرات ${c.rust}`).toBe(c.count);
      // المقارنة قبل عدّ الأعضاء: الفرق يسمّي المتغيّر الغائب، والعدد لا يسمّي
      // إلا رقمًا.
      expect(sorted(ts.map((v) => v.tag)), `متغيّرات ${c.rust} تفترق عن ${c.ts}`).toEqual(
        sorted(rust.map((v) => v.wire)),
      );
      expect(ts.length, `لم تُقرأ أعضاء ${c.ts}`).toBe(c.count);
      for (const variant of rust) {
        const twin = ts.find((v) => v.tag === variant.wire);
        expect(twin, `المتغيّر ${variant.wire} غائب عن ${c.ts}`).toBeDefined();
        const fields = (twin as TsVariant).fields;
        expect(
          sorted(fields.filter((f) => !c.carrier.includes(f))),
          `حمولة ${variant.wire} تفترق بين ${c.rust} و${c.ts}`,
        ).toEqual(sorted(variant.fields));
        expect(
          c.carrier.filter((f) => !fields.includes(f)),
          `العضو ${variant.wire} يُغفل حقل البنية الحاملة`,
        ).toEqual([]);
      }
    });
  }

  /**
   * الاتجاه العكسي: ما ترسله الواجهة يجب أن تستطيع النواة قراءته.
   *
   * ‏`RawValue` هو النوع الوحيد الذي يعبر الحدّ نحو الداخل، و`value.rs`
   * يشتقّ له `Deserialize` لا `Serialize`. صيغةٌ تعلنها الواجهة ولا يعرفها
   * serde ليست حقلًا فارغًا بل رفضٌ لكل الطلب: النموذج لا يُخطَّط أصلًا.
   */
  it('يطابق RawValue في اتجاه الإرسال', () => {
    const item = requireItem('enum', 'RawValue');
    expect(item.attrs.join(' '), 'RawValue يُقرأ لا يُكتب').toContain('Deserialize');
    expect(sorted(enumVariants(item).map((v) => v.wire))).toEqual(['flag', 'path', 'text']);
  });
});

// ── الحقول ─────────────────────────────────────────────────────────────

/**
 * حقولٌ تكتبها النواة ولا تسمّيها الواجهة.
 *
 * مسموحة **فقط** في الأشكال المفتوحة (`extends Record<string, unknown>`)،
 * ومجمّدة اسمًا اسمًا كي تبقى قرارًا مُراجَعًا لا صمتًا. المساواة تامّة:
 * حقلٌ جديد غير مذكور هنا يُسقط الاختبار.
 */
const UNDECLARED: Record<string, string[]> = {
  // ‏`InputSummary` تسطّح `InputKind`، والواجهة تصف ذلك النوع في اتحاد
  // ‏`InputKind` المستقل — وهو مفحوصٌ أعلاه بمتغيّراته وحمولته — بدل أن
  // تكرّر حقوله هنا. تكرارها كان سيعني وصفين لشيء واحد داخل `ipc.ts` نفسه.
  //
  // وهذا الاستثناء الوحيد الباقي: كل ما عداه يُطابَق حقلًا حقلًا.
  InputSummary: ['ext', 'kind', 'max_len'],
};

/**
 * حقولٌ قد يغيب مفتاحها عن السلك، والواجهة تعلنها كأنها حاضرة دومًا.
 *
 * لا استثناء اليوم، والخانة مُبقاة كي يبقى الاستثناء — إن لزم — قرارًا
 * مكتوبًا يُراجَع لا فحصًا يُلَيَّن. و«الغياب» طريقان يلتقيان عند الواجهة:
 * `skip_serializing_if = "Option::is_none"` الذي يحذف المفتاح نفسه لا يجعله
 * `null`، و`enum` مسطَّح يكتب `produced` مع متغيّر و`code` مع آخر. في
 * الحالين النوع الصادق `x?: T`، وبدون العلامة يصدّق `tsc` حضورًا لا يضمنه
 * السلك فتصير `=== null` مقارنةً لا تصدق أبدًا.
 */
const ABSENT_BUT_REQUIRED: Record<string, string[]> = {};

describe('حقول كل بنية تعبر الحدّ', () => {
  const cases: Array<{ rust: string; ts: string }> = [
    { rust: 'OperationSummary', ts: 'OperationSummary' },
    { rust: 'InputSummary', ts: 'InputSummary' },
    { rust: 'PlanResponse', ts: 'PlanResponse' },
    { rust: 'ExplainToken', ts: 'ExplainToken' },
    { rust: 'ToolView', ts: 'ToolView' },
    { rust: 'EstimateView', ts: 'EstimateView' },
    { rust: 'Entry', ts: 'JournalEntry' },
    { rust: 'WireError', ts: 'CoreErrorShape' },
    { rust: 'RunOutput', ts: 'RunOutputEvent' },
    { rust: 'RunFinished', ts: 'RunFinishedEvent' },
  ];

  for (const c of cases) {
    it(`يطابق ${c.rust} ↔ ${c.ts}`, () => {
      const rust = structWireFields(c.rust);
      const wire = sorted(rust.names);
      const declared = sorted(tsShapeFields(c.ts));
      expect(wire.length, `لم تُقرأ حقول ${c.rust}`).toBeGreaterThan(1);
      expect(declared.length, `لم تُقرأ حقول ${c.ts}`).toBeGreaterThan(1);

      // اتجاهٌ لا استثناء له: حقلٌ تعلنه الواجهة ولا ترسله النواة يعني قراءة
      // ‏`undefined` بينما يعد `tsc` بقيمة.
      expect(
        declared.filter((f) => !wire.includes(f)),
        `${c.ts} تعلن حقولًا لا ترسلها ${c.rust}`,
      ).toEqual([]);

      // والاتجاه الآخر: ما ترسله النواة ولا تسمّيه الواجهة.
      const undeclared = wire.filter((f) => !declared.includes(f));
      expect(undeclared, `${c.ts} لا تسمّي حقولًا ترسلها ${c.rust}`).toEqual(
        sorted(UNDECLARED[c.ts] ?? []),
      );
      if (undeclared.length > 0) {
        expect(
          tsShape(c.ts).open,
          `${c.ts} تُغفل حقولًا وهي شكلٌ مغلق: لا مكان لها في النوع أصلًا`,
        ).toBe(true);
      }
    });

    it(`يعلن اختياريًا كل ما قد يغيب في ${c.ts}`, () => {
      const rust = structWireFields(c.rust);
      const shape = tsShape(c.ts);
      const declared = new Set(tsShapeFields(c.ts));
      // ما لا تسمّيه الواجهة أصلًا يحكمه `UNDECLARED` أعلاه لا الاختيارية.
      const required = sorted([...rust.skipped, ...rust.conditional]).filter(
        (f) => declared.has(f) && !tsMayBeAbsent(shape, f),
      );
      expect(required, `${c.ts}: حقولٌ قد تغيب وهي معلَنة كأنها حاضرة دومًا`).toEqual(
        sorted(ABSENT_BUT_REQUIRED[c.ts] ?? []),
      );
    });
  }

  it('يفحص اختياريةً على حقول موجودة فعلًا، لا على قوائم فارغة', () => {
    // حارسٌ على الحارس: الفحص أعلاه يمرّ صامتًا لو صار طرفاه فارغين، فيُثبَّت
    // هنا أن لكل بنيةٍ تحمل حقولًا قابلة للغياب حقولٌ مرصودة ومعلَنة معًا.
    const entry = structWireFields('Entry');
    expect(sorted([...entry.skipped, ...entry.conditional])).toEqual([
      'code',
      'cwd',
      'duration_ms',
      'produced',
      'reason',
    ]);
    const shape = tsShape('JournalEntry');
    for (const f of ['duration_ms', 'cwd', 'produced', 'reason', 'code']) {
      expect(tsShapeFields('JournalEntry'), `JournalEntry لا تسمّي ${f}`).toContain(f);
      expect(tsMayBeAbsent(shape, f), `${f} معلَنة إلزامية في JournalEntry`).toBe(true);
    }
    expect(structWireFields('RunFinished').conditional).toContain('signal');
    expect(tsShapeFields('RunFinishedEvent'), 'RunFinishedEvent لا تسمّي signal').toContain(
      'signal',
    );
  });
});
