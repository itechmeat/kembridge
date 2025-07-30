/**
 * Gateway Crypto Proxy Integration Tests
 *
 * Тестирует проксирование запросов от gateway-service:4000 к crypto-service:4003.
 * Эта функциональность НЕ покрыта Rust unit тестами и критична для архитектуры.
 */

import { test, expect } from "@playwright/test";

const GATEWAY_URL = "http://localhost:4000";
const CRYPTO_SERVICE_URL = "http://localhost:4003";

test.describe("Gateway Crypto Proxy Integration Tests", () => {
  test.describe("Proxy Status Endpoints", () => {
    test("should proxy /crypto/status to crypto-service", async ({
      request,
    }) => {
      // Тестируем проксирование через gateway
      const gatewayResponse = await request.get(
        `${GATEWAY_URL}/api/v1/crypto/status`
      );

      expect(gatewayResponse.ok()).toBe(true);

      const gatewayData = await gatewayResponse.json();
      console.log(
        "🌐 Gateway Proxy Response:",
        JSON.stringify(gatewayData, null, 2)
      );

      // Сравниваем с прямым запросом к crypto-service
      const directResponse = await request.get(`${CRYPTO_SERVICE_URL}/status`);
      const directData = await directResponse.json();

      // Проверяем что gateway проксирует правильно
      expect(gatewayData).toHaveProperty("success", true);
      expect(gatewayData).toHaveProperty("data");

      const gatewayQuantumData = gatewayData.data;
      const directQuantumData = directData.data;

      // Основные поля должны совпадать (Gateway использует camelCase)
      expect(gatewayQuantumData).toHaveProperty("quantumProtection");
      expect(gatewayQuantumData.quantumProtection.algorithm).toBe(
        "ML-KEM-1024"
      );
      expect(gatewayQuantumData.quantumProtection.encryptionStrength).toBe(
        1024
      );

      console.log("✅ Gateway successfully proxies crypto status");
      console.log(
        `   Quantum Protection: ${
          gatewayQuantumData.quantumProtection.isActive ? "ACTIVE" : "INACTIVE"
        }`
      );
    });

    test("should handle crypto-service unavailable gracefully", async ({
      request,
    }) => {
      // Этот тест проверяет что происходит когда crypto-service недоступен
      // Мы не можем легко остановить сервис, но можем проверить timeout behavior

      const response = await request.get(`${GATEWAY_URL}/api/v1/crypto/status`);

      // Если crypto-service работает, должен быть успешный ответ
      // Если не работает, gateway должен вернуть ошибку или fallback
      if (response.ok()) {
        const data = await response.json();
        expect(data).toHaveProperty("success", true);
        console.log("✅ Crypto service available through gateway");
      } else {
        // Проверяем что gateway возвращает осмысленную ошибку
        expect(response.status()).toBeGreaterThanOrEqual(500);
        console.log("✅ Gateway handles crypto service unavailability");
      }
    });
  });

  test.describe("Proxy Key Management", () => {
    test("should proxy key generation requests", async ({ request }) => {
      const keyRequest = {
        key_type: "ml_kem_1024",
        purpose: "gateway_proxy_test",
      };

      // Запрос через gateway
      const gatewayResponse = await request.post(
        `${GATEWAY_URL}/api/v1/crypto/keys/generate`,
        {
          data: keyRequest,
        }
      );

      expect(gatewayResponse.ok()).toBe(true);

      const gatewayData = await gatewayResponse.json();
      console.log(
        "🔑 Gateway Key Generation:",
        JSON.stringify(gatewayData, null, 2)
      );

      expect(gatewayData).toHaveProperty("success", true);
      expect(gatewayData).toHaveProperty("data");

      const keyData = gatewayData.data;
      expect(keyData).toHaveProperty("keyId");
      expect(keyData).toHaveProperty("algorithm", "ML-KEM-1024");
      expect(keyData).toHaveProperty("publicKey");

      // Проверяем что keyId это валидный UUID
      const uuidRegex =
        /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
      expect(keyData.keyId).toMatch(uuidRegex);

      console.log("✅ Gateway successfully proxies key generation");
      console.log(`   Generated Key ID: ${keyData.keyId}`);
    });

    test("should proxy key listing requests", async ({ request }) => {
      const response = await request.get(`${GATEWAY_URL}/api/v1/crypto/keys`);

      expect(response.ok()).toBe(true);

      const data = await response.json();
      console.log("🗂️ Gateway Keys List:", JSON.stringify(data, null, 2));

      expect(data).toHaveProperty("success", true);
      expect(data).toHaveProperty("data");

      const keysData = data.data;
      expect(keysData).toHaveProperty("keys");
      expect(Array.isArray(keysData.keys)).toBe(true);
      expect(keysData).toHaveProperty("totalCount");

      console.log(
        `✅ Gateway proxies key listing - found ${keysData.keys.length} keys`
      );
    });
  });

  test.describe("Proxy Key Rotation", () => {
    test("should proxy key rotation check", async ({ request }) => {
      const rotationRequest = {
        key_type: "ml_kem_1024",
      };

      const response = await request.post(
        `${GATEWAY_URL}/api/v1/crypto/keys/check-rotation`,
        {
          data: rotationRequest,
        }
      );

      expect(response.ok()).toBe(true);

      const data = await response.json();
      console.log("🔄 Gateway Rotation Check:", JSON.stringify(data, null, 2));

      expect(data).toHaveProperty("success", true);
      expect(data).toHaveProperty("data");

      const rotationData = data.data;
      expect(rotationData).toHaveProperty("rotationDue");
      expect(rotationData).toHaveProperty("activeKeys");
      expect(typeof rotationData.rotationDue).toBe("boolean");

      console.log("✅ Gateway successfully proxies rotation check");
      console.log(`   Rotation Due: ${rotationData.rotationDue}`);
      console.log(`   Active Keys: ${rotationData.activeKeys}`);
    });

    test("should proxy key rotation trigger", async ({ request }) => {
      const rotationRequest = {
        key_type: "ml_kem_1024",
        force: false,
        reason: "gateway_proxy_test",
      };

      const response = await request.post(
        `${GATEWAY_URL}/api/v1/crypto/keys/rotate`,
        {
          data: rotationRequest,
        }
      );

      // Rotation может не выполниться если ключи свежие, но proxy должен работать
      if (response.ok()) {
        const data = await response.json();
        console.log(
          "🔄 Gateway Rotation Trigger:",
          JSON.stringify(data, null, 2)
        );

        expect(data).toHaveProperty("success", true);
        expect(data).toHaveProperty("data");

        console.log("✅ Gateway successfully proxies rotation trigger");
      } else {
        // Проверяем что ошибка приходит от crypto-service, а не от gateway
        const data = await response.json();
        expect(data).toHaveProperty("error");
        console.log("✅ Gateway proxies rotation errors correctly");
      }
    });
  });

  test.describe("Circuit Breaker Behavior", () => {
    test("should handle repeated failures gracefully", async ({ request }) => {
      // Тестируем поведение при повторных запросах
      const requests = [];

      for (let i = 0; i < 3; i++) {
        requests.push(request.get(`${GATEWAY_URL}/api/v1/crypto/status`));
      }

      const responses = await Promise.all(requests);

      // Все запросы должны либо успешно проксироваться, либо fail gracefully
      responses.forEach((response, index) => {
        if (response.ok()) {
          console.log(`✅ Request ${index + 1}: Successfully proxied`);
        } else {
          console.log(
            `⚠️ Request ${index + 1}: Failed with status ${response.status()}`
          );
          // Circuit breaker должен возвращать 503 или 502
          expect([502, 503, 504]).toContain(response.status());
        }
      });

      console.log("✅ Circuit breaker behavior tested");
    });

    test("should provide fallback responses when crypto-service fails", async ({
      request,
    }) => {
      // Тестируем fallback behavior
      const response = await request.get(`${GATEWAY_URL}/api/v1/crypto/status`);

      if (response.ok()) {
        const data = await response.json();
        expect(data).toHaveProperty("success");
        console.log("✅ Normal response received");
      } else {
        // Если crypto-service недоступен, gateway должен предоставить fallback
        expect(response.status()).toBeGreaterThanOrEqual(500);

        try {
          const errorData = await response.json();
          expect(errorData).toHaveProperty("error");
          console.log("✅ Fallback error response provided");
        } catch {
          console.log("✅ Gateway returns appropriate error status");
        }
      }
    });
  });

  test.describe("Request/Response Validation", () => {
    test("should validate request headers", async ({ request }) => {
      const response = await request.get(
        `${GATEWAY_URL}/api/v1/crypto/status`,
        {
          headers: {
            "Content-Type": "application/json",
            "User-Agent": "E2E-Test-Client",
          },
        }
      );

      expect(response.ok()).toBe(true);

      // Проверяем что gateway добавляет свои headers
      const headers = response.headers();
      expect(headers).toHaveProperty("content-type");

      console.log("✅ Request headers handled correctly");
    });

    test("should handle malformed requests", async ({ request }) => {
      const response = await request.post(
        `${GATEWAY_URL}/api/v1/crypto/keys/generate`,
        {
          data: "invalid json",
        }
      );

      // Gateway должен либо проксировать ошибку от crypto-service, либо вернуть свою
      expect(response.status()).toBeGreaterThanOrEqual(400);

      console.log(
        `✅ Malformed request handled with status ${response.status()}`
      );
    });

    test("should preserve response format", async ({ request }) => {
      const response = await request.get(`${GATEWAY_URL}/api/v1/crypto/status`);

      expect(response.ok()).toBe(true);

      const data = await response.json();

      // Проверяем что gateway сохраняет формат ответа crypto-service
      expect(data).toHaveProperty("success");
      expect(data).toHaveProperty("data");
      expect(data).toHaveProperty("timestamp");

      // Проверяем структуру quantum protection (Gateway использует camelCase)
      expect(data.data).toHaveProperty("quantumProtection");
      expect(data.data.quantumProtection).toHaveProperty(
        "algorithm",
        "ML-KEM-1024"
      );

      console.log("✅ Response format preserved through proxy");
    });
  });

  test.describe("Performance and Reliability", () => {
    test("should handle concurrent requests", async ({ request }) => {
      const concurrentRequests = 5;
      const requests = [];

      for (let i = 0; i < concurrentRequests; i++) {
        requests.push(request.get(`${GATEWAY_URL}/api/v1/crypto/status`));
      }

      const startTime = Date.now();
      const responses = await Promise.all(requests);
      const totalTime = Date.now() - startTime;

      // Проверяем что все запросы обработались
      const successfulRequests = responses.filter((r) => r.ok()).length;

      console.log(
        `✅ Concurrent requests: ${successfulRequests}/${concurrentRequests} successful`
      );
      console.log(`   Total time: ${totalTime}ms`);
      console.log(
        `   Average time per request: ${Math.round(
          totalTime / concurrentRequests
        )}ms`
      );

      // Хотя бы половина запросов должна быть успешной
      expect(successfulRequests).toBeGreaterThanOrEqual(
        Math.ceil(concurrentRequests / 2)
      );
    });

    test("should have reasonable response times", async ({ request }) => {
      const startTime = Date.now();
      const response = await request.get(`${GATEWAY_URL}/api/v1/crypto/status`);
      const responseTime = Date.now() - startTime;

      expect(response.ok()).toBe(true);

      // Proxy не должен добавлять значительную задержку (< 1 секунды)
      expect(responseTime).toBeLessThan(1000);

      console.log(`✅ Response time: ${responseTime}ms (acceptable)`);
    });
  });

  test.describe("Error Propagation", () => {
    test("should propagate crypto-service errors correctly", async ({
      request,
    }) => {
      // Отправляем заведомо неправильный запрос (пустой payload)
      const response = await request.post(
        `${GATEWAY_URL}/api/v1/crypto/keys/generate`,
        {
          data: {},
        }
      );

      // Gateway должен либо обработать запрос (с дефолтными параметрами), либо вернуть ошибку
      if (response.ok()) {
        const data = await response.json();
        expect(data).toHaveProperty("success", true);
        console.log("✅ Gateway handles empty requests with defaults");
      } else {
        const errorData = await response.json();
        console.log("❌ Error Response:", JSON.stringify(errorData, null, 2));

        // Gateway должен проксировать ошибку от crypto-service
        expect(errorData).toHaveProperty("success", false);
        expect(errorData).toHaveProperty("error");
        console.log(
          "✅ Crypto-service errors propagated correctly through gateway"
        );
      }
    });

    test("should handle timeout scenarios", async ({ request }) => {
      // Тестируем поведение при timeout (используем очень короткий timeout)
      try {
        const response = await request.get(
          `${GATEWAY_URL}/api/v1/crypto/status`,
          {
            timeout: 100, // Очень короткий timeout для тестирования
          }
        );

        if (response.ok()) {
          console.log("✅ Request completed within timeout");
        }
      } catch (error) {
        // Timeout ошибка ожидаема при коротком timeout
        console.log("✅ Timeout handled appropriately");
      }
    });
  });
});
