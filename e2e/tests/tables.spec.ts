import { test, expect } from '@playwright/test';
import { generateTestToken, loginWithToken } from './helpers';

test.describe('Table Management', () => {
  test.beforeEach(async ({ page }) => {
    const token = generateTestToken('e2e-user', 300);
    await loginWithToken(page, token);
  });

  test('create a new table', async ({ page }) => {
    await page.fill('#new-table-name', 'test_users');
    await page.click('button:has-text("Create Table")');
    await expect(page.locator('#table-list')).toContainText('test_users');
  });

  test('table appears in table list', async ({ page }) => {
    await page.fill('#new-table-name', 'products');
    await page.click('button:has-text("Create Table")');
    await expect(page.locator('#table-list a:has-text("products")')).toBeVisible();
  });

  test('navigate to table detail page', async ({ page }) => {
    await page.fill('#new-table-name', 'orders');
    await page.click('button:has-text("Create Table")');
    await page.click('a:has-text("orders")');
    await expect(page).toHaveURL(/\/tables\/orders/);
    await expect(page.locator('h2')).toContainText('orders');
  });

  test('drop a table from dashboard', async ({ page }) => {
    await page.fill('#new-table-name', 'temp_table');
    await page.click('button:has-text("Create Table")');
    await expect(page.locator('#table-list')).toContainText('temp_table');

    page.on('dialog', dialog => dialog.accept());
    await page.click('#table-temp_table button:has-text("Drop")');
    await expect(page.locator('#table-list')).not.toContainText('temp_table');
  });

  test('drop a table from table detail page', async ({ page }) => {
    await page.fill('#new-table-name', 'to_delete');
    await page.click('button:has-text("Create Table")');
    await page.click('a:has-text("to_delete")');

    page.on('dialog', dialog => dialog.accept());
    await page.click('.danger-zone button:has-text("Drop Table")');
    await expect(page).toHaveURL('/dashboard');
  });
});
