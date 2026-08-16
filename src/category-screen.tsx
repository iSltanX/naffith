/**
 * شاشة القسم — عملياته، وسؤالها: «أيّ عملية؟».
 *
 * ## لماذا شاشةٌ ثالثة بين الفئات والعملية
 *
 * قائمةٌ واحدة تحمل سبعًا وأربعين عملية ليست قائمة، هي جدار. والتقسيم إلى
 * أقسامٍ في شاشةٍ واحدة (عناوين وفواصل) كان يبقي الجدار ويُضيف إليه تمريرًا.
 * الشاشة الوسطى تجعل كل عرضٍ يجيب سؤالًا واحدًا: الفئات «في أيّ باب؟»،
 * والقسم «أيّ عملية؟».
 *
 * ## غير المتاح يُعرض ولا يُخفى
 *
 * قسمٌ فيه ستّ عمليات تعمل منها أربع يعرض الستّ، والاثنتان معطّلتان بسببهما
 * المكتوب تحتهما. إخفاؤهما كان يجعل المستخدم يظنّ أن المنتج لا يفعل ذلك
 * أصلًا، بدل أن يعرف أن أداةً واحدة تنقصه — وهو فرقٌ بين «غير ممكن» و«ينقصك
 * كذا».
 */
import { useEffect, useRef } from 'react';
import type { CategoryCard, OperationCard } from './library';
import { isAvailable } from './library';
import { t } from './i18n';
import { OperationTile } from './library-tiles';
import './library-screen.css';

interface Props {
  category: CategoryCard;
  operations: OperationCard[];
  favouriteIds: string[];
  onOpenOperation: (opId: string) => void;
  onToggleFavourite: (opId: string) => void;
  onBack: () => void;
}

export default function CategoryScreen(props: Props): JSX.Element {
  const { category, operations, favouriteIds, onOpenOperation, onToggleFavourite, onBack } = props;
  const heading = useRef<HTMLHeadingElement>(null);

  // البؤرة على العنوان عند بلوغ الشاشة: أوّل ما يُنطق اسمُ القسم، لا زرّ
  // الرجوع. والاعتمادية على المعرّف لا على الكائن: الانتقال من قسمٍ إلى آخر
  // انتقالُ شاشةٍ كامل وإن بقي المكوّن نفسه مركَّبًا.
  useEffect(() => {
    heading.current?.focus();
  }, [category.id]);

  const available = operations.filter((o) => isAvailable(o.availability)).length;
  const partial = available < operations.length;

  return (
    <section className="lib" aria-labelledby="cat-heading">
      <header className="cat__head">
        <button type="button" className="btn btn--ghost btn--sm cat__back" onClick={onBack}>
          <svg viewBox="0 0 24 24" aria-hidden="true" data-directional>
            <use href="#i-chevron" />
          </svg>
          {t('nav.back.library')}
        </button>

        <div className="cat__title-row">
          <span className="cat__icon" aria-hidden="true">
            <svg viewBox="0 0 24 24">
              <use href={category.icon} />
            </svg>
          </span>
          <div className="ops__intro">
            <h2 id="cat-heading" className="t-page-title ops__title" tabIndex={-1} ref={heading}>
              {t(category.titleKey)}
            </h2>
            <p className="t-body-sec ops__sub">{t(category.descriptionKey)}</p>
          </div>
        </div>

        {/* العدد يُحسب من البطاقات المعروضة لا يُمرَّر: أي رقمٍ يأتي من مكانٍ
            آخر يمكن أن يخالف ما تراه العين. */}
        <p className="t-caption ops__count">
          {operations.length === 1 ? (
            t('ops.count.one')
          ) : (
            <>
              <span className="num">{operations.length}</span> {t('ops.count.many')}
            </>
          )}
          {partial && (
            <>
              {' · '}
              <span className="num">{available}</span> {t('lib.category.available')}
            </>
          )}
        </p>
      </header>

      {operations.length === 0 ? (
        <div className="ops__state">
          <svg viewBox="0 0 24 24" aria-hidden="true" className="ops__state-icon">
            <use href="#i-info" />
          </svg>
          <p className="t-body-sec">{t('lib.category.empty')}</p>
        </div>
      ) : (
        <ul className="ops__grid">
          {operations.map((card) => (
            <OperationTile
              key={card.id}
              card={card}
              isFavourite={favouriteIds.includes(card.id)}
              onSelect={onOpenOperation}
              onToggleFavourite={onToggleFavourite}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
