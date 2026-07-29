/**
 * قائمة العمليات — الشاشة الأولى، وسؤالها واحد: «ماذا تريد أن تنفّذ؟».
 *
 * ## ليس في هذا الملف اسمُ عمليةٍ واحد
 *
 * البطاقات تصل جاهزةً من `list_operations()` بعد أن يمرّرها المنسّق على
 * `toCards`، فلا يبقى للشاشة إلا أن ترسم ما وصل: عنوانٌ من مفتاح، ووصفٌ من
 * مفتاح، وأيقونةٌ اشتُقّت من الفئة، وتوفّرٌ اشتُقّ من شكل المدخلات.
 *
 * البديل المرفوض خريطةٌ هنا من `op_id` إلى عنوانٍ وأيقونةٍ وترتيب. هي أسرع
 * كتابةً وأدقّ تحكّمًا في المظهر، لكنها فهرسٌ ثانٍ يتقادم صامتًا: عمليةٌ تُضاف
 * في النواة تظهر بلا اسم، وأخرى تُحذف تبقى معروضة، ولا شيء يكسر حتى يشتكي
 * مستخدم. الاختبار المرافق يحقن خمس عملياتٍ لا وجود لها في هذا البناء ويطالب
 * بظهورها كاملةً، ويقرأ هذا المصدر نفسه ليتأكّد أنه خلا من أي معرّف عملية.
 *
 * ## لماذا التخطيط شبكةٌ تتدفّق لا صفٌّ مرسوم بيد
 *
 * عدد العمليات مجهولٌ بالتعريف — هو ما تُدرجه النواة اليوم لا ما رُسم أمس.
 * فالتخطيط يجب أن يصحّ عند بطاقةٍ واحدة وعند اثنتي عشرة دون تدخّل، ولهذا
 * `auto-fill` لا `auto-fit` في العرض: الثانية تمطّ البطاقة الوحيدة على عرض
 * النافذة كلها فتبدو لوحةً لا خيارًا.
 */
import { useId } from 'react';
import type { CoreErrorShape } from './ipc';
import type { OperationCard } from './operations';
import { errorText, t } from './i18n';
import './operations-list.css';

/**
 * حالة الفهرس كما يراها المنسّق.
 *
 * `failed` تحمل خطأ النواة كاملًا لا رايةً منطقية: الرسالة العامة تقول إن
 * الفهرس لم يُقرأ، والمفتاح القادم من النواة يقول لماذا. حذفُه كان يجعل كل
 * أعطال البدء تبدو واحدة.
 */
export type OperationsState =
  | { status: 'loading' }
  | { status: 'ready'; cards: OperationCard[] }
  | { status: 'failed'; error: CoreErrorShape };

interface Props {
  state: OperationsState;
  onSelect: (opId: string) => void;
  onRetry: () => void;
  onOpenLog: () => void;
  onOpenSettings: () => void;
}

/**
 * بطاقة عملية.
 *
 * العنصر زرّ حقيقي لا `div` عليه `onClick`: التبويب والمسافة وEnter تأتي من
 * المتصفّح، ولا يحتاج أحدٌ إلى إعادة اختراع تفعيلٍ بلوحة المفاتيح.
 *
 * وغير المدعومة تُعطَّل بـ`disabled` لا بـ`aria-disabled`: الثانية تُبقي الزرّ
 * قابلًا للنقر ثم تعتمد على `if` في المعالج كي لا يحدث شيء — تعطيلٌ بالنيّة
 * لا بالبنية، وسطرٌ واحد يسقط منه يفتح شاشةً لا تعرف الواجهة رسمها. ثمن
 * `disabled` أن الزرّ يخرج من ترتيب التبويب فلا يُقرأ سببُ المنع منه، ولذلك
 * السبب مكتوبٌ نصًّا ظاهرًا **خارج** الزرّ لا في تلميحٍ لا يبلغه إلا الفأرة.
 */
function OperationTile({
  card,
  onSelect,
}: {
  card: OperationCard;
  onSelect: (opId: string) => void;
}) {
  const uid = useId();
  const whyId = `${uid}-why`;
  const unsupported = card.availability.state !== 'available';

  return (
    <li className="ops__cell">
      <button
        type="button"
        className={`card op-card ${unsupported ? 'op-card--off' : 'card--interactive'}`}
        disabled={unsupported}
        aria-describedby={unsupported ? whyId : undefined}
        onClick={() => onSelect(card.id)}
      >
        <span className="op-card__icon" aria-hidden="true">
          {/* الأيقونة معرّفٌ في لوحة الرموز جاء مع البطاقة. أي خريطةٍ هنا من
              العملية إلى رمزٍ كانت ستعيد الفهرس إلى الواجهة من بابٍ خلفي. */}
          <svg viewBox="0 0 24 24">
            <use href={card.icon} />
          </svg>
        </span>

        <span className="op-card__text">
          <span className="t-card-title op-card__title">{t(card.titleKey)}</span>
          <span className="t-body-sec op-card__desc">{t(card.descriptionKey)}</span>
          {unsupported && (
            <span className="chip chip--neutral op-card__chip">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <use href="#i-info" />
              </svg>
              {t('ops.unavailable')}
            </span>
          )}
        </span>

        {!unsupported && (
          <span className="op-card__go" aria-hidden="true">
            {/* السهم يشير إلى جهة المضيّ، فيَنعكس مع اتجاه الصفحة. */}
            <svg viewBox="0 0 24 24" data-directional>
              <use href="#i-chevron" />
            </svg>
          </span>
        )}
      </button>

      {unsupported && (
        <p id={whyId} className="t-caption op-card__why">
          {t('ops.unavailable.why')}
        </p>
      )}
    </li>
  );
}

