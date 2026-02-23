import { test, expect } from '@playwright/test';

const uniqueId = () => Math.random().toString(36).substring(2, 10);

async function registerUser(page: import('@playwright/test').Page) {
  const username = `bottest_${uniqueId()}`;
  await page.goto('/register');
  await page.locator('label:has-text("Username") + input').fill(username);
  await page.locator('label:has-text("Email") + input').fill(`${username}@test.local`);
  await page.locator('label:has-text("Password") + input').fill('TestPass1234');
  await page.getByRole('button', { name: /Register/ }).click();
  await expect(page).toHaveURL('/', { timeout: 10000 });
  return username;
}

test.describe('Bot Creation & Library Workflow', () => {
  test('create a new bot from library page, see it in the list', async ({ page }) => {
    await registerUser(page);
    await expect(page.getByRole('heading', { name: 'Bot Library' })).toBeVisible();

    // Click "+ New Bot"
    await page.getByRole('button', { name: '+ New Bot' }).click();
    await expect(page).toHaveURL(/\/editor\/\d+/, { timeout: 5000 });

    // Navigate back to library
    await page.getByRole('link', { name: 'Bot Library' }).click();
    await expect(page).toHaveURL('/');
    await page.waitForTimeout(1000);

    // Should see at least one bot
    await expect(page.locator('table')).toBeVisible({ timeout: 3000 });
    const rows = page.locator('table tbody tr');
    expect(await rows.count()).toBeGreaterThanOrEqual(1);
  });

  test('delete a bot from the library using inline confirm', async ({ page }) => {
    await registerUser(page);

    // Create a bot first
    await page.getByRole('button', { name: '+ New Bot' }).click();
    await expect(page).toHaveURL(/\/editor\/\d+/, { timeout: 5000 });

    // Go back to library
    await page.getByRole('link', { name: 'Bot Library' }).click();
    await expect(page).toHaveURL('/');
    await page.waitForTimeout(1000);

    // Should have at least one bot row
    const rows = page.locator('table tbody tr');
    const countBefore = await rows.count();
    expect(countBefore).toBeGreaterThanOrEqual(1);

    // Click Delete on the first bot — should show Confirm/Cancel
    const firstRow = rows.first();
    await firstRow.getByRole('button', { name: 'Delete' }).click();
    await expect(firstRow.getByRole('button', { name: 'Confirm' })).toBeVisible();
    await expect(firstRow.getByRole('button', { name: 'Cancel' })).toBeVisible();

    // Click Confirm to actually delete
    await firstRow.getByRole('button', { name: 'Confirm' }).click();
    await page.waitForTimeout(1000);

    // Should have one fewer bot
    const countAfter = await rows.count();
    expect(countAfter).toBe(countBefore - 1);
  });

  test('cancel bot deletion keeps the bot in the list', async ({ page }) => {
    await registerUser(page);

    // Create a bot
    await page.getByRole('button', { name: '+ New Bot' }).click();
    await expect(page).toHaveURL(/\/editor\/\d+/, { timeout: 5000 });

    await page.getByRole('link', { name: 'Bot Library' }).click();
    await expect(page).toHaveURL('/');
    await page.waitForTimeout(1000);

    const rows = page.locator('table tbody tr');
    const countBefore = await rows.count();
    expect(countBefore).toBeGreaterThanOrEqual(1);

    // Click Delete, then Cancel
    const firstRow = rows.first();
    await firstRow.getByRole('button', { name: 'Delete' }).click();
    await expect(firstRow.getByRole('button', { name: 'Confirm' })).toBeVisible();
    await firstRow.getByRole('button', { name: 'Cancel' }).click();

    // Delete button should reappear, count unchanged
    await expect(firstRow.getByRole('button', { name: 'Delete' })).toBeVisible();
    expect(await rows.count()).toBe(countBefore);
  });

  test('bot editor page loads for a new bot', async ({ page }) => {
    await registerUser(page);

    await page.getByRole('button', { name: '+ New Bot' }).click();
    await expect(page).toHaveURL(/\/editor\/\d+/, { timeout: 5000 });

    await page.waitForTimeout(2000);
    const bodyText = await page.locator('main').textContent();
    expect(bodyText!.length).toBeGreaterThan(0);
  });
});
