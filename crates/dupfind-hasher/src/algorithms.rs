use sha2::{Digest, Sha256};
use std::io::{self, Read};

// 重新导出 trait 以便外部通过 algorithms 模块访问
pub use dupfind_core::HashAlgorithm;

/// SHA-256 算法
pub struct Sha256Algo;

impl HashAlgorithm for Sha256Algo {
    fn hash(&self, reader: &mut dyn Read) -> io::Result<String> {
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 131_072]; // 128 KiB
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn name(&self) -> &'static str {
        "SHA-256"
    }
}

/// BLAKE3 算法（比 SHA-256 快数倍，默认推荐）
pub struct Blake3Algo;

impl HashAlgorithm for Blake3Algo {
    fn hash(&self, reader: &mut dyn Read) -> io::Result<String> {
        let mut hasher = blake3::Hasher::new();
        let mut buf = [0u8; 131_072];
        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(hasher.finalize().to_hex().to_string())
    }

    fn name(&self) -> &'static str {
        "BLAKE3"
    }
}
