/// 设备 ID 生成
use rand::Rng;

/// 生成 52 位十六进制设备 ID
pub fn generate_device_id() -> String {
    let hex_chars = b"0123456789ABCDEF";
    let mut rng = rand::thread_rng();
    (0..52)
        .map(|_| hex_chars[rng.gen_range(0..16)] as char)
        .collect()
}

/// 生成随机 hex 字符串（指定字节数，输出 hex）
pub fn random_hex(bytes: usize) -> String {
    let mut rng = rand::thread_rng();
    let data: Vec<u8> = (0..bytes).map(|_| rng.gen()).collect();
    hex::encode(&data)
}

/// 生成 WNMCID
pub fn generate_wnmcid() -> String {
    let mut rng = rand::thread_rng();
    let chars = b"abcdefghijklmnopqrstuvwxyz";
    let random_str: String = (0..6)
        .map(|_| chars[rng.gen_range(0..26)] as char)
        .collect();
    let timestamp = chrono::Utc::now().timestamp_millis();
    format!("{}.{}.01.0", random_str, timestamp)
}
