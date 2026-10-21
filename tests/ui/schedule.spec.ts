import { test, expect } from '@playwright/test';

test('schedule preview validates duplicate boundaries and timezone before showing transitions', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: /Backgrounds/i }).click();
  const panel = page.locator('.schedule-panel');
  await expect(panel.getByRole('heading', { name: 'Let the TV change with the day' })).toBeVisible();
  await expect(panel.getByText(/Preview only · no TV write is available/)).toBeVisible();

  await page.locator('#rule-1-kind').selectOption('fixed');
  await page.locator('#rule-1-time').fill('07:00');
  await expect(panel.getByRole('alert').filter({ hasText: 'Duplicate fixed-time boundary with rule 2.' })).toBeVisible();
  await expect(panel.getByRole('alert').filter({ hasText: 'Duplicate fixed-time boundary with rule 1.' })).toBeVisible();
  await expect(panel.getByRole('button', { name: 'Preview upcoming changes' })).toBeDisabled();

  await page.locator('#rule-1-time').fill('20:00');
  await page.locator('#schedule-timezone').fill('Not/AZone');
  await expect(panel.getByRole('alert').filter({ hasText: 'valid IANA timezone' })).toBeVisible();
  await page.locator('#schedule-timezone').fill('Europe/London');
  await expect(panel.getByRole('button', { name: 'Preview upcoming changes' })).toBeEnabled();

  await panel.getByRole('button', { name: 'Preview upcoming changes' }).click();
  await expect(panel.getByRole('heading', { name: /Next changes/ })).toBeVisible();
  await expect(panel.getByText(/All times use Europe\/London/)).toBeVisible();
  expect(await page.evaluate(() => '__TAURI_INTERNALS__' in window)).toBe(false);
});

test('schedule rules can be added and removed with keyboard-accessible controls', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: /Backgrounds/i }).click();
  const panel = page.locator('.schedule-panel');
  await expect(panel.getByRole('heading', { name: 'Rule 2' })).toBeVisible();
  await panel.getByRole('button', { name: 'Add daily rule' }).click();
  await expect(panel.getByRole('heading', { name: 'Rule 3' })).toBeVisible();
  const remove = panel.getByRole('button', { name: 'Remove rule 3' });
  await remove.focus();
  await page.keyboard.press('Enter');
  await expect(panel.getByRole('heading', { name: 'Rule 3' })).toHaveCount(0);
  await expect(panel.getByText('2 of 24')).toBeVisible();
});
