# H4. Интеграция реальных Bridge контрактов

## 🎯 Цель

Заменить мок-реализации bridge контрактов на реальные Ethereum контракты для более впечатляющего демо с блокчейн взаимодействием.

## 📋 Анализ текущего состояния

### Проблемы

- Методы `lock_eth_tokens()` и `unlock_eth_tokens()` в EthereumAdapter используют мок-реализации
- Отсутствуют реальные bridge контракты на Sepolia testnet
- Нет ABI для bridge контрактов
- Отсутствует генерация proof транзакций

### Файлы для изменения

- `backend/crates/kembridge-blockchain/src/ethereum/adapter.rs` - основной адаптер
- `backend/crates/kembridge-blockchain/src/ethereum/contracts.rs` - контракты
- `backend/src/constants.rs` - адреса контрактов
- Возможно новые файлы для ABI и контрактов

## 🔧 План реализации

### Фаза 1: Анализ и подготовка

1. Изучить существующую архитектуру blockchain адаптеров
2. Проверить зависимости и структуру проекта
3. Убедиться что проект собирается

### Фаза 2: Простые bridge контракты

1. Создать минимальные Solidity контракты для lock/unlock
2. Деплоить контракты на Sepolia testnet
3. Получить реальные адреса контрактов
4. Добавить ABI в код

### Фаза 3: Интеграция с кодом

1. Заменить мок-адреса на реальные в constants.rs
2. Обновить EthereumAdapter для работы с реальными контрактами
3. Реализовать реальные методы lock/unlock
4. Интегрировать с quantum crypto

### Фаза 4: Тестирование и валидация

1. Протестировать с небольшими суммами ETH
2. Проверить lock/unlock операции
3. Убедиться что транзакции попадают в блокчейн
4. Добавить базовую генерацию proof

## 📊 Ожидаемые результаты

### Минимальные требования для demo

- ✅ Реальные bridge контракты на Sepolia testnet
- ✅ Замена мок-реализаций на реальные вызовы контрактов
- ✅ Транзакции ETH lock/unlock в реальном блокчейне
- ✅ Базовая генерация proof транзакций

### Nice-to-have (если время позволит)

- Event listeners для автоматического отслеживания
- Расширенная валидация proof
- Интеграция с NEAR контрактами

## 🔗 Зависимости

### Требует (должно быть выполнено ранее)

- ✅ Phase 4.1: Ethereum Adapter базовая реализация
- ✅ Phase 4.3: BridgeService интеграция
- ✅ Phase 3: Quantum crypto система
- ✅ Sepolia testnet RPC конфигурация

### Блокирует (зависит от этого пункта)

- H5: NEAR Bridge контракты (может использовать похожий подход)
- Advanced proof generation (можно реализовать позже)
- Production deployment (требует mainnet контракты)

## 🚨 Ограничения

### Что НЕ реализуем в этой фазе

- Сложные multi-sig контракты (оставляем для P-фазы)
- Интеграция с реальными NEAR контрактами (H5)
- Полная proof validation система (P-фаза)

### Причины ограничений

- Фокус на Ethereum части для demo
- Время ограничено для хакатона
- NEAR интеграция требует отдельной реализации

## 🎪 Demo сценарий

После реализации должен работать следующий поток:

1. Пользователь инициирует bridge операцию ETH → NEAR
2. EthereumAdapter вызывает реальный bridge контракт на Sepolia
3. Транзакция lock создается в реальном блокчейне
4. Система получает реальный transaction hash
5. Proof генерируется на основе реального блокчейна
6. Unlock операция (пока мок, но с реальными данными)

## 🔧 Технические детали

### Простой Bridge контракт

```solidity
// Минимальный пример для demo
contract SimpleBridge {
    mapping(bytes32 => bool) public processedHashes;

    event TokensLocked(address indexed user, uint256 amount, string recipientChain, bytes32 quantumHash);
    event TokensUnlocked(address indexed user, uint256 amount, bytes32 quantumHash);

    function lockTokens(string memory recipientChain, bytes32 quantumHash) external payable;
    function unlockTokens(address recipient, uint256 amount, bytes32 quantumHash) external;
}
```

### Интеграция с EthereumAdapter

```rust
// Пример реального вызова контракта
pub async fn lock_eth_tokens(&self, ...) -> Result<H256, EthereumError> {
    let contract = self.get_bridge_contract().await?;
    let tx = contract.method("lockTokens", (recipient_chain, quantum_hash))?
        .value(amount)
        .send()
        .await?;
    Ok(tx.tx_hash())
}
```

Это обеспечит более реалистичную демонстрацию возможностей KEMBridge на хакатоне.

**Критический путь**: Деплой контрактов → интеграция → тестирование
