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

use std::lazy::SyncLazy;

pub static REMARKS: SyncLazy<Vec<String>> = SyncLazy::new(|| {
  serde_json::from_str(include_str!("../../content/json/remarks.json")).unwrap()
});

pub fn module(router: &mut windmark::Router) {
  crate::route::track_mount(
    router,
    "/remarks",
    "Fuwn's remarks",
    Box::new(|context| {
      crate::success!(
        format!(
          "# REMARKS\n\n{}",
          REMARKS
            .iter()
            .map(|r| format!("* {}", r))
            .collect::<Vec<String>>()
            .join("\n")
        ),
        context
      )
    }),
  );
}
