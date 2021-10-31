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
use actix_web::{web, HttpResponse, Responder};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use serde::{Deserialize, Serialize};

use crate::SETTINGS;

pub mod routes {
    pub struct Proxy {
        pub update: &'static str,
        pub page: &'static str,
    }

    impl Proxy {
        pub const fn new() -> Self {
            Self {
                update: "/api/v1/update",
                page: "/{username}/{post}",
            }
        }
        pub fn get_page(&self, username: &str, post: &str) -> String {
            self.page
                .replace("{username}", username)
                .replace("{post}", post)
        }
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/schema.graphql",
    query_path = "schemas/query.graphql",
    response_derives = "Debug"
)]
struct GetPost;

#[my_codegen::get(path = "crate::V1_API_ROUTES.proxy.page")]
async fn page(path: web::Path<(String, String)>) -> impl Responder {
    let post_id = path.1.split("-").last();
    if post_id.is_none() {
        return HttpResponse::BadRequest();
    }
    let id = post_id.unwrap().to_string();

    let vars = get_post::Variables { id };

    const URL: &str = "https://medium.com/_/graphql";

    let client = reqwest::Client::new();
    let res = post_graphql::<GetPost, _>(&client, URL, vars)
        .await
        .unwrap();
    println!("{:?}", res);

    let response_data: get_post::ResponseData = res.data.expect("missing response data");
    for p in response_data
        .post
        .unwrap()
        .content
        .unwrap()
        .body_model
        .unwrap()
        .paragraphs
        .unwrap()
        .iter()
    {
        println!("paragraph content: {:?}", p.as_ref().unwrap());
    }
    //        .bodyModel
    //        .paragraphs
    //        .iter();
    //    println!("{:?}", response_data);

    HttpResponse::Ok()
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(page);
}

#[cfg(test)]
mod tests {
    use actix_web::{http::StatusCode, test, App};

    use crate::services;
    use crate::*;

    use super::*;

    //    #[actix_rt::test]
    //    async fn deploy_update_works() {
    //        let app = test::init_service(App::new().configure(services)).await;
    //
    //        let page = page.unwrap();
    //
    //        let mut payload = ProxyEvent {
    //            secret: page.secret.clone(),
    //            branch: page.branch.clone(),
    //        };
    //
    //        let resp = test::call_service(
    //            &app,
    //            test::TestRequest::post()
    //                .uri(V1_API_ROUTES.deploy.update)
    //                .set_json(&payload)
    //                .to_request(),
    //        )
    //        .await;
    //        assert_eq!(resp.status(), StatusCode::OK);
    //
    //        payload.secret = page.branch.clone();
    //
    //        let resp = test::call_service(
    //            &app,
    //            test::TestRequest::post()
    //                .uri(V1_API_ROUTES.deploy.update)
    //                .set_json(&payload)
    //                .to_request(),
    //        )
    //        .await;
    //        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    //    }
}
