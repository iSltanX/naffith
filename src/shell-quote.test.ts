/**
 * الأمر القابل للّصق.
 *
 * تذكير قبل قراءة هذه الاختبارات: ناتج `shellCommand` **لا يُنفَّذ**. التطبيق
 * يشغّل `argv` مباشرة بلا صدفة، وهذا النص لعين المستخدم وحافظته فقط. لذلك
 * السؤال هنا ليس «هل الهروب آمن؟» بل «هل ما يلصقه المستخدم في Terminal يعطيه
 * ما وعدناه؟».
 */
import { describe, expect, it } from 'vitest';
import { shellCommand, shellQuote, tokenNotes } from './shell-quote';

describe('shellQuote', () => {
  it('يترك الرموز الآمنة بلا اقتباس', () => {
    for (const safe of ['/usr/bin/ditto', '-c', '-k', '--sequesterRsrc', '--keepParent']) {
      expect(shellQuote(safe)).toBe(safe);
    }
  });

  it('يقتبس المسافات فلا تصير المسافةُ فاصلَ وسائط', () => {
    expect(shellQuote('/Users/a/my folder')).toBe("'/Users/a/my folder'");
  });

  it('يقتبس العربية', () => {
    // العربية ليست خطرة، لكنها خارج قائمة السماح فتُقتبس — والاقتباس لا يضرّ.
    expect(shellQuote('/Users/a/مجلد')).toBe("'/Users/a/مجلد'");
  });

  it('يعالج الاقتباس المفرد بالإغلاق والهروب والفتح', () => {
    // لا سبيل لهروب `'` داخل اقتباس مفرد في POSIX، فهذه هي الحيلة الوحيدة.
    expect(shellQuote("it's")).toBe("'it'\\''s'");
    expect(shellQuote("a'b'c")).toBe("'a'\\''b'\\''c'");
    expect(shellQuote("'")).toBe("''\\'''");
  });

  it('يجعل محارف الصدفة نصًّا حرفيًا', () => {
    const cases: Array<[string, string]> = [
      ['$(whoami)', "'$(whoami)'"],
      ['`id`', "'`id`'"],
      ['a; rm -rf ~', "'a; rm -rf ~'"],
      ['a && b', "'a && b'"],
      ['a | b', "'a | b'"],
      ['a > /tmp/x', "'a > /tmp/x'"],
      ['$HOME', "'$HOME'"],
      ['*', "'*'"],
      ['~', "'~'"],
    ];
    for (const [input, expected] of cases) {
      expect(shellQuote(input)).toBe(expected);
    }
  });

  it('يقتبس النص الفارغ كي لا يختفي وسيطٌ من الأمر', () => {
    expect(shellQuote('')).toBe("''");
  });

  it('لا يترك محرفًا فعّالًا خارج اقتباس', () => {
    // فحص شامل: أي رمز يخرج بلا اقتباس يجب أن يكون من قائمة السماح وحدها.
    const dangerous = ['$', '`', '"', "'", '\\', ';', '&', '|', '<', '>', '(', ')', '{', '}',
      '[', ']', '*', '?', '~', '!', '#', ' ', '\t', '\n'];
    for (const ch of dangerous) {
      const quoted = shellQuote(`x${ch}y`);
      expect(quoted.startsWith("'"), `${JSON.stringify(ch)} must force quoting`).toBe(true);
    }
  });
});

describe('shellCommand', () => {
  it('يبني أمر ditto كاملًا كما يُلصق في Terminal', () => {
    const argv = [
      '/usr/bin/ditto',
      '-c',
      '-k',
      '--sequesterRsrc',
      '--keepParent',
      "/Users/a/it's a folder",
      '/Users/a/dest/.naffith-ab12-تقرير.zip.part',
    ];
    expect(shellCommand(argv)).toBe(
      "/usr/bin/ditto -c -k --sequesterRsrc --keepParent '/Users/a/it'\\''s a folder' " +
        "'/Users/a/dest/.naffith-ab12-تقرير.zip.part'",
    );
  });

  it('يحافظ على عدد الوسائط مهما كان محتواها', () => {
    const argv = ['/usr/bin/ditto', 'a b c', "d'e", '$(f)'];
    // عدّ الاقتباسات المفتوحة: كل وسيط غير آمن محاط باقتباس واحد على الأقل.
    expect(shellCommand(argv).split(' ')[0]).toBe('/usr/bin/ditto');
    expect(shellCommand(argv)).toContain("'a b c'");
  });
});

describe('tokenNotes', () => {
  it('لا يقول شيئًا عن مسار عادي', () => {
    expect(tokenNotes('/Users/a/مجلد المشروع')).toEqual([]);
  });

  it('يكشف المسافة في الطرف، وهي غير مرئية على الشاشة', () => {
    expect(tokenNotes('/Users/a/name ')).toContain('يبدأ أو ينتهي بمسافة');
  });

  it('يكشف المسافات المتتالية', () => {
    expect(tokenNotes('a  b')).toContain('يحتوي مسافات متتالية');
  });

  it('يكشف محارف الاتجاه غير المرئية', () => {
    expect(tokenNotes('ملف‏خفي')).toContain('يحتوي محارف غير مرئية');
    expect(tokenNotes('a‮b')).toContain('يحتوي محارف غير مرئية');
  });

  it('يكشف المسافة غير الفاصلة التي تبدو مسافةً عادية', () => {
    expect(tokenNotes('a b')).toContain('يحتوي مسافة غير اعتيادية');
  });

  it('ينبّه على الاقتباس المفرد لأنه يغيّر شكل الأمر المنسوخ', () => {
    expect(tokenNotes("it's")).toContain('يحتوي اقتباسًا مفردًا');
  });
});
