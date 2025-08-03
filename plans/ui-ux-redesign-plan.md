# Kembridge — план редизайна мобильного UI/UX

Документ описывает цели, требования, архитектурные решения и пошаговый план работ по редизайну фронтенда. Файл создан для внутреннего пользования команды и будет служить чеклистом выполнения задач.

## 1) Бизнес- и UX-цели

- Объединить Swap и Bridge в единую мобильную форму на главной странице, без отдельных страниц.
- Максимально упростить сценарий перевода активов между сетями, сохраняя 100% текущего функционала.
- Минимизировать когнитивную нагрузку: прогрессивное раскрытие (bottom sheet, модальные окна, тултипы).
- Быстрое получение котировок и прозрачная детализация комиссий.
- Единый строгий тёмный минималистичный визуальный стиль, контрастный и удобочитаемый.

## 2) Ограничения и обязательные правила

- Рабочая среда для проверки UI: http://localhost:4100/
- Нельзя трогать устаревшую папку backend (микросервисы работают отдельно).
- Не удаляем функционал — только реорганизация и улучшение UX-подачи.
- Никаких UI-библиотек (Tailwind и т.п.).
- Строгий TypeScript без any.
- Тексты и комментарии в коде — английский, только MD-документы — русский (по месту).
- Используем глобальные переменные/константы проекта, избегаем хардкода.
- Никаких моков/заглушек — отображаем реальные ошибки, но без падений фронта/бэка.

## 3) Ключевые UX-принципы

- Мобильное только: один главный экран, нижняя навигация, крупные касаемые цели.
- Прогрессивное раскрытие: выбор сети/токена/квоты/подтверждения — через BottomSheet/Modal.
- Визуальный фокус на главном действии (Get quotes / Swap).
- Единый ритм отступов, шрифтов, радиусов, теней через дизайн-токены (CSS-переменные).
- Стабильные состояния: loading/empty/error/disabled для каждого интерактивного элемента.
- Предсказуемые ошибки с понятными причинами и действиями.

## 4) Информационная архитектура

Единый экран «Home»:

- Верхняя панель: статус кошелька, сеть, доступный баланс (кратко), вход к настройкам/уведомлениям.
- Блок «Swap & Bridge»:
  - From: asset + network
  - To: asset + network
  - Amount
  - Secondary data: balance, USD эквивалент (если доступно), подсказки.
  - CTA: Get quotes → Quote list → Review → Broadcast.
- Лента активов/истории — укороченная версия, с переходами в детали в модалях.
- Нижняя навигация: сохранён текущий функционал (перенастроим маршрутизацию, но UX — мобильный).

## 5) Глобальные дизайн-токены и тема (CSS Custom Properties)

Цель — максимум переиспользуемых CSS-переменных, единый токенизированный подход.

- Цвета: фоны, поверхности, бордеры, интерактивные состояния, акценты, успех/внимание/ошибка.
- Типографика: размеры, высота строки, веса.
- Интервалы: scale (4/8), контейнерные паддинги, межблочные отступы.
- Радиусы: sm/md/lg/round.
- Тени/обводки: focus/hover/pressed с контрастом для тёмной темы.
- Анимации: duration, easing.

План по файлам токенов/темы:

- [`frontend/src/styles/abstracts/_variables.scss`](frontend/src/styles/abstracts/_variables.scss)
- [`frontend/src/styles/abstracts/_mixins.scss`](frontend/src/styles/abstracts/_mixins.scss)
- [`frontend/src/styles/base/_reset.scss`](frontend/src/styles/base/_reset.scss)
- [`frontend/src/styles/base/_globals.scss`](frontend/src/styles/base/_globals.scss)

Если каких-то файлов нет — создадим, сохранив текущую организацию стилей.

## 6) Компонентная архитектура (мобильные UI-компоненты)

Все базовые UI-компоненты храним отдельно, по требованию проекта:

- Button, IconButton, Input, Select, TextField, Badge, Chip, Divider
- BottomSheet (универсальный), Modal (существует), Tooltip, Spinner/Loader
- AmountInput (фиче-специфичный атом), TokenPill, NetworkPill
- QuoteCard, FeeRow, ReviewCard
- Toast/InlineAlert для ошибок и статусов
- FormRow/FieldLabel/HelperText

