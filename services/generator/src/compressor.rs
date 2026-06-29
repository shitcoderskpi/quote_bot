use std::io::Read;

#[derive(Debug, thiserror::Error)]
pub enum CompressorError {
    #[error("zstd decompression failed: {0}")]
    Decompress(#[from] std::io::Error),
}

pub fn decompress(input: &[u8]) -> Result<String, CompressorError> {
    let mut decoder = zstd::Decoder::new(input)?;
    let mut out = String::new();
    decoder.read_to_string(&mut out)?;
    Ok(out)
}

pub fn compress(input: &str, level: i32) -> Result<Vec<u8>, CompressorError> {
    let out = zstd::bulk::compress(input.as_bytes(), level)?;
    Ok(out)
}
