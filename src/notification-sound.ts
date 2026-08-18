/**
 * نغمة اكتمالٍ ناجح — تُصنَّع لا تُحمَّل.
 *
 * ## لماذا لا صوت نظام macOS الحقيقي
 *
 * الواجهة لا تملك صلاحية قراءة `/System/Library/Sounds` ولا أمر IPC يعيد
 * محتوى ملفٍّ منه — وإضافة أمرٍ عاشر لأجل نغمةٍ كانت ستوسّع سطح الهجوم من
 * أجل بايتاتٍ لا تستحقّ صلاحيةً جديدة، وتكسر الاختبار الذي يثبّت سطح الأوامر
 * تسعةً (`security.rs`). ولا صوتٌ مرفق كملفّ: ملفّاتٌ صوتية حقيقية تحتاج
 * مصدرًا وترخيصًا، وهذا المنتج لا ينسخ من macOS شيئًا لم يوثّق مصدره —
 * الأيقونة نفسها مثالٌ سابق على هذا الحذر.
 *
 * ‏Web Audio API تصنع نغمةً محليةً بلا شبكة ولا ملفٍّ مرفق ولا صلاحيةٍ جديدة:
 * نغمتان قصيرتان صاعدتان، أقرب ما تكون لطابع «Tink» — لا نسخة منه، توليدٌ
 * مستقلّ بنفس الروح.
 *
 * ## لماذا لا سياقًا واحدًا يعيش طوال الجلسة
 *
 * هذه النغمة تُطلق مرّةً كل تشغيلٍ ناجح — نادرًا، لا في حلقة. سياقُ صوتٍ
 * جديد لكل نغمة أبسط من سياقٍ واحد يُدار طوال حياة التطبيق (لا حالة `closed`/
 * `suspended` يُتحقَّق منها)، وأرخص بما يكفي عند هذا التكرار.
 */

/** يبحث عن مُنشئ `AudioContext` في نافذة المتصفّح، بلا افتراض وجوده. */
function audioContextConstructor(): typeof AudioContext | undefined {
  if (typeof window === 'undefined') return undefined;
  const w = window as typeof window & { webkitAudioContext?: typeof AudioContext };
  return w.AudioContext ?? w.webkitAudioContext;
}

/**
 * يشغّل نغمة الاكتمال. لا ترمي أبدًا — بيئةٌ بلا `AudioContext` (اختبارُ وحدة،
 * متصفّحٌ قديم) تعني أن لا نغمة تُشغَّل، لا خطأً يُقاطع بقيّة المعالجة.
 */
export function playSuccessChime(): void {
  const Ctor = audioContextConstructor();
  if (!Ctor) return;

  try {
    const context = new Ctor();
    const now = context.currentTime;
    // نغمتان: الأولى قصيرة خافتة تمهّد، والثانية أعلى نبرةً وأطول قليلًا —
    // نفس بنية «سؤالٍ أُجيب» التي تُقرأ إيجابية بلا حاجةٍ لكلمة.
    const notes: ReadonlyArray<readonly [frequency: number, start: number, duration: number]> = [
      [880, 0, 0.09],
      [1318.5, 0.07, 0.16],
    ];

    for (const [frequency, start, duration] of notes) {
      const oscillator = context.createOscillator();
      const gain = context.createGain();
      oscillator.type = 'sine';
      oscillator.frequency.value = frequency;

      // مغلّفٌ سريع الصعود بطيء الهبوط — نقرةٌ لا صفير: صعودٌ خطّي في 12ms
      // يمنع طقّة بدء مفاجئة، وهبوطٌ أُسّي يُقلَّد به تلاشي جرسٍ حقيقي.
      gain.gain.setValueAtTime(0, now + start);
      gain.gain.linearRampToValueAtTime(0.16, now + start + 0.012);
      gain.gain.exponentialRampToValueAtTime(0.0001, now + start + duration);

      oscillator.connect(gain);
      gain.connect(context.destination);
      oscillator.start(now + start);
      oscillator.stop(now + start + duration + 0.02);
    }

    // الإغلاق بعد انتهاء أطول نغمة بمهلةٍ صغيرة — لا حاجة له بعدها، وتركه
    // مفتوحًا يراكم سياقات صوتٍ صامتة عبر تشغيلاتٍ كثيرة في نفس الجلسة.
    const last = notes.reduce((max, [, start, duration]) => Math.max(max, start + duration), 0);
    window.setTimeout(
      () => {
        void context.close().catch(() => {});
      },
      Math.ceil((last + 0.05) * 1000),
    );
  } catch {
    // فشل تشغيل نغمةٍ ليس عطلًا يستحقّ مقاطعة التشغيل الذي نجح للتوّ.
  }
}
