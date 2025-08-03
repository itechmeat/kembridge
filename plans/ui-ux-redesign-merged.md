# Kembridge — объединённый план редизайна мобильного UI/UX

Документ объединяет лучшие части из двух планов: [`plans/redesign-plan.md`](plans/redesign-plan.md) и [`plans/ui-ux-redesign-plan.md`](plans/ui-ux-redesign-plan.md). Служит единым источником правды (SSOT) и чеклистом выполнения.

## 1) Цели и текущий статус

### Бизнес- и UX-цели

- Объединить Swap и Bridge в единый мобильный сценарий на главной странице без отдельных страниц.
- **КРИТИЧНО**: Сохранить 100% текущего wallet функционала с главной страницы (WalletPage)
- Интегрировать wallet функционал в главную страницу через модальные окна и компоненты
- Минимизировать когнитивную нагрузку: прогрессивное раскрытие через BottomSheet/Modal/Tooltip.
- Быстрое и прозрачное получение котировок и комиссий.
- Единый строгий тёмный минималистичный стиль, высокий контраст и читабельность.

### Текущий статус (консолидировано)

- Этап 1: фундамент — завершён.
- Этап 2: базовые UI — завершён (по факту реализации в [`plans/redesign-plan.md`](plans/redesign-plan.md)).
- Этап 3: Unified Swap/Bridge на главной — завершён (по факту реализации в [`plans/redesign-plan.md`](plans/redesign-plan.md)).
- Прогресс: 3 из 11+ этапов завершены; продолжаем с интеграционных фич, маршрутизации и качественных критериев.

## 2) Ограничения и обязательные правила

- Рабочая среда для проверки UI: http://localhost:4100/
- Нельзя трогать устаревшую папку backend (микросервисы работают отдельно).
- **КРИТИЧНО**: Не удаляем функционал — проводим реорганизацию и улучшение UX.
- **ВАЖНО**: При переделке дизайна постоянно сверяться с тестами в @e2e-tests и обновлять селекторы
- Без UI-библиотек (Tailwind и т.п.).
- Строгий TypeScript без any.
- Тексты и комментарии в коде — английский; MD-документы — русский при необходимости.
- Используем глобальные константы/переменные, избегаем хардкода.
- Никаких моков/заглушек — реальные ошибки отображаются без падений фронта/бэка.

## 3) UX-принципы

- Мобильное только: один главный экран, bottom navigation, крупные касаемые цели.
- Прогрессивное раскрытие: выбор сети/токена/квоты/подтверждения — через BottomSheet/Modal.
- Визуальный фокус на главном действии (Get quotes / Swap).
- Единый ритм отступов, типографики, радиусов, теней через дизайн‑токены (CSS Custom Properties).
- Стабильные состояния: loading / empty / error / success / disabled с предсказуемыми сообщениями и действиями.

## 4) Информационная архитектура

### Единый экран «Home» с интегрированным wallet функционалом:

**Верхняя панель:**
- Статус кошелька (подключен/отключен)
- Текущая сеть
- Краткий баланс
- Доступ к настройкам/уведомлениям
- **Wallet Status Indicators** (из WalletPage)

**Блок подключения кошелька (если не подключен):**
- Ethereum Wallet кнопка
- NEAR Wallet кнопка  
- Статус аутентификации
- Security indicators

**Блок «Swap & Bridge» (основной, если кошелек подключен):**
- From: asset + network
- To: asset + network
- Amount
- Secondary: баланс, USD-эквивалент (если доступно), подсказки
- CTA: Get quotes → Quote list → Review → Broadcast

**Wallet Management (через модальные окна):**
- User Profile (tier, risk profile)
- Quick Actions
- Security Status
- Logout функционал

**Лента активов/истории:**
- Компактная, детали — в модалях
- Transaction History

**Нижняя навигация:**
- Сохраняем функционал, навигация адаптирована под мобильный UX

## 5) Дизайн‑система (CSS Custom Properties / SCSS токены)

Палитра, типографика, интервалы, радиусы, состояния фокуса/hover/pressed, анимации (duration/easing) — уже заданы и используются. Базовые референсы: $bg-primary #0D0D0D, акцент $accent-primary #00D9FF. Поддерживаем контраст ≥ WCAG AA.

