import { defineConfig } from '@playwright/test';
import base from './playwright.config';
export default defineConfig({...base,testDir:'tests/docs',testMatch:'screenshots.capture.ts',workers:1,use:{...base.use,viewport:{width:1360,height:1000},reducedMotion:'reduce'}});
