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

  test('bot editor page loads for a new bot', async ({ page }) => {
    await registerUser(page);

    await page.getByRole('button', { name: '+ New Bot' }).click();
    await expect(page).toHaveURL(/\/editor\/\d+/, { timeout: 5000 });

    await page.waitForTimeout(2000);
    const bodyText = await page.locator('main').textContent();
    expect(bodyText!.length).toBeGreaterThan(0);
  });
});
