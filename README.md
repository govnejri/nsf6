# Not so Far

Репозиторий веб-приложения для хакатона Decentraton 4.0, кейс 2 от inDrive. 

## Описание

Not so Far - веб-приложение для анализа и визуализации анонимизированных данных о поездках пользователей сервиса inDrive.

Аналитики могут использовать данные для выявления популярных маршрутов, 
оценки загруженности дорог и опасных участков, 
а так же получать предупреждения об аномальных поездках.

Водители могут оптимизировать маршруты и искать зоны с высоким спросом.

Пользователи могут видеть популярные направления и планировать поездки.

## Технологии

- База данных: PostgreSQL
- Backend: Rust, Actix-web
- Frontend: 2Gis MapGL, TypeScript, jQuery, Tailwind CSS, PostCSS
- Контейнеризация: Docker, Docker Compose


## Команда
- omga: Frontend, API, писал этот файл | [LinkedIn](https://www.linkedin.com/in/omgaxd/)
- c0st1nus: Backend, БД | [LinkedIn](https://www.linkedin.com/in/konstantin-koshevoy-336608324/)
- govnejri: ML, анализ аномалий | [LinkedIn](https://www.linkedin.com/in/bekzat-uteulin-98082b2b6/)
- tsu: Дизайн, презентация | [LinkedIn](https://www.linkedin.com/in/saltanat-tlegen-b43138380/)

## Полезные ссылки
- [Демо](https://indrive.notsofar.live)
- [Папа с презентацией](https://drive.google.com/drive/folders/1_iUVDPaoIMY0XAkwV2OdvYzY7o-DGjer?usp=sharing)
- [Сайт хакатона Decentraton 4.0](https://astanahub.com/en/event/decentrathon-4-0)

# Для разработчиков

## Установка и запуск
1. Клонируйте репозиторий:
    ```bash
    git clone git@github.com:NIS-Qostanai-Touhou-Doujin-Circle/nsf6.git
    cd nsf6
    ```
2. Создайте файл `.env` в корне проекта и добавьте переменные окружения:
    - DATABASE_URL: URL подключения к базе данных PostgreSQL.
    - RUST_LOG: уровень логирования для backend (например, info, debug).
    - POINTS_WEBHOOK_URL: URL для вебхука ML-анализа аномальности точек маршрута
    
    Пример содержимого файла `.env`:
    ```
    DATABASE_URL=postgres://nsf6:yourpassword@127.0.0.1:5432/nsf6_db
    RUST_LOG=info
    POINTS_WEBHOOK_URL=http://127.0.0.1:8080/api/zaglushka
    ```
3. Запустите контейнеры с помощью Docker Compose:
    ```bash
    docker compose up --build
    ```
    Приложение будет доступно по адресу `http://localhost:8080`.

## Разработка

### Установка зависимостей
Убедитесь, что у вас установлены Cargo, Node.js и npm. Затем выполните:

```bash
npm install
```

### Запуск

Для запуска в режиме разработки (авто-перезагрузка при изменениях фронтенда):
```bash
docker compose -f postgre-only-docker-compose.yaml up -d
```
Дождитесь запуска БД. Далее:
```bash
npm start
```
Приложение будет доступно по адресу `http://localhost:8080`. 
Любые изменения в коде фронтенда будут автоматически применяться с задержкой 1-3 сек.

### Сборка фронтенда
Для сборки фронтенда выполните:
```bash
npm run build
```
Собранные файлы будут помещены в папку `web/out`.
