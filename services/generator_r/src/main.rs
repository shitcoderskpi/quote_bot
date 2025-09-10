use redis::{Commands, RedisResult};
use serde_json::Value;
use std::env;
use std::fs;
use std::process::Command;
use std::time::Instant;
use unicode_segmentation::UnicodeSegmentation;
use image::{open, imageops};

fn main() -> RedisResult<()> {
    let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let redis_url = format!("redis://{}/", redis_host);
    let client = redis::Client::open(redis_url)?;
    let mut con = client.get_connection()?;

    println!("Listening for jobs on 'generate:jobs' queue...");

    loop {
        let (_queue, job_data): (String, String) = con.blpop("generate:jobs", 0.0)?;
        let start_time = Instant::now();
        println!("Received job: {}", job_data);

        let json_value: Value = match serde_json::from_str(&job_data) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("JSON parse error: {}", e);
                continue;
            }
        };

        // Берем template как SVG
        let svg_template = match json_value.get("template") {
            Some(Value::String(s)) => s.clone(),
            _ => {
                eprintln!("Missing 'template' key in JSON");
                continue;
            }
        };

        let output_dir = "./images";
        fs::create_dir_all(output_dir).ok();
        let tmp_svg_path = format!("{}/temp.svg", output_dir);
        let base_png = format!("{}/base.png", output_dir);
        let output_file = format!("{}/output.png", output_dir);

        // Получаем font_size и начальное смещение X из JSON (по желанию)
        let font_size = json_value.get("font_size").and_then(|v| v.as_i64()).unwrap_or(15) as u32;
        let start_x = json_value.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        // 1) Обрабатываем SVG: заменяем эмодзи на пробелы и запоминаем позиции
        let mut emoji_positions = vec![];
        let mut processed_template = String::new();
        let mut offset_x = start_x;

        for g in svg_template.graphemes(true) {
            let c = g.chars().next().unwrap();
            if is_emoji(c) {
                let codepoint = get_emoji_codepoint(g);
                let path = format!("./twemoji/{}.png", codepoint);
                if fs::metadata(&path).is_ok() {
                    emoji_positions.push((path, offset_x, 0, font_size));
                    processed_template.push(' '); // заменяем эмодзи на пробел
                } else {
                    processed_template.push_str(g);
                }
            } else {
                processed_template.push_str(g);
            }
            offset_x += (font_size as f32 * 0.6) as i32; // горизонтальный отступ
        }

        // Сохраняем обработанный SVG
        if let Err(e) = fs::write(&tmp_svg_path, &processed_template) {
            eprintln!("Failed to write temporary SVG: {}", e);
            continue;
        }

        // 2) Конвертируем SVG в PNG
        let status = Command::new("rsvg-convert")
            .arg(&tmp_svg_path)
            .arg("-o")
            .arg(&base_png)
            .arg("--dpi-x")
            .arg("96")
            .arg("--dpi-y")
            .arg("96")
            .status();

        if let Ok(s) = status {
            if !s.success() {
                eprintln!("rsvg-convert failed with code: {}", s);
                let _ = fs::remove_file(&tmp_svg_path);
                continue;
            }
        } else {
            eprintln!("Failed to execute rsvg-convert");
            let _ = fs::remove_file(&tmp_svg_path);
            continue;
        }

        // 3) Накладываем эмодзи на их позиции
        if let Err(e) = overlay_emojis(&base_png, &output_file, &emoji_positions) {
            eprintln!("Failed to overlay emojis: {}", e);
        }

        // Удаляем временные файлы
        let _ = fs::remove_file(&tmp_svg_path);
        let _ = fs::remove_file(&base_png);

        let duration = start_time.elapsed();
        println!("Job done! Saved to {} in {:?}", output_file, duration);
    }
}

/// Проверка символа на эмодзи
fn is_emoji(c: char) -> bool {
    match c {
        '\u{1F600}'..='\u{1F64F}' |
        '\u{1F300}'..='\u{1F5FF}' |
        '\u{1F680}'..='\u{1F6FF}' |
        '\u{1F1E0}'..='\u{1F1FF}' |
        '\u{2600}'..='\u{26FF}' |
        '\u{2700}'..='\u{27BF}' |
        '\u{1F900}'..='\u{1F9FF}' |
        '\u{1FA70}'..='\u{1FAFF}' => true,
        _ => false,
    }
}

/// Codepoint для Twemoji
fn get_emoji_codepoint(grapheme: &str) -> String {
    grapheme.chars().map(|ch| format!("{:x}", ch as u32)).collect::<Vec<_>>().join("-")
}

/// Накладывает PNG эмодзи на базовое изображение
fn overlay_emojis(
    base_png_path: &str,
    output_png_path: &str,
    emojis: &[(String, i32, i32, u32)]
) -> Result<(), Box<dyn std::error::Error>> {
    let mut img = open(base_png_path)?;
    for (png_path, x, y, size) in emojis {
        if let Ok(emoji_img) = open(png_path) {
            let resized = emoji_img.resize_exact(*size, *size, image::imageops::FilterType::Lanczos3);
            imageops::overlay(&mut img, &resized, *x as i64, *y as i64);
        }
    }
    img.save(output_png_path)?;
    Ok(())
}
