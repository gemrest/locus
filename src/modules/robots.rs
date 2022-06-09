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

use windmark::Response;

pub fn module(router: &mut windmark::Router) {
  crate::route::track_mount(
    router,
    "/robots.txt",
    "Crawler traffic manager; for robots, not humans",
    Box::new(|context| {
      let mut content =
        include_str!(concat!("../../content/meta/robots.txt")).to_string();

      if let Ok(response) = reqwest::blocking::get(
        "https://gist.githubusercontent.com/Fuwn/bc3bf8b8966bb123b48af7d9dfb857c3/raw/cbc1f13685231f0e4c6c2ccc7ec8aa2cd46d377b/locus_robots.txt",
      ) {
        if let Ok(response_content) = response.text() {
          content = response_content;
        }
      }

      crate::route::cache(&context, &content);

      Response::Success(content)
    }),
  );
}
