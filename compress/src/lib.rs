/**
 * 通用函数库
 */
extern crate libc;
extern crate lz4;

use std::vec::Vec;
use std::io::{Result as IoResult, Read, Write, ErrorKind};

use lz4::{BlockSize, BlockMode, ContentChecksum, EncoderBuilder, Decoder};

/*
* 压缩级别
*/
pub enum CompressLevel {
    Low = 0x1,
    Mid = 0x5,
    High = 0xa,
}

/*
* 同步压缩指定的二进制数据
*/
pub fn compress(src: &[u8], dst: &mut Vec<u8>, level: CompressLevel) -> IoResult<()> {
    dst.truncate(0);
    EncoderBuilder::new()
                    .block_size(BlockSize::Max64KB)
                    .block_mode(BlockMode::Linked)
                    .checksum(ContentChecksum::ChecksumEnabled)
                    .level(level as u32)
                    .auto_flush(true)
                    .build(dst)
                    .and_then(|mut encoder| {
                        encoder.write_all(src)?;
                        let (_, result) = encoder.finish();
                        result
                    })
}

/*
* 同步解压指定的二进制数据
*/
pub fn uncompress(src: &[u8], dst: &mut Vec<u8>) -> IoResult<()> {
    dst.truncate(0);
    Decoder::new(src)
            .and_then(|mut decoder| {
                loop {
                    match decoder.read_to_end(dst) {
                        Ok(_) => {
                            ()
                        },
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                            //重复读
                            continue;
                        },
                        Err(e) => {
                            return Err(e);
                        },
                    }
                    let (_, result) = decoder.finish();
                    return result;
                }
            })
}
