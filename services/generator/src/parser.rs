use std::str::FromStr;

#[derive(Debug)]
pub struct SvgMessage {
    pub data: String,
}

#[derive(Debug, Clone, Copy)]
pub enum WrapMode {
    Word,
    Char,
    WordChar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VAlignment {
    Top,
    Baseline,
}

#[derive(Debug, Clone)]
pub struct TextEntity {
    pub ty: String,
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug)]
pub struct TextMessage {
    pub x: i32,
    pub y: i32,
    pub wrap_width: i32,
    pub alignment: Alignment,
    pub valignment: VAlignment,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub color: String,
    pub bg_color: Option<String>,
    pub monospace_color: Option<String>,
    pub link_color: Option<String>,
    pub entities_key: Option<String>,
    pub text: String,
}

#[derive(Debug)]
pub struct ParsedMessage {
    pub header: String,
    pub svg: SvgMessage,
    pub texts: Vec<TextMessage>,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unexpected end of input")]
    UnexpectedEnd,
    #[error("invalid integer: {0}")]
    InvalidInt(String),
    #[error("invalid float: {0}")]
    InvalidFloat(String),
    #[error("payload too short: expected {expected} bytes, got {got}")]
    PayloadTooShort { expected: usize, got: usize },
    #[error("unknown message type: {0}")]
    UnknownType(i32),
}

fn read_until<'a>(input: &mut &'a str, delim: char) -> Result<&'a str, ParseError> {
    match input.find(delim) {
        Some(pos) => {
            let token = &input[..pos];
            *input = &input[pos + delim.len_utf8()..];
            Ok(token)
        }
        None => {
            let token = *input;
            *input = "";
            if token.is_empty() {
                Err(ParseError::UnexpectedEnd)
            } else {
                Ok(token)
            }
        }
    }
}

fn parse_int<T: FromStr>(s: &str) -> Result<T, ParseError> {
    s.trim().parse::<T>().map_err(|_| ParseError::InvalidInt(s.to_string()))
}

fn parse_float(s: &str) -> Result<f32, ParseError> {
    s.trim().parse::<f32>().map_err(|_| ParseError::InvalidFloat(s.to_string()))
}

fn parse_alignment(val: i32) -> (Alignment, VAlignment) {
    let alignment = match val % 10 {
        1 => Alignment::Center,
        2 => Alignment::Right,
        _ => Alignment::Left,
    };
    let valignment = match val / 10 {
        1 => VAlignment::Baseline,
        _ => VAlignment::Top,
    };
    (alignment, valignment)
}

/// Parse a JSON array of text entities into typed structs.
/// Shared by layout (metric-affecting entities) and wire-format parsing.
pub fn parse_entities(json_val: &serde_json::Value) -> Vec<TextEntity> {
    let mut entities = Vec::new();
    if let Some(arr) = json_val.as_array() {
        for item in arr {
            if let (Some(ty), Some(offset), Some(length)) = (
                item.get("type").and_then(|v| v.as_str()),
                item.get("offset").and_then(|v| v.as_u64()),
                item.get("length").and_then(|v| v.as_u64()),
            ) {
                entities.push(TextEntity {
                    ty: ty.to_string(),
                    offset: offset as usize,
                    length: length as usize,
                });
            }
        }
    }
    entities
}

pub fn parse_entities_map(json: Option<&serde_json::Value>) -> std::collections::HashMap<String, Vec<TextEntity>> {
    let mut map = std::collections::HashMap::new();
    if let Some(val) = json {
        if let Some(obj) = val.as_object() {
            for (k, v) in obj {
                map.insert(k.clone(), parse_entities(v));
            }
        }
    }
    map
}

pub(crate) fn parse_text(mut data: &str) -> Result<TextMessage, ParseError> {
    let x = parse_int(read_until(&mut data, ';')?)?;
    let y = parse_int(read_until(&mut data, ';')?)?;
    let wrap_width = parse_int(read_until(&mut data, ';')?)?;

    let align_val = parse_int::<i32>(read_until(&mut data, ';')?)?;
    let (alignment, valignment) = parse_alignment(align_val);

    let font_family = read_until(&mut data, ';')?.to_string();
    let font_size = parse_float(read_until(&mut data, ';')?)?;
    let font_weight = parse_int(read_until(&mut data, ';')?)?;
    let color = read_until(&mut data, ';')?.to_string();
    
    let bg_color = match read_until(&mut data, ';') {
        Ok(c) => Some(c.to_string()),
        Err(_) => None,
    };
    let bg_color = bg_color.filter(|s| !s.is_empty());
    
    let monospace_color = match read_until(&mut data, ';') {
        Ok(c) => Some(c.to_string()),
        Err(_) => None,
    };
    let monospace_color = monospace_color.filter(|s| !s.is_empty());
    
    let link_color = match read_until(&mut data, ';') {
        Ok(c) => Some(c.to_string()),
        Err(_) => None,
    };
    let link_color = link_color.filter(|s| !s.is_empty());
    
    let entities_key = match read_until(&mut data, ';') {
        Ok(c) => Some(c.to_string()),
        Err(_) => None,
    };
    let entities_key = entities_key.filter(|s| !s.is_empty());

    let text = data.to_string();

    Ok(TextMessage {
        x, y, wrap_width, alignment, valignment,
        font_family, font_size, font_weight,
        color, bg_color, monospace_color, link_color,
        entities_key, text,
    })
}

pub fn parse(mut input: &str) -> Result<ParsedMessage, ParseError> {
    let mut msg = ParsedMessage {
        header: String::new(),
        svg: SvgMessage { data: String::new() },
        texts: Vec::new(),
    };

    while !input.is_empty() {
        let type_str = read_until(&mut input, ';')?;
        let len_str = read_until(&mut input, ';')?;

        let msg_type: i32 = parse_int(type_str)?;
        let len: usize = parse_int(len_str)?;

        if input.len() < len {
            return Err(ParseError::PayloadTooShort { expected: len, got: input.len() });
        }

        let data = &input[..len];
        input = &input[len..];

        if input.starts_with(',') {
            input = &input[1..];
        }

        match msg_type {
            0 => msg.svg = SvgMessage { data: data.to_string() },
            1 => msg.texts.push(parse_text(data)?),
            2 => msg.header = data.to_string(),
            t => return Err(ParseError::UnknownType(t)),
        }
    }

    Ok(msg)
}
