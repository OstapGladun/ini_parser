# ini_parser

**ini_parser** — бібліотека та утиліта командного рядка на Rust для парсингу `.ini` файлів. Вона створена з використанням `pest` для реалізації PEG (Parsing Expression Grammar).

## Можливості

* **Типізований парсинг**: Розрізняє рядки, числа (цілі та з комою) та булеві значення (`true`/`false`/`yes`/`no`).
* **Гнучкі імена**: Підтримує секції та ключі з крапками (напр., `[database.prod]` або `server.host = ...`).
* **Коментарі**: Коректно ігнорує коментарі на весь рядок (`#` або `;`) та вбудовані коментарі (в кінці рядка).
* **CLI та Бібліотека**: Може використовуватися як утиліта в терміналі або як залежність у вашому проекті.
* **Надійна обробка помилок**: Використовує `thiserror` для бібліотеки та `anyhow` для бінарного файлу.



## Технічний опис процесу парсингу

Парсинг виконується за допомогою `pest` на основі граматики, визначеної у файлі `src/grammar.pest`.

### 1. Граматика (Grammar)

Парсер визначає структуру INI файлу за допомогою наступних правил. Граматика підтримує пропуск пробілів та коментарів.


```rust
NEWLINE = _{ "\r\n" | "\n" | "\r" }
WHITESPACE = _{ " " | "\t" }

LINE_COMMENT = _{ (";" | "#") ~ (!NEWLINE ~ ANY)* }
INLINE_COMMENT = _{ (";" | "#") ~ (!NEWLINE ~ ANY)* }

string_quoted = @{ "\"" ~ (!("\"" | NEWLINE) ~ ANY)* ~ "\"" }
boolean = @{ "true" | "false" | "yes" | "no" }
number = @{ ("-" | "+")? ~ (ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)?) }
string_simple = @{ !("\"") ~ (!(";" | "#" | NEWLINE) ~ ANY)+ }

value = { string_quoted | boolean | number | string_simple }

identifier = @{ (ASCII_ALPHANUMERIC | "_" | "-")+ }
dotted_identifier = @{ identifier ~ ("." ~ identifier)* }

pair = { dotted_identifier ~ WHITESPACE* ~ "=" ~ WHITESPACE* ~ value ~ (WHITESPACE* ~ INLINE_COMMENT)? }
section = { "[" ~ dotted_identifier ~ "]" }

line = { (section | pair | LINE_COMMENT | NEWLINE) }
file = { SOI ~ line* ~ EOI }
```

### 2. Як використовуються результати

Логіка в `src/lib.rs` (функція `parse_ini`) отримує дерево розбору (AST) від `pest` і перетворює його у структуровані дані Rust.

**Вихідна структура:** `HashMap<String, HashMap<String, IniValue>>`

* **Зовнішній `HashMap<String, ...>`**: Карта, де ключ — це назва секції (`String`).
    * **Глобальна секція**: Усі пари ключ-значення, що знаходяться до першої декларації `[section]`, зберігаються під спеціальним "глобальним" ключем `""` (порожній рядок).
* **Внутрішній `HashMap<String, IniValue>`**: Карта пар ключ-значення для конкретної секції.
* **`IniValue`**: Це `enum`, який зберігає типизовані дані:
    * `IniValue::String(String)`
    * `IniValue::Number(f64)`
    * `IniValue::Boolean(bool)`

**Приклад:**

Вхідний файл `.ini`:
```ini
global_key = true

[server]
host = "127.0.0.1"
port = 8080
```

Буде перетворено у таку структуру (умовно):
```rust
{
    "": {
        "global_key": Boolean(true)
    },
    "server": {
        "host": String("127.0.0.1"),
        "port": Number(8080.0)
    }
}
```

---

## Використання CLI

Проект включає CLI, зібраний за допомогою `clap`.

**Отримання допомоги:**
```bash
cargo run -- --help
```

**Парсинг файлу:**
```bash
cargo run -- parse --file examples/example.ini
```

**Відображення авторів:**
```bash
cargo run -- credits
```

---

## Скрипти `Makefile`

Для зручності розробки надається `Makefile`.

* `make run ARGS="..."`: Запускає програму з аргументами (напр., `make run ARGS="parse --file examples/example.ini"`).
* `make test`: Запускає всі юніт-тести.
* `make fmt`: Форматує код за допомогою `cargo fmt`.
* `make lint`: Перевіряє код за допомогою `cargo clippy`.
* `make check`: Виконує `fmt`, `lint` та `test` по черзі (для перевірки перед комітом).
* `make publish`: Запускає `check` і (якщо успішно) публікує крейт на `crates.io`.