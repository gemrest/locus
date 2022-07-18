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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Commit {
  sha: String,
  url: String,
}

#[derive(Serialize, Deserialize)]
struct Tags {
  pub name:    String,
  zipball_url: String,
  tarball_url: String,
  commit:      Commit,
  node_id:     String,
}

pub fn module(router: &mut windmark::Router) {
  crate::route::track_mount(
    router,
    "/api/sydney/version",
    "Sydney's version",
    Box::new(move |context| {
      let mut content = "0.0.0".to_string();

      if let Ok(response) = reqwest::blocking::Client::new()
        .get("https://api.github.com/repos/gemrest/sydney/tags")
        .header(
          "User-Agent",
          format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )
        .send()
      {
        if let Ok(response_content) = response.json::<Vec<Tags>>() {
          let response_content: Vec<Tags> = response_content;

          if let Some(first_tag) = response_content.get(0) {
            content = first_tag.name.clone();
          }

          if let Some(just_tag) = content.get(1..) {
            content = just_tag.to_string();
          }
        }
      }

      crate::route::cache(&context, &content);

      windmark::Response::Success(content)
    }),
  );
}
