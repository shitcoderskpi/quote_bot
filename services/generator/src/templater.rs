use serde::Deserialize;
use minijinja::Environment;

use crate::layout::QuoteLayout;

#[derive(Deserialize, Debug)]
pub struct InputMessage {
    pub header: Option<serde_json::Value>,
    pub entities: Option<serde_json::Value>,
    pub username: Option<String>,
    pub user_status: Option<String>,
    pub user_role: Option<String>,
    pub content: Option<String>,
    pub image: Option<String>,
    pub dpi: Option<f32>,
    pub theme: Option<String>,
}

/// Font specification extracted from a template text block.
#[derive(Debug, Clone)]
pub struct FontSpec {
    pub family: String,
    pub size: f32,
    pub weight: f32,
}

impl Default for FontSpec {
    fn default() -> Self {
        Self {
            family: "sans-serif".to_string(),
            size: 15.0,
            weight: 400.0,
        }
    }
}

/// A single block in a parsed template file.
#[derive(Debug)]
pub struct TemplateBlock {
    /// Wire-format message type (0=SVG, 1=Text, 3=RichText).
    pub block_type: i32,
    /// The minijinja template body (everything after `type;byte_len;`).
    pub body_template: String,
    /// Pre-extracted font info for text/rich-text blocks.
    pub font: Option<FontSpec>,
}

/// A parsed template file, ready for font lookups and rendering.
/// Created once per template and reused for layout computation and payload generation.
#[derive(Debug)]
pub struct ParsedTemplate {
    pub blocks: Vec<TemplateBlock>,
}

impl ParsedTemplate {
    /// Parse a template file into structured blocks.
    /// Extracts font specs from text blocks (type 1 and 3) at parse time.
    pub fn parse(template_str: &str) -> Self {
        let mut blocks = Vec::new();

        for raw_block in template_str.split(",\n") {
            let raw_block = raw_block.trim_end_matches(',');
            if raw_block.is_empty() {
                continue;
            }

            let parts: Vec<&str> = raw_block.splitn(3, ';').collect();
            if parts.len() < 3 {
                continue;
            }

            let block_type: i32 = match parts[0].trim().parse() {
                Ok(t) => t,
                Err(_) => continue,
            };
            let body_template = parts[2].to_string();

            // Extract font spec from text/rich-text blocks.
            // Body fields: x;y;wrap;align;family;size;weight;...
            // Font is at positions 4, 5, 6 within the body.
            let font = if block_type == 1 || block_type == 3 {
                let body_parts: Vec<&str> = body_template.splitn(8, ';').collect();
                if body_parts.len() >= 7 {
                    Some(FontSpec {
                        family: body_parts[4].trim().to_string(),
                        size: body_parts[5].trim().parse().unwrap_or(15.0),
                        weight: body_parts[6].trim().parse().unwrap_or(400.0),
                    })
                } else {
                    None
                }
            } else {
                None
            };

            blocks.push(TemplateBlock {
                block_type,
                body_template,
                font,
            });
        }

        ParsedTemplate { blocks }
    }

    /// Find the font spec for a text block whose body contains the given marker.
    /// Falls back to default font if no matching block is found.
    pub fn font_for(&self, marker: &str) -> FontSpec {
        for block in &self.blocks {
            if (block.block_type == 1 || block.block_type == 3)
                && block.body_template.contains(marker)
            {
                if let Some(ref font) = block.font {
                    return font.clone();
                }
            }
        }
        FontSpec::default()
    }

    /// Render all template blocks into wire-format payload string.
    /// Prepends the header block (type 2) and renders each template block
    /// with the given input message and computed layout.
    pub fn render(
        &self,
        msg: &InputMessage,
        layout: &QuoteLayout,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let env = Environment::new();

        let ctx = minijinja::context! {
            // Content variables
            username => msg.username.as_deref().unwrap_or(""),
            user_status => msg.user_status.as_deref(),
            user_role => msg.user_role.as_deref().unwrap_or("member"),
            content => msg.content.as_deref().unwrap_or("").trim_end(),
            entities => msg.entities.as_ref()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "[]".to_string()),
            image => msg.image.as_deref().unwrap_or(""),
            // Layout — canvas
            svg_width => layout.canvas.width,
            svg_height => layout.canvas.height,
            // Layout — bubble
            bubble_width => layout.bubble.width,
            bubble_height => layout.bubble.height,
            // Layout — text wrap
            wrap_width => layout.wrap_width,
        };

        let mut payload = String::new();

        // Header block (type 2)
        let header_str = msg.header.as_ref()
            .map(|h| h.to_string())
            .unwrap_or_else(|| "{}".to_string());
        payload.push_str("2;");
        payload.push_str(&header_str.len().to_string());
        payload.push(';');
        payload.push_str(&header_str);
        payload.push(',');

        // Template blocks
        for block in &self.blocks {
            if msg.user_status.is_none() && block.body_template.contains("user_status") {
                continue;
            }

            let rendered = env.render_str(&block.body_template, &ctx)?;

            payload.push_str(&block.block_type.to_string());
            payload.push(';');
            payload.push_str(&rendered.len().to_string());
            payload.push(';');
            payload.push_str(&rendered);
            payload.push(',');
        }

        Ok(payload)
    }
}
