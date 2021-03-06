// This file is part of Locus <https://github.com/gemrest/locus>.
// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.
//
// Copyright (C) 2022-2022 Fuwn <contact@fuwn.me>
// SPDX-License-Identifier: GPL-3.0-only

mod blog;
mod contact;
mod interests;
mod random;
mod remarks;
// mod robots;
mod api;
mod router;
pub mod search;
mod sitemap;
mod skills;
mod r#static;
mod uptime;

pub fn module(router: &mut windmark::Router) {
  crate::statelesses!(
    router, uptime, sitemap, search, remarks, blog, random, r#static, router,
    skills, contact, interests, api,
  );
}
