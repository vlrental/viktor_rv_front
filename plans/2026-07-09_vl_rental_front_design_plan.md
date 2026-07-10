# План: VL Rental — фронт с нуля (Dioxus web, чистый дизайн без бэка)

## Контекст
- Папка: `C:\work\web_sites\rv_rental\viktor_rv_front` (пустая, полный ноль).
- Дизайн: Pencil `C:/code/tools/pencil/info_center.pen` — набор фреймов «VL Rental — …» (десктоп ~1440)
  и «Mobile — …» (390), + компоненты Listing Card, Mobile Nav, Mobile Footer.
- Этап 1 = только дизайн/вёрстка (статичные данные-моки), подключение бэка — потом.

## Лог просьб и решений владельца (000.PLAN.8)
- [2026-07-09] Просьба: новый сайт RV rental, чисто фронт с нуля, в этой папке, Dioxus «как обычно».
- [2026-07-09] Просьба: сначала дизайн, потом всё подключаем.
- [2026-07-09] Даны node ID всех фреймов дизайна в Pencil.

## Дизайн-токены (из Pencil vl-*)
- forest #1B3A28 / forest-2 #10271B / ink #17261C — тёмно-зелёные
- surface #F5F3EE, mint #EAF2EC, white/cloud #FFFFFF — фоны
- accent #D7A24A (янтарь), coral #EA6A4B, sage #7A9A80
- hair #E7E4DB, line #D6DDD0, muted #59685E
- Шрифты: заголовки Newsreader, текст Inter, моно Geist Mono (Google Fonts)

## Архитектура
- Dioxus 0.7.9 web + router. Один крейт `viktor_rv_front`.
- CSS: один `assets/main.css` с CSS-переменными-токенами + классами; @media ≤768px для мобильной версии.
- `src/components/`: header (нав), footer, listing_card, общие мелочи.
- `src/pages/`: home, catalog, rv_detail, boat_detail, checkout, confirmed, contact, about,
  attractions, restaurants, cooler_trailers, delivery, rv_sales, terms, book_boat.
- Данные-моки: локальные структуры в `src/data.rs` (листинги RV/лодок) — потом заменятся на API.
- Картинки: экспорт из Pencil-фреймов в `assets/img/` (или плейсхолдеры-градиенты где не критично).

## Этапы
1. ✅ План.
2. ✅ Скаффолд: Cargo.toml, Dioxus.toml, main.rs, роутер (15 роутов), main.css с токенами. cargo check зелёный.
3. ✅ Header (overlay на Home / светлый на остальных) + Footer по дизайну.
4. ✅ Home (RWhwx) целиком: hero+search, RV-карточки, featured, лодки, how-it-works, services, CTA. Сверено в превью (порт 8090, конфиг rv-preview).
5. ✅ Catalog (kTkmm): header+компактный search, фильтры (чекбоксы/слайдер/чипы), грид 3 кол.
6-8. ✅ Остальные 13 страниц — 7 параллельных суб-агентов (detail×2, checkout+confirmed, contact+about, attractions+restaurants, cooler+delivery, sales+terms, book_boat). Все отчитались: pixel-per-export, свои css в assets/css/.
9. ✅ Финал: общий cargo check зелёный, 0 warnings; обход всех 15 роутов в превью — контент на месте, консоль без ошибок; мобильная адаптация проверена (одна колонка, нет h-scroll).

## Известные хвосты
- preview_screenshot в Browser-панели стабильно таймаутится (глюк панели, страница живая — сверка шла через snapshot/inspect/eval). Визуальную пиксель-сверку скриншотами добить при живом просмотре.
- Всё интерактивное — моки: поиск/фильтры/формы не подключены (этап «подключаем» — следующий).
- Строки на английском по дизайну; i18n-слой не заводился (решить при подключении).

## Решения по ходу
- Иконки: lucide icon-font с CDN (unpkg lucide-static), компонент Icon{name,size,color}.
- Картинки дизайна скопированы из C:\code\tools\pencil\assets\vl → assets/img (30 шт).
- CSS: main.css (токены+глобальные+home+catalog+footer), по-страничные файлы assets/css/*.css.
- Данные-моки: src/data.rs (Listing/Boat, catalog_listings 8 позиций).
- Превью: .claude/launch.json (idyll_v2) — конфиг rv-preview, порт 8090, cmd cd в папку проекта.

## Проверка
- `cargo check` после каждого этапа; `dx serve` + сверка с Pencil-кадрами (preview / скриншоты).