/**
 * عدد المتاح.
 *
 * يُحسب من البطاقات لا يُمرَّر: أي رقمٍ يأتي من مكانٍ آخر يمكن أن يخالف ما
 * تراه العين. والرقم يبقى في `.num` كي لا ينشقّ عن جملته في نصٍّ من اليمين.
 */
function CountLine({ cards }: { cards: OperationCard[] }) {
  const available = cards.filter((c) => c.availability.state === 'available').length;
  return (
    <p className="t-caption ops__count">
      {available === 1 ? (
        t('ops.count.one')
      ) : (
        <>
          <span className="num">{available}</span> {t('ops.count.many')}
        </>
      )}
    </p>
  );
}

export default function OperationsList(props: Props): JSX.Element {
  const { state, onSelect, onRetry, onOpenLog, onOpenSettings } = props;
  const ready = state.status === 'ready' ? state.cards : null;

  return (
    <section className="ops" aria-labelledby="ops-heading">
      <header className="ops__head">
        <div className="ops__intro">
          {/* `h1` للتطبيق كله في الغلاف، فترويسة الشاشة `h2`. حجمها من الصنف
              لا من مستواها: المستوى بنيةٌ للقارئ، والحجم قرارٌ بصري. */}
          <h2 id="ops-heading" className="t-page-title">
            {t('ops.heading')}
          </h2>
          <p className="t-body-sec ops__sub">{t('ops.subheading')}</p>
        </div>

        {/* السجلّ والإعدادات حاضران لا منافسان: أزرارٌ شبحية في طرف الترويسة،
            بعد العمليات في ترتيب القراءة وقبلها في متناول اليد. */}
        <div className="ops__quiet">
          <button type="button" className="btn btn--ghost btn--sm" onClick={onOpenLog}>
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-history" />
            </svg>
            {t('nav.log')}
          </button>
          <button type="button" className="btn btn--ghost btn--sm" onClick={onOpenSettings}>
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-settings" />
            </svg>
            {t('nav.settings')}
          </button>
        </div>
      </header>

      {/* منطقةٌ حيّة **ثابتة** يتبدّل ما بداخلها. لو كانت كل حالةٍ منطقةً حيّة
          تُركَّب عند ظهورها لضاع الإعلان عن نصفها، ولو ضُمّت الشبكة إليها
          لأُعلنت البطاقات كلها في كل تحميل. */}
      <div className="ops__live" role="status" aria-live="polite">
        {state.status === 'loading' && (
          <div className="ops__state">
            <span className="spinner" aria-hidden="true" />
            <p className="t-body-sec">{t('ops.loading')}</p>
          </div>
        )}

        {ready && ready.length === 0 && (
          <div className="ops__state">
            <svg viewBox="0 0 24 24" aria-hidden="true" className="ops__state-icon">
              <use href="#i-info" />
            </svg>
            <p className="t-body-sec">{t('ops.empty')}</p>
          </div>
        )}

        {ready && ready.length > 0 && <CountLine cards={ready} />}
      </div>

      {/* العطل `alert` لا `status`: قائمةٌ لم تُقرأ توقف المستخدم، ولا تُذكر
          بين ما يمرّ. وخارج المنطقة الحيّة أعلاه كي لا يُعلَن مرّتين. */}
      {state.status === 'failed' && (
        <div className="ops__state ops__state--bad" role="alert">
          <span className="chip chip--danger">
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-error" />
            </svg>
            {t('state.failed')}
          </span>
          <p className="t-body">{t('ops.failed')}</p>
          {/* سببُ النواة كما أعلنته. الرسالة العامة تقول «لم تُقرأ»، وهذا
              السطر يقول «لماذا» — وبدونه تتشابه كل أعطال البدء. */}
          <p className="t-caption">{errorText(state.error.key, state.error.detail)}</p>
          <button type="button" className="btn btn--quiet" onClick={onRetry}>
            {t('ops.retry')}
          </button>
        </div>
      )}

      {ready && ready.length > 0 && (
        <ul className="ops__grid">
          {ready.map((card) => (
            <OperationTile key={card.id} card={card} onSelect={onSelect} />
          ))}
        </ul>
      )}
    </section>
  );
}
