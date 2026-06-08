use chrono::{Datelike, Local, NaiveDate, Weekday, Duration};
use std::env;
use sys_locale::get_locale;
use std::io::{stdout, Write, BufRead, BufReader, stdin};
use std::fs::{self, File};
use std::path::PathBuf;
use std::collections::BTreeMap;
use crossterm::{
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{MoveTo, Hide, Show},
    event::{read, Event, KeyCode, KeyEventKind},
};

const MONTHS_RU: [&str; 12] = [
    "Январь", "Февраль", "Март", "Апрель", "Май", "Июнь",
    "Июль", "Август", "Сентябрь", "Октябрь", "Ноябрь", "Декабрь",
];
const MONTHS_EN: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const GRAY: &str = "\x1b[90m";
const INVERT: &str = "\x1b[7m";
const RESET: &str = "\x1b[0m";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum EventDate {
    Always,
    Yearly(i32),
    Specific(NaiveDate),
}

impl EventDate {
    fn parse(input: &str) -> Result<Self, ()> {
        let input = input.trim();
        let parts: Vec<&str> = input.split('.').collect();
        
        if parts.len() != 3 {
            return Err(());
        }

        let day_str = parts[0];
        let month_str = parts[1];
        let mut year_str = parts[2].to_string();

        if day_str == "00" && month_str == "00" {
            if year_str == "00" || year_str == "0000" {
                return Ok(EventDate::Always);
            }
            if let Ok(year) = year_str.parse::<i32>() {
                let full_year = if year_str.len() == 2 { 2000 + year } else { year };
                return Ok(EventDate::Yearly(full_year));
            }
            return Err(());
        }

        if year_str.len() == 2 {
            year_str = format!("20{}", year_str);
        }

        let full_date_str = format!("{}.{}.{}", day_str, month_str, year_str);
        if let Ok(date) = NaiveDate::parse_from_str(&full_date_str, "%d.%m.%Y") {
            Ok(EventDate::Specific(date))
        } else {
            Err(())
        }
    }

    fn to_formatted_string(&self, _is_ru: bool) -> String {
        match self {
            EventDate::Always => "00.00.0000".to_string(),
            EventDate::Yearly(year) => format!("00.00.{}", year),
            EventDate::Specific(date) => date.format("%d.%m.%Y").to_string(),
        }
    }

    fn to_display_string(&self, is_ru: bool) -> String {
        match self {
            EventDate::Always => {
                if is_ru { "[Всегда]".to_string() } else { "[Always]".to_string() }
            }
            EventDate::Yearly(year) => {
                if is_ru { format!("[{} год]", year) } else { format!("[Year {}]", year) }
            }
            EventDate::Specific(date) => {
                date.format("%d.%m.%Y").to_string()
            }
        }
    }
}

struct EventStorage {
    events: BTreeMap<EventDate, Vec<String>>,
}

fn print_help() {
    println!("Usage: dcal [options]");
    println!();
    println!("Options:");
    println!("  -h          Show this help message");
    println!("  -v          Show version information");
    println!("  -<1-12>     Display specified number of months starting from current");
    println!("  -c          Display 3 months, with the current month in the center");
    println!("  -g          Display the current year fully (4x3 grid)");
    println!("  -x<year>    Display the specified year fully (e.g., -x2021)");
    println!("  -b          Draw a beautiful border around each month (can be combined)");
    println!("  -e          Force English language output");
    println!("  -r          Force Russian language output");
    println!("  -m          Start the week on Sunday instead of Monday");
    println!("  -i          Interactive mode (navigate months with arrow keys, exit with Q/Esc)");
    println!("  -w          Display ISO-8601 week numbers on the left side of the calendar grid.");
    println!("  -d          Enable events display. Highlights days with events in green and");
    println!("              lists descriptions below the grid for the rendered period.");
    println!("              Data file path: ~/.config/dcal/events.txt");
    println!("              Format: DD.MM.YYYY - event description (e.g., 23.05.2026 - Clean room)");
    println!("  -l          List all existing events from database chronologically.");
    println!("  -a          Interactive event manager console. Features:");
    println!("              [l] list events, [a] add event with date validation, [d] delete event by index.");
}

