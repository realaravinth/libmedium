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
use std::{collections::HashMap, hash::Hash};

use crate::data::*;
use crate::proxy::StringUtils;
use get_post::*;

#[derive(Eq, PartialEq)]
enum PostitionType {
    Start,
    End,
}

struct ListState {
    in_uli: bool,
    in_oli: bool,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            in_uli: false,
            in_oli: false,
        }
    }
}

struct Markup<'a, 'b> {
    markup: &'a GetPostPostContentBodyModelParagraphsMarkups,
    p: &'a GetPostPostContentBodyModelParagraphs,
    pos_type: PostitionType,
    gists: &'b Option<Vec<(String, crate::data::GistContent)>>,
}

impl<'a, 'b> Markup<'a, 'b> {
    fn start(
        p: &GetPostPostContentBodyModelParagraphs,
        gists: &'b Option<Vec<(String, crate::data::GistContent)>>,
        pindex: usize,
        state: &mut ListState,
    ) -> String {
        let list = Self::list_close(p, state);
        let resp = if p.type_ == "IMG" {
            let metadata = p.metadata.as_ref().unwrap();
            format!(
                r#"<figure><img width="{}" src="{}" /> <figcaption>"#,
                metadata.original_width.as_ref().unwrap(),
                crate::V1_API_ROUTES.proxy.get_medium_asset(&metadata.id)
            )
        } else if p.type_ == "P" {
            "<p>".into()
        } else if p.type_ == "PRE" {
            "<pre>".into()
        } else if p.type_ == "BQ" {
            "<blockquote>".into()
        } else if p.type_ == "H1" {
            "<h1>".into()
        } else if p.type_ == "H2" {
            "<h2>".into()
        } else if p.type_ == "H3" {
            if pindex == 0 {
                log::debug!("caught heading");
                "".into()
            } else {
                "<h3>".into()
            }
        } else if p.type_ == "H4" {
            "<h4>".into()
        } else if p.type_ == "H5" {
            "<h5>".into()
        } else if p.type_ == "H6" {
            "<h6>".into()
        } else if p.type_ == "IFRAME" {
            let src = &p
                .iframe
                .as_ref()
                .unwrap()
                .media_resource
                .as_ref()
                .unwrap()
                .href;
            if src.contains("gist.github.com") {
                let gist_id = crate::data::Data::get_gist_id(src);
                let (_, gist) = gists
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|(id, _)| id == gist_id)
                    .as_ref()
                    .unwrap();

                let mut gists = String::default();
                for file in &gist.files {
                    gists += &format!(
                        r#"<div class="code-block gist-block">{}</div>"#,
                        file.get_html_content()
                    );
                }
                format!(
                    r#"<div class="gist_container">{gists}
                     <a class="gist_link" href="{}" target="_blank">See gist on GitHub</a>"#,
                    &gist.html_url
                )
            } else {
                format!(r#"<iframe src="{src}" frameborder="0">"#)
            }
        } else if p.type_ == "ULI" {
            if state.in_uli {
                "<li>".into()
            } else {
                state.in_uli = true;
                "<ul><li>".into()
            }
        } else if p.type_ == "OLI" {
            if state.in_oli {
                "<li>".into()
            } else {
                state.in_oli = true;
                "<ol><li>".into()
            }
        } else {
            log::info!("Unknown type");
            r#"
                <p class="libmedium__meta">
                    <b>From LibMedium:</b> LibMedium is built by reverse
                    engineering the Meduim's internal API. This post contains
                    markup(formatting rules) that we are unaware of.
                    Please report this URL <a
                    href="https://github.com/realaravinth/libmedium/issues/1"
                    rel="noreferrer">on our bug tracker</a> so that we can
                    improve page rendering.
                    <br />
                    Alternatively, you can also email me at realaravinth at batsense dot net!
                  </p>
            <span>"#
                .into()
        };

        match list {
            Some(list) => format!("{list}{resp}"),
            None => resp,
        }
    }

