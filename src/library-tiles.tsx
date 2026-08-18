/**
 * البطاقات المشتركة بين شاشة الفئات وشاشة القسم.
 *
 * ملفٌّ مستقلّ لا نسختان: البطاقة نفسها تظهر في أربعة مواضع — الأقسام، نتائج
 * البحث، المفضّلة، المستخدَمة حديثًا، وقائمة القسم — وكتابتها في كلٍّ منها
 * كانت تعني أن إصلاح تعطيلٍ في موضعٍ يُنسى في الأربعة الباقية.
 *
 * ولا يوجد في هذا الملف معرّف قسمٍ ولا عملية: كل ما يُعرض جاء في البطاقة.
 */
import { useId } from 'react';
import type { CategoryCard, OperationCard } from './library';
import { findCategory } from './library';
import { t } from './i18n';

/**
 * سببُ تعذّر العملية، مصوغًا.
 *
 * السبب يُكتب نصًّا **ظاهرًا خارج الزرّ** لا في تلميحٍ لا تبلغه إلا الفأرة:
 * الزرّ المعطَّل يخرج من ترتيب التبويب، فلا يُقرأ منه شيء. واسم الأداة الناقصة
 * جزءٌ من الرسالة لأنه الشيء الوحيد الذي يستطيع المستخدم أن يفعل حياله شيئًا.
 */
function unavailableReason(card: OperationCard): string {
  const a = card.availability;
  if (a.state === 'tool_missing') return `${t('ops.unavailable.tool')} ⁦${a.tool}⁩`;
  if (a.state === 'unsupported') return t('ops.unavailable.why');
  return '';
}

/**
 * بطاقة عملية.
 *
 * العنصر زرّ حقيقي لا `div` عليه `onClick`: التبويب والمسافة وEnter تأتي من
 * المتصفّح، ولا يحتاج أحدٌ إلى إعادة اختراع تفعيلٍ بلوحة المفاتيح.
 *
 * وغير المتاحة تُعطَّل بـ`disabled` لا بـ`aria-disabled`: الثانية تُبقي الزرّ
 * قابلًا للنقر ثم تعتمد على `if` في المعالج كي لا يحدث شيء — تعطيلٌ بالنيّة
 * لا بالبنية، وسطرٌ واحد يسقط منه يفتح شاشةً لا تعرف الواجهة رسمها.
 *
 * ## زرّ التفضيل خارج البطاقة لا داخلها
 *
 * زرٌّ داخل زرّ ليس HTML صالحًا، ويجعل النقر على النجمة يفتح العملية أحيانًا
 * حسب أين وقعت الضغطة. فهو أخٌ للبطاقة في نفس الخلية، ووسمُه يذكر اسم العملية
 * كي لا تُقرأ عشرُ نجماتٍ متطابقة في القائمة.
 */
