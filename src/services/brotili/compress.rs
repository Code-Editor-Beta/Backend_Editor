use brotli::CompressorReader;
use std::io::Read;

//brotli compression with param 9-22
pub fn compress_brotli(data: &[u8]) -> Vec<u8> {
    let mut reader = CompressorReader::new(data, 4096, 9, 22);
    let mut compressed = Vec::new();
    reader.read_to_end(&mut compressed).unwrap();
    compressed
}
