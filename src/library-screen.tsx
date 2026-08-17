/**
 * شاشة الفئات — جذر المكتبة، وسؤالها: «في أيّ باب؟».
 *
 * ## ليس في هذا الملف اسمُ قسمٍ ولا عددُ عمليات
 *
 * الأقسام تصل من `list_categories()` بعدديها محسوبين في النواة من الفهرس
 * نفسه، والعمليات من `list_operations()`. لا يوجد هنا — ولا في أي ملفٍ في
 * الواجهة — رقمٌ مكتوب بيد يقول «ثماني عمليات». اختبارٌ مرافق يحقن أقسامًا
 * وعملياتٍ لا وجود لها في هذا البناء ويطالب بظهورها كاملةً بأعدادها، ويقرأ
 * هذا المصدر نفسه ليتأكّد أنه خلا من أي معرّف قسمٍ أو عملية.
 *
 * ## ترتيب الشاشة، ولماذا هو هذا الترتيب
 *
 * البحث، ثم شبكة الأقسام، ثم المفضّلة، ثم المستخدَمة حديثًا.
 *
 * جُرِّب العكس أوّلًا — المفضّلة والمستخدَمة حديثًا فوق الشبكة — بحجّةٍ معقولة:
 * من يفتح التطبيق في يومه العاشر يعرف ما يريد، فتقديمُ الشبكة يجعله يمرّ
 * عليها كل مرّة. وسقطت الحجّة على القياس: الصفّان معًا يبلغان ارتفاع الشاشة
 * تقريبًا، فتنزل شبكة الأقسام **تحت الطيّة**، ويفتح المستخدمُ الجديد شاشةً
 * اسمها «الفئات» ولا يرى فيها فئةً واحدة.
 *
 * فالشبكة أوّلًا: هي ما يسمّي الشاشة، ولا يجوز أن يحتاج المرء تمريرًا ليرى ما
 * جاء من أجله. والصفّان بعدها، ويختفيان حين يفرغان — فأول تشغيلٍ لا يرى إلا
 * الشبكة، وهو بالضبط ما يحتاجه.
 *
 * ## البحث يحلّ محلّ الشاشة ولا يضيف إليها
 *
 * حين يكتب المستخدم شيئًا تُستبدل الشاشة كلها بالنتائج. البديل — إبقاء الشبكة
 * وإضافة النتائج فوقها — يجعل الصفحة تطول والعين تبحث عن الجديد فيها.
 */
import { useCallback, useEffect, useId, useMemo, useRef, useState } from 'react';
import type { CoreErrorShape } from './ipc';
import type { CategoryCard, OperationCard } from './library';
import { isAvailable, search } from './library';
import { errorText, t } from './i18n';
import { CategoryTile, OperationTile } from './library-tiles';
import StatePanel from './state-panel';
import './library-screen.css';

/**
 * حالة المكتبة كما يراها المنسّق.
 *
 * `failed` تحمل خطأ النواة كاملًا لا رايةً منطقية: الرسالة العامة تقول إن
 * المكتبة لم تُقرأ، والمفتاح القادم من النواة يقول لماذا. حذفُه كان يجعل كل
 * أعطال البدء تبدو واحدة.
 */
export type LibraryState =
  | { status: 'loading' }
  | { status: 'ready'; categories: CategoryCard[]; operations: OperationCard[] }
  | { status: 'failed'; error: CoreErrorShape };

interface Props {
  state: LibraryState;
  favourites: OperationCard[];
  recents: OperationCard[];
  favouriteIds: string[];
  onOpenCategory: (categoryId: string) => void;
  onOpenOperation: (opId: string) => void;
  onToggleFavourite: (opId: string) => void;
  onRetry: () => void;
  initialQuery?: string;
  onQueryChange?: (query: string) => void;
}

/**
 * صفٌّ من بطاقات العمليات بعنوانه. يختفي كاملًا حين يفرغ.
 *
 * مكوّنٌ لا نسختان مكتوبتان: «المفضّلة» و«المستخدَمة حديثًا» متطابقان بنيويًا،
 * وكتابتهما مرّتين كانت تعني أن تحسينًا في أحدهما يُنسى في الآخر.
 */
function OperationRow({
  titleKey,
  hintKey,
  cards,
  categories,
  favouriteIds,
  onOpen,
  onToggleFavourite,
}: {
  titleKey: string;
  hintKey: string;
  cards: OperationCard[];
  categories: CategoryCard[];
  favouriteIds: string[];
  onOpen: (opId: string) => void;
  onToggleFavourite: (opId: string) => void;
}) {
  if (cards.length === 0) return null;
  return (
    <section className="lib__row" aria-labelledby={`row-${titleKey}`}>
      <div className="lib__row-head">
        <h3 id={`row-${titleKey}`} className="t-section-title">
          {t(titleKey)}
        </h3>
        <p className="t-caption lib__row-hint">{t(hintKey)}</p>
      </div>
      <ul className="ops__grid">
        {cards.map((card) => (
          <OperationTile
            key={card.id}
            card={card}
            categories={categories}
            isFavourite={favouriteIds.includes(card.id)}
            onSelect={onOpen}
            onToggleFavourite={onToggleFavourite}
          />
        ))}
      </ul>
    </section>
  );
}