Файлы:

- [`frontend/src/styles/abstracts/_variables.scss`](frontend/src/styles/abstracts/_variables.scss)
- [`frontend/src/styles/abstracts/_mixins.scss`](frontend/src/styles/abstracts/_mixins.scss)
- [`frontend/src/styles/base/_reset.scss`](frontend/src/styles/base/_reset.scss)
- [`frontend/src/styles/base/_globals.scss`](frontend/src/styles/base/_globals.scss)

Если каких-то файлов нет — создаём, сохраняя текущую структуру.

## 6) Компонентная архитектура

Базовые UI (готовы):

- Button, IconButton, Input, Select, Card, Modal, BottomSheet, Toast, Badge, Spinner, Switch (при необходимости).

Фиче‑атомы/молекулы:

- AmountInput, TokenPill, NetworkPill, QuoteCard, FeeRow, ReviewCard, FormRow, FieldLabel, HelperText, Tooltip, Divider, Chip, InlineAlert.

Структурный стандарт:

- Папка компонента: ComponentName/
- Файлы: ComponentName.ts(x) и ComponentName.module.scss

Примеры путей (уточняются по фактическим размещениям):

- [`frontend/src/components/ui/Button/Button.ts`](frontend/src/components/ui/Button/Button.ts)
- [`frontend/src/components/ui/Button/Button.module.scss`](frontend/src/components/ui/Button/Button.module.scss)
- [`frontend/src/components/ui/IconButton/IconButton.ts`](frontend/src/components/ui/IconButton/IconButton.ts)
- [`frontend/src/components/ui/IconButton/IconButton.module.scss`](frontend/src/components/ui/IconButton/IconButton.module.scss)
- [`frontend/src/components/ui/BottomSheet/BottomSheet.ts`](frontend/src/components/ui/BottomSheet/BottomSheet.ts)
- [`frontend/src/components/ui/BottomSheet/BottomSheet.module.scss`](frontend/src/components/ui/BottomSheet/BottomSheet.module.scss)
- [`frontend/src/components/ui/Tooltip/Tooltip.ts`](frontend/src/components/ui/Tooltip/Tooltip.ts)
- [`frontend/src/components/ui/Tooltip/Tooltip.module.scss`](frontend/src/components/ui/Tooltip/Tooltip.module.scss)
- [`frontend/src/components/ui/Toast/Toast.ts`](frontend/src/components/ui/Toast/Toast.ts)
- [`frontend/src/components/ui/Toast/Toast.module.scss`](frontend/src/components/ui/Toast/Toast.module.scss)
- [`frontend/src/components/ui/Spinner/Spinner.ts`](frontend/src/components/ui/Spinner/Spinner.ts)
- [`frontend/src/components/ui/Spinner/Spinner.module.scss`](frontend/src/components/ui/Spinner/Spinner.module.scss)

Фиче‑контейнеры (реюз существующих bridge‑компонентов):

- [`frontend/src/components/bridge/SwapForm/SwapForm.ts`](frontend/src/components/bridge/SwapForm/SwapForm.ts)
- [`frontend/src/components/bridge/TokenSelector/TokenSelector.ts`](frontend/src/components/bridge/TokenSelector/TokenSelector.ts)
- [`frontend/src/components/bridge/PriceQuote/PriceQuote.ts`](frontend/src/components/bridge/PriceQuote/PriceQuote.ts)
- [`frontend/src/components/bridge/SwapConfirmation/SwapConfirmation.ts`](frontend/src/components/bridge/SwapConfirmation/SwapConfirmation.ts)
- [`frontend/src/components/bridge/TransactionProgress/TransactionProgress.ts`](frontend/src/components/bridge/TransactionProgress/TransactionProgress.ts)
- Новый объединяющий контейнер:
  - [`frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.ts`](frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.ts)
  - [`frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.module.scss`](frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.module.scss)

Задача — инкапсулировать существующие фичи в мобильные контейнеры и переиспользуемые атомы, не ломая функциональность.

## 7) Маршрутизация и экраны

