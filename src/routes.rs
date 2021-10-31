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

use crate::meta::routes::Meta;
use crate::proxy::routes::Proxy;

pub const ROUTES: Routes = Routes::new();

pub struct Routes {
    pub meta: Meta,
    pub proxy: Proxy,
}

impl Routes {
    pub const fn new() -> Self {
        Self {
            meta: Meta::new(),
            proxy: Proxy::new(),
        }
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    crate::meta::services(cfg);
    crate::proxy::services(cfg);
}
