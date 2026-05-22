# dcal (v0.5.0)

A modern, fast, and standalone CLI calendar utility written in Rust. It serves as a drop-in or enhanced replacement for the classic `cal`/`ncal` commands, featuring auto-localization, grid alignment, customizable month ranges, interactive navigation, and todo/event integration.

## Screenshots
[<img src="screenshots/01.png" width="400">](screenshots/01.png) [<img src="screenshots/04.png" width="400">](screenshots/04.png)
[<img src="screenshots/03.png" width="400">](screenshots/03.png) [<img src="screenshots/02.png" width="400">](screenshots/02.png)

[Читать на русском языке](#русский)

---

## Features
* **Zero Dependencies:** Compiles into a single independent binary.
* **Auto-Localization:** Automatically detects system locale (supports Russian and English; defaults to English).
* **Flexible Grids:** Displays multiple months horizontally, perfectly aligned.
* **Custom Months Range:** View from 1 to 12 months ahead or look up any specific year.
* **Beautiful Borders:** Optional clean box borders around month blocks.
* **Week Counts:** Displays total ISO-8601 week count for full-year views.
* **Interactive TUI Mode:** Smoothly navigate through months or years using arrow keys.
* **Todo/Events Integration:** Automatically highlights dates with events in green and lists descriptions directly below the calendar grid.

## Installation on Arch Linux

### Manual Build
Ensure you have `cargo` installed:
```bash
sudo pacman -S rust
```
Clone the repository and build the release version:
```bash
git clone https://github.com/1mesles1/dcal
cd dcal
cargo build --release
sudo cp target/release/dcal /usr/local/bin/
```

### Using PKGBUILD
You can build a native Arch package manually using the provided `PKGBUILD`:
```bash
git clone https://github.com/1mesles1/dcal
cd dcal
makepkg -si
```

## Usage & Flags
* `dcal` — Display current month (auto-localized).
* `dcal -3` — Display 3 months horizontally starting from current.
* `dcal -c` — Display 3 months with the current month strictly in the center.
* `dcal -g` — Display the full current year (aligned in a clean 4x3 grid).
* `dcal -x2021` — Display the specified year fully (e.g., year 2021).
* `dcal -b` — Draw a clean border around month blocks (can be combined with other flags like `-gb` or `-cb`).
* `dcal -m` — Start the week on Sunday instead of Monday.
* `dcal -e` / `dcal -r` — Force English or Russian output regardless of system locale (the leftmost flag takes priority).
* `dcal -v` / `dcal -h` — Show version or help message.
* `dcal -i` — **Interactive Mode**. Navigate months with **Left/Right/Up/Down Arrow Keys**. Press **Q** or **Esc** to exit. *Tip: If combined with year mode (`-gi`), Arrow Keys will skip an entire year!*
* `dcal -w` — Display ISO-8601 week numbers on the left side of the calendar grid.
* `dcal -d` — **Events Mode**. Highlights days with events in green and lists descriptions below the grid for the rendered period.
  * Data file path: `~/.config/dcal/events.txt` (automatically created on first launch).
  * Format inside the file: `DD.MM.YYYY - event description` (e.g., `23.05.2026 - Clean room`).
* `dcal -l` — List all existing events from the database chronologically and exit.
* `dcal -a` — Launch the built-in **Interactive Event Manager Console**. Features its own command menu: `[l]` list events, `[a]` add event with date validation, `[d]` recursively delete events by index, and `[q]` return to shell.

---

<a name="русский"></a>
# dcal (Русский)

Современная, быстрая и полностью независимая консольная утилита-календарь, написанная на Rust. Создана как замена классическим командам `cal`/`ncal` с поддержкой автоматической локализации, выравниванием сеток, интерактивным листанием, красивыми рамками в терминале и интеграцией с файлом планов.

## Особенности
* **Ноль зависимостей:** Компилируется в один независимый бинарный файл.
* **Авто-локализация:** Автоматически определяет язык системы (поддерживает русский и английский, по умолчанию английский).
* **Выравнивание в ширину:** Выводит несколько месяцев горизонтально в ряд с идеальной геометрией.
* **Гибкие диапазоны:** Позволяет смотреть от 1 до 12 месяцев вперед или выводить конкретный год целиком.
* **Красивые рамки:** Опциональная отрисовка аккуратных псевдографических границ вокруг месяцев.
* **Подсчет недель:** Отображает общее количество недель в году по стандарту ISO-8601 при выводе за год.
* **Интерактивный режим:** Плавное листание месяцев или лет с помощью стрелок клавиатуры в реальном времени.
* **Интеграция с планами/задачами:** Подсвечивает зелёным цветом дни с запланированными делами и выводит список описаний под календарной сеткой.
* `dcal -v` / `dcal -h` — Показать версию или справку.
* `dcal -i` — Интерактивный режим (листание месяцев стрелками, выход на Q или Esc).
* `dcal -w` — Отобразить порядковые номера недель года по стандарту ISO-8601 слева от календарной сетки.
* `dcal -d` — **Режим событий**. Выделяет зеленым цветом дни с событиями и отображает описания событий под сеткой за отображаемый период.
* `dcal -l` — Быстрый вывод всех существующих в базе напоминаний в хронологическом порядке.
* `dcal -a` — Запуск интерактивного **Менеджера задач**. Открывает внутреннюю консоль управления: `[l]` показать записи, `[a]` добавить запись с валидацией дат, `[d]` удаление по номерам с автообновлением списка, `[q]` выход.

## Работа с файлом задач (`-d`)
Файл создаётся автоматически при первом запуске программы по пути: `~/.config/dcal/events.txt`.
Вы можете открыть его в любом текстовом редакторе и добавлять задачи в строгом формате:
```text
23.05.2026 - Убраться по дому
25.05.2026 - Важная встреча
```

## Установка на Arch Linux

### Ручная сборка
Убедитесь, что в системе установлен `cargo`:
```bash
sudo pacman -S rust
```
Склонируйте репозиторий и соберите релизную версию:
```bash
git clone https://github.com/1mesles1/dcal
cd dcal
cargo build --release
sudo cp target/release/dcal /usr/local/bin/
```

### Сборка через PKGBUILD
Вы можете собрать нативный пакет для Arch Linux, используя готовый `PKGBUILD`:
```bash
git clone https://github.com/1mesles1/dcal
cd dcal
makepkg -si
```

