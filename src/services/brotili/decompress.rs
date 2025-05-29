use brotli::Decompressor;
use std::io::Read;

//brotli decompression
pub fn decompress_brotli(data: &[u8]) -> Vec<u8> {
    let mut decompressed = Vec::new();
    let mut reader = Decompressor::new(data, 4096);
    reader.read_to_end(&mut decompressed).unwrap();
    decompressed
}
