use dupfind::hasher::algorithms::{Blake3Algo, HashAlgorithm, Sha256Algo};
use proptest::prelude::*;

// 属性测试：生成随机字节序列，验证哈希算法的一致性属性。
//
// 核心不变式（invariants）：
// 1. 确定性 — 相同输入 → 相同输出
// 2. 碰撞抵抗 — 不同输入 → 不同输出（概率性保证）
// 3. 长度无关 — 输出长度不受输入大小影响

proptest! {
    /// 相同内容总是产生相同哈希值（确定性）
    #[test]
    fn prop_same_content_same_hash_sha256(data in any::<Vec<u8>>()) {
        let algo = Sha256Algo;
        let h1 = algo.hash(&mut std::io::Cursor::new(&data)).unwrap();
        let h2 = algo.hash(&mut std::io::Cursor::new(&data)).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn prop_same_content_same_hash_blake3(data in any::<Vec<u8>>()) {
        let algo = Blake3Algo;
        let h1 = algo.hash(&mut std::io::Cursor::new(&data)).unwrap();
        let h2 = algo.hash(&mut std::io::Cursor::new(&data)).unwrap();
        assert_eq!(h1, h2);
    }

    /// 不同内容产生不同哈希（碰撞抵抗的弱验证）
    #[test]
    fn prop_different_content_different_hash_sha256(a in any::<Vec<u8>>(), b in any::<Vec<u8>>()) {
        prop_assume!(a != b);
        let algo = Sha256Algo;
        let h1 = algo.hash(&mut std::io::Cursor::new(&a)).unwrap();
        let h2 = algo.hash(&mut std::io::Cursor::new(&b)).unwrap();
        // SHA-256 碰撞概率极低，不等内容应当不等哈希
        assert_ne!(h1, h2);
    }

    #[test]
    fn prop_different_content_different_hash_blake3(a in any::<Vec<u8>>(), b in any::<Vec<u8>>()) {
        prop_assume!(a != b);
        let algo = Blake3Algo;
        let h1 = algo.hash(&mut std::io::Cursor::new(&a)).unwrap();
        let h2 = algo.hash(&mut std::io::Cursor::new(&b)).unwrap();
        assert_ne!(h1, h2);
    }

    /// 输出长度固定：SHA-256 始终输出 64 字符十六进制
    #[test]
    fn prop_sha256_output_length(data in any::<Vec<u8>>()) {
        let algo = Sha256Algo;
        let hash = algo.hash(&mut std::io::Cursor::new(&data)).unwrap();
        assert_eq!(hash.len(), 64);
        // 验证全为十六进制字符
        assert!(hash.chars().all(|c: char| c.is_ascii_hexdigit()));
    }

    /// 输出长度固定：BLAKE3 默认 64 字符十六进制
    #[test]
    fn prop_blake3_output_length(data in any::<Vec<u8>>()) {
        let algo = Blake3Algo;
        let hash = algo.hash(&mut std::io::Cursor::new(&data)).unwrap();
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c: char| c.is_ascii_hexdigit()));
    }

    /// 空输入也能正确哈希（不崩溃）
    #[test]
    fn prop_empty_input_works(data in prop::collection::vec(0u8..=255, 0..=0)) {
        let algo = Blake3Algo;
        let hash = algo.hash(&mut std::io::Cursor::new(&data)).unwrap();
        assert_eq!(hash.len(), 64);
    }
}
