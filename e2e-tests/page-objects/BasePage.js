/**
 * Base Page Object Model for KEMBridge E2E Tests
 * Provides common functionality for all page objects
 */

export class BasePage {
  constructor(page) {
    this.page = page;
  }

  // Navigation helpers
  async goto(url) {
    await this.page.goto(url);
    await this.waitForPageLoad();
  }

  async waitForPageLoad() {
    await this.page.waitForLoadState('networkidle');
  }

  // Element helpers
  async waitForElement(selector, timeout = 10000) {
    return await this.page.waitForSelector(selector, { timeout });
  }

  async waitForText(text, timeout = 10000) {
    return await this.page.waitForSelector(`text=${text}`, { timeout });
  }

  async clickElement(selector) {
    await this.page.click(selector);
  }

  async fillInput(selector, value) {
    await this.page.fill(selector, value);
  }

  async getText(selector) {
    return await this.page.textContent(selector);
  }

  async isVisible(selector) {
    return await this.page.isVisible(selector);
  }

  // Common actions
  async screenshot(name) {
    await this.page.screenshot({ path: `screenshots/${name}.png` });
  }

  async waitForTimeout(ms) {
    await this.page.waitForTimeout(ms);
  }

  // Error handling helpers
  async handleDialog(accept = true, promptText = '') {
    this.page.on('dialog', async dialog => {
      if (accept) {
        await dialog.accept(promptText);
      } else {
        await dialog.dismiss();
      }
    });
  }

  async interceptNetworkRequests(urlPattern, response) {
    await this.page.route(urlPattern, route => {
      route.fulfill(response);
    });
  }

  async clearNetworkInterceptions() {
    await this.page.unroute('**/*');
  }
}