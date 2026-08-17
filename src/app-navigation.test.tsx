// @vitest-environment jsdom
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import AppShell from './app-navigation';
import { t } from './i18n';
import type { CategoryCard } from './library';
import type { Screen } from './nav';

const FILES: CategoryCard = {
  id: 'files',
  titleKey: 'cat.files.title',
  descriptionKey: 'cat.files.description',
  icon: '#i-files',
  kind: 'operations',
  sortOrder: 10,
  operationCount: 2,
  availableCount: 2,
};

afterEach(cleanup);

function view(screenState: Screen = { name: 'operations-list' }) {
  return render(
    <AppShell
      screen={screenState}
      categories={[FILES]}
      activeCategoryId={
        screenState.name === 'category-view' || screenState.name === 'operation-view'
          ? FILES.id
          : undefined
      }
      onOpenLibrary={vi.fn()}
      onOpenCategory={vi.fn()}
      onOpenLog={vi.fn()}
      onOpenSettings={vi.fn()}
    >
      <main>content</main>
    </AppShell>,
  );
}

describe('تنقّل الشريط المضغوط', () => {
  it('يمنح كل زر اسمًا مرئيًا مصمّمًا ولا يعتمد على title الأصلي', () => {
    const { container } = view();
    const buttons = [...container.querySelectorAll<HTMLButtonElement>('.app-nav__item')];

    expect(buttons).toHaveLength(4);
    for (const button of buttons) {
      expect(button.getAttribute('aria-label')).toBeTruthy();
      expect(button.getAttribute('title')).toBeNull();
      const tooltip = button.querySelector('.app-nav__tooltip');
      expect(tooltip?.textContent?.trim()).toBeTruthy();
      expect(tooltip?.getAttribute('aria-hidden')).toBe('true');
    }
  });

  it.each<Screen>([
    { name: 'category-view', categoryId: FILES.id },
    { name: 'operation-view', opId: 'files.example' },
  ])('يبقي المكتبة وجهة الصفحة الوحيدة في الشاشة $name', (screenState) => {
    const { container } = view(screenState);
    const current = [...container.querySelectorAll<HTMLElement>('[aria-current="page"]')];

    expect(current).toHaveLength(1);
    expect(current[0]?.getAttribute('aria-label')).toBe(t('nav.operations'));

    const category = screen.getByRole('button', {
      name: `${t('nav.operations')}: ${t(FILES.titleKey)}`,
    });
    expect(category.getAttribute('aria-current')).toBeNull();
    expect(category.classList.contains('is-ancestor')).toBe(true);
  });
});
