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

type SkillTree = HashMap<String, Vec<HashMap<String, Option<Vec<String>>>>>;

static SKILL_TREE: SyncLazy<SkillTree> = SyncLazy::new(|| {
  serde_json::from_str(include_str!("../../content/json/skills.json")).unwrap()
});

pub fn module(router: &mut windmark::Router) {
  let mut skills = SKILL_TREE
    .iter()
    .map(|(category, skills)| {
      format!(
        "## {}\n\n{}",
        category,
        skills
          .iter()
          .map(|items| {
            items
              .iter()
              .map(|(item, areas)| {
                format!(
                  "* {}{}",
                  item,
                  areas.clone().map_or_else(
                    || "".to_string(),
                    |known_areas| format!(": {}", known_areas.join(", "))
                  )
                )
              })
              .collect::<Vec<String>>()
              .join("\n")
          })
          .collect::<Vec<String>>()
          .join("\n")
      )
    })
    .collect::<Vec<String>>();

  skills.sort();

  crate::route::track_mount(
    router,
    "/skills",
    "A Few Skills of Fuwn",
    Box::new(move |context| {
      crate::success!(format!("# Skills\n\n{}", skills.join("\n")), context)
    }),
  );
}
