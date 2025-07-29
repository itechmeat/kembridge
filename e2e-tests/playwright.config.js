import { defineConfig, devices } from '@playwright/test';

/**
 * KEMBridge E2E Test Configuration
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests',
  timeout: 60000, // 60 seconds per test
  expect: {
    timeout: 10000, // 10 seconds for assertions
  },
  fullyParallel: false, // Run tests sequentially for wallet interactions
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker for wallet tests
  reporter: [
    ['html'],
    ['list'],
    ['json', { outputFile: 'test-results.json' }]
  ],
  
  use: {
    // Base URL for testing
    baseURL: 'http://localhost:4100',
    
    // Browser settings
    headless: process.env.CI ? true : false,
    viewport: { width: 1280, height: 720 },
    
    // Screenshots and videos
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    
    // Timeouts
    actionTimeout: 10000,
    navigationTimeout: 30000,
    
    // Trace collection
    trace: 'on-first-retry',
  },

  // Test projects for different scenarios
  projects: [
    {
      name: 'chromium',
      use: { 
        ...devices['Desktop Chrome'],
        // Chrome with wallet simulation
        launchOptions: {
          args: [
            '--disable-web-security',
            '--disable-features=VizDisplayCompositor'
          ]
        }
      },
    },
  ],

  // Local dev server (if needed)
  webServer: {
    command: 'make health-quick',
    port: 4100,
    reuseExistingServer: !process.env.CI,
    timeout: 30000,
  },
});