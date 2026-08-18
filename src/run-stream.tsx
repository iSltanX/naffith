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

export type StreamPresentation =
  | 'waiting'
  | 'stdout'
  | 'stderr'
  | 'silent'
  | 'truncated'
  | 'dropped';

/** Page 14 Run/Stream state is selected from typed events, never parsed prose. */
export function streamPresentation(
  lines: readonly StreamLine[],
  phase: StreamPhase,
): StreamPresentation {
  if (droppedCount(lines) > 0 || lines.some(({ event }) => event.stream === 'omitted')) {
    return 'dropped';
  }
  if (lines.some(({ event }) => event.stream === 'truncated')) return 'truncated';
  if (lines.length === 0) {
    return phase === 'running' || phase === 'cancelling' ? 'waiting' : 'silent';
  }
  if (lines.some(({ event }) => event.stream === 'stderr')) return 'stderr';
  return 'stdout';
}

export default function RunStream({
  lines,
  phase,
}: {
  lines: readonly StreamLine[];
  phase: StreamPhase;
}) {
  // لا مجرى قبل التشغيل: الخطة ليست تشغيلًا، وقسمٌ فارغ يَعِد بما لم يبدأ.
  if (phase === 'idle' || phase === 'planning') return null;

  const presentation = streamPresentation(lines, phase);
  const headerTitle = t(`stream.state.${presentation}.title`);
  const headerMeta = t(`stream.state.${presentation}.meta`);
  // ‏`dropped` هنا أيضًا: الإسقاط يعني أن **بعض** الأسطر ضاعت — من مقدّمة ما
  // حفظته الواجهة، أو بعلامة `omitted` وسط المجرى — لا أن كل ما وصل ضاع.
  // إخفاء الذيل المحفوظ بالكامل عند أي إسقاط كان يعني أن أطول التشغيلات
  // إخراجًا (‏`cargo build`، `npm test`) — وهي أكثرها حاجةً إلى أن يُقرأ خرجها
  // — تعرض لا شيء إطلاقًا.
  const showLines =
    presentation === 'stdout' ||
    presentation === 'stderr' ||
    presentation === 'truncated' ||
    presentation === 'dropped';

  return (
    <section
      className={`stream stream--${presentation}`}
      aria-label={headerTitle}
    >
      <div className="stream__header">
        <span className="stream__heading">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-terminal" />
          </svg>
          {headerTitle}
        </span>
        <span className="t-caption stream__count">{headerMeta}</span>
      </div>

      <div className="stream__body">
        {presentation === 'waiting' ? (
            <div className="stream__waiting">
              <p className="stream__waiting-body">{t('stream.state.waiting.body')}</p>
              <p className="t-caption stream__waiting-state">
                <span className="stream__waiting-dot" aria-hidden="true" />
                {t('stream.state.waiting.footer')}
              </p>
            </div>
        ) : presentation === 'silent' ? (
          <p className="stream__state-copy">{t('stream.state.silent.body')}</p>
        ) : showLines ? (
          <>
            {/* الإسقاط يفسَّر فوق الذيل المحفوظ، لا بدلًا منه — انظر توثيق
                `showLines` أعلاه. */}
            {presentation === 'dropped' && (
              <p className="stream__state-copy">{t('stream.state.dropped.body')}</p>
            )}
            {/* المجرى سجلٌّ قابل للمراجعة لا منطقةٌ حيّة عملاقة. حالة التشغيل
                الموجزة تُعلنها `RunningState`؛ أمّا إعلان كل سطرٍ يصل فيقاطع قارئ
                الشاشة بلا انتهاء، خصوصًا مع أوامر البحث والتقارير الطويلة. */}
            <ol className="stream__lines">
              {lines.map(({ seq, event }) => (
                <li key={seq} className={`stream__line stream__line--${event.stream}`}>
                  {event.stream === 'truncated' || event.stream === 'omitted' ? (
                    <span className="t-caption stream__notice">
                      {t(event.stream === 'omitted' ? 'stream.omitted' : 'stream.truncated')}{' '}
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
          </>
        ) : null}

        {(presentation === 'truncated' || presentation === 'dropped') && (
          <p className="t-caption stream__footer">
            {t(`stream.state.${presentation}.footer`)}
          </p>
        )}
      </div>
    </section>
  );
}