Структура каждого компонента:

- Папка компонента: `ComponentName`
- Файлы: `ComponentName.ts` и `ComponentName.module.scss`

План по файлам:

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

Фиче-компоненты (поверх UI-атомов), реюзая существующие bridge-компоненты:

- [`frontend/src/components/bridge/SwapForm/SwapForm.ts`](frontend/src/components/bridge/SwapForm/SwapForm.ts)
- [`frontend/src/components/bridge/TokenSelector/TokenSelector.ts`](frontend/src/components/bridge/TokenSelector/TokenSelector.ts)
- [`frontend/src/components/bridge/PriceQuote/PriceQuote.ts`](frontend/src/components/bridge/PriceQuote/PriceQuote.ts)
- [`frontend/src/components/bridge/SwapConfirmation/SwapConfirmation.ts`](frontend/src/components/bridge/SwapConfirmation/SwapConfirmation.ts)
- [`frontend/src/components/bridge/TransactionProgress/TransactionProgress.ts`](frontend/src/components/bridge/TransactionProgress/TransactionProgress.ts)
- Новый объединяющий контейнер для главной формы:
  - [`frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.ts`](frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.ts)
  - [`frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.module.scss`](frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.module.scss)

Задача — не ломать текущие фичи, а инкапсулировать их в мобильные контейнеры и переиспользуемые атомы.

## 7) Маршрутизация и экраны

- Главная: объединённый «Swap & Bridge» виджет.
  - Правка существующей страницы:
    - [`frontend/src/pages/HomePage/HomePage.tsx`](frontend/src/pages/HomePage/HomePage.tsx)
    - [`frontend/src/pages/HomePage/HomePage.module.scss`](frontend/src/pages/HomePage/HomePage.module.scss)
- Страницы `/swap` и `/bridge` больше не применяются в навигации для мобильного UX, но функционал остаётся доступным внутри главного экрана через компоненты.
  - Маршруты не удаляем сразу: возможно мягкий редирект на Home, либо скрытие в навигации без удаления кода.

## 8) Состояния и ошибки

Для каждого интерактивного компонента: loading / empty / error / success / disabled.

- Общий шаблон отображения ошибок через Toast/InlineAlert, детальные причины в bottom sheet по нажатию «Details».
- Ошибки API — без падений, но с информированием пользователя и возможностью повтора.

## 9) Доступность (a11y)

- Контраст ≥ WCAG AA.
- Фокус-стили видимые на тёмной теме.
- Размеры касаемых целей 44x44pt+.
- Чёткие лэйблы полей, aria-атрибуты в модалях и листах.

## 10) Производительность

- Ленивая подгрузка модальных селекторов (TokenSelector, NetworkSelector).
- Мемоизация тяжёлых списков.
- Дебаунс инпутов суммы/поиска токенов.

## 11) Контроль качества

- Проверка линтера и TS без any после каждого крупного изменения.
- Ручное тестирование основной воронки: Select From/To → Amount → Get quotes → Review → Confirm → Broadcast.
- Мониторинг ошибок в консоли браузера при разработке. Любые ошибки устраняются сразу.

## 12) Пошаговый план работ (чеклист)

### ✅ Этап 1: Архитектура и токены (ЗАВЕРШЁН)

