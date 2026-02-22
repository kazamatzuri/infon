import { test, expect } from '@playwright/test';

test.describe('Navigation & Layout', () => {
  test('navbar renders with all public links', async ({ page }) => {
    await page.goto('/');
    const nav = page.locator('nav.app-nav');
    await expect(nav).toBeVisible();

    // App title
    await expect(nav.locator('h1')).toHaveText('Infon Arena');

    // Public nav links always visible
    await expect(nav.getByRole('link', { name: 'Bot Library' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Editor' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Leaderboard' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Tournaments' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Game' })).toBeVisible();
  });

  test('unauthenticated user sees Login/Register links', async ({ page }) => {
    await page.goto('/');
    const nav = page.locator('nav.app-nav');

    await expect(nav.getByRole('link', { name: 'Login' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Register' })).toBeVisible();

    // Authenticated-only links should NOT be visible
    await expect(nav.getByRole('link', { name: 'Challenge' })).not.toBeVisible();
    await expect(nav.getByRole('link', { name: 'Teams' })).not.toBeVisible();
    await expect(nav.getByRole('link', { name: 'API Keys' })).not.toBeVisible();
  });

  test('clicking nav links navigates to correct pages', async ({ page }) => {
    await page.goto('/');

    await page.getByRole('link', { name: 'Leaderboard' }).click();
    await expect(page).toHaveURL('/leaderboard');
    await expect(page.getByRole('heading', { name: 'Leaderboards' })).toBeVisible();

    await page.getByRole('link', { name: 'Tournaments' }).click();
    await expect(page).toHaveURL('/tournaments');
    await expect(page.getByRole('heading', { name: 'Tournaments' })).toBeVisible();

    await page.getByRole('link', { name: 'Game' }).click();
    await expect(page).toHaveURL('/game');

    await page.getByRole('link', { name: 'Bot Library' }).click();
    await expect(page).toHaveURL('/');
    await expect(page.getByRole('heading', { name: 'Bot Library' })).toBeVisible();
  });
});