export default function LibraryScreen(props: Props): JSX.Element {
  const {
    state,
    favourites,
    recents,
    favouriteIds,
    onOpenCategory,
    onOpenOperation,
    onToggleFavourite,
    onRetry,
    initialQuery = '',
    onQueryChange,
  } = props;

  const [query, setQuery] = useState(initialQuery);
  const [showUnavailableDetails, setShowUnavailableDetails] = useState(false);
  const heading = useRef<HTMLHeadingElement>(null);
  const box = useRef<HTMLInputElement>(null);
  const uid = useId();

  // الشاشة هي الوجهة التي يُرجَع إليها من كل شاشة، وأوّل ما يلي الترحيب — فهي
  // أكثر الشاشات وصولًا، وأشدّها فقدًا للبؤرة: الزرّ الذي ضُغط يُنزع من الشجرة
  // فتسقط البؤرة إلى `body` ويستأنف Tab من رأس المستند. نقلُها إلى العنوان
  // يجعل أوّل ما يُنطق سؤالَ الشاشة، وأوّل ما يليه أفعالها.
  useEffect(() => {
    heading.current?.focus();
  }, []);

  const ready = state.status === 'ready' ? state : null;

  const results = useMemo(() => {
    if (!ready) return null;
    return search(query, ready.operations, ready.categories);
  }, [ready, query]);

  const updateQuery = useCallback((next: string) => {
    setQuery(next);
    setShowUnavailableDetails(false);
    onQueryChange?.(next);
  }, [onQueryChange]);

  /**
   * اختصارٌ واحد: `/` يضع البؤرة في مربّع البحث.
   *
   * المستمع على المستند لا على المربّع، وإلّا لم يعمل إلا وهو مركَّز أصلًا —
   * أي لم يعمل. والحارس يمنعه داخل أي حقل: من يكتب `/` في مسارٍ يريد شرطةً
   * مائلة لا قفزةً إلى مكانٍ آخر.
   */
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key !== '/' || event.metaKey || event.ctrlKey || event.altKey) return;
      const active = document.activeElement;
      const typing =
        active instanceof HTMLInputElement ||
        active instanceof HTMLTextAreaElement ||
        (active instanceof HTMLElement && active.isContentEditable);
      if (typing) return;
      event.preventDefault();
      box.current?.focus();
      box.current?.select();
    }
    document.addEventListener('keydown', onKeyDown);
    return () => document.removeEventListener('keydown', onKeyDown);
  }, []);

  /** Escape يُفرغ البحث ويُبقي البؤرة: المخرج الآمن لا يُخرج من المربّع. */
  const onSearchKeyDown = useCallback((event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Escape') {
      event.preventDefault();
      updateQuery('');
    }
  }, [updateQuery]);

  const searchId = `${uid}-search`;
  const searching = results?.active === true;
  const found =
    searching ? results.categories.length + results.operations.length : 0;
  const searchUnavailable =
    results?.active === true &&
    results.categories.length === 0 &&
    results.operations.length > 0 &&
    results.operations.every((card) => !isAvailable(card.availability));

  return (
    <section className="lib" aria-labelledby="lib-heading">
      <div className="lib__hero">
        <header className="ops__head">
          <div className="ops__intro">
            <h2 id="lib-heading" className="t-page-title ops__title" tabIndex={-1} ref={heading}>
              {t(searching ? 'lib.search.title' : 'lib.heading')}
            </h2>
            <p
              className="t-body-sec ops__sub"
              role={searching ? 'status' : undefined}
              aria-live={searching ? 'polite' : undefined}
            >
              {searching ? (
                <>
                  <span className="num">{found}</span> {t('lib.search.summary')}{' '}
                  <bdi>‹{query.trim()}›</bdi>
                </>
              ) : (
                t('lib.subheading')
              )}
            </p>
          </div>
        </header>

        <div
          className={`field field--search lib__search${
            searching && found === 0 ? ' field--no-results' : ''
          }`}
        >
          <label className="visually-hidden" htmlFor={searchId}>
            {t('lib.search.label')}
          </label>
          <svg viewBox="0 0 24 24" aria-hidden="true" className="field__icon">
            <use href="#i-search" />
          </svg>
          <input
            id={searchId}
            ref={box}
            type="search"
            className="lib__search-input"
            placeholder={t('lib.search.placeholder')}
            value={query}
            onChange={(e) => updateQuery(e.target.value)}
            onKeyDown={onSearchKeyDown}
            autoComplete="off"
            spellCheck={false}
          />
        </div>
      </div>

      {/* منطقةٌ حيّة **ثابتة** يتبدّل ما بداخلها. لو كانت كل حالةٍ منطقةً حيّة
          تُركَّب عند ظهورها لضاع الإعلان عن نصفها، ولو ضُمّت الشبكة إليها
          لأُعلنت البطاقات كلها في كل تحميل. */}
      <div className="ops__live" role="status" aria-live="polite">
        {state.status === 'loading' && (
          <StatePanel
            title={t('lib.loading')}
            body={t('lib.loading.body')}
            busy
          />
        )}
      </div>

      {state.status === 'failed' && (
        <StatePanel
          title={t('lib.failed.title')}
          body={`${t('lib.failed.body')} ${errorText(state.error.key, state.error.detail)}`}
          tone="danger"
          action={t('ops.retry')}
          onAction={onRetry}
        />
      )}

      {/* ── نتائج البحث ────────────────────────────────────────────── */}
      {results?.active === true && (
        <>
          {searchUnavailable && !showUnavailableDetails && (
            <StatePanel
              title={t('lib.search.unavailable.title')}
              body={t('lib.search.unavailable.body')}
              tone="warning"
              action={t('lib.search.unavailable.action')}
              onAction={() => setShowUnavailableDetails(true)}
            />
          )}

          {(!searchUnavailable || showUnavailableDetails) && results.categories.length > 0 && (
            <section className="lib__row" aria-labelledby="found-categories">
              <div className="lib__row-head">
                <h3 id="found-categories" className="t-section-title">
                  {t('lib.search.categories')}
                </h3>
              </div>
              <ul className="ops__grid ops__grid--categories">
                {results.categories.map((card) => (
                  <CategoryTile key={card.id} card={card} onSelect={onOpenCategory} />
                ))}
              </ul>
            </section>
          )}

          {(!searchUnavailable || showUnavailableDetails) && results.operations.length > 0 && (
            <section className="lib__row" aria-labelledby="found-operations">
              <div className="lib__row-head">
                <h3 id="found-operations" className="t-section-title">
                  {t('lib.search.operations')}
                </h3>
              </div>
              <ul className="ops__grid ops__grid--operations">
                {results.operations.map((card) => (
                  <OperationTile
                    key={card.id}
                    card={card}
                    /* نتيجةُ بحثٍ بلا اسم قسمها تترك المستخدم يخمّن أين وجدها،
                       فلا يستطيع العودة إليها إلا بالبحث مرّةً أخرى. */
                    categories={ready?.categories ?? []}
                    isFavourite={favouriteIds.includes(card.id)}
                    onSelect={onOpenOperation}
                    onToggleFavourite={onToggleFavourite}
                  />
                ))}
              </ul>
            </section>
          )}

          {found === 0 && (
            <StatePanel
              title={t('lib.search.empty.title')}
              body={t('lib.search.empty.body')}
              action={t('action.return.library')}
              onAction={() => updateQuery('')}
            />
          )}
        </>
      )}

      {/* ── المكتبة، حين لا بحث ────────────────────────────────────── */}
      {ready && results?.active === false && (
        <>
          {ready.categories.length === 0 ? (
            <StatePanel
              title={t('lib.empty')}
              body={t('lib.empty.body')}
              action={t('action.return.library')}
              onAction={onRetry}
            />
          ) : (
            <ul className="ops__grid ops__grid--categories" aria-label={t('lib.search.categories')}>
              {ready.categories.map((card) => (
                <CategoryTile key={card.id} card={card} onSelect={onOpenCategory} />
              ))}
            </ul>
          )}
          <OperationRow
            titleKey="lib.favourites.title"
            hintKey="lib.favourites.hint"
            cards={favourites}
            categories={ready.categories}
            favouriteIds={favouriteIds}
            onOpen={onOpenOperation}
            onToggleFavourite={onToggleFavourite}
          />
          <OperationRow
            titleKey="lib.recents.title"
            hintKey="lib.recents.hint"
            cards={recents}
            categories={ready.categories}
            favouriteIds={favouriteIds}
            onOpen={onOpenOperation}
            onToggleFavourite={onToggleFavourite}
          />
        </>
      )}
    </section>
  );
}

/** يُصدَّر للاختبار: عدد ما يُعرض من عملياتٍ متاحة في صفٍّ ما. */
export function availableIn(cards: OperationCard[]): number {
  return cards.filter((c) => isAvailable(c.availability)).length;
}
