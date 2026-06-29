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

#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
pub struct TextMessage {
    pub x: i32,
    pub y: i32,
    pub wrap_width: i32,
    pub alignment: Alignment,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub color: String,
    pub text: String,
}

#[derive(Debug)]
pub struct ParsedMessage {
    pub chat_id: String,
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

fn parse_text(mut data: &str) -> Result<TextMessage, ParseError> {
    let x = parse_int(read_until(&mut data, ';')?)?;
    let y = parse_int(read_until(&mut data, ';')?)?;
    let wrap_width = parse_int(read_until(&mut data, ';')?)?;

    let alignment = match parse_int::<i32>(read_until(&mut data, ';')?)? {
        1 => Alignment::Center,
        2 => Alignment::Right,
        _ => Alignment::Left,
    };

    let font_family = read_until(&mut data, ';')?.to_string();
    let font_size = parse_float(read_until(&mut data, ';')?)?;
    let font_weight = parse_int(read_until(&mut data, ';')?)?;
    let color = read_until(&mut data, ';')?.to_string();
    let text = data.to_string();

    Ok(TextMessage { x, y, wrap_width, alignment, font_family, font_size, font_weight, color, text })
}

pub fn parse(mut input: &str) -> Result<ParsedMessage, ParseError> {
    let mut msg = ParsedMessage {
        chat_id: String::new(),
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
            2 => msg.chat_id = data.to_string(),
            t => return Err(ParseError::UnknownType(t)),
        }
    }

    Ok(msg)
}
