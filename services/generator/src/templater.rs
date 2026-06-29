use serde::Deserialize;
use minijinja::Environment;
use crate::config;
use crate::layout::LayoutDimensions;

#[derive(Deserialize, Debug)]
pub struct InputMessage {
    #[serde(rename = "chat-id")]
    pub chat_id: Option<i64>,
    pub username: Option<String>,
    pub user_status: Option<String>,
    pub content: Option<String>,
    pub image: Option<String>,
}

pub fn render_payload(
    cfg: &config::Config,
    msg: &InputMessage,
    dims: &LayoutDimensions,
) -> Result<String, String> {
    let template_str = std::fs::read_to_string(&cfg.template_path)
        .map_err(|e| format!("Failed to read template file: {}", e))?;
    let env = Environment::new();
    
    let ctx = minijinja::context! {
        username => msg.username.as_deref().unwrap_or(""),
        user_status => msg.user_status.as_deref(),
        content => msg.content.as_deref().unwrap_or("").trim_end(),
        image => msg.image.as_deref().unwrap_or(""),
        wrap_width => dims.wrap_width,
        bubble_width => dims.bubble_width,
        svg_width => dims.svg_width,
        bubble_height => dims.bubble_height,
        svg_height => dims.svg_height,
        status_x => dims.status_x,
    };

    let mut payload = String::new();
    
    let chat_id_str = msg.chat_id.unwrap_or(0).to_string();
    payload.push_str("2;");
    payload.push_str(&chat_id_str.len().to_string());
    payload.push_str(";");
    payload.push_str(&chat_id_str);
    payload.push(',');

    for block in template_str.split(",\n") {
        let block = block.trim_end_matches(',');
        if block.is_empty() {
            continue;
        }

        let parts: Vec<&str> = block.splitn(3, ';').collect();
        if parts.len() < 3 {
            continue;
        }

        let block_type = parts[0];
        let template_body = parts[2];

        if msg.user_status.is_none() && template_body.contains("user_status") {
            continue;
        }

        let rendered = env.render_str(template_body, &ctx)
            .map_err(|e| format!("Template render error: {}", e))?;
            
        payload.push_str(block_type);
        payload.push(';');
        payload.push_str(&rendered.len().to_string());
        payload.push(';');
        payload.push_str(&rendered);
        payload.push(',');
    }

    Ok(payload)
}
