import { test, expect } from '@playwright/test';
import { generateTestToken, loginWithToken } from './helpers';

test.describe('Column Management', () => {
  const tableName = 'col_test_' + Date.now();

  test.beforeEach(async ({ page }) => {
    const token = generateTestToken('e2e-user', 300);
    await loginWithToken(page, token);
  });

  test('add a TEXT column', async ({ page }) => {
    const tbl = 'coltest_text_' + Date.now();
    await page.fill('#new-table-name', tbl);
    await page.click('button:has-text("Create Table")');
    await page.click(`a:has-text("${tbl}")`);

    await page.fill('#new-col-name', 'username');
    await page.selectOption('#new-col-type', 'TEXT');
    await page.click('button:has-text("Add Column")');
    await expect(page.locator('#column-list')).toContainText('username');
    await expect(page.locator('#column-list')).toContainText('TEXT');
  });

  test('add an INTEGER column', async ({ page }) => {
    const tbl = 'coltest_int_' + Date.now();
    await page.fill('#new-table-name', tbl);
    await page.click('button:has-text("Create Table")');
    await page.click(`a:has-text("${tbl}")`);

    await page.fill('#new-col-name', 'age');
    await page.selectOption('#new-col-type', 'INTEGER');
    await page.click('button:has-text("Add Column")');
    await expect(page.locator('#column-list')).toContainText('age');
    await expect(page.locator('#column-list')).toContainText('INTEGER');
  });

  test('remove a column', async ({ page }) => {
    const tbl = 'coltest_rm_' + Date.now();
    await page.fill('#new-table-name', tbl);
    await page.click('button:has-text("Create Table")');
    await page.click(`a:has-text("${tbl}")`);

    await page.fill('#new-col-name', 'temp_column');
    await page.selectOption('#new-col-type', 'TEXT');
    await page.click('button:has-text("Add Column")');
    await expect(page.locator('#column-list')).toContainText('temp_column');

    page.on('dialog', dialog => dialog.accept());
    await page.click('#column-list button:has-text("Remove")');
    await expect(page.locator('#column-list')).not.toContainText('temp_column');
  });

  test('id column cannot be removed (no Remove button)', async ({ page }) => {
    const tbl = 'coltest_pk_' + Date.now();
    await page.fill('#new-table-name', tbl);
    await page.click('button:has-text("Create Table")');
    await page.click(`a:has-text("${tbl}")`);

    const idRow = page.locator('tr:has-text("id")');
    await expect(idRow.locator('button:has-text("Remove")')).toHaveCount(0);
  });

  test('add multiple columns and verify all present', async ({ page }) => {
    const tbl = 'coltest_multi_' + Date.now();
    await page.fill('#new-table-name', tbl);
    await page.click('button:has-text("Create Table")');
    await page.click(`a:has-text("${tbl}")`);

    const columns = [
      { name: 'name', type: 'TEXT' },
      { name: 'email', type: 'TEXT' },
      { name: 'score', type: 'REAL' },
    ];
    for (const col of columns) {
      await page.fill('#new-col-name', col.name);
      await page.selectOption('#new-col-type', col.type);
      await page.click('button:has-text("Add Column")');
      await expect(page.locator('#column-list')).toContainText(col.name);
    }
    for (const col of columns) {
      await expect(page.locator('#column-list')).toContainText(col.name);
      await expect(page.locator('#column-list')).toContainText(col.type);
    }
  });
});
