/// API 模块 - 对应 Node.js 版本的 module/ 目录
///
/// 每个 API 接口是一个异步方法，通过宏批量生成

use crate::request::{ApiClient, ApiResponse, CryptoType, RequestOption};
use crate::error::Result;
use md5::{Md5, Digest};
use serde_json::{json, Value};
use std::collections::HashMap;

/// 通用查询参数
#[derive(Debug, Clone, Default)]
pub struct Query {
    pub params: HashMap<String, String>,
    pub cookie: Option<String>,
    pub proxy: Option<String>,
    pub real_ip: Option<String>,
    pub random_cn_ip: bool,
    pub ua: Option<String>,
    pub e_r: Option<bool>,
    pub domain: Option<String>,
}

impl Query {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置参数
    pub fn param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    /// 设置 cookie
    pub fn cookie(mut self, cookie: &str) -> Self {
        self.cookie = Some(cookie.to_string());
        self
    }

    /// 获取参数值，若不存在则返回默认值
    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    /// 获取参数值，若不存在则返回默认值
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.params
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// 构造 RequestOption
    fn to_option(&self, crypto: CryptoType) -> RequestOption {
        RequestOption {
            crypto,
            cookie: self.cookie.clone(),
            ua: self.ua.clone(),
            proxy: self.proxy.clone(),
            real_ip: self.real_ip.clone(),
            random_cn_ip: self.random_cn_ip,
            e_r: self.e_r,
            domain: self.domain.clone(),
            check_token: false,
        }
    }
}

// ============================================================
//  API 接口实现
// ============================================================

impl ApiClient {
    // ---- 歌曲相关 ----

