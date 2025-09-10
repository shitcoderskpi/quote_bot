use redis::{Commands, RedisResult};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn main() -> RedisResult<()> {
    // Шаг 1: Подключение к Redis
    // Сначала мы устанавливаем соединение с Redis-сервером.
    let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let redis_url = format!("redis://{}/", redis_host);
    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_connection()?;

    println!("Listening for jobs on 'generate:jobs' queue...");

    // Шаг 2: Запуск основного цикла обработки заданий
    // Программа будет постоянно ждать новых заданий в очереди.
    loop {
        // Шаг 3: Получение задания из очереди (блокирующий вызов)
        // Команда BLPOP блокирует соединение, пока не появится новое задание.
        let (_queue, job_data): (String, String) = con.blpop("generate:jobs", 0.0)?;

        // Запускаем таймер сразу после получения задачи.
        let start_time = Instant::now();
        println!("Received job: {}", job_data);

        // Шаг 4: Парсинг JSON-данных
        // Проверяем, что полученные данные имеют правильный формат.
        let json_value: Value = match serde_json::from_str(&job_data) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("JSON parse error: {}", e);
                continue;
            }
        };
        println!("Processing job received from queue...");

        // Шаг 5: Извлечение SVG-шаблона
        let svg_template: String = match json_value.get("template") {
            Some(Value::String(s)) => s.clone(),
            _ => {
                eprintln!("Missing 'template' key in JSON");
                continue;
            }
        };

        // Шаг 6: Подготовка и замена данных в SVG
        // Создаем папку для сохранения готового изображения и заменяем эмодзи.
        let output_dir = "./images";
        if let Err(e) = fs::create_dir_all(output_dir) {
            eprintln!("Failed to create output dir: {}", e);
            continue;
        }

        let output_file = format!("{}/output.png", output_dir);

        // Заменяем эмодзи на локальные изображения.
        let svg_final = svg_template
            .replace("😀", r#"<image href="./twemoji_png/1f424.png" width="20" height="20"/>"#)
            .replace("🎉", r#"<image href="./twemoji_png/1f424.png" width="20" height="20"/>"#)
            .replace("❤️", r#"<image href="./twemoji_png/1f424.png" width="20" height="20"/>"#)
            .replace("🔥", r#"<image href="./twemoji_png/1f424.png" width="20" height="20"/>"#);

        // Шаг 7: Сохранение временного SVG-файла
        let tmp_svg_path = format!("{}/temp.svg", output_dir);
        if let Err(e) = fs::write(&tmp_svg_path, svg_final) {
            eprintln!("Failed to write temporary SVG: {}", e);
            continue;
        }

        // Конвертируем временный SVG-файл в PNG с помощью rsvg-convert.
        let status = Command::new("rsvg-convert")
            .arg(&tmp_svg_path)
            .arg("-o")
            .arg(&output_file)
            .status();

        // Шаг 8: Обработка результата и очистка
        match status {
            Ok(s) if s.success() => {
                // Измеряем общее время выполнения.
                let duration = start_time.elapsed();
                // Выводим результат и время в лог.
                println!("Job done! Saved to {} in {:?}", output_file, duration);
                // Удаляем временный файл.
                let _ = fs::remove_file(&tmp_svg_path);
            }
            Ok(s) => eprintln!("rsvg-convert failed with code: {}", s),
            Err(e) => eprintln!("Failed to execute rsvg-convert: {}", e),
        }
    }
}
