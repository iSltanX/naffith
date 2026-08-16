/**
 * مجرى التشغيل — ما طبعته الأداة نفسها.
 *
 * ## العطل الذي وُلد منه هذا الملف
 *
 * النواة تبثّ `run://output` سطرًا سطرًا: `stdout` و`stderr` مفصولين، وعلامةَ
 * قصٍّ عند بلوغ سقفها (‏`MAX_OUTPUT_LINES` في `executor.rs`). وكان الجسر
 * (`onRunOutput` في `ipc.ts`) معرَّفًا ومحروسًا بعقد السلك — **ولا مستدعي له في
 * الواجهة كلّها**. أي أن التطبيق يشغّل أداةً على قرص المستخدم ثم يُهمل كل ما
 * تقوله، ويعرض عند الفشل «لم تكتمل العملية» بلا سببٍ واحد.
 *
 * وهذا نقيض ما يبيعه المنتج: «افعلْ ما تريد، وافهمْ ما فعلت».
 *
 * ## القواعد الثلاث
 *
 * 1. **الخرج نصُّ الأداة لا نصُّنا.** لا يُترجَم ولا يُصاغ ولا يُلخَّص. يُعرض على
 *    سطح «سَطْر» بخطّه الأحادي واتجاهه LTR، لأنه مادّةٌ تقنية لا جملةٌ عربية.
 * 2. **`stderr` يُميَّز ولا يُخفى.** أداةٌ ناجحة قد تكتب في `stderr` تحذيرًا،
 *    وأداةٌ فاشلة تكتب فيه السبب. دمجُ المجريين يخفي أيّهما قال ماذا.
 * 3. **القصُّ يُعلَن.** مجرًى مقصوصٌ بلا إعلان يجعل المستخدم يبني حكمه على
 *    نصفِ ما قالته الأداة وهو يحسبه كلَّه.
 *
 * ## ولماذا يعيش هنا لا في «نَفِّذ»
 *
 * «نَفِّذ» السطح الهادئ: حقولٌ وملخّصٌ بلغة المستخدم وفعلٌ وحالة. و«سَطْر» الطبقة
 * التقنية المكشوفة: الأمر، ووسائطه، وسياسته. وخرجُ الأداة الخام من مادّة الثانية
 * قطعًا. وبهذا يرث المجرى سلوكَ اللوحة المستقرّ: مطويّةٌ حتى يوجد محتوى، وتتوسّع
 * حين يوجد، وتفاصيلها قابلة للطيّ.
 */
import { useEffect, useState } from 'react';
import type { RunOutputEvent } from './ipc';
import { t } from './i18n';

/** ما تعرضه الشاشة. `Phase` في `app.tsx` هو المصدر، ويُمرَّر مقروءًا فقط. */
export type StreamPhase = 'idle' | 'planning' | 'running' | 'cancelling' | 'finished';

/**
 * أقصى ما يُحتفظ به من أسطر في الواجهة.
 *
 * سقف النواة ‎5000‎ سطر، ورسمُ خمسة آلاف صفٍّ في DOM يجعل الشاشة تتلجلج على
 * أداةٍ مسهبة. والمحفوظ هو **آخر** ما وصل لا أوّله: مجرًى يُقرأ للتشخيص يُقرأ من
 * ذيله — آخر ما قالته الأداة قبل أن تتوقّف هو ما يشرح توقّفها.
 *
 * والإسقاط يُعلَن كما يُعلَن قصُّ النواة، وبصياغةٍ أخرى لأنه شيءٌ آخر: ذاك «لم
 * تُبَثّ»، وهذا «بُثّت ولم تُحفظ».
 */
export const MAX_KEPT_LINES = 1_000;

/** سطرٌ محفوظ: حمولة الحدث مع ترقيمٍ ثابت يصلح مفتاحًا للرسم. */
export interface StreamLine {
  seq: number;
  event: RunOutputEvent;
}

/**
 * يضمّ حدثًا جديدًا إلى ما حُفظ، ويقصّ من الأوّل عند بلوغ السقف.
 *
 * مفصولةٌ عن المكوّن ومصدَّرة كي تُختبر وحدها: منطقُ الاحتفاظ هو ما يمكن أن يُخطئ
 * صامتًا (ينمو بلا حدّ، أو يقصّ الذيل بدل الرأس)، واختبارُه عبر الرسم يخفيه بين
 * ثلاثين صفًّا.
 */
export function appendLine(kept: readonly StreamLine[], event: RunOutputEvent): StreamLine[] {
  const seq = (kept[kept.length - 1]?.seq ?? -1) + 1;
  const next = [...kept, { seq, event }];
  return next.length > MAX_KEPT_LINES ? next.slice(next.length - MAX_KEPT_LINES) : next;
}

/** عدد ما أُسقط من أوّل المجرى: أوّل رقمٍ محفوظ هو ما يقيسه. */
export function droppedCount(kept: readonly StreamLine[]): number {
  return kept[0]?.seq ?? 0;
}

