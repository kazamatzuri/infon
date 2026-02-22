import { test, expect } from '@playwright/test';

test.describe('Leaderboard Page', () => {
  test('renders with correct heading and tab buttons', async ({ page }) => {
    await page.goto('/leaderboard');
    await expect(page.getByRole('heading', { name: 'Leaderboards' })).toBeVisible();

    // All three tabs should be visible
    const tab1v1 = page.getByRole('button', { name: '1v1 Elo' });
    const tabFfa = page.getByRole('button', { name: 'FFA' });
    const tab2v2 = page.getByRole('button', { name: '2v2 Teams' });

    await expect(tab1v1).toBeVisible();
    await expect(tabFfa).toBeVisible();
    await expect(tab2v2).toBeVisible();
  });

  test('switching tabs loads different content', async ({ page }) => {
    await page.goto('/leaderboard');
    await page.waitForTimeout(1000);

    // Click FFA tab
    await page.getByRole('button', { name: 'FFA' }).click();
    await page.waitForTimeout(500);
    // Page should still be on /leaderboard
    await expect(page).toHaveURL('/leaderboard');

    // Click 2v2 tab
    await page.getByRole('button', { name: '2v2 Teams' }).click();
    await page.waitForTimeout(500);
    // Should show the "coming soon" or empty state for 2v2
    const has2v2Content = await page.getByText(/coming soon|No entries/i).isVisible().catch(() => false);
    const hasTable = await page.locator('table').isVisible().catch(() => false);
    expect(has2v2Content || hasTable).toBeTruthy();
  });

  test('shows table headers when entries exist', async ({ page }) => {
    await page.goto('/leaderboard');
    await page.waitForTimeout(1500);

    const hasTable = await page.locator('table').isVisible().catch(() => false);
    if (hasTable) {
      // Verify expected columns
      await expect(page.locator('th').filter({ hasText: '#' })).toBeVisible();
      await expect(page.locator('th').filter({ hasText: 'Bot' })).toBeVisible();
      await expect(page.locator('th').filter({ hasText: 'Rating' })).toBeVisible();
      await expect(page.locator('th').filter({ hasText: 'Games' })).toBeVisible();
    }
  });
});
