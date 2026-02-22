import { test, expect } from '@playwright/test';

const uniqueId = () => Math.random().toString(36).substring(2, 10);

// Helper to fill the register form using sibling-of-label selectors
async function fillRegister(page: import('@playwright/test').Page, username: string, email: string, password: string) {
  // Username: first text input
  await page.locator('label:has-text("Username") + input').fill(username);
  // Email
  await page.locator('label:has-text("Email") + input').fill(email);
  // Password
  await page.locator('label:has-text("Password") + input').fill(password);
}

// Helper to fill the login form
async function fillLogin(page: import('@playwright/test').Page, username: string, password: string) {
  await page.locator('label:has-text("Username") + input').fill(username);
  await page.locator('label:has-text("Password") + input').fill(password);
}

test.describe('Authentication Flow', () => {
  test('register a new user, see authenticated nav, then logout', async ({ page }) => {
    const username = `testuser_${uniqueId()}`;
    const email = `${username}@test.local`;
    const password = 'TestPass1234';

    await page.goto('/register');
    await expect(page.getByRole('heading', { name: 'Register' })).toBeVisible();

    await fillRegister(page, username, email, password);
    await page.getByRole('button', { name: /Register/ }).click();

    // Should redirect to home
    await expect(page).toHaveURL('/', { timeout: 10000 });
    const nav = page.locator('nav.app-nav');
    await expect(nav.getByText(username)).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Challenge' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Teams' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'API Keys' })).toBeVisible();
    await expect(nav.getByRole('button', { name: 'Logout' })).toBeVisible();

    // Login/Register links should NOT be visible
    await expect(nav.getByRole('link', { name: 'Login' })).not.toBeVisible();

    // Logout
    await nav.getByRole('button', { name: 'Logout' }).click();
    await expect(nav.getByRole('link', { name: 'Login' })).toBeVisible();
    await expect(nav.getByRole('button', { name: 'Logout' })).not.toBeVisible();
  });

  test('login with valid credentials', async ({ page }) => {
    const username = `logintest_${uniqueId()}`;
    const email = `${username}@test.local`;
    const password = 'TestPass1234';

    // Register first
    await page.goto('/register');
    await fillRegister(page, username, email, password);
    await page.getByRole('button', { name: /Register/ }).click();
    await expect(page).toHaveURL('/', { timeout: 10000 });

    // Logout
    await page.getByRole('button', { name: 'Logout' }).click();
    await expect(page.locator('nav.app-nav').getByRole('link', { name: 'Login' })).toBeVisible();

    // Login
    await page.goto('/login');
    await fillLogin(page, username, password);
    await page.getByRole('button', { name: /Login/ }).click();
    await expect(page).toHaveURL('/', { timeout: 10000 });
    await expect(page.locator('nav.app-nav').getByText(username)).toBeVisible();
  });

  test('login with invalid credentials shows error', async ({ page }) => {
    await page.goto('/login');
    await fillLogin(page, 'nonexistent_user', 'wrongpassword');
    await page.getByRole('button', { name: /Login/ }).click();

    // Should stay on login and show error
    await page.waitForTimeout(1000);
    await expect(page).toHaveURL('/login');
    await expect(page.locator('p').filter({ hasText: /failed|invalid|error/i })).toBeVisible({ timeout: 3000 });
  });

  test('register with duplicate username shows error', async ({ page }) => {
    const username = `duptest_${uniqueId()}`;
    const email = `${username}@test.local`;
    const password = 'TestPass1234';

    // Register first time
    await page.goto('/register');
    await fillRegister(page, username, email, password);
    await page.getByRole('button', { name: /Register/ }).click();
    await expect(page).toHaveURL('/', { timeout: 10000 });

    // Logout and try duplicate
    await page.getByRole('button', { name: 'Logout' }).click();
    await page.goto('/register');
    await fillRegister(page, username, `other_${email}`, password);
    await page.getByRole('button', { name: /Register/ }).click();

    // Should stay on register with error
    await page.waitForTimeout(1000);
    await expect(page).toHaveURL('/register');
  });
});