/**
 * هل يُفتح القسم من تلقائه؟
 *
 * أثناء التشغيل: نعم — المجرى هو الشيء الوحيد الذي يتحرّك في الشاشة، وطيُّه
 * يجعل التطبيق يبدو واقفًا. وبعد فشلٍ أو إشارة: نعم — هناك يُبحث عن السبب.
 * وبعد نجاح: لا — الأداة الناجحة صامتة عادةً، وقسمٌ مفتوح على «لم تطبع شيئًا»
 * ضجيج.
 */
export function shouldOpen(phase: StreamPhase, status: string | undefined): boolean {
  if (phase === 'running' || phase === 'cancelling') return true;
  return phase === 'finished' && status !== undefined && status !== 'success';
}

export default function RunStream({
  lines,
  phase,
  status,
}: {
  lines: readonly StreamLine[];
  phase: StreamPhase;
  /**
   * حالة الناتج، أو `undefined` قبل انتهاء التشغيل.
   *
   * `| undefined` صريحة لا `?:` وحدها: `exactOptionalPropertyTypes` يفرّق بين
   * «الخاصّية غائبة» و«حاضرةٌ بقيمة `undefined`»، والغلاف يمرّر `outcome?.status`
   * فيمرّر الثانية.
   */
  status?: string | undefined;
}) {
  const auto = shouldOpen(phase, status);

  /**
   * الفتح حالةٌ يملكها المكوّن، لا خاصّيةٌ محسوبة في كل رسم.
   *
   * `<details open={auto}>` وحدها تعيد فرض القيمة عند كل إعادة رسم — وسطرٌ جديد
   * يصل كل بضع مئات من الميلي ثانية يعني إعادة رسم — فينفتح القسم في وجه من
   * أغلقه. والأثر يُشعل الفتح عند تبدّل `auto` وحده، فيبقى إغلاق المستخدم
   * محترمًا حتى يتغيّر الطور فعلًا.
   */
  const [open, setOpen] = useState(auto);
  useEffect(() => {
    if (auto) setOpen(true);
  }, [auto]);

  // لا مجرى قبل التشغيل: الخطة ليست تشغيلًا، وقسمٌ فارغ يَعِد بما لم يبدأ.
  if (phase === 'idle' || phase === 'planning') return null;

  const dropped = droppedCount(lines);
  const running = phase === 'running' || phase === 'cancelling';

  return (
    <details
      className="stream"
      open={open}
      onToggle={(e) => setOpen(e.currentTarget.open)}
    >
      <summary className="t-label satr__section satr__section--fold">
        {t('stream.heading')}
        {/* العدّاد في الوسم: يُقرأ وهو مطويّ، فيُعرف أن هناك ما يُفتح. */}
        <span className="t-caption stream__count">
          <bdi className="num">{lines.length}</bdi> {t('stream.lines')}
        </span>
        <svg className="arg__caret" viewBox="0 0 24 24" aria-hidden="true">
          <use href="#i-chevron-down" />
        </svg>
      </summary>

      <div className="stream__body">
        {dropped > 0 && (
          <p className="t-caption stream__notice">
            {t('stream.dropped')} <bdi className="num">{dropped}</bdi>
          </p>
        )}

        {lines.length === 0 ? (
          <p className="t-caption stream__empty">
            {running ? t('stream.waiting') : t('stream.silent')}
          </p>
        ) : (
          /* منطقةٌ حيّة مهذّبة: الأسطر تصل تباعًا، وإعلانُ كل سطر يقاطع من يقرأ
             بالصوت بلا انتهاء. `polite` تجعل قارئ الشاشة يقول ما وصل عند أوّل
             صمت، وهو السلوك الصحيح لمجرًى لا لخبر. */
          <ol className="stream__lines" aria-live="polite">
            {lines.map(({ seq, event }) => (
              <li key={seq} className={`stream__line stream__line--${event.stream}`}>
                {event.stream === 'truncated' ? (
                  <span className="t-caption stream__notice">
                    {t('stream.truncated')}{' '}
                    <bdi className="num">{event.line.dropped}</bdi>
                  </span>
                ) : (
                  <>
                    <span className="stream__tag t-caption" aria-hidden="true">
                      {t(event.stream === 'stderr' ? 'stream.stderr' : 'stream.stdout')}
                    </span>
                    {/* نصُّ الأداة كما هو: خطٌّ أحادي، اتجاهٌ LTR، ومسافاته
                        محفوظة. و`overflow-wrap: normal` في الأنماط كي لا يُشقّ
                        مسارٌ في منتصفه — القاعدة نفسها التي يتبعها الأمر. */}
                    <code className="stream__text">{event.line}</code>
                  </>
                )}
              </li>
            ))}
          </ol>
        )}
      </div>
    </details>
  );
}