    fn end(
        p: &GetPostPostContentBodyModelParagraphs,
        pindex: usize,
        state: &mut ListState,
    ) -> String {
        let resp: String = if p.type_ == "IMG" {
            "</figcaption></figure>".into()
        } else if p.type_ == "P" {
            "</p>".into()
        } else if p.type_ == "PRE" {
            "</pre>".into()
        } else if p.type_ == "BQ" {
            "</blockquote>".into()
        } else if p.type_ == "H1" {
            "</h1>".into()
        } else if p.type_ == "H2" {
            "</h2>".into()
        } else if p.type_ == "H3" {
            if pindex == 0 {
                log::debug!("caught heading");
                "".into()
            } else {
                "</h3>".into()
            }
        } else if p.type_ == "H4" {
            "</h4>".into()
        } else if p.type_ == "H5" {
            "</h5>".into()
        } else if p.type_ == "H6" {
            "</h6>".into()
        } else if p.type_ == "IFRAME" {
            let src = &p
                .iframe
                .as_ref()
                .unwrap()
                .media_resource
                .as_ref()
                .unwrap()
                .href;
            if src.contains("gist.github.com") {
                "</div>".into()
            } else {
                "</iframe>".into()
            }
        } else if p.type_ == "OLI" || p.type_ == "ULI" {
            "</li>".into()
        } else {
            "</span>".into()
        };
        if state.in_oli {
            if p.type_ != "OLI" {
                state.in_oli = false;
                format!("</ol>{resp}")
            } else {
                resp
            }
        } else if state.in_uli {
            if p.type_ != "ULI" {
                state.in_uli = false;
                format!("</ul>{resp}")
            } else {
                resp
            }
        } else {
            resp
        }
    }

    fn list_close(
        p: &GetPostPostContentBodyModelParagraphs,
        state: &mut ListState,
    ) -> Option<String> {
        if state.in_oli {
            if p.type_ != "OLI" {
                state.in_oli = false;
                return Some(format!("</ol>"));
            }
        };
        if state.in_uli {
            if p.type_ != "ULI" {
                state.in_uli = false;
                return Some(format!("</ul>"));
            }
        };
        None
    }

    fn apply_markup(&self, pindex: usize) -> String {
        if self.markup.type_ == "A" {
            if let Some(anchor_type) = &self.markup.anchor_type {
                if anchor_type == "LINK" {
                    if self.pos_type == PostitionType::Start {
                        format!(
                            r#"<a rel="noreferrer" href="{}">"#,
                            self.markup.href.as_ref().unwrap()
                        )
                    } else {
                        "</a>".into()
                    }
                } else if anchor_type == "USER" {
                    if self.pos_type == PostitionType::Start {
                        format!(
                            r#"<a rel="noreferrer" href="https://medium.com/u/{}">"#,
                            self.markup.user_id.as_ref().unwrap()
                        )
                    } else {
                        "</a>".into()
                    }
                } else {
                    //             log::error!("unknown markup.anchor_type: {:?} post id {}", anchor_type, id);
                    if self.pos_type == PostitionType::Start {
                        "<span>".into()
                    } else {
                        "</span>".into()
                    }
                }
            } else {
                //             log::error!("unknown markup.anchor_type: {:?} post id {}", anchor_type, id);
                if self.pos_type == PostitionType::Start {
                    "<span>".into()
                } else {
                    "</span>".into()
                }
            }
        } else if self.markup.type_ == "PRE" {
            if self.pos_type == PostitionType::Start {
                "<pre>".into()
            } else {
                "</pre>".into()
            }
        } else if self.markup.type_ == "EM" {
            if self.pos_type == PostitionType::Start {
                "<em>".into()
            } else {
                "</em>".into()
            }
        } else if self.markup.type_ == "STRONG" {
            if self.pos_type == PostitionType::Start {
                "<strong>".into()
            } else {
                "</strong>".into()
            }
        } else if self.markup.type_ == "CODE" {
            if self.pos_type == PostitionType::Start {
                "<code>".into()
            } else {
                "</code>".into()
            }
        } else {
            // log::error!("unknown markup.type_: {:?} post id {}", markup.type_, id);
            if self.pos_type == PostitionType::Start {
                log::info!("Unknown type");
                r#"
                <p class="libmedium__meta">
                    <b>From LibMedium:</b> LibMedium is built by reverse
                    engineering the Meduim's internal API. This post contains
                    markup(formatting rules) that we are unaware of.
                    Please report this URL <a
                    href="https://github.com/realaravinth/libmedium/issues/1"
                    rel="noreferrer">on our bug tracker</a> so that we can
                    improve page rendering.
                    <br />
                    Alternatively, you can also email me at realaravinth at batsense dot net!
                  </p>
            <span>"#
                    .into()
            } else {
                "</span>".into()
            }
        }
    }
}

