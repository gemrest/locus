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

use std::{collections::HashMap, lazy::SyncLazy};

type InterestMap = HashMap<String, HashMap<String, String>>;

static INTEREST_MAP: SyncLazy<InterestMap> = SyncLazy::new(|| {
  serde_json::from_str(include_str!("../../content/json/interests.json"))
    .unwrap()
});

pub fn module(router: &mut windmark::Router) {
  let mut contacts = INTEREST_MAP
    .iter()
    .map(|(category, contacts)| {
      format!("## {}\n\n{}", category, {
        let mut contacts = contacts
          .iter()
          .map(|(tag, href)| format!("=> {} {}", href, tag))
          .collect::<Vec<_>>();

        contacts.sort();

        contacts.join("\n")
      })
    })
    .collect::<Vec<_>>();

  contacts.sort();

  crate::route::track_mount(
    router,
    "/interests",
    "A Few Interests of Fuwn",
    Box::new(move |context| {
      crate::success!(
        format!(
          "# Interests\n\nA collection of things that I think are pretty neat \
           and I am considering using more in the future.\n\n{}",
          contacts.join("\n")
        ),
        context
      )
    }),
  );
}
