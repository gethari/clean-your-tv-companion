import { defineConfig } from '@playwright/test';
export default defineConfig({testDir:'tests/ui',use:{baseURL:'http://127.0.0.1:1420',headless:true},webServer:{command:'pnpm dev',url:'http://127.0.0.1:1420',reuseExistingServer:!process.env.CI},reporter:'list'});
