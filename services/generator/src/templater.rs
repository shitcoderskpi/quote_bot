use serde::Deserialize;
use minijinja::Environment;
use tracing::error;
use vello_svg::usvg::ImageKind::SVG;
use crate::layout::QuoteLayout;
use crate::parser::{self, ParsedMessage, SvgMessage};

const DEFAULT_FONT_FAMILY: &str = "sans-serif";
const DEFAULT_FONT_SIZE: f32 = 15.0;
const DEFAULT_FONT_WEIGHT: f32 = 400.0;

const SVG_BLOCK: u8 = 0;
const TEXT_BLOCK: u8 = 1;
const CHAT_ID_BLOCK: u8 = 2;
const RICH_TEXT_BLOCK: u8 = 3;

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
            family: DEFAULT_FONT_FAMILY.to_string(),
            size: DEFAULT_FONT_SIZE,
            weight: DEFAULT_FONT_WEIGHT,
        }
    }
}

/// A single block in a parsed template file.
#[derive(Debug)]
pub struct TemplateBlock {
    /// Wire-format message type (0=SVG, 1=Text, 3=RichText).
    pub block_type: u8,
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

            let block_type: u8 = match parts[0].trim().parse() {
                Ok(t) => t,
                Err(_) => continue,
            };
            let body_template = parts[2].to_string();

            // Extract font spec from text/rich-text blocks.
            // Body fields: x;y;wrap;align;family;size;weight;...
            // Font is at positions 4, 5, 6 within the body.
            let font = (block_type == TEXT_BLOCK || block_type == RICH_TEXT_BLOCK)
                .then(|| {
                    let body_parts: Vec<&str> = body_template.splitn(8, ';').collect();

                    if body_parts.len() >= 7 {
                        Some(FontSpec {
                            family: body_parts[4].trim().to_string(),
                            size: body_parts[5].trim().parse().unwrap_or(DEFAULT_FONT_SIZE),
                            weight: body_parts[6].trim().parse().unwrap_or(DEFAULT_FONT_WEIGHT),
                        })
                    } else {
                        error!("Error: Invalid body_template format. Expected at least 7 parts, found {}", body_parts.len());
                        None
                    }
                }).flatten();

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
            if (block.block_type == TEXT_BLOCK || block.block_type == RICH_TEXT_BLOCK)
                && block.body_template.contains(marker)
            {
                if let Some(ref font) = block.font {
                    return font.clone();
                }
            }
        }
        FontSpec::default()
    }

    /// Build a ParsedMessage directly without going through COFFIN.
    pub fn build_message(
        &self,
        msg: &InputMessage,
        layout: &QuoteLayout,
        env: &Environment,
    ) -> Result<ParsedMessage, Box<dyn std::error::Error>> {

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

        let mut svg = SvgMessage { data: String::new() };
        let mut texts = Vec::new();

        let header = msg.header.as_ref()
            .map(|h| h.to_string())
            .unwrap_or_else(|| "{}".to_string());

        for block in &self.blocks {
            if msg.user_status.is_none() && block.body_template.contains("user_status") {
                continue;
            }

            let rendered = env.render_str(&block.body_template, &ctx)?;

            match block.block_type {
                SVG_BLOCK => svg.data = rendered,
                TEXT_BLOCK=> texts.push(parser::parse_text(&rendered)?),
                RICH_TEXT_BLOCK => texts.push(parser::parse_rich_text(&rendered)?),
                _ => return Err(format!("Unknown block type: {}", block.block_type).into()),
            }
        }

        Ok(ParsedMessage {
            header,
            svg,
            texts,
        })
    }
}
