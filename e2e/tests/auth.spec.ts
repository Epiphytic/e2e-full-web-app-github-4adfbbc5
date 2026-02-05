import { test, expect } from '@playwright/test';
import { generateTestToken, loginWithToken } from './helpers';

test.describe('Authentication', () => {
  test('login page is accessible', async ({ page }) => {
    await page.goto('/login');
    await expect(page.locator('h2')).toHaveText('Login');
    await expect(page.locator('#token')).toBeVisible();
  });

  test('login with valid short-lived JWT token redirects to dashboard', async ({ page }) => {
    const token = generateTestToken('e2e-user', 60);
    await loginWithToken(page, token);
    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('h2')).toHaveText('Database Tables');
  });

  test('login with expired JWT token shows error', async ({ page }) => {
    const token = generateTestToken('e2e-user', -10);
    await page.goto('/login');
    await page.fill('#token', token);
    await page.click('button[type="submit"]');
    await expect(page.locator('#login-error')).toBeVisible();
    await expect(page.locator('#login-error')).toHaveText('Invalid token');
  });

  test('.well-known/jwks.json endpoint returns valid JWK set', async ({ request }) => {
    const response = await request.get('/.well-known/jwks.json');
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(body.keys).toHaveLength(1);
    expect(body.keys[0].kty).toBe('RSA');
    expect(body.keys[0].alg).toBe('RS256');
    expect(body.keys[0].n).toBeTruthy();
    expect(body.keys[0].e).toBeTruthy();
  });
});