fn get_config_path() -> (PathBuf, PathBuf) {
    let home_dir = env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let config_dir = home_dir.join(".config").join("dcal");
    let file_path = config_dir.join("events.txt");
    (config_dir, file_path)
}

fn load_events() -> EventStorage {
    let mut storage = BTreeMap::new();
    let (config_dir, file_path) = get_config_path();

    if !config_dir.exists() { let _ = fs::create_dir_all(&config_dir); }
    if !file_path.exists() { let _ = File::create(&file_path); }

    if let Ok(file) = File::open(&file_path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Some(pos) = line.find(" - ") {
                let date_str = line[..pos].trim();
                let desc = line[pos + 3..].trim().to_string();
                // Используем наш новый умный парсер вместо NaiveDate::parse_from_str
                if let Ok(event_date) = EventDate::parse(date_str) {
                    storage.entry(event_date).or_insert_with(Vec::new).push(desc);
                }
            }
        }
    }
    EventStorage { events: storage }
}

fn save_all_events(storage: &EventStorage) {
    let (_, file_path) = get_config_path();
    if let Ok(mut file) = File::create(&file_path) {
        for (event_date, descs) in &storage.events {
            for desc in descs {
                // Используем наш метод форматирования, чтобы в файл писались красивые нули
                let date_str = event_date.to_formatted_string(true); 
                let _ = writeln!(file, "{} - {}", date_str, desc);
            }
        }
    }
}

fn print_all_events_cli(storage: &EventStorage, is_ru: bool) {
    if storage.events.is_empty() {
        println!("{}", if is_ru { "Напоминаний не найдено." } else { "No events found." });
        return;
    }
    for (event_date, descs) in &storage.events {
        for desc in descs {
            println!("{} - {}", event_date.to_formatted_string(is_ru), desc);
        }
    }
}

fn handle_interactive_manager(is_ru: bool) {
    let mut storage = load_events();
    
    let menu_msg = if is_ru {
        "\n--- МЕНЕДЖЕР ЗАДАЧ dcal ---\n[l] Показать все записи\n[a] Добавить запись\n[d] Удалить запись\n[q] Выйти в терминал\nВыберите действие: "
    } else {
        "\n--- dcal EVENT MANAGER ---\n[l] List all events\n[a] Add new event\n[d] Delete event\n[q] Quit to terminal\nChoose action: "
    };

    loop {
        print!("{}", menu_msg);
        stdout().flush().unwrap();
        let mut choice = String::new();
        stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim().to_lowercase();

        if choice == "q" { break; }
        else if choice == "l" {
            print_all_events_cli(&storage, is_ru);
        } else if choice == "a" {
            handle_sub_add(&mut storage, is_ru);
        } else if choice == "d" {
            handle_sub_delete(&mut storage, is_ru);
        }
    }
}

// Убедись, что в самом верху файла (или внутри функции) доступен Local из chrono:
// use chrono::Local;

fn handle_sub_add(storage: &mut EventStorage, is_ru: bool) {
    let today = chrono::Local::now().naive_local().date();
    let today_str = today.format("%d.%m.%Y").to_string();

    if is_ru {
        println!("Сегодня: {}", today_str);
    } else {
        println!("Today is: {}", today_str);
    }

    let msg_date = if is_ru { "Введите дату (ДД.ММ.ГГГГ (ГГ) / 00.00.ГГГГ / 00.00.ГГ) или 'q' для отмены: " } else { "Enter date (DD.MM.YYYY (YY) / 00.00.YYYY / 00.00.YY) or 'q' to cancel: " };
    let msg_desc = if is_ru { "Введите описание задачи или 'q' для отмены: " } else { "Enter event description or 'q' to cancel: " };
    let msg_err = if is_ru { "Ошибка: Неверный формат даты! Попробуйте снова." } else { "Error: Invalid date format! Please try again." };
    
    let validated_date;
    loop {
        print!("{}", msg_date);
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "q" || input == "Q" { return; }

        // Используем наш новый умный парсер, который сам обработает и YY, и нули
        if let Ok(event_date) = EventDate::parse(input) {
            validated_date = event_date;
            break;
        } else { println!("{}", msg_err); }
    }

    print!("{}", msg_desc);
    stdout().flush().unwrap();
    let mut desc = String::new();
    stdin().read_line(&mut desc).unwrap();
    let desc = desc.trim().to_string();
    if desc == "q" || desc == "Q" { return; }

    storage.events.entry(validated_date.clone()).or_insert_with(Vec::new).push(desc);
    save_all_events(storage);
    
    // Красиво выводим то, что ввёл пользователь (нули или обычную дату)
    let formatted_date = validated_date.to_formatted_string(is_ru);
    if is_ru {
        println!("Успешно добавлено на {}!", formatted_date);
    } else {
        println!("Successfully added to {}!", formatted_date);
    }
}