- Главная: объединённый «Swap & Bridge» виджет.
- Обновляем:
  - [`frontend/src/pages/HomePage/HomePage.tsx`](frontend/src/pages/HomePage/HomePage.tsx)
  - [`frontend/src/pages/HomePage/HomePage.module.scss`](frontend/src/pages/HomePage/HomePage.module.scss)
- Страницы /swap и /bridge из навигации скрыты для мобильного UX; функционал доступен из главного экрана.
- Редиректы — мягкие; код не удаляем до завершения миграции.

## 8) Состояния и ошибки

- Для каждого интерактивного компонента: loading / empty / error / success / disabled.
- Ошибки: Toast/InlineAlert + BottomSheet «Details» для причин/step‑by‑step действий.
- Ошибки API — без падений, информируем пользователя, даём возможность повтора.

## 9) Доступность (a11y)

- Контраст не ниже WCAG AA.
- Видимые focus‑стили на тёмной теме.
- Минимальные касаемые цели 44–48px.
- Чёткие лэйблы полей, aria‑атрибуты в модалях/листах, клавиатурная навигация.

## 10) Производительность

- Ленивая подгрузка модальных селекторов (TokenSelector, NetworkSelector).
- Мемоизация тяжёлых списков/элементов.
- Debounce для инпутов суммы/поиска.
- Виртуализация длинных списков.

## 11) Контроль качества

- Линтер и TS checks без any после каждого крупного изменения.
- Ручное тестирование основной воронки: Select From/To → Amount → Get quotes → Review → Confirm → Broadcast.
- Мониторинг ошибок в консоли — устраняем сразу.

## 12) Пошаговый план работ (чеклист, объединённый и актуализированный)

Этап 1: Архитектура и токены — ЗАВЕРШЁН

