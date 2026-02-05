import * as jwt from 'jsonwebtoken';
import * as fs from 'fs';
import * as path from 'path';
import { Page } from '@playwright/test';

const PRIVATE_KEY_PATH = path.join(__dirname, '../../keys/private.pem');

export function generateTestToken(subject: string = 'test-user', ttlSeconds: number = 300): string {
  const privateKey = fs.readFileSync(PRIVATE_KEY_PATH, 'utf-8');
  const now = Math.floor(Date.now() / 1000);
  const payload = {
    sub: subject,
    iat: now,
    exp: now + ttlSeconds,
    iss: 'sqlite-editor',
  };
  return jwt.sign(payload, privateKey, { algorithm: 'RS256' });
}

export async function loginWithToken(page: Page, token: string): Promise<void> {
  await page.goto('/login');
  await page.fill('#token', token);
  await page.click('button[type="submit"]');
  await page.waitForURL('/dashboard');
}
