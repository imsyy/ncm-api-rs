/// IP 工具 - 随机中国 IP 生成
use rand::Rng;

/// 生成随机中国 IP 地址（简化版，使用常见 IP 段）
pub fn generate_random_chinese_ip() -> String {
    let mut rng = rand::thread_rng();

    // 常见中国 IP 段前缀
    let prefixes: &[(u8, u8, u8)] = &[
        (116, 25, 94),
        (112, 0, 255),
        (114, 0, 255),
        (119, 0, 255),
        (120, 0, 255),
        (121, 0, 255),
        (122, 0, 255),
        (123, 0, 255),
        (124, 0, 255),
        (125, 0, 255),
        (182, 0, 255),
        (183, 0, 255),
        (211, 0, 255),
        (218, 0, 255),
        (220, 0, 255),
        (221, 0, 255),
        (222, 0, 255),
        (223, 0, 255),
    ];

    let (first, min_second, max_second) = prefixes[rng.gen_range(0..prefixes.len())];
    let second = rng.gen_range(min_second..=max_second);
    let third: u8 = rng.gen_range(1..=255);
    let fourth: u8 = rng.gen_range(1..=254);

    format!("{}.{}.{}.{}", first, second, third, fourth)
}
