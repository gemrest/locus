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

use std::{collections::HashMap, fmt::Display};

pub struct Item {
  fields: HashMap<String, String>,
}
impl Item {
  pub fn builder() -> Self {
    Self {
      fields: HashMap::new(),
    }
  }

  pub fn add_field(&mut self, key: &str, value: &str) -> &mut Self {
    self.fields.insert(key.to_string(), value.to_string());

    self
  }
}
impl Display for Item {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "<item>{}</item>",
      self.fields.iter().fold(String::new(), |mut acc, (k, v)| {
        acc.push_str(&format!("<{}>{}</{}>", k, v, k));
        acc
      })
    )
  }
}

#[derive(Clone)]
pub struct Writer {
  content: String,
  fields:  HashMap<String, String>,
  link:    String,
}
impl Writer {
  pub fn builder() -> Self {
    Self {
      content: String::new(),
      fields:  HashMap::default(),
      link:    "".to_string(),
    }
  }

  pub fn add_link(&mut self, link: &str) -> &mut Self {
    self.link = link.to_string();

    self
  }

  pub fn add_field(&mut self, key: &str, value: &str) -> &mut Self {
    self.fields.insert(key.to_string(), value.to_string());

    self
  }

  pub fn add_item(&mut self, item: &Item) -> &mut Self {
    self.content.push_str(&item.to_string());

    self
  }
}
impl Display for Writer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss xmlns:atom=\"http://www.w3.org/2005/Atom\" \
       version=\"2.0\"><channel>{}<atom:link href=\"{}\" rel=\"self\" \
       type=\"application/rss+xml\" />{}</channel></rss>",
      self.fields.iter().fold(String::new(), |mut acc, (k, v)| {
        acc.push_str(&format!("<{}>{}</{}>", k, v, k));
        acc
      }),
      self.link,
      self.content
    )
  }
}