- [x] Дизайн‑токены и тёмная тема SCSS: \_variables.scss, \_mixins.scss, базовые стили
- [x] Исправлены критические ошибки консоли (API/WebSocket)
- [x] Линтер и TypeScript checker без ошибок
- [x] Базовая тёмная тема (#0D0D0D фон, #00D9FF акцент)
- [x] Мобильные миксины и анимации

Этап 2: Базовые UI‑компоненты — ЗАВЕРШЁН

- [x] Button (варианты и состояния)
- [x] IconButton
- [x] Input
- [x] Select/Dropdown
- [x] Card
- [x] Modal + BottomSheet
- [x] Toast
- [x] Badge/StatusIndicator/SecurityLevel
- [x] Spinner (исправлена SASS rgba/currentColor)

Этап 3: Unified Swap/Bridge на Home — ЗАВЕРШЁН

- [x] Компонент/контейнер Unified Swap/Bridge (SwapBridgeInterface/SwapBridgeCard)
- [x] Выбор токенов From/To
- [x] Переключатель режимов Swap/Bridge (если применимо)
- [x] Amount + баланс/fiat‑эквивалент
- [x] Котировки и комиссии
- [x] CTA: Get Quote/Swap/Confirm
- [x] HeroMain/SwapSection (брендинг/логика)
- [x] Обновление HomePage
- [x] Стабильные состояния и ошибки на пути

Этап 4: Селекторы и модальные листы — В ПРОЦЕССЕ

- [ ] TokenSelector с поиском/фильтрацией (lazy)
- [ ] NetworkSelector (lazy)
- [ ] Детальные модалы/BottomSheet для комиссий, деталей транзакций
- [ ] Шаблон Details для ошибок/статусов

Этап 5: Навигация и лейаут — В ПРОЦЕССЕ

- [ ] Обновить MobileLayout под новую тему/ритм
- [ ] TopBar с балансом и статусом кошелька
- [ ] BottomNavigation (иконки/маршруты, скрыть /swap и /bridge в UI)
- [ ] Меню/Settings доступ через модалку/лист

Этап 6: Кошелёк и аутентификация — **КРИТИЧЕСКИЙ ЭТАП**

- [ ] **Анализ текущего WalletPage функционала**
  - [ ] Документировать все компоненты WalletPage
  - [ ] Выделить критический функционал для swap операций
  - [ ] Определить зависимости между wallet и bridge функционалом

- [ ] **Создание wallet компонентов для главной страницы**
  - [ ] WalletConnectionCard (onboarding состояние)
  - [ ] WalletStatusIndicator (подключен/аутентифицирован)
  - [ ] UserProfileCard (tier, risk profile)
  - [ ] SecurityStatusCard (quantum protection, risk analysis)
  - [ ] QuickActionsPanel (быстрые действия)

- [ ] **Интеграция wallet в главную страницу**
  - [ ] Условная логика отображения (wallet connected/disconnected)
  - [ ] Wallet-зависимые состояния для SwapForm
  - [ ] Модальные окна для wallet management
  - [ ] Сохранение всех путей аутентификации (Ethereum/NEAR)

- [ ] **Обновление e2e тестов**
  - [ ] Обновить селекторы для новых wallet компонентов
  - [ ] Адаптировать тесты аутентификации под новую структуру
  - [ ] Проверить все wallet-related тестовые сценарии
  - [ ] Обновить page objects (AuthPage, WalletPage)

- [ ] **Миграция функционала**
  - [ ] Перенести AuthManager в новую структуру
  - [ ] Адаптировать useWallet, useAuthStatus hooks
  - [ ] Сохранить все security indicators
  - [ ] Протестировать полный цикл wallet → auth → swap

Этап 7: E2E тестирование и селекторы — **ОБЯЗАТЕЛЬНЫЙ**

- [ ] **Обновление тестовых селекторов**
  - [ ] Обновить TestSelectors class под новую структуру
  - [ ] Адаптировать ethWalletButton, nearWalletButton селекторы
  - [ ] Обновить navigation selectors (homeLink, bridgeLink)
  - [ ] Проверить все wallet-related селекторы

- [ ] **Адаптация тестовых сценариев**
  - [ ] wallet-authentication.spec.js
  - [ ] enhanced-auth.spec.ts
  - [ ] wallet-mock.spec.js
  - [ ] security-penetration.spec.ts
  - [ ] transaction-flow.spec.js

- [ ] **Page Objects обновления**
  - [ ] AuthPage.ts - новые методы для интегрированного wallet
  - [ ] Создать HomePage page object
  - [ ] Обновить navigation helpers
  - [ ] Адаптировать wallet-helpers.js

- [ ] **Константы и конфигурация**
  - [ ] Обновить SELECTORS в constants.js
  - [ ] Адаптировать API_ENDPOINTS если нужно
  - [ ] Проверить TEST_DATA актуальность

## 18) Критические требования к wallet интеграции

### Функционал который ОБЯЗАТЕЛЬНО должен остаться:

1. **Подключение кошельков:**
   - Ethereum Wallet (MetaMask и др.)
   - NEAR Wallet
   - Статусы подключения и аутентификации

2. **Пользовательский профиль:**
   - User tier (отображение уровня)
   - Risk profile (уровень риска)
   - Welcome сообщения

3. **Quick Actions:**
   - Быстрые действия для пользователя
   - Доступ к основным функциям

4. **Security Status:**
   - Quantum protection indicators
   - Risk analysis display
   - Security alerts

5. **Управление сессией:**
   - Logout функционал
   - Переподключение кошелька
   - Обработка ошибок аутентификации

### Принципы интеграции:

- **Прогрессивное раскрытие**: wallet функционал доступен через модальные окна/BottomSheet
- **Контекстная доступность**: wallet статус всегда виден в header
- **Условная логика**: swap форма адаптируется под статус wallet
- **Сохранение UX**: все пути пользователя остаются доступными

### E2E тестирование:

- **Обязательное обновление** всех селекторов при изменении структуры
- **Регрессионное тестирование** wallet функционала после каждого изменения
- **Совместимость** с существующими тестовыми сценариями
- **Документирование** изменений в page objects

## 19) Чеклист совместимости с тестами

- [ ] Все wallet кнопки сохраняют доступные селекторы
- [ ] Navigation links работают с существующими тестами
- [ ] Form elements доступны через role-based селекторы
- [ ] Status indicators имеют правильные aria-labels
- [ ] Modal/BottomSheet компоненты тестируемы
- [ ] Error states корректно отображаются для тестов
- [ ] Loading states имеют правильные селекторы
- [ ] Security indicators доступны для проверки
