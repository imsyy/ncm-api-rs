<div align="center">

# NCM API Rust SDK

**网易云音乐 API Rust 原生实现**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-WTFPL-brightgreen.svg)](LICENSE)
[![Tokio](https://img.shields.io/badge/async-tokio-blue?logo=rust)](https://tokio.rs/)

从 [NeteaseCloudMusicApi Enhanced](https://github.com/NeteaseCloudMusicApiEnhanced/api-enhanced) 移植的 Rust 原生 SDK，无需 Node.js 运行时

</div>

---

## 项目简介

本项目是 [NeteaseCloudMusicApi Enhanced](https://github.com/NeteaseCloudMusicApiEnhanced/api-enhanced) 的 Rust 原生实现。通过跨站请求伪造 (CSRF) 和伪造请求头，调用网易云音乐官方 API，提供与 Node.js 版本 1:1 对应的接口。

所有 API 方法统一使用 `Query` 对象传参，与 Node.js 版本保持一致的调用风格，同时充分利用 Rust 的类型安全和零成本抽象特性。

## 特点

- **纯 Rust 实现** - 无需 Node.js 运行时，独立编译部署
- **完整加密支持** - 完整实现 weapi / eapi / linuxapi 三种加密方式
- **极低内存占用** - ~5MB vs Node.js ~50-100MB
- **异步非阻塞** - 基于 tokio + reqwest 的异步请求
- **300+ 开箱即用的 API 接口** - 与 Node.js 版本 1:1 对应
- **模块化设计** - 每个 API 独立文件，易于扩展和维护
- **统一的参数模式** - 所有接口使用 `Query` 对象传参，用法简洁一致
- **代理支持** - 支持 HTTP / SOCKS5 代理
- **IP 伪装** - 支持 realIP 和 randomCNIP 功能
- **HTTP 服务模式** - 内置 Axum HTTP 服务器，可直接替代 Node.js 版提供 REST API

## 快速开始

### 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
ncm-api = { git = "https://github.com/NeteaseCloudMusicApiEnhanced/api-enhanced.git", branch = "feat/rust-sdk" }
tokio = { version = "1", features = ["full"] }
```

或使用本地路径（开发时）：

```toml
[dependencies]
ncm-api = { path = "../rust-sdk" }
tokio = { version = "1", features = ["full"] }
```

### 基础使用

```rust
use ncm_api::{create_client, Query};

#[tokio::main]
async fn main() {
    // 创建客户端（不带 cookie）
    let client = create_client(None);

    // 搜索歌曲
    let query = Query::new()
        .param("keywords", "晴天 周杰伦")
        .param("type", "1")       // 1=歌曲
        .param("limit", "10");
    let result = client.cloudsearch(&query).await.unwrap();
    println!("{}", result.body);

    // 获取歌曲详情
    let query = Query::new().param("ids", "186016");
    let detail = client.song_detail(&query).await.unwrap();
    println!("{}", detail.body);

    // 获取歌词
    let query = Query::new().param("id", "186016");
    let lyric = client.lyric(&query).await.unwrap();
    println!("{}", lyric.body["lrc"]["lyric"]);

    // 获取播放链接
    let query = Query::new()
        .param("id", "186016")
        .param("level", "standard");
    let url = client.song_url_v1(&query).await.unwrap();
    println!("{}", url.body);
}
```

### 带 Cookie 使用（登录后的接口）

```rust
use ncm_api::{create_client, Query};

#[tokio::main]
async fn main() {
    // 方式一：创建客户端时传入 cookie
    let client = create_client(Some("MUSIC_U=xxx; __csrf=xxx".to_string()));

    let query = Query::new();
    let songs = client.recommend_songs(&query).await.unwrap();

    // 方式二：在 Query 中传入 cookie（覆盖客户端 cookie）
    let query = Query::new().cookie("MUSIC_U=xxx; __csrf=xxx");
    let fm = client.personal_fm(&query).await.unwrap();
}
```

### Query 参数说明

`Query` 是所有 API 的统一参数载体，支持链式调用：

```rust
let mut query = Query::new()
    // 业务参数 - 对应 Node.js 版本中 req.query 传入的参数
    .param("id", "186016")
    .param("limit", "30")
    .param("offset", "0")
    // 可选：覆盖 cookie
    .cookie("MUSIC_U=xxx");

// 可选：设置代理
query.proxy = Some("socks5://127.0.0.1:1080".to_string());
// 可选：设置真实 IP（伪装 X-Real-IP 请求头）
query.real_ip = Some("116.25.146.177".to_string());
// 可选：使用随机中国 IP
query.random_cn_ip = true;

// 读取参数
query.get("id");              // Some("186016")
query.get_or("limit", "30");  // "30"
```

## 调用前须知

> 本项目仅供学习使用，请尊重版权，请勿利用此项目从事商业行为或进行破坏版权行为

> 不要频繁调用登录接口，不然可能会被风控，登录状态还存在就不要重复调用登录接口

> 部分接口如登录接口不能调用太频繁，否则可能会触发 503 错误或者 IP 高频错误，若需频繁调用，需要准备 IP 代理池

> 由于网易限制，此项目在国外服务器或部分国内云服务上使用会受到限制，如 `460 cheating异常`，如需解决，可使用 `real_ip` 参数，传进国内 IP 解决

> 301 错误基本都是没登录就调用了需要登录的接口，如果登录了还是提示 301，基本都是 cookie 问题

### Cookie 说明

登录接口返回的 `ApiResponse` 中包含 `cookie` 字段（`Vec<String>`），可以保存到本地后在后续请求中使用：

```rust
// 登录
let query = Query::new()
    .param("phone", "13xxx")
    .param("password", "xxx");
let result = client.login_cellphone(&query).await?;

// 保存 cookie
let cookies = result.cookie.join("; ");

// 后续请求使用 cookie
let query = Query::new().cookie(&cookies);
let account = client.user_account(&query).await?;
```

也可以直接从浏览器中获取 cookie 值，只需要其中 key 为 `MUSIC_U` 的数据即可：

```rust
let query = Query::new().cookie("MUSIC_U=xxxx");
```

### realIP 参数

在国外服务器或 Vercel 等云服务上使用时，需要设置 `real_ip` 参数传入国内 IP：

```rust
let mut query = Query::new().param("id", "1969519579");
query.real_ip = Some("116.25.146.177".to_string());
let result = client.song_url_v1(&query).await?;
```

### randomCNIP 参数

也可以使用随机中国 IP 功能，无需手动指定 IP：

```rust
let mut query = Query::new().param("id", "1969519579");
query.random_cn_ip = true;
let result = client.song_url_v1(&query).await?;
```

### 代理支持

在 Query 参数中设置 proxy 即可让该次请求使用代理：

```rust
let mut query = Query::new().param("id", "33894312");
query.proxy = Some("http://121.196.226.246:84".to_string());
// 也支持 socks5 代理
// query.proxy = Some("socks5://127.0.0.1:1080".to_string());
let result = client.song_url(&query).await?;
```

---

## HTTP 服务模式

除了作为 Rust 库直接调用，本项目还支持以 HTTP 服务器模式运行，提供与 Node.js 版本完全兼容的 REST API 接口，前端可以无缝切换。

### 启动服务器

```bash
# 编译并运行（默认监听 0.0.0.0:3000）
cargo run --features server --bin ncm-server

# 或先编译再运行
cargo build --release --features server --bin ncm-server
./target/release/ncm-server
```

### 环境变量配置

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `NCM_HOST` | 监听地址 | `0.0.0.0` |
| `NCM_PORT` | 监听端口 | `3000` |
| `CORS_ALLOW_ORIGIN` | CORS 允许的 Origin | `*`（允许所有） |

```bash
# 示例：自定义端口和 CORS
NCM_HOST=127.0.0.1 NCM_PORT=8080 CORS_ALLOW_ORIGIN=http://localhost:5173 cargo run --features server --bin ncm-server
```

### 前端调用示例

接口路径与 Node.js 版完全一致，方法名中的下划线 `_` 转换为斜杠 `/`：

```js
// 搜索歌曲
const res = await axios.get('/cloudsearch', { params: { keywords: '海阔天空' } })

// 获取歌曲详情
const res = await axios.get('/song/detail', { params: { ids: '347230' } })

// POST 方式调用
const res = await axios.post('/login/cellphone', {
  phone: '138xxxx8000',
  password: 'xxx',
})

// 带 Cookie 调用需要登录的接口
const res = await axios.get('/user/playlist', {
  params: { uid: '32953014', cookie: 'MUSIC_U=xxx' },
})
```

### 路由映射规则

| 方法名 | HTTP 路由 | 说明 |
|--------|-----------|------|
| `song_detail` | `/song/detail` | 下划线转斜杠（默认规则） |
| `login_cellphone` | `/login/cellphone` | 同上 |
| `daily_signin` | `/daily_signin` | 特殊路由，保留下划线 |
| `fm_trash` | `/fm_trash` | 特殊路由，保留下划线 |
| `personal_fm` | `/personal_fm` | 特殊路由，保留下划线 |
| `avatar_upload` | `/avatar/upload` | POST multipart/form-data |
| `voice_upload` | `/voice/upload` | POST multipart/form-data |

所有路由均支持 GET 和 POST 两种请求方式（上传接口仅 POST）。

### 作为库集成到你的项目

如果你已有 Axum 项目，可以直接集成路由：

```rust
use ncm_api::{create_client, server::{build_app, build_app_with_config, ServerConfig}};

// 方式一：快速构建
let app = build_app(create_client(None));

// 方式二：自定义配置
let config = ServerConfig {
    host: "127.0.0.1".to_string(),
    port: 8080,
    cors_origin: Some("http://localhost:5173".to_string()),
};
let app = build_app_with_config(create_client(None), &config);

// 启动
let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
axum::serve(listener, app).await?;
```

### 自动路由注册

新增 API 时无需手动注册路由。`build.rs` 会在编译期自动扫描 `src/api/mod.rs` 中的模块声明，生成对应的路由注册代码。只需：

1. 创建 `src/api/xxx_yyy.rs` 文件，实现 `ApiClient` 方法
2. 在 `src/api/mod.rs` 中添加 `mod xxx_yyy;`

路由 `/xxx/yyy` 会自动注册，无需其他操作。

---

## API 文档

**完整接口文档请查看 [docs/API.md](docs/API.md)**，涵盖全部接口的详细参数说明和调用示例。

### 接口速查表

<details>
<summary><strong>登录相关</strong>（16 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `login` | 邮箱登录 |
| `login_cellphone` | 手机号登录 |
| `login_qr_key` | 二维码 key 生成 |
| `login_qr_create` | 二维码生成 |
| `login_qr_check` | 二维码扫码状态 |
| `login_refresh` | 刷新登录 |
| `login_status` | 登录状态 |
| `logout` | 退出登录 |
| `register_anonimous` | 游客登录 |
| `register_cellphone` | 注册/修改密码 |
| `captcha_sent` | 发送验证码 |
| `captcha_verify` | 验证验证码 |
| `cellphone_existence_check` | 检测手机号是否注册 |
| `activate_init_profile` | 初始化昵称 |
| `nickname_check` | 重复昵称检测 |
| `rebind` | 更换绑定手机 |

</details>

<details>
<summary><strong>用户相关</strong>（32 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `user_detail` | 用户详情 |
| `user_detail_new` | 用户详情(新) |
| `user_account` | 账号信息 |
| `user_subcount` | 收藏计数 |
| `user_level` | 用户等级 |
| `user_binding` | 绑定信息 |
| `user_bindingcellphone` | 绑定手机 |
| `user_replacephone` | 更换手机 |
| `user_update` | 更新用户信息 |
| `user_playlist` | 用户歌单 |
| `user_playlist_create` | 创建歌单列表 |
| `user_playlist_collect` | 收藏歌单列表 |
| `user_follows` | 关注列表 |
| `user_followeds` | 粉丝列表 |
| `user_follow_mixed` | 关注的用户/歌手 |
| `user_mutualfollow_get` | 是否互相关注 |
| `user_event` | 用户动态 |
| `user_record` | 播放记录 |
| `user_dj` | 用户电台 |
| `user_audio` | 用户音频 |
| `user_comment_history` | 历史评论 |
| `user_medal` | 用户徽章 |
| `user_social_status` | 用户状态 |
| `user_social_status_edit` | 编辑状态 |
| `user_social_status_rcmd` | 相同状态用户 |
| `user_social_status_support` | 支持的状态 |
| `follow` | 关注/取消关注 |
| `pl_count` | 私信和通知数量 |
| `countries_code_list` | 国家编码列表 |
| `setting` | 用户设置 |
| `get_userids` | 获取用户 ID |
| `avatar_upload` | 上传头像 |

</details>

<details>
<summary><strong>歌曲相关</strong>（30 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `song_detail` | 歌曲详情 |
| `song_url` | 歌曲播放链接 |
| `song_url_v1` | 歌曲播放链接(新) |
| `song_url_v1_302` | 302 重定向到歌曲 URL |
| `song_url_ncmget` | NCM 获取歌曲 URL |
| `song_url_match` | 歌曲解锁匹配 |
| `song_download_url` | 歌曲下载链接 |
| `song_download_url_v1` | 歌曲下载链接(新) |
| `check_music` | 音乐是否可用 |
| `lyric` | 歌词 |
| `lyric_new` | 逐字歌词 |
| `like` | 喜欢音乐 |
| `likelist` | 喜欢的音乐列表 |
| `song_like_check` | 歌曲是否喜爱 |
| `scrobble` | 听歌打卡 |
| `song_order_update` | 调整歌曲顺序 |
| `song_chorus` | 副歌时间 |
| `song_wiki_summary` | 歌曲百科 |
| `song_music_detail` | 歌曲音质详情 |
| `song_red_count` | 红心数量 |
| `song_dynamic_cover` | 动态封面 |
| `song_downlist` | 会员下载记录 |
| `song_monthdownlist` | 本月下载记录 |
| `song_singledownlist` | 已购买单曲 |
| `song_purchased` | 已购买歌曲 |
| `song_lyrics_mark` | 歌词摘录信息 |
| `song_lyrics_mark_add` | 添加歌词摘录 |
| `song_lyrics_mark_del` | 删除歌词摘录 |
| `song_lyrics_mark_user_page` | 我的歌词本 |
| `audio_match` | 听歌识曲 |

</details>

<details>
<summary><strong>搜索相关</strong>（8 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `search` | 搜索 |
| `cloudsearch` | 搜索(更全) |
| `search_default` | 默认搜索关键词 |
| `search_hot` | 热搜列表(简略) |
| `search_hot_detail` | 热搜列表(详细) |
| `search_suggest` | 搜索建议 |
| `search_multimatch` | 搜索多重匹配 |
| `search_match` | 搜索匹配 |

</details>

<details>
<summary><strong>歌单相关</strong>（28 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `playlist_create` | 新建歌单 |
| `playlist_delete` | 删除歌单 |
| `playlist_subscribe` | 收藏/取消收藏歌单 |
| `playlist_subscribers` | 歌单收藏者 |
| `playlist_detail` | 歌单详情 |
| `playlist_detail_dynamic` | 歌单详情动态 |
| `playlist_detail_rcmd_get` | 相关歌单推荐 |
| `playlist_track_all` | 歌单所有歌曲 |
| `playlist_tracks` | 添加/删除歌曲 |
| `playlist_track_add` | 收藏视频到歌单 |
| `playlist_track_delete` | 删除歌单视频 |
| `playlist_update` | 更新歌单 |
| `playlist_desc_update` | 更新歌单描述 |
| `playlist_name_update` | 更新歌单名 |
| `playlist_tags_update` | 更新歌单标签 |
| `playlist_cover_update` | 歌单封面上传 |
| `playlist_order_update` | 调整歌单顺序 |
| `playlist_update_playcount` | 更新播放量 |
| `playlist_catlist` | 歌单分类 |
| `playlist_hot` | 热门歌单分类 |
| `playlist_category_list` | 歌单分类列表 |
| `playlist_highquality_tags` | 精品歌单标签 |
| `playlist_privacy` | 歌单隐私设置 |
| `playlist_mylike` | 我喜欢的歌单 |
| `playlist_video_recent` | 最近播放视频 |
| `playlist_import_name_task_create` | 歌单导入 |
| `playlist_import_task_status` | 导入任务状态 |
| `related_playlist` | 相关歌单 |

</details>

<details>
<summary><strong>评论相关</strong>（15 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `comment_music` | 歌曲评论 |
| `comment_album` | 专辑评论 |
| `comment_playlist` | 歌单评论 |
| `comment_mv` | MV 评论 |
| `comment_dj` | 电台节目评论 |
| `comment_video` | 视频评论 |
| `comment_event` | 动态评论 |
| `comment_floor` | 楼层评论 |
| `comment_hot` | 热门评论 |
| `comment_like` | 点赞评论 |
| `comment_new` | 新版评论 |
| `comment_hug_list` | 抱一抱列表 |
| `comment_info_list` | 评论统计 |
| `hug_comment` | 抱一抱评论 |
| `starpick_comments_summary` | 精选评论摘要 |

</details>

<details>
<summary><strong>歌手相关</strong>（16 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `artists` | 歌手信息 |
| `artist_detail` | 歌手详情 |
| `artist_detail_dynamic` | 歌手详情动态 |
| `artist_songs` | 歌手全部歌曲 |
| `artist_album` | 歌手专辑 |
| `artist_desc` | 歌手描述 |
| `artist_mv` | 歌手 MV |
| `artist_list` | 歌手分类列表 |
| `artist_sub` | 收藏/取消收藏歌手 |
| `artist_sublist` | 收藏的歌手列表 |
| `artist_top_song` | 歌手热门歌曲 |
| `artist_fans` | 歌手粉丝 |
| `artist_follow_count` | 歌手关注数 |
| `artist_new_mv` | 关注歌手新 MV |
| `artist_new_song` | 关注歌手新歌 |
| `artist_video` | 歌手视频 |

</details>

<details>
<summary><strong>专辑相关</strong>（11 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `album` | 专辑内容 |
| `album_detail` | 专辑详情 |
| `album_detail_dynamic` | 专辑动态信息 |
| `album_sub` | 收藏/取消收藏专辑 |
| `album_sublist` | 已收藏专辑列表 |
| `album_newest` | 最新专辑 |
| `album_new` | 新碟上架 |
| `album_list` | 数字专辑列表 |
| `album_list_style` | 专辑风格列表 |
| `album_privilege` | 专辑歌曲音质 |
| `album_songsaleboard` | 专辑销量榜 |

</details>

<details>
<summary><strong>MV 相关</strong>（10 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `mv_all` | 全部 MV |
| `mv_first` | 最新 MV |
| `mv_exclusive_rcmd` | 网易出品 MV |
| `mv_detail` | MV 数据 |
| `mv_detail_info` | MV 点赞转发评论数 |
| `mv_url` | MV 地址 |
| `mv_sub` | 收藏/取消收藏 MV |
| `mv_sublist` | 收藏的 MV 列表 |
| `personalized_mv` | 推荐 MV |
| `top_mv` | MV 排行 |

</details>

<details>
<summary><strong>视频相关</strong>（13 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `video_group_list` | 视频标签列表 |
| `video_category_list` | 视频分类列表 |
| `video_group` | 标签下的视频 |
| `video_timeline_all` | 全部视频列表 |
| `video_timeline_recommend` | 推荐视频 |
| `video_detail` | 视频详情 |
| `video_detail_info` | 视频点赞转发评论数 |
| `video_url` | 视频播放地址 |
| `video_sub` | 收藏视频 |
| `related_allvideo` | 相关视频 |
| `mlog_url` | Mlog 地址 |
| `mlog_to_video` | Mlog 转视频 |
| `mlog_music_rcmd` | Mlog 音乐推荐 |

</details>

<details>
<summary><strong>电台相关</strong>（25 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `dj_banner` | 电台 banner |
| `dj_personalize_recommend` | 电台个性推荐 |
| `dj_subscriber` | 电台订阅者列表 |
| `dj_catelist` | 电台分类列表 |
| `dj_category_excludehot` | 非热门电台分类 |
| `dj_category_recommend` | 电台分类推荐 |
| `dj_detail` | 电台详情 |
| `dj_hot` | 热门电台 |
| `dj_radio_hot` | 电台 - 类别热门 |
| `dj_program` | 电台节目列表 |
| `dj_program_detail` | 电台节目详情 |
| `dj_program_toplist` | 节目排行榜 |
| `dj_program_toplist_hours` | 节目24小时榜 |
| `dj_recommend` | 电台推荐 |
| `dj_recommend_type` | 电台推荐类型 |
| `dj_sub` | 订阅/取消订阅电台 |
| `dj_sublist` | 订阅的电台列表 |
| `dj_toplist` | 电台排行榜 |
| `dj_toplist_hours` | 电台24小时榜 |
| `dj_toplist_newcomer` | 电台新人榜 |
| `dj_toplist_pay` | 付费电台榜 |
| `dj_toplist_popular` | 电台热门榜 |
| `dj_paygift` | 付费精品 |
| `dj_today_perfered` | 今日优选 |
| `dj_radio_top` | 电台排行 |

</details>

<details>
<summary><strong>推荐相关</strong>（25 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `personalized` | 推荐歌单 |
| `personalized_newsong` | 推荐新音乐 |
| `personalized_djprogram` | 推荐电台 |
| `personalized_privatecontent` | 独家放送(入口) |
| `personalized_privatecontent_list` | 独家放送列表 |
| `recommend_songs` | 每日推荐歌曲 |
| `recommend_resource` | 每日推荐歌单 |
| `recommend_songs_dislike` | 不喜欢推荐 |
| `history_recommend_songs` | 历史日推 |
| `history_recommend_songs_detail` | 历史日推详情 |
| `program_recommend` | 推荐节目 |
| `homepage_block_page` | 首页-发现 |
| `homepage_dragon_ball` | 首页圆形图标入口 |
| `banner` | 首页 Banner |
| `daily_signin` | 每日签到 |
| `personal_fm` | 私人 FM |
| `personal_fm_mode` | 私人 FM 模式 |
| `fm_trash` | 垃圾桶 |
| `playmode_intelligence_list` | 心动模式/智能播放 |
| `playmode_song_vector` | 随机播放模式 |
| `simi_song` | 相似歌曲 |
| `simi_artist` | 相似歌手 |
| `simi_playlist` | 相似歌单 |
| `simi_mv` | 相似 MV |
| `simi_user` | 相似用户 |

</details>

<details>
<summary><strong>排行榜相关</strong>（11 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `toplist` | 所有榜单 |
| `toplist_detail` | 所有榜单摘要 |
| `toplist_detail_v2` | 所有榜单摘要 v2 |
| `toplist_artist` | 歌手榜 |
| `top_song` | 新歌速递 |
| `top_album` | 新碟上架 |
| `top_artists` | 热门歌手 |
| `top_mv` | MV 排行 |
| `top_playlist` | 歌单(网友精选碟) |
| `top_playlist_highquality` | 精品歌单 |
| `top_list` | 排行榜详情 |

</details>

<details>
<summary><strong>云盘相关</strong>（9 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `user_cloud` | 云盘数据 |
| `user_cloud_detail` | 云盘数据详情 |
| `user_cloud_del` | 云盘歌曲删除 |
| `cloud` | 云盘上传 |
| `cloud_import` | 云盘导入 |
| `cloud_match` | 云盘歌曲匹配纠正 |
| `cloud_lyric_get` | 云盘歌词 |
| `cloud_upload_token` | 上传凭证 |
| `cloud_upload_complete` | 完成上传 |

</details>

<details>
<summary><strong>私信/动态相关</strong>（15 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `msg_private` | 私信列表 |
| `msg_private_history` | 私信历史 |
| `msg_comments` | 评论通知 |
| `msg_forwards` | 转发通知 |
| `msg_notices` | 通知消息 |
| `msg_recentcontact` | 最近联系人 |
| `send_text` | 发送文本私信 |
| `send_song` | 发送歌曲私信 |
| `send_playlist` | 发送歌单私信 |
| `send_album` | 发送专辑私信 |
| `share_resource` | 分享到动态 |
| `resource_like` | 资源点赞 |
| `event` | 动态列表 |
| `event_del` | 删除动态 |
| `event_forward` | 转发动态 |

</details>

<details>
<summary><strong>VIP/会员</strong>（9 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `vip_info` | VIP 信息 |
| `vip_info_v2` | VIP 信息 v2 |
| `vip_sign` | 黑胶乐签打卡 |
| `vip_sign_info` | 打卡信息 |
| `vip_tasks` | VIP 任务列表 |
| `vip_timemachine` | 时光机 |
| `vip_growthpoint` | 成长值基本信息 |
| `vip_growthpoint_details` | 成长值明细 |
| `vip_growthpoint_get` | 领取成长值奖励 |

</details>

<details>
<summary><strong>云贝</strong>（11 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `yunbei` | 云贝签到信息 |
| `yunbei_info` | 云贝信息 |
| `yunbei_sign` | 云贝签到 |
| `yunbei_today` | 今日云贝 |
| `yunbei_expense` | 云贝支出 |
| `yunbei_receipt` | 云贝收入 |
| `yunbei_tasks` | 云贝任务 |
| `yunbei_tasks_todo` | 待完成任务 |
| `yunbei_task_finish` | 完成任务 |
| `yunbei_rcmd_song` | 云贝推歌 |
| `yunbei_rcmd_song_history` | 推歌历史 |

</details>

<details>
<summary><strong>听歌足迹</strong>（12 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `listen_data_year_report` | 年度听歌足迹 |
| `listen_data_today_song` | 今日收听 |
| `listen_data_total` | 总收听时长 |
| `listen_data_realtime_report` | 本周/本月时长 |
| `listen_data_report` | 收听报告 |
| `recent_listen_list` | 最近听歌列表 |
| `record_recent_song` | 最近播放歌曲 |
| `record_recent_album` | 最近播放专辑 |
| `record_recent_playlist` | 最近播放歌单 |
| `record_recent_dj` | 最近播放电台 |
| `record_recent_video` | 最近播放视频 |
| `record_recent_voice` | 最近播放声音 |

</details>

<details>
<summary><strong>风格/曲风</strong>（7 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `style_list` | 曲风列表 |
| `style_detail` | 曲风详情 |
| `style_song` | 曲风歌曲 |
| `style_album` | 曲风专辑 |
| `style_artist` | 曲风歌手 |
| `style_playlist` | 曲风歌单 |
| `style_preference` | 曲风偏好 |

</details>

<details>
<summary><strong>数字专辑</strong>（4 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `digital_album_detail` | 数字专辑详情 |
| `digital_album_ordering` | 购买数字专辑 |
| `digital_album_purchased` | 已购数字专辑 |
| `digital_album_sales` | 数字专辑销量 |

</details>

<details>
<summary><strong>声音/播客</strong>（9 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `voice_upload` | 上传音频 |
| `voice_delete` | 删除音频 |
| `voice_detail` | 音频详情 |
| `voice_lyric` | 音频歌词 |
| `voicelist_list` | 声音列表 |
| `voicelist_detail` | 声音列表详情 |
| `voicelist_search` | 搜索声音列表 |
| `voicelist_list_search` | 搜索声音 |
| `voicelist_trans` | 声音转换 |

</details>

<details>
<summary><strong>音乐人</strong>（8 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `musician_sign` | 音乐人签到 |
| `musician_tasks` | 音乐人任务 |
| `musician_tasks_new` | 音乐人新任务 |
| `musician_vip_tasks` | VIP 任务 |
| `musician_data_overview` | 数据概览 |
| `musician_play_trend` | 播放趋势 |
| `musician_cloudbean` | 云豆数量 |
| `musician_cloudbean_obtain` | 领取云豆 |

</details>

<details>
<summary><strong>粉丝中心</strong>（5 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `fanscenter_overview_get` | 粉丝中心概览 |
| `fanscenter_trend_list` | 粉丝趋势 |
| `fanscenter_basicinfo_age_get` | 年龄分布 |
| `fanscenter_basicinfo_gender_get` | 性别分布 |
| `fanscenter_basicinfo_province_get` | 省份分布 |

</details>

<details>
<summary><strong>UGC 百科</strong>（7 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `ugc_song_get` | 歌曲百科 |
| `ugc_artist_get` | 歌手百科 |
| `ugc_album_get` | 专辑百科 |
| `ugc_mv_get` | MV 百科 |
| `ugc_detail` | 百科详情 |
| `ugc_artist_search` | 搜索歌手百科 |
| `ugc_user_devote` | 用户贡献 |

</details>

<details>
<summary><strong>一起听</strong>（9 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `listentogether_room_create` | 创建房间 |
| `listentogether_room_check` | 检查房间 |
| `listentogether_accept` | 接受邀请 |
| `listentogether_status` | 房间状态 |
| `listentogether_heatbeat` | 心跳 |
| `listentogether_play_command` | 播放指令 |
| `listentogether_sync_list_command` | 同步列表指令 |
| `listentogether_sync_playlist_get` | 获取同步歌单 |
| `listentogether_end` | 结束房间 |

</details>

<details>
<summary><strong>广播电台</strong>（5 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `broadcast_category_region_get` | 分类/地区信息 |
| `broadcast_channel_list` | 全部电台 |
| `broadcast_channel_currentinfo` | 电台信息 |
| `broadcast_channel_collect_list` | 我的收藏 |
| `broadcast_sub` | 收藏/取消电台 |

</details>

<details>
<summary><strong>其他</strong>（17 个）</summary>

| 方法名 | 说明 |
|--------|------|
| `hot_topic` | 热门话题 |
| `topic_detail` | 话题详情 |
| `topic_detail_event_hot` | 话题热门动态 |
| `topic_sublist` | 收藏的专栏 |
| `calendar` | 音乐日历 |
| `batch` | 批量请求 |
| `api` | 通用 API 代理 |
| `inner_version` | 内部版本号 |
| `weblog` | 日志上报 |
| `eapi_decrypt` | EAPI 解密 |
| `sign_happy_info` | 乐签信息 |
| `signin_progress` | 签到进度 |
| `summary_annual` | 年度总结 |
| `threshold_detail_get` | 达人认证门槛 |
| `creator_authinfo_get` | 创作者认证信息 |
| `sheet_list` | 乐谱列表 |
| `sheet_preview` | 乐谱预览 |
| `aidj_content_rcmd` | AI DJ 推荐 |
| `music_first_listen_info` | 回忆坐标 |
| `verify_get_qr` | 验证二维码 |
| `verify_qrcodestatus` | 二维码状态 |

</details>

## 扩展接口

在 `src/api/` 中添加新文件即可，模式非常简单：

```rust
// src/api/your_new_api.rs
use crate::request::{ApiClient, ApiResponse, CryptoType};
use crate::error::Result;
use serde_json::json;
use super::Query;

impl ApiClient {
    pub async fn your_new_api(&self, query: &Query) -> Result<ApiResponse> {
        let data = json!({
            "id": query.get_or("id", ""),
        });
        self.request("/api/your/endpoint", data, query.to_option(CryptoType::Weapi)).await
    }
}
```

然后在 `src/api/mod.rs` 中注册：

```rust
mod your_new_api;
```

对应的 Node.js 接口参考 [NeteaseCloudMusicApi Enhanced](https://github.com/NeteaseCloudMusicApiEnhanced/api-enhanced) 的 `module/` 目录。

## 致谢

- [NeteaseCloudMusicApi Enhanced](https://github.com/NeteaseCloudMusicApiEnhanced/api-enhanced) - 原始 Node.js 实现
- [Binaryify/NeteaseCloudMusicApi](https://github.com/Binaryify/NeteaseCloudMusicApi) - 原始项目

## License

[WTFPL](LICENSE) - Do What The Fuck You Want To Public License