- [x] Создать/актуализировать дизайн-токены и тёмную тему SCSS: `_variables.scss`, `_mixins.scss`, базовые стили.
- [x] Исправить критические ошибки консоли (бесконечные API запросы, WebSocket подключения).
- [x] Проверить работу линтера и TypeScript checker.
- [x] Установить базовую темную тему с CSS переменными (#0D0D0D фон, #00D9FF акцент).
- [x] Создать мобильные миксины и анимации для UI компонентов.

### 📋 Этап 2: UI-атомы (В ПРОЦЕССЕ)

- [ ] Реализовать `Button`, `IconButton` с вариациями (primary, ghost, danger), состояниями.
- [ ] Добавить `Spinner`, `Toast`, `Tooltip`, `Divider`, `Chip`.
- [ ] Реализовать универсальный `BottomSheet` для селекторов/просмотров.
- [ ] Обновить существующие UI компоненты под новую темную тему.

### 📋 Этап 3: Фичи Swap/Bridge (ПЛАНИРУЕТСЯ)

- [ ] Создать контейнер `SwapBridgeCard` и встроить существующие `SwapForm`, `TokenSelector`, `PriceQuote`.
- [ ] Обеспечить сценарий: Get quotes → Quote list → Review → Broadcast в виде последовательных листов/модалей.
- [ ] Учесть отображение комиссий (LP fee, slippage, service fee), детали — отдельный лист.

### 📋 Этап 4: Маршрутизация/страницы (ПЛАНИРУЕТСЯ)

- [ ] Обновить главную страницу и нижнюю навигацию для мобильного UX.
- [ ] Скрыть прямые входы `/swap` и `/bridge` из UI, оставить мягкий редирект/переиспользование.

### 📋 Этап 5: Состояния и ошибки (ПЛАНИРУЕТСЯ)

- [ ] Единая система состояний компонентов.
- [ ] Единый паттерн обработки ошибок без падений (UI уведомления + подробности по клику).

### 📋 Этап 6: Качество и тесты (ПЛАНИРУЕТСЯ)

- [x] Линтер + TS-чеки без any (базовая проверка выполнена).
- [ ] Ручные сценарии E2E на dev-версии по основным флоу.
- [x] Уборка консольных ошибок/ворнингов (критические исправлены).

## 🎯 Текущий статус реализации

### ✅ Завершённые задачи:

1. **Базовая архитектура CSS**:
   - Система CSS переменных с темной темой (#0D0D0D, #00D9FF)
   - Мобильные миксины и анимации
   - Глобальные стили и утилитарные классы

2. **Исправление критических ошибок**:
   - Остановлены бесконечные API запросы к AI-сервисам
   - Исправлены множественные WebSocket подключения/отключения
   - Улучшена инициализация NEAR wallet modal

3. **Проверка качества кода**:
   - ESLint проходит без ошибок
   - TypeScript checker проходит без ошибок
   - Консоль браузера очищена от спама ошибок

### 🔄 Следующий этап: UI-атомы

Готов к реализации основных UI компонентов под темную тему:
- Button/IconButton с состояниями
- Modal/BottomSheet компоненты  
- Toast/Tooltip для уведомлений
- Spinner/Loading состояния

### 📊 Прогресс: 20% выполнено

Этап 1 из 6 полностью завершён. Фундамент для мобильной темной темы готов.

## 13) Риски и смягчение

- Риск ломки существующих фич при объединении страниц — решение: инкрементальная интеграция через новый контейнер `SwapBridgeCard`, без удаления исходных модулей.
- Риск сложности темы — решение: токены + Story-like стенд на Home для визуальной проверки атомов.
- Риск деградации производительности списков — решение: виртуализация/мемоизация + lazy модальные.

## 14) Критерии готовности (Definition of Done)

- Главная страница обеспечивает полный цикл Swap/Bridge, без перехода на отдельные страницы.
- Единый тёмный стиль через CSS-переменные.
- Нет ошибок в консоли на основных флоу.
- Линтер и TS проходят без any и без ошибок.
- Функционал исходных страниц сохранён логически (через новые UI-паттерны), ничего не удалено.

## 15) Карта предстоящих изменений по файлам (ориентир)

- Токены/стили (создать/актуализировать):

  - [`frontend/src/styles/abstracts/_variables.scss`](frontend/src/styles/abstracts/_variables.scss)
  - [`frontend/src/styles/abstracts/_mixins.scss`](frontend/src/styles/abstracts/_mixins.scss)
  - [`frontend/src/styles/base/_globals.scss`](frontend/src/styles/base/_globals.scss)

- Базовые UI (создать):

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

- Фиче-контейнер для объединения Swap+Bridge (создать):

  - [`frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.ts`](frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.ts)
  - [`frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.module.scss`](frontend/src/components/bridge/SwapBridgeCard/SwapBridgeCard.module.scss)

- Главный экран (изменить):
  - [`frontend/src/pages/HomePage/HomePage.tsx`](frontend/src/pages/HomePage/HomePage.tsx)
  - [`frontend/src/pages/HomePage/HomePage.module.scss`](frontend/src/pages/HomePage/HomePage.module.scss)

Примечание: фактический состав и расположение некоторых файлов уточним при интеграции — приоритет на переиспользование существующих модулей, рефакторинг без удаления функциональности.
