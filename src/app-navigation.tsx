import type { ReactNode } from 'react';
import { t } from './i18n';
import type { CategoryCard } from './library';
import type { Screen } from './nav';
import type { SidebarIconSize } from './settings';

interface AppShellProps {
  screen: Screen;
  categories: CategoryCard[];
  activeCategoryId?: string | undefined;
  viewportMode?: 'scroll' | 'fixed';
  /** إعداد «حجم الأيقونات في الشريط الجانبي». `medium` هو مقاس اليوم قبل
   *  هذا الإعداد — فمن لم يمرّره لا يرى تغييرًا. */
  iconSize?: SidebarIconSize;
  onOpenLibrary: () => void;
  onOpenCategory: (categoryId: string) => void;
  onOpenLog: () => void;
  onOpenSettings: () => void;
  children: ReactNode;
}

function screenRelationship(screen: Screen, destination: 'library' | 'log' | 'settings') {
  if (destination === 'log') return screen.name === 'run-log' ? 'current' : 'none';
  if (destination === 'settings') return screen.name === 'settings' ? 'current' : 'none';
  if (
    screen.name === 'operations-list' ||
    screen.name === 'category-view' ||
    screen.name === 'operation-view'
  ) {
    return 'current';
  }
  return 'none';
}

function itemClass(relationship: 'current' | 'ancestor' | 'none', extra = '') {
  return ['app-nav__item', relationship === 'ancestor' ? 'is-ancestor' : '', extra]
    .filter(Boolean)
    .join(' ');
}

/** Persistent product navigation. The desktop identity keeps it on the
 * physical right in RTL: expanded above 960px and a 56px icon rail below it. */
export default function AppShell({
  screen,
  categories,
  activeCategoryId,
  viewportMode = 'scroll',
  iconSize = 'medium',
  onOpenLibrary,
  onOpenCategory,
  onOpenLog,
  onOpenSettings,
  children,
}: AppShellProps): JSX.Element {
  const libraryRelationship = screenRelationship(screen, 'library');
  const logRelationship = screenRelationship(screen, 'log');
  const settingsRelationship = screenRelationship(screen, 'settings');
  const operationCategories = categories.filter((category) => category.kind === 'operations');

  return (
    <div className="app-frame">
      <div className="app-frame__main">
        <div className="app-titlebar" data-tauri-drag-region aria-hidden="true" />
        <div className={`app-frame__viewport app-frame__viewport--${viewportMode}`}>{children}</div>
      </div>

      <aside className={`app-nav app-nav--icon-${iconSize}`} aria-label={t('nav.operations')}>
        {/* اسم المنتج وحده. و«سَطْر» ليس جزءًا منه: هو اسم لوحة الأوامر،
            ويظهر حيث تظهر تلك اللوحة لا في علامة التطبيق. */}
        <div className="app-nav__brand" aria-hidden="true">
          <span className="app-nav__brand-full">{t('app.naffith')}</span>
          <span className="app-nav__brand-rail">ن</span>
        </div>

        <nav className="app-nav__primary" aria-label={t('nav.operations')}>
          <button
            type="button"
            className={itemClass(libraryRelationship)}
            aria-current={libraryRelationship === 'current' ? 'page' : undefined}
            aria-label={t('nav.operations')}
            onClick={onOpenLibrary}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-categories" />
            </svg>
            <span className="app-nav__label">{t('nav.operations')}</span>
            <span className="app-nav__tooltip" aria-hidden="true">
              {t('nav.operations')}
            </span>
          </button>

          <div className="app-nav__categories">
            {operationCategories.map((category) => {
              const selected = activeCategoryId === category.id;
              // The library is the single page-level destination for its
              // category and operation descendants. A selected category is
              // context within that page, never a second `aria-current` page.
              const relationship = selected ? 'ancestor' : 'none';
              return (
                <button
                  key={category.id}
                  type="button"
                  className={itemClass(relationship, 'app-nav__item--category')}
                  aria-label={`${t('nav.operations')}: ${t(category.titleKey)}`}
                  onClick={() => onOpenCategory(category.id)}
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <use href={category.icon} />
                  </svg>
                  <span className="app-nav__label">{t(category.titleKey)}</span>
                  <span className="app-nav__tooltip" aria-hidden="true">
                    {t(category.titleKey)}
                  </span>
                </button>
              );
            })}
          </div>
        </nav>

        <nav className="app-nav__utility" aria-label={t('nav.settings')}>
          <button
            type="button"
            className={itemClass(logRelationship)}
            aria-current={logRelationship === 'current' ? 'page' : undefined}
            aria-label={t('nav.log')}
            onClick={onOpenLog}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-history" />
            </svg>
            <span className="app-nav__label">{t('nav.log')}</span>
            <span className="app-nav__tooltip" aria-hidden="true">
              {t('nav.log')}
            </span>
          </button>
          <button
            type="button"
            className={itemClass(settingsRelationship)}
            aria-current={settingsRelationship === 'current' ? 'page' : undefined}
            aria-label={t('nav.settings')}
            onClick={onOpenSettings}
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href="#i-settings" />
            </svg>
            <span className="app-nav__label">{t('nav.settings')}</span>
            <span className="app-nav__tooltip" aria-hidden="true">
              {t('nav.settings')}
            </span>
          </button>
        </nav>
      </aside>
    </div>
  );
}
