# KEMBridge – Cross‑Chain **Swap** Extension Guide (Hackathon Add‑On)

### One Click. Any Chain. Quantum Safe.

> **Goal**   Показать, _зачем_ и _как_ за неделю превратить готовый квантозащищённый мост ETH ↔ NEAR в **cross‑chain swap DEX** (без выпуска \$KEMB).

---

## 0. Почему добавить Swap‑слой прямо на хакатоне?

| Критерий                      | Только Bridge                         | Bridge + Swap                                                     | Как усиливается квантовая защита & Risk‑AI                                            |
| ----------------------------- | ------------------------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| **Пользовательская ценность** | Перевод активов между сетями.         | "Один клик — ETH → USDC на другой сети".                          | Рыночные данные (slippage, котировка) шифруются KEM‑подписями: MITM не подменит цену. |
| **Дифференциация**            | Мостов много, даже с фишками.         | Первый quantum‑safe **cross‑chain DEX** с AI‑риск‑менеджером.     | Risk Engine теперь использует волатильность пары и gas‑price для real‑time scoring.   |
| **Monetisation**              | <0.1 % bridge‑fee.                    | Spread‑fee + партнёрские рефы 1inch, объём ×5–10.                 | Появляется понятная выручка → токеномика \$KEMB (пост‑MVP).                           |
| **Demo Wow‑factor**           | FSM, timeout, KEM – но «чёрный ящик». | Зрелищная цепочка: lock → swap → unlock с живым графом прогресса. | Показывает extensibility архитектуры FSM.                                             |
| **Риск демонстрации**         | Низкий.                               | Средний, но снижается фичефлагом `ENABLE_SWAP`.                   | Bridge остаётся работоспособным, даже если Fusion API упадёт.                         |

➡ **Вывод:** +5 дней работы дают \~15 % прироста баллов за _Innovation_ и _Product‑Market Fit_ при минимальном риске.

---

## 1. TL;DR — объём работ

1. Интегрировать **1inch Fusion+ SDK** на обеих сетях.
2. Добавить ветку FSM `Lock → SwapOut → Unlock`.
3. Реализовать эндпоинты `/api/swap/quote` и `/api/swap`.
4. Сделать форму **SwapForm** + историю «Swaps».
5. Токены описываются в БД → новая пара = скрипт + иконка.

⌚ **Backend ≈ 3 дн., Frontend ≈ 1 д., QA ≈ 0.5 д.**

---

## 2. Architecture Delta

```
User → Frontend (SwapForm)
      ↓ REST
Backend  ──┬── BridgeService  (Phase 4.3)
           ├── PriceOracleService   ← NEW (6.1)
           ├── OneinchAdapter      ← NEW (6.2)
           └── SwapOrchestrator    ← NEW (6.3)

↳ Chain A (ETH)  lock()
↳ Chain B (NEAR) mint() → 1inch.swap() → burn()/unlock()
```

**Новые компоненты**

- **OneinchAdapter** — гейт к Fusion API.
- **PriceOracleService** — USD‑цена через Chainlink / TWAP.
- **SwapOrchestrator** — управляет веткой FSM и откатами.

---

## 3. Backend Work Breakdown

### 3.1 Dependencies

```bash
npm i @1inch/fusion-sdk @chainlink/contracts --save
```

Env‑vars:

```text
ONEINCH_API="https://api.1inch.dev/fusion/"
CHAINLINK_FEED_ETH_USD="0x..."
```

### 3.2 Новые сервисы

| 🚩  | Package           | Responsibility                    |
| --- | ----------------- | --------------------------------- |
| 🧩  | `services/price`  | `getUsdPrice(chainId, token)`     |
| 🛡️  | `services/swap`   | `quoteSwap()` / `executeSwap()`   |
| 🔄  | `fsm/transitions` | `Lock → Quote → SwapOut → Unlock` |

### 3.3 REST API

| Verb | Path              | Body                                           | Response                      |
| ---- | ----------------- | ---------------------------------------------- | ----------------------------- |
| GET  | `/api/swap/quote` | `fromToken, toToken, amount`                   | `{ minOut, slippage, route }` |
| POST | `/api/swap`       | `fromToken, toToken, amount, minOut, deadline` | `{ txHash, status }`          |

### 3.4 DB Migration `V6__token_meta.sql`

```sql
CREATE TABLE token_meta (
  id SERIAL PRIMARY KEY,
  chain_id INT,
  address  VARCHAR(42),
  symbol   VARCHAR(16),
  decimals SMALLINT,
  icon_url TEXT,
  CONSTRAINT u_token UNIQUE(chain_id, address)
);
```

---

## 4. Smart‑Contract Notes

- HTLC не нужен — используем атомарность моста + Fusion.
- В NEAR‑контракт добавить `swap_callback()` для пост‑обработки.

---

## 5. Frontend Tasks

1. **SwapForm.tsx** (TokenSelect, Amount, "Min received").
2. **History > Swaps** таб (reuse BridgeHistory).
3. Reuse `TxProgressModal` для статуса.

---

## 6. DevOps / CI

- Новые env‑vars в `docker-compose.override.yml`.
- Job `swap_e2e` в GitHub CI.
- Фичефлаг `ENABLE_SWAP` (env) ↔ toggler в Helm.

---

## 7. Testing Checklist

- Unit: `quoteSwap()` ≥ 0.
- FSM: happy / rollback.
- E2E: ETH → USDC (Sepolia↔Testnet) ≤ 2 мин, slippage ≤ 1 %.

---

## 8. Добавление новой пары

1. Проверить, что торгуется на 1inch.
2. `INSERT INTO token_meta ...` + иконка.
3. `pnpm run sync-tokens`.
4. `pnpm run test:swap --pair=NEW/USDC`.

> ⏱ ≈ 45 мин. под PR.

---

## 9. Rollback Strategy

- `ENABLE_SWAP=false` → эндпоинты выключены, Bridge живёт.
- Ошибки пишутся в `swap_status` и видно в UI.

---

## 10. Suggested Timeline

| День | Задача                              |
| ---- | ----------------------------------- |
| 1    | OneinchAdapter + PriceOracleService |
| 2    | SwapOrchestrator + FSM              |
| 3    | REST + миграция + unit‑tests        |
| 4    | SwapForm + History + e2e            |
| 5    | Buffer / QA / Demo‑script           |

---

## 11. Acceptance Criteria

- ✅ Swap ETH ↔ USDC Sepolia ↔ NEAR Testnet ≤ 2 мин, slippage ≤ 1 %.
- ✅ Rollback при цене < minOut.
- ✅ Поддержка ≥ 5 новых токенов без кода.

---
