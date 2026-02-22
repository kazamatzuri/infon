import { test, expect } from '@playwright/test';

test.describe('Game Viewer Page', () => {
  test('shows setup UI or live game', async ({ page }) => {
    await page.goto('/game');
    await page.waitForTimeout(2000);

    const hasSetup = await page.getByText('Start a Game').isVisible().catch(() => false);
    const hasLive = await page.getByText('Live Game').isVisible().catch(() => false);
    expect(hasSetup || hasLive).toBeTruthy();

    if (hasSetup) {
      // If bots exist, we should see map selector and player slots
      const hasBots = await page.locator('#map-select').isVisible().catch(() => false);
      const noBots = await page.getByText('No bots in your library').isVisible().catch(() => false);
      // One of these must be true
      expect(hasBots || noBots).toBeTruthy();

      if (hasBots) {
        // Start Game button should exist
        await expect(page.getByRole('button', { name: /Start Game/ })).toBeVisible();
        await expect(page.getByRole('button', { name: /Add Player/ })).toBeVisible();
      } else {
        // Should have a link to create a bot
        await expect(page.getByText('Create a bot first')).toBeVisible();
      }
    } else if (hasLive) {
      await expect(page.getByRole('button', { name: /Stop Game/ })).toBeVisible();
    }
  });

  test('shows error when starting with no bots selected', async ({ page }) => {
    await page.goto('/game');
    await page.waitForTimeout(2000);

    const startBtn = page.getByRole('button', { name: /Start Game/ });
    const hasStart = await startBtn.isVisible().catch(() => false);
    if (!hasStart) {
      test.skip();
      return;
    }

    await startBtn.click();
    await expect(page.getByText('Select at least 2 bots')).toBeVisible({ timeout: 2000 });
  });

  test('add and remove player slots when bots exist', async ({ page }) => {
    await page.goto('/game');
    await page.waitForTimeout(2000);

    const addBtn = page.getByRole('button', { name: /Add Player/ });
    const hasAdd = await addBtn.isVisible().catch(() => false);
    if (!hasAdd) {
      test.skip();
      return;
    }

    // Count initial player name inputs (should be 2)
    const initialInputs = page.locator('input[placeholder="Name"]');
    expect(await initialInputs.count()).toBe(2);

    // Add a third slot
    await addBtn.click();
    expect(await page.locator('input[placeholder="Name"]').count()).toBe(3);

    // Remove buttons should be visible for all 3 slots
    const removeButtons = page.getByTitle('Remove player');
    expect(await removeButtons.count()).toBe(3);

    // Remove one
    await removeButtons.last().click();
    expect(await page.locator('input[placeholder="Name"]').count()).toBe(2);
  });
});
