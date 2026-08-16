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
  onOpenLog: () => void;
  onOpenSettings: () => void;
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
  favouriteIds,
  onOpen,
  onToggleFavourite,
}: {
  titleKey: string;
  hintKey: string;
  cards: OperationCard[];
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
    onOpenLog,
    onOpenSettings,
  } = props;

  const [query, setQuery] = useState('');
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
      setQuery('');
    }
  }, []);

  const searchId = `${uid}-search`;
  const found =
    results?.active === true ? results.categories.length + results.operations.length : 0;

  return (
    <section className="lib" aria-labelledby="lib-heading">
      <header className="ops__head">
        <div className="ops__intro">
          {/* `h1` للتطبيق كله في الغلاف، فترويسة الشاشة `h2`. حجمها من الصنف
              لا من مستواها: المستوى بنيةٌ للقارئ، والحجم قرارٌ بصري. */}
          <h2 id="lib-heading" className="t-page-title ops__title" tabIndex={-1} ref={heading}>
            {t('lib.heading')}
          </h2>
          <p className="t-body-sec ops__sub">{t('lib.subheading')}</p>
        </div>

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

      {/* مربّع البحث. `type="search"` لا `text`: المتصفّح يمنحه زرّ المسح
          وسلوك Escape الأصليّين، وقارئ الشاشة ينطقه «بحث» بلا وسمٍ إضافي.

          والصندوق `.field` من نظام التصميم لا صنفًا خاصًّا بهذه الشاشة: منه
          يأخذ ارتفاع عناصر الإدخال وحشوتها وحدَّها الواحد وحلقةَ تركيزها، فهو
          والحقول في شاشة العملية شيءٌ واحد بالبناء لا بالمصادفة. وكان
          `class="input"` — ولا قاعدة بهذا الاسم في المشروع — فيُرسم بأثاث
          الوكيل الخام تحته حدُّنا وفوقه حلقةُ تركيزٍ ثانية. */}
      <div className="field field--search">
        <label className="visually-hidden" htmlFor={searchId}>
          {t('lib.search.label')}
        </label>
        {/* أيقونةٌ واحدة داخل الحقل، وهي ابنُه الأوّل: موضعُها من ترتيب الصفّ
            لا من تثبيتٍ مطلق فوقه بحشوةٍ محسوبة له. */}
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
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={onSearchKeyDown}
          autoComplete="off"
          spellCheck={false}
        />
      </div>

      {/* منطقةٌ حيّة **ثابتة** يتبدّل ما بداخلها. لو كانت كل حالةٍ منطقةً حيّة
          تُركَّب عند ظهورها لضاع الإعلان عن نصفها، ولو ضُمّت الشبكة إليها
          لأُعلنت البطاقات كلها في كل تحميل. */}
      <div className="ops__live" role="status" aria-live="polite">
        {state.status === 'loading' && (
          <div className="ops__state">
            <span className="spinner" aria-hidden="true" />
            <p className="t-body-sec">{t('lib.loading')}</p>
          </div>
        )}
        {results?.active === true && (
          <p className="t-caption ops__count">
            {found === 0 ? (
              t('lib.search.none')
            ) : (
              <>
                <span className="num">{found}</span> {t('lib.search.count')}
              </>
            )}
          </p>
        )}
      </div>

      {state.status === 'failed' && (
        <div className="ops__state ops__state--bad" role="alert">
          <span className="chip chip--danger">
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-error" />
            </svg>
            {t('state.failed')}
          </span>
          <p className="t-body">{t('lib.failed')}</p>
          <p className="t-caption">{errorText(state.error.key, state.error.detail)}</p>
          <button type="button" className="btn btn--quiet" onClick={onRetry}>
            {t('ops.retry')}
          </button>
        </div>
      )}

      {/* ── نتائج البحث ────────────────────────────────────────────── */}
      {results?.active === true && (
        <>
          {results.categories.length > 0 && (
            <section className="lib__row" aria-labelledby="found-categories">
              <div className="lib__row-head">
                <h3 id="found-categories" className="t-section-title">
                  {t('lib.search.categories')}
                </h3>
              </div>
              <ul className="ops__grid">
                {results.categories.map((card) => (
                  <CategoryTile key={card.id} card={card} onSelect={onOpenCategory} />
                ))}
              </ul>
            </section>
          )}

          {results.operations.length > 0 && (
            <section className="lib__row" aria-labelledby="found-operations">
              <div className="lib__row-head">
                <h3 id="found-operations" className="t-section-title">
                  {t('lib.search.operations')}
                </h3>
              </div>
              <ul className="ops__grid">
                {results.operations.map((card) => (
                  <OperationTile
                    key={card.id}
                    card={card}
                    /* نتيجةُ بحثٍ بلا اسم قسمها تترك المستخدم يخمّن أين وجدها،
                       فلا يستطيع العودة إليها إلا بالبحث مرّةً أخرى. */
                    categories={ready?.categories ?? []}
                    showCategory
                    isFavourite={favouriteIds.includes(card.id)}
                    onSelect={onOpenOperation}
                    onToggleFavourite={onToggleFavourite}
                  />
                ))}
              </ul>
            </section>
          )}

          {found === 0 && (
            <div className="ops__state">
              <svg viewBox="0 0 24 24" aria-hidden="true" className="ops__state-icon">
                <use href="#i-info" />
              </svg>
              <p className="t-body-sec">{t('lib.search.empty')}</p>
            </div>
          )}
        </>
      )}

      {/* ── المكتبة، حين لا بحث ────────────────────────────────────── */}
      {ready && results?.active === false && (
        <>
          <section className="lib__row" aria-labelledby="lib-categories">
            <div className="lib__row-head">
              <h3 id="lib-categories" className="t-section-title">
                {t('lib.categories.title')}
              </h3>
              <p className="t-caption lib__row-hint">{t('lib.categories.hint')}</p>
            </div>
            {ready.categories.length === 0 ? (
              <div className="ops__state">
                <svg viewBox="0 0 24 24" aria-hidden="true" className="ops__state-icon">
                  <use href="#i-info" />
                </svg>
                <p className="t-body-sec">{t('lib.empty')}</p>
              </div>
            ) : (
              <ul className="ops__grid">
                {ready.categories.map((card) => (
                  <CategoryTile key={card.id} card={card} onSelect={onOpenCategory} />
                ))}
              </ul>
            )}
          </section>
          <OperationRow
            titleKey="lib.favourites.title"
            hintKey="lib.favourites.hint"
            cards={favourites}
            favouriteIds={favouriteIds}
            onOpen={onOpenOperation}
            onToggleFavourite={onToggleFavourite}
          />
          <OperationRow
            titleKey="lib.recents.title"
            hintKey="lib.recents.hint"
            cards={recents}
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
