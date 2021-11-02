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
use actix_web::web;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Client;
use sled::{Db, Tree};

#[derive(Clone)]
pub struct Data {
    pub client: Client,
    cache: Db,
    pub posts: Tree,
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schemas/schema.graphql",
    query_path = "schemas/query.graphql",
    response_derives = "Debug, Serialize, Deserialize, Clone"
)]
pub struct GetPost;

pub type PostResp = get_post::GetPostPost;

pub type AppData = web::Data<Data>;

impl Data {
    pub fn new() -> AppData {
        let cache = sled::open("posts_cache").unwrap();
        let posts = cache.open_tree("posts").unwrap();
        AppData::new(Self {
            client: Client::new(),
            cache,
            posts,
        })
    }

    pub async fn get_post(&self, id: &str) -> PostResp {
        match self.posts.get(id) {
            Ok(Some(v)) => bincode::deserialize(&v[..]).unwrap(),
            _ => {
                let vars = get_post::Variables { id: id.to_owned() };
                const URL: &str = "https://medium.com/_/graphql";

                let res = post_graphql::<GetPost, _>(&self.client, URL, vars)
                    .await
                    .unwrap();
                let res = res.data.expect("missing response data").post.unwrap();
                self.posts
                    .insert(id, bincode::serialize(&res).unwrap())
                    .unwrap();
                res
            }
        }
    }
}