fn handle_sub_delete(storage: &mut EventStorage, is_ru: bool) {
    let msg_prompt = if is_ru { "Введите номер записи для удаления или 'q' для выхода в меню: " } else { "Enter item number to delete or 'q' to return: " };
    let msg_err = if is_ru { "Ошибка: Неверный номер! Попробуйте снова." } else { "Error: Invalid number! Please try again." };

    loop {
        if storage.events.is_empty() {
            println!("{}", if is_ru { "Список пуст." } else { "List is empty." });
            return;
        }

        let today = chrono::Local::now().naive_local().date();
        let today_str = today.format("%d.%m.%Y").to_string();
        if is_ru {
            println!("Сегодня: {}", today_str);
        } else {
            println!("Today is: {}", today_str);
        }

        // Плоский список теперь хранит EventDate в качестве первого элемента кортежа
        let mut flat_list = Vec::new();
        for (event_date, descs) in &storage.events {
            for desc in descs {
                flat_list.push((event_date.clone(), desc.clone()));
            }
        }

        for (idx, (event_date, desc)) in flat_list.iter().enumerate() {
            println!("{}) {} - {}", idx + 1, event_date.to_formatted_string(is_ru), desc);
        }

        print!("{}", msg_prompt);
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "q" || input == "Q" { return; }

        if let Ok(num) = input.parse::<usize>() {
            if num > 0 && num <= flat_list.len() {
                let (target_date, target_desc) = &flat_list[num - 1];
                let deleted_date_str = target_date.to_formatted_string(is_ru);
                
                if let Some(descs) = storage.events.get_mut(target_date) {
                    if let Some(pos) = descs.iter().position(|x| x == target_desc) {
                        descs.remove(pos);
                    }
                }
                if storage.events.get(target_date).map_or(false, |v| v.is_empty()) {
                    storage.events.remove(target_date);
                }
                save_all_events(storage);
                
                if is_ru {
                    println!("Запись от {} успешно удалена!\n", deleted_date_str);
                } else {
                    println!("Event from {} successfully deleted!\n", deleted_date_str);
                }
                continue; 
            }
        }
        println!("{}", msg_err);
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_help();
            std::process::exit(0);
        } else if arg == "-v" || arg == "--version" {
            println!("dcal version 0.7.1");
            std::process::exit(0);
        }
    }

    let locale = get_locale().unwrap_or_else(|| "en".to_string());
    let mut is_ru = locale.starts_with("ru");
    let mut lang_overridden = false;

    let now = Local::now().date_naive();
    let current_year = now.year();
    let current_month = now.month() as i32;

    let mut start_year = current_year;
    let mut start_month = current_month;
    let mut months_count = 1;
    let mut use_border = false;
    let mut sunday_first = false;
    let mut cols_count = 4;
    let mut mode_selected = false;
    let mut show_weeks_total = false;
    let mut is_year_mode = false;
    let mut interactive_mode = false;
    let mut show_events = false;
    let mut manager_mode = false;
    let mut list_only_mode = false;
    let mut show_week_numbers = false;

    for arg in &args {
        if !arg.starts_with('-') || arg.len() < 2 {
            eprintln!("Error: Unknown argument {}", arg);
            std::process::exit(1);
        }
        if arg.starts_with("-x") {
            if let Ok(year) = arg[2..].parse::<i32>() {
                start_year = year; start_month = 1; months_count = 12; cols_count = 4;
                mode_selected = true; show_weeks_total = true; is_year_mode = true;
                continue;
            } else {
                eprintln!("Error: Invalid year format after -x");
                std::process::exit(1);
            }
        }

        let mut chars = arg.chars().skip(1).peekable();
        while let Some(c) = chars.next() {
            if c == 'b' { use_border = true; }
            else if c == 'm' { sunday_first = true; }
            else if c == 'i' { interactive_mode = true; }
            else if c == 'd' { show_events = true; }
            else if c == 'w' { show_week_numbers = true; }
            else if c == 'a' { manager_mode = true; }
            else if c == 'l' { list_only_mode = true; }
            else if c == 'e' { if !lang_overridden { is_ru = false; lang_overridden = true; } }
            else if c == 'r' { if !lang_overridden { is_ru = true; lang_overridden = true; } }
            else if c == 'g' {
                start_year = current_year; start_month = 1; months_count = 12; cols_count = 4;
                mode_selected = true; show_weeks_total = true; is_year_mode = true;
            } else if c == 'c' {
                let prev_month_date = NaiveDate::from_ymd_opt(current_year, current_month as u32, 1).unwrap() - Duration::days(1);
                start_year = prev_month_date.year(); start_month = prev_month_date.month() as i32;
                months_count = 3; cols_count = 3; mode_selected = true;
            } else if c.is_ascii_digit() {
                let mut num_str = c.to_string();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_digit() { num_str.push(chars.next().unwrap()); } else { break; }
                }
                if let Ok(count) = num_str.parse::<i32>() {
                    if (1..=12).contains(&count) {
                        months_count = count; cols_count = if count < 4 { count as usize } else { 4 };
                        mode_selected = true;
                    } else {
                        eprintln!("Error: Number of months must be between 1 and 12");
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: Unknown flag -{}", c);
                std::process::exit(1);
            }
        }
    }

    if list_only_mode {
        let storage = load_events();
        print_all_events_cli(&storage, is_ru);
        return;
    }

    if manager_mode {
        handle_interactive_manager(is_ru);
        return;
    }

    if !mode_selected { cols_count = 1; }
    let event_storage = load_events();
    if interactive_mode {
        crossterm::terminal::enable_raw_mode().unwrap();
        execute!(stdout(), EnterAlternateScreen, Hide).unwrap();
        
        loop {
            execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
            
            let mut months_to_render = Vec::new();
            let mut y = start_year;
            let mut m = start_month;

            for _ in 0..months_count {
                months_to_render.push((y, m));
                m += 1;
                if m > 12 {
                    m = 1;
                    y += 1;
                }
            }

            let chunks: Vec<&[(i32, i32)]> = months_to_render.chunks(cols_count).collect();
            for (i, chunk) in chunks.iter().enumerate() {
                print_months_row_interactive(chunk, is_ru, now, use_border, sunday_first, is_year_mode, show_events, &event_storage, show_week_numbers);
                if i < chunks.len() - 1 {
                    print!("\r\n");
                }
            }

            if show_weeks_total {
                if let Some(last_day_of_year) = NaiveDate::from_ymd_opt(start_year, 12, 28) {
                    let total_weeks = last_day_of_year.iso_week().week();
                    print!("\r\n");
                    if is_ru {
                        print!("Всего недель в {} году: {}", start_year, total_weeks);
                    } else {
                        print!("Total weeks in year {}: {}", start_year, total_weeks);
                    }
                }
            }

            if show_events {
                print_events_list_interactive(is_ru, &months_to_render, &event_storage);
            }
            
            stdout().flush().unwrap();

            if let Event::Key(key) = read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Right | KeyCode::Up => {
                            if is_year_mode {
                                start_year += 1;
                            } else {
                                start_month += 1;
                                if start_month > 12 {
                                    start_month = 1;
                                    start_year += 1;
                                }
                            }
                        }
                        KeyCode::Left | KeyCode::Down => {
                            if is_year_mode {
                                start_year -= 1;
                            } else {
                                start_month -= 1;
                                if start_month < 1 {
                                    start_month = 12;
                                    start_year -= 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        execute!(stdout(), Show, LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
    } else {
        let mut months_to_render = Vec::new();
        let mut y = start_year;
        let mut m = start_month;

        for _ in 0..months_count {
            months_to_render.push((y, m));
            m += 1;
            if m > 12 {
                m = 1;
                y += 1;
            }
        }

        let chunks: Vec<&[(i32, i32)]> = months_to_render.chunks(cols_count).collect();
        for (i, chunk) in chunks.iter().enumerate() {
            print_months_row(chunk, is_ru, now, use_border, sunday_first, is_year_mode, show_events, &event_storage, show_week_numbers);
            if i < chunks.len() - 1 {
                println!();
            }
        }

        if show_weeks_total {
            if let Some(last_day_of_year) = NaiveDate::from_ymd_opt(start_year, 12, 28) {
                let total_weeks = last_day_of_year.iso_week().week();
                println!();
                if is_ru {
                    println!("Всего недель в {} году: {}", start_year, total_weeks);
                } else {
                    println!("Total weeks in year {}: {}", start_year, total_weeks);
                }
            }
        }

        if show_events {
            print_events_list(is_ru, &months_to_render, &event_storage);
        }
    }
}

fn generate_month_lines(year: i32, month: i32, is_ru: bool, today: NaiveDate, use_border: bool, sunday_first: bool, required_weeks: usize, show_events: bool, storage: &EventStorage, show_week_numbers: bool) -> Vec<String> {
    let mut lines = Vec::new();

    let month_name = if is_ru { MONTHS_RU[(month - 1) as usize] } else { MONTHS_EN[(month - 1) as usize] };
    let header_text = format!("{} {}", month_name, year);
    
    let wdays_text = if sunday_first {
        if is_ru {
            format!("{red}Вс{reset} Пн Вт Ср Чт Пт {red}Сб{reset}", red = RED, reset = RESET)
        } else {
            format!("{red}Su{reset} Mo Tu We Th Fr {red}Sa{reset}", red = RED, reset = RESET)
        }
    } else {
        if is_ru {
            format!("Пн Вт Ср Чт Пт {red}Сб Вс{reset}", red = RED, reset = RESET)
        } else {
            format!("Mo Tu We Th Fr {red}Sa Su{reset}", red = RED, reset = RESET)
        }
    };

    let wnum_header_pad = if show_week_numbers { "   " } else { "" };

    if use_border {
        lines.push(format!("{}┌────────────────────┐", wnum_header_pad));
        lines.push(format!("{}│{:^20}│", wnum_header_pad, header_text));
        lines.push(format!("{}├────────────────────┤", wnum_header_pad));
        lines.push(format!("{}│{}│", wnum_header_pad, wdays_text));
    } else {
        lines.push(format!("{}{:<20}", wnum_header_pad, format!("    {}", header_text)));
        lines.push(format!("{}{}", wnum_header_pad, wdays_text));
    }

    let first_day = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
    let weekday_offset = if sunday_first {
        first_day.weekday().num_days_from_sunday() as usize
    } else {
        first_day.weekday().num_days_from_monday() as usize
    };

    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap()
    };
    let total_days = next_month_date.signed_duration_since(first_day).num_days() as usize;

    let mut current_line = String::new();
    for _ in 0..weekday_offset {
        current_line.push_str("   ");
    }

    let mut temp_week_lines = Vec::new();
    let mut day_idx = weekday_offset;
    
    for day in 1..=total_days {
        let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap();
        let wday = date.weekday();
        let is_today = date == today;
        let is_weekend = wday == Weekday::Sat || wday == Weekday::Sun;
        let has_event = show_events && storage.events.contains_key(&EventDate::Specific(date));

        let mut day_str = format!("{:>2}", day);

        if is_today && has_event {
            day_str = format!("{}{}{}{}", INVERT, GREEN, day_str, RESET);
        } else if is_today {
            let color = if is_weekend { RED } else { "" };
            day_str = format!("{}{}{}{}", INVERT, color, day_str, RESET);
        } else if has_event {
            day_str = format!("{}{}{}", GREEN, day_str, RESET);
        } else if is_weekend {
            day_str = format!("{}{}{}", RED, day_str, RESET);
        }

        current_line.push_str(&day_str);
        
        day_idx += 1;
        if day_idx % 7 == 0 {
            let start_day_of_week = date - Duration::days(6);
            let week_num = start_day_of_week.iso_week().week();
            
            temp_week_lines.push((week_num, current_line.clone()));
            current_line.clear();
        } else {
            current_line.push(' ');
        }
    }

    if !current_line.is_empty() {
        let last_date = NaiveDate::from_ymd_opt(year, month as u32, total_days as u32).unwrap();
        let week_num = last_date.iso_week().week();
        
        while strip_ansi_len(&current_line) < 20 {
            current_line.push(' ');
        }
        temp_week_lines.push((week_num, current_line));
    }

    for (wnum, mut grid_line) in temp_week_lines {
        let prefix = if show_week_numbers {
            format!("{}{:>2}{} ", GRAY, wnum, RESET)
        } else {
            "".to_string()
        };
        
        if use_border {
            grid_line = format!("│{}│", grid_line);
        }
        lines.push(format!("{}{}", prefix, grid_line));
    }

    let target_grid_height = if use_border { 4 + required_weeks } else { 2 + required_weeks };
    
    while lines.len() < target_grid_height {
        let mut empty_line = String::new();
        while empty_line.len() < 20 {
            empty_line.push(' ');
        }
        
        let prefix = if show_week_numbers { "   " } else { "" };
        let grid_part = if use_border { format!("│{}│", empty_line) } else { empty_line };
        lines.push(format!("{}{}", prefix, grid_part));
    }

    if use_border {
        lines.push(format!("{}└────────────────────┘", wnum_header_pad));
    }

    lines
}

fn get_required_weeks_for_month(year: i32, month: i32, sunday_first: bool) -> usize {
    let first_day = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
    let weekday_offset = if sunday_first {
        first_day.weekday().num_days_from_sunday() as usize
    } else {
        first_day.weekday().num_days_from_monday() as usize
    };
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap()
    };
    let total_days = next_month_date.signed_duration_since(first_day).num_days() as usize;
    
    (weekday_offset + total_days + 6) / 7
}


fn print_months_row(chunk: &[(i32, i32)], is_ru: bool, today: NaiveDate, use_border: bool, sunday_first: bool, is_year_mode: bool, show_events: bool, storage: &EventStorage, show_week_numbers: bool) {
    let mut required_weeks = 4;
    if is_year_mode {
        required_weeks = 6;
    } else {
        for &(year, month) in chunk {
            let weeks = get_required_weeks_for_month(year, month, sunday_first);
            if weeks > required_weeks {
                required_weeks = weeks;
            }
        }
    }

    let mut all_months_lines = Vec::new();
    for &(year, month) in chunk {
        all_months_lines.push(generate_month_lines(year, month, is_ru, today, use_border, sunday_first, required_weeks, show_events, storage, show_week_numbers));
    }

    let total_output_rows = if use_border { required_weeks + 5 } else { required_weeks + 2 };
    let block_width = if use_border { 22 } else { 20 };
    let actual_width = if show_week_numbers { block_width + 3 } else { block_width };

    for line_idx in 0..total_output_rows {
        let mut row_output = String::new();
        for month_lines in &all_months_lines {
            let raw_line = &month_lines[line_idx];
            let mut padded_line = raw_line.clone();
            
            let visible_len = strip_ansi_len(&raw_line);
            if visible_len < actual_width {
                for _ in 0..(actual_width - visible_len) {
                    padded_line.push(' ');
                }
            }

            row_output.push_str(&padded_line);
            row_output.push_str("    ");
        }
        println!("{}", row_output.trim_end());
    }
}

fn print_months_row_interactive(chunk: &[(i32, i32)], is_ru: bool, today: NaiveDate, use_border: bool, sunday_first: bool, is_year_mode: bool, show_events: bool, storage: &EventStorage, show_week_numbers: bool) {
    let mut required_weeks = 4;
    if is_year_mode {
        required_weeks = 6;
    } else {
        for &(year, month) in chunk {
            let weeks = get_required_weeks_for_month(year, month, sunday_first);
            if weeks > required_weeks {
                required_weeks = weeks;
            }
        }
    }

    let mut all_months_lines = Vec::new();
    for &(year, month) in chunk {
        all_months_lines.push(generate_month_lines(year, month, is_ru, today, use_border, sunday_first, required_weeks, show_events, storage, show_week_numbers));
    }

    let total_output_rows = if use_border { required_weeks + 5 } else { required_weeks + 2 };
    let block_width = if use_border { 22 } else { 20 };
    let actual_width = if show_week_numbers { block_width + 3 } else { block_width };

    for line_idx in 0..total_output_rows {
        let mut row_output = String::new();
        for month_lines in &all_months_lines {
            let raw_line = &month_lines[line_idx];
            let mut padded_line = raw_line.clone();
            
            let visible_len = strip_ansi_len(&raw_line);
            if visible_len < actual_width {
                for _ in 0..(actual_width - visible_len) {
                    padded_line.push(' ');
                }
            }

            row_output.push_str(&padded_line);
            row_output.push_str("    ");
        }
        print!("{}\r\n", row_output.trim_end());
    }
}

fn print_events_list(is_ru: bool, months: &[(i32, i32)], storage: &EventStorage) {
    let mut header_printed = false;

    let ensure_header = |hp: &mut bool| {
        if !*hp {
            println!();
            *hp = true;
        }
    };

    // 1. Выводим всегда актуальные задачи
    if let Some(descs) = storage.events.get(&EventDate::Always) {
        for desc in descs {
            ensure_header(&mut header_printed);
            // ЗАМЕНИЛИ ТУТ:
            println!("{} - {}", EventDate::Always.to_display_string(is_ru), desc);
        }
    }

    let mut rendered_years = Vec::new();
    for &(year, _) in months {
        if !rendered_years.contains(&year) {
            rendered_years.push(year);
        }
    }

    // 2. Выводим задачи на год
    for year in rendered_years {
        if let Some(descs) = storage.events.get(&EventDate::Yearly(year)) {
            for desc in descs {
                ensure_header(&mut header_printed);
                // ЗАМЕНИЛИ ТУТ:
                println!("{} - {}", EventDate::Yearly(year).to_display_string(is_ru), desc);
            }
        }
    }

    // 3. Выводим обычные задачи
    for &(year, month) in months {
        let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap() - Duration::days(1)
        };

        for (event_date, descs) in &storage.events {
            if let EventDate::Specific(date) = event_date {
                if date >= &start_date && date <= &end_date {
                    for desc in descs {
                        ensure_header(&mut header_printed);
                        // ЗАМЕНИЛИ ТУТ (используем метод для единообразия):
                        println!("{} - {}", event_date.to_display_string(is_ru), desc);
                    }
                }
            }
        }
    }
}

fn print_events_list_interactive(is_ru: bool, months: &[(i32, i32)], storage: &EventStorage) {
    let mut header_printed = false;

    let ensure_header = |hp: &mut bool| {
        if !*hp {
            print!("\r\n");
            *hp = true;
        }
    };

    if let Some(descs) = storage.events.get(&EventDate::Always) {
        for desc in descs {
            ensure_header(&mut header_printed);
            // ЗАМЕНИЛИ ТУТ:
            print!("{} - {}\r\n", EventDate::Always.to_display_string(is_ru), desc);
        }
    }

    let mut rendered_years = Vec::new();
    for &(year, _) in months {
        if !rendered_years.contains(&year) {
            rendered_years.push(year);
        }
    }

    for year in rendered_years {
        if let Some(descs) = storage.events.get(&EventDate::Yearly(year)) {
            for desc in descs {
                ensure_header(&mut header_printed);
                // ЗАМЕНИЛИ ТУТ:
                print!("{} - {}\r\n", EventDate::Yearly(year).to_display_string(is_ru), desc);
            }
        }
    }

    for &(year, month) in months {
        let start_date = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap() - Duration::days(1)
        } else {
            NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap() - Duration::days(1)
        };

        for (event_date, descs) in &storage.events {
            if let EventDate::Specific(date) = event_date {
                if date >= &start_date && date <= &end_date {
                    for desc in descs {
                        ensure_header(&mut header_printed);
                        // ЗАМЕНИЛИ ТУТ:
                        print!("{} - {}\r\n", event_date.to_display_string(is_ru), desc);
                    }
                }
            }
        }
    }
}

fn strip_ansi_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_ansi = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\x1b' {
            in_ansi = true;
            i += 1;
            continue;
        }
        if in_ansi {
            if chars[i] == 'm' {
                in_ansi = false;
            }
            i += 1;
            continue;
        }
        len += 1;
        i += 1;
    }
    len
}

