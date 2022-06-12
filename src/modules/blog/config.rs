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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Entry {
  description:   Option<String>,
  author:        Option<String>,
  created:       Option<String>,
  last_modified: Option<String>,
  name:          Option<String>,
}
impl Entry {
  pub const fn description(&self) -> &Option<String> { &self.description }

  pub const fn author(&self) -> &Option<String> { &self.author }

  pub const fn name(&self) -> &Option<String> { &self.name }

  pub const fn created(&self) -> &Option<String> { &self.created }

  pub const fn last_modified(&self) -> &Option<String> { &self.last_modified }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Blog {
  name:        Option<String>,
  description: Option<String>,
  posts:       Option<HashMap<String, Entry>>,
}
impl Blog {
  pub const fn description(&self) -> &Option<String> { &self.description }

  pub const fn name(&self) -> &Option<String> { &self.name }

  pub const fn posts(&self) -> &Option<HashMap<String, Entry>> { &self.posts }

  pub fn from_string(string: &str) -> serde_json::Result<Self> {
    serde_json::from_str(string)
  }
}
