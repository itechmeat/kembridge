# KEMBridge NEAR Contract Development Plan

## Цель

Превратить простой HelloContract в полноценный bridge контракт как описано в FUNC.md

## Флоу реализации

```
1. Код функциональности → 2. Build → 3. Deploy → 4. Test → 5. Фикс багов → повтор 2-4
```

**ВАЖНО: ПО 1 МЕТОДУ ЗА РАЗ!**

**Для каждого метода:**

1. ✍️ Пишем код ОДНОГО метода
2. 🔍 ИСПРАВЛЯЕМ ВСЕ ЯВНЫЕ ОШИБКИ В КОДЕ (импорты, синтаксис и т.д.)
3. 🔨 `make build` - проверяем компиляцию
4. 🚀 `make deploy` - деплоим на testnet
5. 🧪 Создаем unit-тесты и добавляем интеграционные тесты в `scripts/test-contract.sh`
6. 🧪 Запускаем `make test` для проверки всей функциональности
7. 🐛 Если есть ошибки - фиксим и повторяем с шага 3
8. ✅ Переходим к следующему методу

**КРИТИЧЕСКИ ВАЖНО:**

- ЛЮБОЕ изменение кода (даже исправление тестов) требует build + deploy!
- НЕ МЕНЯТЬ КОД без последующего build + deploy!
- Build должен быть БЕЗ WARNINGS! Исправлять все предупреждения!
- При добавлении нового метода ОБЯЗАТЕЛЬНО добавить тест в `scripts/test-contract.sh`!

**НЕ ДЕЛАТЬ СРАЗУ МНОГО МЕТОДОВ!!!**

**ВАЖНО: При изменении структуры контракта:**

- Если структура несовместима со старым состоянием, удалить старое состояние или развернуть на новом аккаунте
- Записать в инструкцию как правильно удалять только контракт (не аккаунт)

**ВАЖНО: При добавлении тестов в `scripts/test-contract.sh`:**

- Для view методов использовать `run_view_test "Test Name" "command" "expected_pattern"`
- Для transaction методов использовать `run_test "Test Name" "command" expected_exit_code`
- Добавлять тесты в логических группах (═══ Group Name ═══)
- Проверять `make test` после каждого добавления нового теста

## Этапы реализации

### ✅ Этап 0: Подготовка (ЗАВЕРШЕН)

- [x] Простой HelloContract
- [x] Rust 1.86.0 setup
- [x] Build/deploy pipeline

### 🔄 Этап 1: Базовая структура и конфигурация (ТЕКУЩИЙ)

**Что делаем:**

- [ ] Создать основную структуру BridgeContract
- [ ] Добавить базовую конфигурацию (min/max amounts, fees)
- [ ] Добавить owner/admin функциональность
- [ ] Добавить паузу контракта

**Методы:**

- `new(owner)` - инициализация с владельцем
- `get_config()` - получить конфигурацию
- `update_config()` - обновить конфигурацию (только owner)
- `set_paused()` - пауза/снятие паузы (только owner)

### 📊 Этап 2: Статистика и просмотр

**Что делаем:**

- [ ] Добавить bridge statistics (total locked/unlocked)
- [ ] Добавить view методы для баланса пользователей
- [ ] Добавить проверку статуса транзакций

**Методы:**

- `get_bridge_stats()` - общая статистика
- `get_locked_balance(account)` - баланс пользователя
- `is_eth_tx_processed(eth_tx_hash)` - проверка обработки Ethereum транзакции

### 🔒 Этап 3: Lock операции ✅

**Что делаем:**

- [x] Реализовать lock_tokens функциональность
- [x] Добавить события для lock операций
- [x] Добавить проверки лимитов и fees

**Методы:**

- `lock_tokens(eth_recipient, quantum_hash)` - заблокировать токены ✅
- Проверки: minimum amount, maximum amount, fees ✅ (`validate_amount`, `calculate_fee`)
- События: LockEvent ✅ (JSON логи)

### 🔓 Этап 4: Unlock операции ✅

**Что делаем:**

- [x] Реализовать unlock_tokens
- [x] Добавить replay protection (отслеживание eth_tx_hash)
- [x] Добавить события для unlock

**Методы:**

- `unlock_tokens(amount, near_recipient, eth_tx_hash, quantum_hash)` - разблокировать токены ✅
- Replay protection через IterableSet<String> ✅ (enhanced `processed_eth_txs`)
- События: UnlockEvent ✅ (JSON логи)
- `mark_eth_tx_processed(eth_tx_hash)` - помечать ETH транзакции ✅

### 🪙 Этап 5: Mint/Burn операции ✅

**Что делаем:**

- [x] Реализовать mint_tokens
- [x] Реализовать burn_tokens
- [x] Полная интеграция с quantum hashes

**Методы:**

- `mint_tokens(recipient, amount, eth_tx_hash, quantum_hash)` - создать wrapped токены ✅
- `burn_tokens(amount, eth_recipient, quantum_hash)` - сжечь wrapped токены ✅ (payable)
- События: `MintEvent`, `BurnEvent` ✅ (JSON логи)
- Replay protection для mint операций ✅

### 🛡️ Этап 6: Безопасность и админка ✅

**Что делаем:**

- [x] Emergency withdraw функциональность
- [x] Transfer ownership (уже был реализован)
- [x] Финальные проверки безопасности

**Методы:**

- `emergency_withdraw(amount)` - экстренный вывод (только owner) ✅
- `transfer_ownership(new_owner)` - передача владения ✅ (уже был)
- `get_contract_balance()` - просмотр баланса контракта ✅
- `get_total_bridge_volume()` - общий объем bridge операций ✅
- События: `EmergencyWithdrawEvent` ✅ (JSON логи)

## Структуры данных

```rust
#[near(contract_state)]
pub struct BridgeContract {
    owner: AccountId,
    paused: bool,
    config: BridgeConfig,
    bridge_stats: BridgeStats,
    user_balances: UnorderedMap<AccountId, Balance>,
    processed_eth_txs: UnorderedSet<String>,
    transactions: Vector<Transaction>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct BridgeConfig {
    min_bridge_amount: Balance,
    max_bridge_amount: Balance,
    bridge_fee_bp: u16, // basis points (100 = 1%)
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct BridgeStats {
    total_locked: Balance,
    total_unlocked: Balance,
    total_minted: Balance,
    total_burned: Balance,
}
```

## Текущий прогресс

- **Этап 0**: ✅ Завершен (Базовая структура)
- **Этап 1**: ✅ Завершен (Конфигурация и админка)
- **Этап 2**: ✅ Завершен (Статистика и просмотр)
- **Этап 3**: ✅ Завершен (Lock операции)
- **Этап 4**: ✅ Завершен (Unlock операции)
- **Этап 5**: ✅ Завершен (Mint/Burn операции)
- **Этап 6**: ✅ Завершен (Безопасность и админка)

**🎉 ВСЕ ЭТАПЫ ЗАВЕРШЕНЫ! 🎉**

## Заметки

- Каждый этап тестируется перед переходом к следующему
- При ошибках откатываемся и фиксим
- Сохраняем работающую версию на каждом этапе