#[derive(Default)]
struct PositionMap<'a, 'b> {
    map: HashMap<i64, Vec<Markup<'a, 'b>>>,
    arr: Vec<i64>,
}

impl<'a, 'b> PositionMap<'a, 'b> {
    fn insert_if_not_exists(&mut self, pos: i64, m: Markup<'a, 'b>) {
        if let Some(markups) = self.map.get_mut(&pos) {
            markups.push(m);
        } else {
            self.map.insert(pos, vec![m]);
            self.arr.push(pos);
        }
    }
}

pub fn apply_markup<'b>(
    data: &PostResp,
    gists: &'b Option<Vec<(String, crate::data::GistContent)>>,
) -> Vec<String> {
    let mut paragraphs: Vec<String> = Vec::with_capacity(data.content.body_model.paragraphs.len());
    let mut state = ListState::default();
    for (pindex, p) in data.content.body_model.paragraphs.iter().enumerate() {
        let mut pos = PositionMap::default();
        if p.type_ == "H3" && pindex == 0 {
            log::debug!("FOUND TOP LEVEL H3. Breaking");
            continue;
        }
        for m in p.markups.iter() {
            let start_markup = Markup {
                markup: &m,
                p,
                gists,
                pos_type: PostitionType::Start,
            };
            pos.insert_if_not_exists(m.start, start_markup);
            let end_markup = Markup {
                markup: &m,
                p,
                gists,
                pos_type: PostitionType::End,
            };

            pos.insert_if_not_exists(m.end, end_markup);
        }

        let mut cur = 0;

        fn incr_cur(cur: usize, point: i64) -> usize {
            let incr = point as usize - cur;
            let post_incr = cur + incr;
            log::debug!(
                "cur before incr: {cur}, incr by: {}, post incr: {}",
                incr,
                post_incr
            );
            post_incr
        }

        let mut content = String::with_capacity(p.text.len());
        content += &Markup::start(&p, &gists, pindex, &mut state);
        pos.arr.sort();
        if let Some(first) = pos.arr.get(0) {
            //content += p.text.substring(cur, *first as usize);
            content += p.text.slice(cur..*first as usize);
            cur = incr_cur(cur, *first);
            for point in pos.arr.iter() {
                //content.push(p.text.substring(start, start + point);
                //            if *point != 0 {

                if cur != *point as usize {
                    //           content += p.text.substring(cur, *point as usize);
                    content += p.text.slice(cur..*point as usize);
                }
                //           }
                let pos_markups = pos.map.get(point).unwrap();
                for m in pos_markups.iter() {
                    content += &m.apply_markup(pindex);
                }
                cur = incr_cur(cur, *point);
            }
            log::debug!("LAST");
            content += p.text.slice(cur..);
            content += &Markup::end(&p, pindex, &mut state);
        } else {
            log::debug!("LAST WITH NO MARKUP");
            content += p.text.slice(cur..);
            content += &Markup::end(&p, pindex, &mut state);
        }
        paragraphs.push(content);
    }
    paragraphs
}
