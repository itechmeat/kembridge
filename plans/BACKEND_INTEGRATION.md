# Frontend Backend Integration

Документация по интеграции фронтенда с бекендом KEMBridge.

## 🎯 Что реализовано

### 1. API Client Infrastructure

**Файлы:**
- `src/services/api/config.ts` - Конфигурация API endpoints и констант
- `src/services/api/apiClient.ts` - Централизованный Axios client с JWT токенами
- `src/services/api/index.ts` - Экспорт всех API сервисов

**Возможности:**
- Автоматическое добавление JWT токенов
- Обработка 401 ошибок с очисткой токенов
- Retry логика и error handling
- TypeScript типизация всех API

### 2. Authentication Services

**Файлы:**
- `src/services/api/authService.ts` - Web3 аутентификация
- `src/hooks/api/useAuth.ts` - React hooks для auth
- `src/components/auth/AuthManager/` - UI компонент аутентификации

**Flow:**
1. Получение nonce для wallet адреса
2. Подпись сообщения через MetaMask/NEAR
3. Верификация подписи на бекенде
4. Получение JWT токена и сохранение

### 3. User Management

**Файлы:**
- `src/services/api/userService.ts` - Управление профилем пользователя
- `src/hooks/api/useUser.ts` - React hooks для user data

**Возможности:**
- Получение профиля пользователя
- Обновление настроек (slippage, quantum protection)
- Risk profiling и tier management
- Wallet addresses management

### 4. Bridge Operations

**Файлы:**
- `src/services/api/bridgeService.ts` - Cross-chain bridge операции
- `src/hooks/api/useBridge.ts` - React hooks для bridge

**Возможности:**
- Получение котировок для свопов
- Инициация cross-chain транзакций
- Отслеживание статуса транзакций
- История операций пользователя

### 5. Real-time Monitoring

**Файлы:**
- `src/services/websocket/wsClient.ts` - WebSocket client
- `src/hooks/websocket/useWebSocket.ts` - React hooks для real-time

**Возможности:**
- Автоматическое подключение при аутентификации
- Отслеживание обновлений транзакций
- Risk alerts в реальном времени
- Системные уведомления

### 6. Updated UI Components

**Файлы:**
- `src/pages/WalletPage/WalletPage.tsx` - Главная страница с интеграцией
- `src/components/auth/AuthManager/` - Компонент аутентификации

**Возможности:**
- Onboarding flow с аутентификацией
- Отображение профиля пользователя
- Интеграция с backend данными
- Logout функциональность

## 🔧 Как использовать

### 1. Аутентификация

```typescript
import { useEthereumAuth, useNearAuth, useAuthStatus } from '@/hooks/api/useAuth';

function AuthComponent() {
  const { isAuthenticated } = useAuthStatus();
  const ethereumAuth = useEthereumAuth();
  const nearAuth = useNearAuth();

  const handleEthAuth = async () => {
    await ethereumAuth.authenticate();
  };

  return (
    <div>
      {!isAuthenticated ? (
        <button onClick={handleEthAuth}>
          Connect Ethereum Wallet
        </button>
      ) : (
        <p>Authenticated!</p>
      )}
    </div>
  );
}
```

### 2. Профиль пользователя

```typescript
import { useUserProfile, useUserTier } from '@/hooks/api/useUser';

function UserProfile() {
  const { data: profile, isLoading } = useUserProfile();
  const { tier, isPremium } = useUserTier();

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <h3>User Tier: {tier}</h3>
      <p>Wallets: {profile?.wallet_addresses.length}</p>
      {isPremium && <p>Premium features available!</p>}
    </div>
  );
}
```

### 3. Bridge операции

```typescript
import { useSwapQuote, useInitSwap, useTrackTransaction } from '@/hooks/api/useBridge';

function SwapComponent() {
  const getQuote = useSwapQuote();
  const initSwap = useInitSwap();
  const { transaction, progress } = useTrackTransaction('tx-id');

  const handleSwap = async () => {
    // 1. Получаем котировку
    const quote = await getQuote.mutateAsync({
      from_chain: 'ethereum',
      to_chain: 'near',
      from_token: 'ETH',
      to_token: 'NEAR',
      amount: '1000000000000000000' // 1 ETH
    });

    // 2. Инициируем своп
    const swapTx = await initSwap.mutateAsync({
      quote_id: quote.quote_id,
      from_wallet_address: '0x...',
      to_wallet_address: 'alice.near'
    });
  };

  return (
    <div>
      <button onClick={handleSwap}>Start Swap</button>
      {transaction && (
        <div>Progress: {progress}%</div>
      )}
    </div>
  );
}
```

### 4. Real-time мониторинг

```typescript
import { useTransactionUpdates, useRiskAlerts } from '@/hooks/websocket/useWebSocket';

function MonitoringComponent() {
  const { updates, latestUpdate } = useTransactionUpdates('tx-id');
  const { alerts, unreadCount } = useRiskAlerts();

  return (
    <div>
      <div>Transaction Updates: {updates.length}</div>
      <div>Latest Status: {latestUpdate?.status}</div>
      <div>Risk Alerts: {unreadCount} unread</div>
    </div>
  );
}
```

## 🔗 Backend Endpoints

Все endpoints автоматически используют конфигурацию из `constants.rs`:

- **Auth**: `/api/v1/auth/*`
- **User**: `/api/v1/user/*`
- **Bridge**: `/api/v1/bridge/*`
- **Quantum**: `/api/v1/crypto/*`
- **Risk**: `/api/v1/risk/*`
- **WebSocket**: `/api/ws`

## 🚀 Следующие шаги

1. **Тестирование интеграции** - Проверить работу в браузере
2. **Error handling** - Улучшить обработку ошибок
3. **Кеширование** - Оптимизировать TanStack Query кеши
4. **UI/UX** - Добавить лоадеры и состояния загрузки
5. **WebSocket reconnection** - Улучшить переподключение

## 🛠️ Конфигурация

Переменные окружения (`.env`):

```bash
VITE_API_URL=http://localhost:4000
VITE_WS_URL=ws://localhost:4000/api/ws
```

## 📦 Зависимости

Новые зависимости уже добавлены в `package.json`:

- `axios` - HTTP client
- `@tanstack/react-query` - Server state management
- Все необходимые типы и hooks

## 🎉 Результат

✅ Полная интеграция фронтенда с бекендом  
✅ Web3 аутентификация через MetaMask и NEAR  
✅ Real-time мониторинг транзакций  
✅ Управление профилем пользователя  
✅ Bridge операции с котировками  
✅ TypeScript типизация всех API  
✅ Modern React patterns с hooks