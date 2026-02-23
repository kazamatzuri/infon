import { test, expect } from '@playwright/test';

test.describe('Navigation & Layout', () => {
  test('navbar renders with public links for unauthenticated user', async ({ page }) => {
    await page.goto('/');
    const nav = page.locator('nav.app-nav');
    await expect(nav).toBeVisible();
    await expect(nav.locator('h1')).toHaveText('Infon Arena');

    // Public nav links visible
    await expect(nav.getByRole('link', { name: 'Leaderboard' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Tournaments' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Docs' })).toBeVisible();

    // Auth-only nav links NOT visible
    await expect(nav.getByRole('link', { name: 'Bot Library' })).not.toBeVisible();
    await expect(nav.getByRole('link', { name: 'Editor' })).not.toBeVisible();
    await expect(nav.getByRole('link', { name: 'Game' })).not.toBeVisible();
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

  test('landing page shows for unauthenticated user', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('heading', { name: 'Infon Arena' })).toBeVisible();
    await expect(page.getByText('Program your bots')).toBeVisible();
    await expect(page.getByRole('link', { name: 'Start Competing' })).toBeVisible();
  });

  test('clicking nav links navigates to correct pages', async ({ page }) => {
    await page.goto('/');

    await page.getByRole('link', { name: 'Leaderboard' }).click();
    await expect(page).toHaveURL('/leaderboard');
    await expect(page.getByRole('heading', { name: 'Leaderboards' })).toBeVisible();

    await page.getByRole('link', { name: 'Tournaments' }).click();
    await expect(page).toHaveURL('/tournaments');
    await expect(page.getByRole('heading', { name: 'Tournaments' })).toBeVisible();

    await page.getByRole('link', { name: 'Docs' }).click();
    await expect(page).toHaveURL('/docs');
    await expect(page.getByRole('heading', { name: 'Lua API Reference' })).toBeVisible();
  });

  test('protected routes redirect to login when unauthenticated', async ({ page }) => {
    for (const path of ['/bots', '/editor', '/game', '/challenge', '/api-keys', '/teams']) {
      await page.goto(path);
      await expect(page).toHaveURL('/login', { timeout: 5000 });
    }
  });
});