export function OperationTile({
  card,
  categories,
  isFavourite,
  onSelect,
  onToggleFavourite,
}: {
  card: OperationCard;
  categories?: CategoryCard[];
  isFavourite: boolean;
  onSelect: (opId: string) => void;
  onToggleFavourite: (opId: string) => void;
}) {
  const uid = useId();
  const whyId = `${uid}-why`;
  const unavailable = card.availability.state !== 'available';
  const unavailableKind =
    card.availability.state === 'tool_missing' ? 'tool-missing' : 'unsupported';
  const availabilityLabel =
    card.availability.state === 'tool_missing'
      ? t('ops.unavailable.tool.label')
      : card.availability.state === 'unsupported'
        ? t('ops.unavailable.unsupported.label')
        : t('ops.available');
  const category = categories ? findCategory(categories, card.category) : undefined;

  return (
    <li className="ops__cell">
      <div className="tile">
        <button
          type="button"
          className={`card op-card op-card--operation ${unavailable ? `op-card--off op-card--${unavailableKind}` : 'card--interactive'}`}
          disabled={unavailable}
          aria-describedby={unavailable ? whyId : undefined}
          onClick={() => onSelect(card.id)}
        >
          {/* الأيقونة نفسها هي العنصر البصري: هوية Page 15 لا تضعها داخل
              بلاطةٍ زخرفية، وتبقي لون Ember لحالة التفاعل. */}
          <svg className="op-card__icon" viewBox="0 0 24 24" aria-hidden="true">
            <use href={card.icon} />
          </svg>

          <span className="op-card__text">
            <span className="t-card-title op-card__title">{t(card.titleKey)}</span>
            <span className="t-body-sec op-card__description">{t(card.descriptionKey)}</span>
            <span className="tile__meta">
              <span className="tile__origin">
                {category && (
                  <>
                    <span className="tile__category">{t(category.titleKey)}</span>
                    <span className="tile__separator" aria-hidden="true">
                      ·
                    </span>
                  </>
                )}
                {/* قيمةٌ تقنية حقيقية، ولذلك وحدها Mono ومعزولة الاتجاه. */}
                <bdi className="tile__tool" dir="ltr">
                  {card.tool}
                </bdi>
              </span>
              {unavailable ? (
                <span className={`tile__availability tile__availability--${unavailableKind}`}>
                  {availabilityLabel}
                </span>
              ) : (
                <span className="tile__availability tile__availability--ok">
                  {t('ops.available')}
                </span>
              )}
            </span>
          </span>
        </button>

        <button
          type="button"
          className={`tile__star ${isFavourite ? 'tile__star--on' : ''}`}
          aria-pressed={isFavourite}
          onClick={() => onToggleFavourite(card.id)}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-favorites" />
          </svg>
          <span className="visually-hidden">
            {t(isFavourite ? 'lib.favourite.remove' : 'lib.favourite.add')} — {t(card.titleKey)}
          </span>
        </button>
      </div>

      {unavailable && (
        <p id={whyId} className="t-caption op-card__why">
          {unavailableReason(card)}
        </p>
      )}
    </li>
  );
}

/**
 * بطاقة قسم.
 *
 * ## اسمٌ وعدد، لا شيء ثالث
 *
 * كان تحت الاسم وصفُ القسم كاملًا ثم العدد. وشبكةٌ من عشر بطاقات كلٌّ منها
 * فقرةٌ تعني أن تُقرأ عشر فقرات كي يُختار بابٌ واحد — وهذا نقيض التصفّح.
 * والوصف باقٍ حيث يُقرأ: ترويسةُ شاشة القسم بعد فتحه (`category-screen.tsx`)،
 * فلا شيء ضاع وإنما انتقل إلى حيث يعني.
 *
 * ## العددان، ولماذا اثنان
 *
 * «‏٦ عمليات» وحدها تَعِد بستّ. فإن كانت أربعٌ منها تعمل على هذا الجهاز
 * والباقي ينتظر أداةً غائبة، فالوعد نصفُ صادق — ويكتشف المستخدم ذلك بعد أن
 * يفتح القسم. السطر الثاني يقول الفرق قبل أن يفتح.
 *
 * وحين يتساوى العددان لا يُعرض إلا واحد: تكرار الرقم نفسه بصيغتين يجعل العين
 * تبحث عن فرقٍ لا وجود له.
 */
export function CategoryTile({
  card,
  onSelect,
}: {
  card: CategoryCard;
  onSelect: (categoryId: string) => void;
}) {
  const journal = card.kind === 'journal';
  const partial = !journal && card.availableCount < card.operationCount;
  const empty = !journal && card.operationCount === 0;
  const state = journal ? 'history' : empty ? 'empty' : partial ? 'partial' : 'available';

  return (
    <li className="ops__cell">
      <button
        type="button"
        className={`card op-card category-card category-card--${state} card--interactive`}
        onClick={() => onSelect(card.id)}
      >
        <svg className="op-card__icon" viewBox="0 0 24 24" aria-hidden="true">
          <use href={card.icon} />
        </svg>

        <span className="op-card__text">
          <span className="t-card-title op-card__title">{t(card.titleKey)}</span>
          <span className={`t-caption tile__count tile__count--${state}`}>
            {journal ? (
              t('lib.category.journal')
            ) : empty ? (
              t('lib.category.empty.count')
            ) : partial ? (
              <>
                <span className="num">{card.availableCount}</span>{' '}
                {t('lib.category.availability.of')}{' '}
                <span className="num">{card.operationCount}</span>{' '}
                {t('lib.category.availability.operations')}
              </>
            ) : card.operationCount === 1 ? (
              t('ops.count.one')
            ) : (
              <>
                <span className="num">{card.operationCount}</span> {t('ops.count.many')}
              </>
            )}
          </span>
        </span>
      </button>
    </li>
  );
}
