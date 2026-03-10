/// 搜索测试 - 打印原始响应查看数据
///
/// 运行: cargo run --example search_test

use ncm_api::create_client;

#[tokio::main]
async fn main() {
    let client = create_client(None);

    // 1. 搜索歌曲
    println!("=== 搜索: 晴天 周杰伦 ===\n");
    match client.cloudsearch("晴天 周杰伦", Some(1), Some(3), None).await {
        Ok(resp) => {
            println!("状态码: {}", resp.status);
            // 打印格式化的 JSON（限制一下长度）
            let json_str = serde_json::to_string_pretty(&resp.body).unwrap();
            // 只打印前 2000 字符
            if json_str.len() > 2000 {
                println!("{}", &json_str[..2000]);
                println!("... (共 {} 字节)", json_str.len());
            } else {
                println!("{}", json_str);
            }

            // 尝试多种可能的路径提取歌曲
            println!("\n--- 解析歌曲 ---");
            let paths = [
                "result.songs",
                "data.songs",
                "songs",
                "result.song.songs",
            ];
            for path in paths {
                let parts: Vec<&str> = path.split('.').collect();
                let mut node = &resp.body;
                let mut found = true;
                for part in &parts {
                    if let Some(n) = node.get(part) {
                        node = n;
                    } else {
                        found = false;
                        break;
                    }
                }
                if found && node.is_array() {
                    println!("✅ 在路径 '{}' 找到歌曲列表!", path);
                    for song in node.as_array().unwrap() {
                        let name = song["name"].as_str().unwrap_or("?");
                        let artist = song["ar"][0]["name"].as_str().unwrap_or("?");
                        let id = song["id"].as_i64().unwrap_or(0);
                        println!("  🎵 {} - {} (ID: {})", name, artist, id);
                    }
                    break;
                }
            }

            // 打印 cookies
            if !resp.cookie.is_empty() {
                println!("\n--- Cookies ({}) ---", resp.cookie.len());
                for c in &resp.cookie {
                    println!("  {}", &c[..c.len().min(80)]);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 搜索失败: {}", e);
            // 如果是 API 错误，尝试打印详细信息
            if let ncm_api::NcmError::Api { code, msg } = &e {
                eprintln!("   code={}, msg={}", code, msg);
            }
        }
    }

    // 2. 歌曲详情
    println!("\n\n=== 歌曲详情: ID 186016 (晴天) ===\n");
    match client.song_detail(&[186016]).await {
        Ok(resp) => {
            println!("状态码: {}", resp.status);
            let json_str = serde_json::to_string_pretty(&resp.body).unwrap();
            if json_str.len() > 1500 {
                println!("{}", &json_str[..1500]);
                println!("... (共 {} 字节)", json_str.len());
            } else {
                println!("{}", json_str);
            }
        }
        Err(e) => eprintln!("❌ 获取详情失败: {}", e),
    }

    // 3. 歌词
    println!("\n\n=== 歌词: ID 186016 (晴天) ===\n");
    match client.lyric(186016).await {
        Ok(resp) => {
            println!("状态码: {}", resp.status);
            if let Some(lrc) = resp.body["lrc"]["lyric"].as_str() {
                // 只打印前 500 字符
                let display = if lrc.len() > 500 { &lrc[..500] } else { lrc };
                println!("{}", display);
                if lrc.len() > 500 {
                    println!("... (共 {} 字符)", lrc.len());
                }
            } else {
                println!("歌词字段: {:?}", resp.body.get("lrc"));
            }
        }
        Err(e) => eprintln!("❌ 获取歌词失败: {}", e),
    }
}
