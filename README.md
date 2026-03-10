# NCM API Rust SDK

网易云音乐 API 的 Rust 原生实现，从 [NeteaseCloudMusicApi](https://github.com/imsyy/NeteaseCloudMusicApi) 移植。

## 特点

- 🦀 纯 Rust 实现，无需 Node.js 运行时
- 🔒 完整实现 weapi / eapi / linuxapi 三种加密方式
- 📦 内存占用极低（~5MB vs Node.js ~50-100MB）
- ⚡ 异步请求，基于 tokio + reqwest
- 🎵 开箱即用的 API 接口（搜索、歌曲、歌词、歌单等）

## 已实现的接口

| 接口 | 方法 | 说明 |
|------|------|------|
| `/song/detail` | `song_detail` | 歌曲详情 |
| `/song/url/v1` | `song_url_v1` | 歌曲播放链接 |
| `/lyric` | `lyric` | 歌词 |
| `/check/music` | `check_music` | 检查音乐是否可用 |
| `/cloudsearch` | `cloudsearch` | 搜索 |
| `/search/default` | `search_default` | 默认搜索关键词 |
| `/search/suggest` | `search_suggest` | 搜索建议 |
| `/user/playlist` | `user_playlist` | 用户歌单 |
| `/playlist/detail` | `playlist_detail` | 歌单详情 |
| `/playlist/track/all` | `playlist_track_all` | 歌单所有歌曲 |
| `/artist/detail` | `artist_detail` | 歌手详情 |
| `/artist/songs` | `artist_songs` | 歌手歌曲 |
| `/album` | `album` | 专辑详情 |
| `/comment/new` | `comment_new` | 新版评论 |
| `/comment/info/list` | `comment_info_list` | 评论统计数据 |
| `/recommend/songs` | `recommend_songs` | 每日推荐 |
| `/personalized` | `personalized` | 推荐歌单 |
| `/login/cellphone` | `login_cellphone` | 手机号登录 |
| `/login/status` | `login_status` | 登录状态 |
| `/user/detail` | `user_detail` | 用户详情 |
| `/personal/fm` | `personal_fm` | 私人 FM |
| `/likelist` | `likelist` | 喜欢的音乐列表 |
| `/like` | `like` | 喜欢音乐 |
| `/banner` | `banner` | 首页 Banner |

## 快速开始

### 添加依赖

```toml
[dependencies]
ncm-api = { git = "https://github.com/imsyy/ncm-api-rs.git" }
tokio = { version = "1", features = ["full"] }
```

### 使用示例

```rust
use ncm_api::create_client;

#[tokio::main]
async fn main() {
    let client = create_client(None);

    // 搜索歌曲
    let result = client.cloudsearch("晴天", Some(1), Some(10), None).await.unwrap();
    println!("{}", result.body);

    // 获取歌词
    let lyric = client.lyric(186016).await.unwrap();
    println!("{}", lyric.body["lrc"]["lyric"]);
}
```

### 带 Cookie 使用（登录后的接口）

```rust
let client = create_client(Some("MUSIC_U=xxx; __csrf=xxx".to_string()));

// 每日推荐（需要登录）
let songs = client.recommend_songs().await.unwrap();
```

### 通用调用（访问未封装的接口）

```rust
use ncm_api::{create_client, CryptoType};
use ncm_api::api::Query;
use serde_json::json;

let client = create_client(None);
let query = Query::new();

let result = client.call(
    "/api/some/endpoint",
    json!({"param": "value"}),
    &query,
    CryptoType::Weapi,
).await.unwrap();
```

## 运行示例

```bash
cargo run --example basic
```

## 扩展接口

在 `src/api.rs` 中添加新方法即可，模式非常简单：

```rust
impl ApiClient {
    pub async fn your_new_api(&self, id: i64) -> Result<ApiResponse> {
        let data = json!({ "id": id });
        let query = Query::new();
        self.request("/api/your/endpoint", data, query.to_option(CryptoType::Weapi))
            .await
    }
}
```

对应的 Node.js 接口参考 [NeteaseCloudMusicApi](https://github.com/imsyy/NeteaseCloudMusicApi) 的 `module/` 目录。

## 致谢

- [NeteaseCloudMusicApi](https://github.com/imsyy/NeteaseCloudMusicApi) - 原始 Node.js 实现

## License

MIT
