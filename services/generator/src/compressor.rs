use std::io::Read;

#[derive(Debug, thiserror::Error)]
pub enum CompressorError {
    #[error("zstd decompression failed: {0}")]
    Decompress(#[from] std::io::Error),
}

pub fn decompress(input: &[u8], out: &mut String) -> Result<(), CompressorError> {
    let mut decoder = zstd::Decoder::new(input)?;
    decoder.read_to_string(out)?;
    Ok(())
}

pub fn max_compressed_size(src_size: usize) -> usize {
    zstd::zstd_safe::compress_bound(src_size)
}

pub fn compress(input: &str, level: i32, out: &mut [u8]) -> std::io::Result<usize> {
    zstd::bulk::compress_to_buffer(input.as_bytes(), out, level)
}
