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
use chrono::{TimeZone, Utc};
use futures::future::join_all;
use reqwest::header::CONTENT_TYPE;
use sailfish::TemplateOnce;

use crate::data::PostResp;
use crate::AppData;

const CACHE_AGE: u32 = 60 * 60 * 24;

pub mod routes {
    pub struct Proxy {
        pub index: &'static str,
        pub page: &'static str,
        pub asset: &'static str,
    }

    impl Proxy {
        pub const fn new() -> Self {
            Self {
                index: "/",
                page: "/{username}/{post}",
                asset: "/asset/medium/{name}",
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
    }
}

// credits @carlomilanesi:
// https://users.rust-lang.org/t/how-to-get-a-substring-of-a-string/1351/11
pub trait StringUtils {
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
    pub date: String,
    pub preview_img: String,
    pub reading_time: usize,
    pub id: String,
    pub gists: Option<Vec<(String, crate::data::GistContent)>>,
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

#[my_codegen::get(path = "crate::V1_API_ROUTES.proxy.page")]
async fn page(path: web::Path<(String, String)>, data: AppData) -> impl Responder {
    let post_id = path.1.split('-').last();
    if post_id.is_none() {
        return HttpResponse::BadRequest().finish();
    }
    let id = post_id.unwrap();

    let post_data = data.get_post(id).await;
    let mut futs = Vec::new();
    let paragraphs = &post_data.content.body_model.paragraphs;

    for p in paragraphs.iter() {
        if p.type_ == "IFRAME" {
            let src = &p
                .iframe
                .as_ref()
                .unwrap()
                .media_resource
                .as_ref()
                .unwrap()
                .href;
            if src.contains("gist.github.com") {
                let gist_id = post_data.get_gist_id(src);
                let fut = data.get_gist(gist_id.to_owned());
                futs.push(fut);
            }
        }
    }
    let gists = if futs.is_empty() {
        None
    } else {
        let x = join_all(futs).await;
        Some(x)
    };

    let date = Utc
        .timestamp_millis(post_data.created_at)
        .format("%b %e, %Y")
        .to_string();
    let reading_time = post_data.reading_time.floor() as usize;
    let preview_img = post_data
        .preview_image
        .as_ref()
        .unwrap()
        .id
        .as_ref()
        .unwrap();
    let preview_img = crate::V1_API_ROUTES.proxy.get_medium_asset(preview_img);

    let page = Post {
        id: id.to_owned(),
        data: post_data,
        date,
        gists,
        reading_time,
        preview_img,
    };

    let page = page.render_once().unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(assets);
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
