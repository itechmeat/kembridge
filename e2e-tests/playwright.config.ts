import { defineConfig, devices } from '@playwright/test';

/**
 * KEMBridge E2E Test Configuration (TypeScript)
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
    ['json', { outputFile: 'test-results.json' }],
    ['junit', { outputFile: 'test-results.xml' }]
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
    trace: 'on-first-retry',
    
    // Timeouts
    actionTimeout: 10000,
    navigationTimeout: 30000,
    
    // Better error handling
    ignoreHTTPSErrors: true,
    
    // Extra HTTP headers
    extraHTTPHeaders: {
      'Accept-Language': 'en-US,en;q=0.9'
    }
  },

  // Test projects - ONLY Chromium (no cross-browser or mobile testing)
  projects: [
    {
      name: 'chromium',
      use: { 
        ...devices['Desktop Chrome'],
        launchOptions: {
          args: [
            '--disable-web-security',
            '--disable-features=VizDisplayCompositor',
            '--disable-background-timer-throttling',
            '--disable-renderer-backgrounding'
          ]
        }
      },
    }
  ],

  // Local dev server configuration
  webServer: {
    command: 'make health-quick',
    port: 4100,
    reuseExistingServer: !process.env.CI,
    timeout: 60000,
    env: {
      NODE_ENV: 'test'
    }
  },
  
  // Global setup and teardown (commented out until created)
  // globalSetup: require.resolve('./tests/global-setup.ts'),
  // globalTeardown: require.resolve('./tests/global-teardown.ts')
});