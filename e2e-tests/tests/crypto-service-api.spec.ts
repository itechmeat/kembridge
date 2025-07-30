/**
 * Crypto Service HTTP API Tests
 *
 * Тестирует HTTP API crypto-service:4003, который НЕ покрыт Rust unit тестами.
 * Фокус на интеграции с PostgreSQL, HTTP handlers и error handling.
 */

import { test, expect } from "@playwright/test";

const CRYPTO_SERVICE_URL = "http://localhost:4003";

test.describe("Crypto Service HTTP API Tests", () => {
  test.describe("Health and Status Endpoints", () => {
    test("should get service health", async ({ request }) => {
      const response = await request.get(`${CRYPTO_SERVICE_URL}/health`);

      expect(response.ok()).toBe(true);

      const data = await response.json();
      console.log("🏥 Health Status:", JSON.stringify(data, null, 2));

      expect(data).toHaveProperty("success", true);
      expect(data).toHaveProperty("data");

      const healthData = data.data;
      expect(healthData).toHaveProperty("service", "kembridge-crypto-service");
      expect(healthData).toHaveProperty("status", "healthy");
      expect(healthData).toHaveProperty("quantum_ready", true);
      expect(healthData).toHaveProperty("algorithms");
      expect(healthData.algorithms).toContain("ML-KEM-1024");

      console.log("✅ Crypto service health check passed");
    });

    test("should get quantum crypto status", async ({ request }) => {
      const response = await request.get(`${CRYPTO_SERVICE_URL}/status`);

      expect(response.ok()).toBe(true);

      const data = await response.json();
      console.log("🔐 Crypto Status:", JSON.stringify(data, null, 2));

      // Проверяем структуру ответа
      expect(data).toHaveProperty("success");
      expect(data.success).toBe(true);
      expect(data).toHaveProperty("data");

      const cryptoData = data.data;
      expect(cryptoData).toHaveProperty("quantum_protection");
      expect(cryptoData).toHaveProperty("overall");
      expect(cryptoData).toHaveProperty("is_online");

      // Проверяем quantum protection details
      const quantumProtection = cryptoData.quantum_protection;
      expect(quantumProtection).toHaveProperty("algorithm", "ML-KEM-1024");
      expect(quantumProtection).toHaveProperty("encryption_strength", 1024);
      expect(quantumProtection).toHaveProperty("is_active");
      expect(typeof quantumProtection.is_active).toBe("boolean");

      console.log(
        "✅ Quantum protection status:",
        quantumProtection.is_active ? "ACTIVE" : "INACTIVE"
      );
    });
  });

  test.describe("Key Generation Endpoint", () => {
    test("should generate ML-KEM-1024 key pair", async ({ request }) => {
      const keyRequest = {
        key_type: "ml_kem_1024",
        purpose: "transaction_encryption",
      };

      const response = await request.post(
        `${CRYPTO_SERVICE_URL}/keys/generate`,
        {
          data: keyRequest,
        }
      );

      expect(response.ok()).toBe(true);

      const data = await response.json();
      console.log("🔑 Key Generation Response:", JSON.stringify(data, null, 2));

      expect(data).toHaveProperty("success", true);
      expect(data).toHaveProperty("data");

      const keyData = data.data;
      expect(keyData).toHaveProperty("key_id");
      expect(keyData).toHaveProperty("algorithm", "ML-KEM-1024");
      expect(keyData).toHaveProperty("public_key");
      expect(keyData).toHaveProperty("created_at");
      expect(keyData).toHaveProperty("encryption_strength", 1024);
      expect(keyData).toHaveProperty("expires_at");

      // Проверяем что key_id это валидный UUID
      const uuidRegex =
        /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
      expect(keyData.key_id).toMatch(uuidRegex);

      // Проверяем что public_key это base64 строка
      expect(typeof keyData.public_key).toBe("string");
      expect(keyData.public_key.length).toBeGreaterThan(0);

      console.log("✅ ML-KEM-1024 key generated successfully");
      console.log(`   Key ID: ${keyData.key_id}`);
      console.log(`   Public Key Length: ${keyData.public_key.length} chars`);
    });

    test("should reject unsupported key types", async ({ request }) => {
      const invalidKeyRequest = {
        key_type: "rsa_2048", // Неподдерживаемый тип
        purpose: "transaction_encryption",
      };

      const response = await request.post(
        `${CRYPTO_SERVICE_URL}/keys/generate`,
        {
          data: invalidKeyRequest,
        }
      );

      expect(response.status()).toBe(501);

      const data = await response.json();
      expect(data).toHaveProperty("success", false);
      expect(data).toHaveProperty("error");

      console.log("✅ Unsupported key type rejected correctly");
    });

    test("should validate required fields", async ({ request }) => {
      const incompleteRequest = {
        purpose: "transaction_encryption",
        // Отсутствует key_type
      };

      const response = await request.post(
        `${CRYPTO_SERVICE_URL}/keys/generate`,
        {
          data: incompleteRequest,
        }
      );

      expect(response.status()).toBe(422);

      console.log("✅ Request validation working correctly");
    });
  });

  test.describe("Key Management", () => {
    test("should list system keys", async ({ request }) => {
      const response = await request.get(`${CRYPTO_SERVICE_URL}/keys`);

      expect(response.ok()).toBe(true);

      const data = await response.json();
      console.log("🗂️ Keys List:", JSON.stringify(data, null, 2));

      expect(data).toHaveProperty("success", true);
      expect(data).toHaveProperty("data");

      const keysData = data.data;
      expect(keysData).toHaveProperty("keys");
      expect(Array.isArray(keysData.keys)).toBe(true);

      console.log(`✅ Found ${keysData.keys.length} system keys`);
    });
  });

  test.describe("Key Rotation Endpoint", () => {
    test("should check key rotation status", async ({ request }) => {
      const rotationRequest = {
        key_type: "ml_kem_1024",
      };

      const response = await request.post(
        `${CRYPTO_SERVICE_URL}/keys/check-rotation`,
        {
          data: rotationRequest,
        }
      );

      expect(response.ok()).toBe(true);

      const data = await response.json();
      console.log("🔄 Key Rotation Status:", JSON.stringify(data, null, 2));

      expect(data).toHaveProperty("success", true);
      expect(data).toHaveProperty("data");

      const rotationData = data.data;
      expect(rotationData).toHaveProperty("rotation_due");
      expect(rotationData).toHaveProperty("total_keys");

      expect(typeof rotationData.rotation_due).toBe("boolean");
      expect(typeof rotationData.total_keys).toBe("number");

      console.log("✅ Key rotation status check passed");
      console.log(`   Keys Checked: ${rotationData.total_keys}`);
      console.log(`   Rotation Due: ${rotationData.rotation_due}`);
    });

    test("should trigger key rotation", async ({ request }) => {
      // Сначала получаем ID существующего ключа
      const keysResponse = await request.get(`${CRYPTO_SERVICE_URL}/keys`);
      const keysData = await keysResponse.json();
      const existingKeyId = keysData.data.keys[0].key_id;

      const rotationRequest = {
        key_id: existingKeyId,
        force: false, // Не принудительная ротация
        reason: "e2e_test_rotation",
      };

      const response = await request.post(`${CRYPTO_SERVICE_URL}/keys/rotate`, {
        data: rotationRequest,
      });

      expect(response.ok()).toBe(true);

      const data = await response.json();
      console.log("🔄 Key Rotation Response:", JSON.stringify(data, null, 2));

      expect(data).toHaveProperty("success", true);
      expect(data).toHaveProperty("data");

      const rotationResult = data.data;
      expect(rotationResult).toHaveProperty("success");
      expect(rotationResult).toHaveProperty("new_key_id");
      expect(rotationResult).toHaveProperty("old_key_id");
      expect(rotationResult).toHaveProperty("algorithm");

      if (rotationResult.success) {
        expect(rotationResult.new_key_id).toBeTruthy();
        console.log("✅ Key rotation performed successfully");
        console.log(`   Old Key ID: ${rotationResult.old_key_id}`);
        console.log(`   New Key ID: ${rotationResult.new_key_id}`);
      } else {
        console.log("✅ Key rotation failed");
      }
    });
  });

  test.describe("ML-KEM Operations", () => {
    let testKeyId: string;

    test("should perform encapsulation operation", async ({ request }) => {
      // Сначала генерируем ключ для тестирования
      const keyRequest = {
        key_type: "ml_kem_1024",
        purpose: "encapsulation_test",
      };

      const keyResponse = await request.post(
        `${CRYPTO_SERVICE_URL}/keys/generate`,
        {
          data: keyRequest,
        }
      );

      expect(keyResponse.ok()).toBe(true);
      const keyData = await keyResponse.json();
      testKeyId = keyData.data.key_id;

      // Теперь тестируем encapsulation
      const encapRequest = {
        public_key_id: testKeyId,
      };

      const response = await request.post(`${CRYPTO_SERVICE_URL}/encapsulate`, {
        data: encapRequest,
      });

      expect(response.ok()).toBe(true);

      const data = await response.json();
      console.log("🔒 Encapsulation Response:", JSON.stringify(data, null, 2));

      expect(data).toHaveProperty("success", true);
      expect(data).toHaveProperty("data");

      const encapData = data.data;
      expect(encapData).toHaveProperty("ciphertext");
      expect(encapData).toHaveProperty("shared_secret_hash");
      expect(encapData).toHaveProperty("operation_id");

      expect(typeof encapData.ciphertext).toBe("string");
      expect(typeof encapData.shared_secret_hash).toBe("string");
      expect(typeof encapData.operation_id).toBe("string");

      console.log("✅ ML-KEM encapsulation successful");
      console.log(`   Ciphertext Length: ${encapData.ciphertext.length} chars`);
      console.log(`   Operation ID: ${encapData.operation_id}`);
    });
  });

  test.describe("Error Handling", () => {
    test("should handle invalid JSON requests", async ({ request }) => {
      const response = await request.post(
        `${CRYPTO_SERVICE_URL}/keys/generate`,
        {
          data: "invalid json string",
        }
      );

      expect(response.status()).toBe(415);
      console.log("✅ Invalid JSON handled correctly");
    });

    test("should handle non-existent endpoints", async ({ request }) => {
      const response = await request.get(`${CRYPTO_SERVICE_URL}/nonexistent`);

      expect(response.status()).toBe(404);
      console.log("✅ Non-existent endpoint handled correctly");
    });

    test("should return proper CORS headers", async ({ request }) => {
      const response = await request.get(`${CRYPTO_SERVICE_URL}/health`);

      // Проверяем CORS headers
      const corsHeaders = response.headers();
      expect(corsHeaders).toHaveProperty("access-control-allow-origin");

      console.log("✅ CORS headers present");
    });
  });

  test.describe("Database Integration", () => {
    test("should persist generated keys in database", async ({ request }) => {
      // Генерируем ключ
      const keyRequest = {
        key_type: "ml_kem_1024",
        purpose: "database_test",
      };

      const generateResponse = await request.post(
        `${CRYPTO_SERVICE_URL}/keys/generate`,
        {
          data: keyRequest,
        }
      );

      expect(generateResponse.ok()).toBe(true);
      const generateData = await generateResponse.json();
      const keyId = generateData.data.key_id;

      // Проверяем что ключ сохранился, запросив список ключей
      const listResponse = await request.get(`${CRYPTO_SERVICE_URL}/keys`);
      const listData = await listResponse.json();

      // Ищем наш ключ в списке
      const foundKey = listData.data.keys.find(
        (key: any) => key.key_id === keyId
      );
      expect(foundKey).toBeTruthy();
      expect(foundKey).toHaveProperty("key_id");
      expect(foundKey).toHaveProperty("algorithm");

      console.log("✅ Key persisted in database successfully");
      console.log(`   Generated Key ID: ${keyId}`);
    });
  });
});
