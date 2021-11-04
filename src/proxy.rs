/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::ops::{Bound, RangeBounds};

use actix_web::{http::header, web, HttpResponse, Responder};
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};

use crate::data::PostResp;
use crate::AppData;

const CACHE_AGE: u32 = 60 * 60 * 24;

pub mod routes {
    pub struct Proxy {
        pub index: &'static str,
        pub page: &'static str,
        pub asset: &'static str,
        pub gist: &'static str,
    }

    impl Proxy {
        pub const fn new() -> Self {
            Self {
                index: "/",
                page: "/{username}/{post}",
                asset: "/asset/medium/{name}",
                gist: "/asset/github-gist",
            }
        }
        pub fn get_page(&self, username: &str, post: &str) -> String {
            self.page
                .replace("{username}", username)
                .replace("{post}", post)
        }

        pub fn get_medium_asset(&self, asset_name: &str) -> String {
            self.asset.replace("{name}", asset_name)
        }

        pub fn get_gist(&self, url: &str) -> String {
            if let Some(gist_id) = url.split('/').last() {
                format!("{}?gist={}", self.gist, urlencoding::encode(gist_id))
            } else {
                url.to_owned()
            }
        }
    }
}

// credits @carlomilanesi:
// https://users.rust-lang.org/t/how-to-get-a-substring-of-a-string/1351/11
trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> &str;
    fn slice(&self, range: impl RangeBounds<usize>) -> &str;
}

impl StringUtils for str {
    fn substring(&self, start: usize, len: usize) -> &str {
        let mut char_pos = 0;
        let mut byte_start = 0;
        let mut it = self.chars();
        loop {
            if char_pos == start {
                break;
            }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_start += c.len_utf8();
            } else {
                break;
            }
        }
        char_pos = 0;
        let mut byte_end = byte_start;
        loop {
            if char_pos == len {
                break;
            }
            if let Some(c) = it.next() {
                char_pos += 1;
                byte_end += c.len_utf8();
            } else {
                break;
            }
        }
        &self[byte_start..byte_end]
    }
    fn slice(&self, range: impl RangeBounds<usize>) -> &str {
        let start = match range.start_bound() {
            Bound::Included(bound) | Bound::Excluded(bound) => *bound,
            Bound::Unbounded => 0,
        };
        let len = match range.end_bound() {
            Bound::Included(bound) => *bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => self.len(),
        } - start;
        self.substring(start, len)
    }
}

#[derive(TemplateOnce)]
#[template(path = "post.html")]
#[template(rm_whitespace = true)]
pub struct Post {
    pub data: PostResp,
    pub id: String,
}

const INDEX: &str = include_str!("../templates/index.html");

#[my_codegen::get(path = "crate::V1_API_ROUTES.proxy.index")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(INDEX)
}

#[my_codegen::get(path = "crate::V1_API_ROUTES.proxy.asset")]
async fn assets(path: web::Path<String>, data: AppData) -> impl Responder {
    let res = data
        .client
        .get(format!("https://miro.medium.com/{}", path))
        .send()
        .await
        .unwrap();
    let headers = res.headers();
    let content_type = headers.get(CONTENT_TYPE).unwrap();
    HttpResponse::Ok()
        .insert_header(header::CacheControl(vec![
            header::CacheDirective::Public,
            header::CacheDirective::Extension("immutable".into(), None),
            header::CacheDirective::MaxAge(CACHE_AGE),
        ]))
        .content_type(content_type)
        .body(res.bytes().await.unwrap())
}

#[derive(Deserialize, Serialize)]
struct GistQuery {
    gist: String,
}

#[derive(Deserialize, Serialize, TemplateOnce)]
#[template(path = "gist.html")]
#[template(rm_whitespace = true)]
pub struct GistContent {
    pub files: Vec<GistFile>,
    pub html_url: String,
}

#[derive(TemplateOnce)]
#[template(path = "gist_error.html")]
#[template(rm_whitespace = true)]
pub struct GistContentError;

#[derive(Deserialize, Serialize)]
pub struct GistFile {
    pub file_name: String,
    pub content: String,
    pub language: String,
    pub raw_url: String,
}

impl GistFile {
    pub fn get_html_content(&self) -> String {
        let mut content = self.content.as_str();
        if self.content.starts_with('"') {
            content = self.content.slice(1..);
        }

        if content.ends_with('"') {
            content = content.slice(..content.len() - 1);
        }
        content.replace("\\t", "  ")
    }
}

#[my_codegen::get(path = "crate::V1_API_ROUTES.proxy.gist")]
async fn get_gist(query: web::Query<GistQuery>, data: AppData) -> impl Responder {
    const URL: &str = "https://api.github.com/gists/";
    let url = format!("{}{}", URL, query.gist);

    let resp = data
        .client
        .get(&url)
        .header(USER_AGENT, "libmedium")
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    if let Some(files) = resp.get("files") {
        if let serde_json::Value::Object(v) = files {
            let mut files = Vec::with_capacity(v.len());
            v.iter().for_each(|(name, file_obj)| {
                let file = GistFile {
                    file_name: name.to_string(),
                    content: file_obj
                        .get("content")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned(),
                    language: file_obj
                        .get("language")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned(),
                    raw_url: file_obj
                        .get("raw_url")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned(),
                };
                files.push(file);
            });
            let gist = GistContent {
                files,
                html_url: resp.get("html_url").unwrap().to_string(),
            };

            return HttpResponse::Ok()
                .content_type("text/html")
                .body(gist.render_once().unwrap());
        }
    };
    let err = GistContentError {};
    HttpResponse::Ok()
        .content_type("text/html")
        .body(err.render_once().unwrap())
}

#[my_codegen::get(path = "crate::V1_API_ROUTES.proxy.page")]
async fn page(path: web::Path<(String, String)>, data: AppData) -> impl Responder {
    let post_id = path.1.split('-').last();
    if post_id.is_none() {
        return HttpResponse::BadRequest().finish();
    }
    let id = post_id.unwrap();

    let page = Post {
        id: id.to_owned(),
        data: data.get_post(id).await,
    }
    .render_once()
    .unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(assets);
    cfg.service(get_gist);
    cfg.service(page);
    cfg.service(index);
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test, App};

    use crate::{services, Data};

    #[actix_rt::test]
    async fn deploy_update_works() {
        let data = Data::new();
        let app = test::init_service(App::new().app_data(data.clone()).configure(services)).await;
        let urls = vec![
            "/@ftrain/big-data-small-effort-b62607a43a8c",
            "/geekculture/rest-api-best-practices-decouple-long-running-tasks-from-http-request-processing-9fab2921ace8",
            "/illumination/5-bugs-that-turned-into-features-e9a0e972a4e7",
            "/",
            "/asset/medium/1*LY2ohYsNa9nOV1Clko3zJA.png",
        ];

        for uri in urls.iter() {
            let resp =
                test::call_service(&app, test::TestRequest::get().uri(uri).to_request()).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }
}