    /// 歌曲详情
    /// 对应 /song/detail
    pub async fn song_detail(&self, ids: &[i64]) -> Result<ApiResponse> {
        let c: Vec<String> = ids.iter().map(|id| format!(r#"{{"id":{}}}"#, id)).collect();
        let data = json!({
            "c": format!("[{}]", c.join(","))
        });
        let query = Query::new();
        self.request("/api/v3/song/detail", data, query.to_option(CryptoType::Weapi))
            .await
    }

    /// 歌曲播放链接
    /// 对应 /song/url/v1
    pub async fn song_url_v1(&self, id: i64, level: &str) -> Result<ApiResponse> {
        let data = json!({
            "ids": format!("[{}]", id),
            "level": level,
            "encodeType": "flac"
        });
        let query = Query::new();
        self.request("/api/song/enhance/player/url/v1", data, query.to_option(CryptoType::default()))
            .await
    }

    /// 歌词
    /// 对应 /lyric
    pub async fn lyric(&self, id: i64) -> Result<ApiResponse> {
        let data = json!({
            "id": id,
            "tv": -1,
            "lv": -1,
            "rv": -1,
            "kv": -1,
            "_nmclfl": 1
        });
        let query = Query::new();
        self.request("/api/song/lyric", data, query.to_option(CryptoType::default()))
            .await
    }

    /// 检查音乐是否可用
    /// 对应 /check/music
    pub async fn check_music(&self, id: i64, br: Option<i64>) -> Result<ApiResponse> {
        let data = json!({
            "ids": [id],
            "br": br.unwrap_or(999000)
        });
        let query = Query::new();
        self.request("/api/song/enhance/player/url", data, query.to_option(CryptoType::Weapi))
            .await
    }

    // ---- 搜索相关 ----

    /// 搜索
    /// 对应 /cloudsearch
    pub async fn cloudsearch(
        &self,
        keywords: &str,
        search_type: Option<i64>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<ApiResponse> {
        let data = json!({
            "s": keywords,
            "type": search_type.unwrap_or(1),
            "limit": limit.unwrap_or(30),
            "offset": offset.unwrap_or(0),
            "total": true
        });
        let query = Query::new();
        self.request("/api/cloudsearch/pc", data, query.to_option(CryptoType::Weapi))
            .await
    }

    /// 默认搜索关键词
    /// 对应 /search/default
    pub async fn search_default(&self) -> Result<ApiResponse> {
        let data = json!({});
        let query = Query::new();
        self.request("/api/search/defaultkeyword/get", data, query.to_option(CryptoType::default()))
            .await
    }

    /// 搜索建议
    /// 对应 /search/suggest
    pub async fn search_suggest(&self, keywords: &str) -> Result<ApiResponse> {
        let data = json!({
            "s": keywords
        });
        let query = Query::new();
        self.request("/api/search/suggest/web", data, query.to_option(CryptoType::Weapi))
            .await
    }

    // ---- 歌单相关 ----

    /// 用户歌单
    /// 对应 /user/playlist
    pub async fn user_playlist(
        &self,
        uid: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<ApiResponse> {
        let data = json!({
            "uid": uid,
            "limit": limit.unwrap_or(30),
            "offset": offset.unwrap_or(0),
            "includeVideo": true
        });
        let query = Query::new();
        self.request("/api/user/playlist", data, query.to_option(CryptoType::Weapi))
            .await
    }

    /// 歌单详情
    /// 对应 /playlist/detail
    pub async fn playlist_detail(&self, id: i64, s: Option<i64>) -> Result<ApiResponse> {
        let data = json!({
            "id": id,
            "n": 100000,
            "s": s.unwrap_or(8)
        });
        let query = Query::new();
        self.request("/api/v6/playlist/detail", data, query.to_option(CryptoType::default()))
            .await
    }

    /// 歌单所有歌曲
    /// 对应 /playlist/track/all
    pub async fn playlist_track_all(
        &self,
        id: i64,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<ApiResponse> {
        let data = json!({
            "id": id,
            "n": 100000,
            "limit": limit.unwrap_or(1000),
            "offset": offset.unwrap_or(0)
        });
        let query = Query::new();
        self.request("/api/v6/playlist/detail", data, query.to_option(CryptoType::default()))
            .await
    }

    // ---- 歌手相关 ----

    /// 歌手详情
    /// 对应 /artist/detail
    pub async fn artist_detail(&self, id: i64) -> Result<ApiResponse> {
        let data = json!({
            "id": id
        });
        let query = Query::new();
        self.request("/api/artist/head/info/get", data, query.to_option(CryptoType::Weapi))
            .await
    }

    /// 歌手歌曲
    /// 对应 /artist/songs
    pub async fn artist_songs(
        &self,
        id: i64,
        order: Option<&str>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<ApiResponse> {
        let data = json!({
            "id": id,
            "private_cloud": "true",
            "work_type": 1,
            "order": order.unwrap_or("hot"),
            "limit": limit.unwrap_or(100),
            "offset": offset.unwrap_or(0)
        });
        let query = Query::new();
        self.request("/api/v1/artist/songs", data, query.to_option(CryptoType::Weapi))
            .await
    }

    // ---- 专辑相关 ----

    /// 专辑详情
    /// 对应 /album
    pub async fn album(&self, id: i64) -> Result<ApiResponse> {
        let data = json!({});
        let query = Query::new();
        self.request(&format!("/api/v1/album/{}", id), data, query.to_option(CryptoType::Weapi))
            .await
    }

    // ---- 评论相关 ----

    /// 新版评论
    /// 对应 /comment/new
    pub async fn comment_new(
        &self,
        id: i64,
        resource_type: i64,
        page_no: Option<i64>,
        page_size: Option<i64>,
        sort_type: Option<i64>,
        cursor: Option<&str>,
    ) -> Result<ApiResponse> {
        let data = json!({
            "rid": id.to_string(),
            "threadId": crate::util::config::RESOURCE_TYPE_MAP
                .get(resource_type.to_string().as_str())
                .map(|prefix| format!("{}{}", prefix, id))
                .unwrap_or_default(),
            "pageNo": page_no.unwrap_or(1),
            "pageSize": page_size.unwrap_or(20),
            "sortType": sort_type.unwrap_or(1),
            "cursor": cursor.unwrap_or("")
        });
        let query = Query::new();
        self.request("/api/v2/resource/comments", data, query.to_option(CryptoType::default()))
            .await
    }

    /// 评论统计数据
    /// 对应 /comment/info/list
    pub async fn comment_info_list(
        &self,
        ids: &[i64],
        resource_type: i64,
    ) -> Result<ApiResponse> {
        // 从 resourceTypeMap 前缀中提取内部类型编号
        let type_id = crate::util::config::RESOURCE_TYPE_MAP
            .get(resource_type.to_string().as_str())
            .map(|prefix| {
                prefix
                    .trim_end_matches('_')
                    .rsplit('_')
                    .next()
                    .unwrap_or("0")
                    .to_string()
            })
            .unwrap_or_else(|| "0".to_string());

        let ids_str: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let data = json!({
            "resourceType": type_id,
            "resourceIds": serde_json::to_string(&ids_str).unwrap()
        });
        let query = Query::new();
        self.request("/api/resource/commentInfo/list", data, query.to_option(CryptoType::Weapi))
            .await
    }

    // ---- 推荐相关 ----

    /// 每日推荐歌曲（需要登录）
    /// 对应 /recommend/songs
    pub async fn recommend_songs(&self) -> Result<ApiResponse> {
        let data = json!({});
        let query = Query::new();
        self.request("/api/v3/discovery/recommend/songs", data, query.to_option(CryptoType::Weapi))
            .await
    }

    /// 推荐歌单
    /// 对应 /personalized
    pub async fn personalized(&self, limit: Option<i64>) -> Result<ApiResponse> {
        let data = json!({
            "limit": limit.unwrap_or(30),
            "total": true,
            "n": 1000
        });
        let query = Query::new();
        self.request("/api/personalized/playlist", data, query.to_option(CryptoType::Weapi))
            .await
    }

    // ---- 用户相关 ----

    /// 手机号登录
    /// 对应 /login/cellphone
    pub async fn login_cellphone(
        &self,
        phone: &str,
        password: Option<&str>,
        md5_password: Option<&str>,
        captcha: Option<&str>,
        country_code: Option<&str>,
    ) -> Result<ApiResponse> {
        let mut data = json!({
            "type": "1",
            "https": "true",
            "phone": phone,
            "countrycode": country_code.unwrap_or("86"),
            "remember": "true"
        });

        if let Some(cap) = captcha {
            data["captcha"] = Value::String(cap.to_string());
        } else if let Some(md5_pwd) = md5_password {
            data["password"] = Value::String(md5_pwd.to_string());
        } else if let Some(pwd) = password {
            // MD5 hash password
            let digest = format!("{:x}", Md5::digest(pwd.as_bytes()));
            data["password"] = Value::String(digest);
        }

        let query = Query::new();
        self.request("/api/w/login/cellphone", data, query.to_option(CryptoType::Weapi))
            .await
    }

    /// 登录状态
    /// 对应 /login/status
    pub async fn login_status(&self) -> Result<ApiResponse> {
        let data = json!({});
        let query = Query::new();
        self.request("/api/w/nuser/account/get", data, query.to_option(CryptoType::Weapi))
            .await
    }

    /// 用户详情
    /// 对应 /user/detail
    pub async fn user_detail(&self, uid: i64) -> Result<ApiResponse> {
        let data = json!({});
        let query = Query::new();
        self.request(&format!("/api/v1/user/detail/{}", uid), data, query.to_option(CryptoType::Weapi))
            .await
    }

    // ---- 私人 FM ----

    /// 私人 FM
    /// 对应 /personal/fm
    pub async fn personal_fm(&self) -> Result<ApiResponse> {
        let data = json!({});
        let query = Query::new();
        self.request("/api/v1/radio/get", data, query.to_option(CryptoType::Weapi))
            .await
    }

    // ---- 喜欢 ----

    /// 喜欢的音乐列表
    /// 对应 /likelist
    pub async fn likelist(&self, uid: i64) -> Result<ApiResponse> {
        let data = json!({
            "uid": uid
        });
        let query = Query::new();
        self.request("/api/song/like/get", data, query.to_option(CryptoType::Weapi))
            .await
    }

    /// 喜欢音乐
    /// 对应 /like
    pub async fn like(&self, id: i64, like: bool) -> Result<ApiResponse> {
        let data = json!({
            "trackId": id,
            "like": like,
            "alg": "itembased"
        });
        let query = Query::new();
        self.request("/api/radio/like", data, query.to_option(CryptoType::Weapi))
            .await
    }

    // ---- Banner ----

    /// 首页 Banner
    /// 对应 /banner
    pub async fn banner(&self, banner_type: Option<i64>) -> Result<ApiResponse> {
        let type_map: HashMap<i64, &str> = [
            (0, "pc"),
            (1, "android"),
            (2, "iphone"),
            (3, "ipad"),
        ]
        .into();

        let client_type = type_map
            .get(&banner_type.unwrap_or(0))
            .unwrap_or(&"pc");

        let data = json!({
            "clientType": client_type
        });
        let query = Query::new();
        self.request("/api/v2/banner/get", data, query.to_option(CryptoType::default()))
            .await
    }

    // ---- 通用请求 ----

    /// 通用 API 调用（使用 Query 对象，适合自定义调用）
    pub async fn call(
        &self,
        uri: &str,
        data: Value,
        query: &Query,
        crypto: CryptoType,
    ) -> Result<ApiResponse> {
        let mut option = query.to_option(crypto);
        if let Some(ref c) = query.cookie {
            option.cookie = Some(c.clone());
        }
        self.request(uri, data, option).await
    }
}
