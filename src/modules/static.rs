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

use crate::batch_mount;

pub fn module(router: &mut windmark::Router) {
  batch_mount!(
    "files",
    router,
    ("/favicon.txt", "This Gemini capsule's icon", "favicon.txt"),
    (
      "/robots.txt",
      "Crawler traffic manager; for robots, not humans",
      "robots.txt"
    ),
  );

  batch_mount!(
    "pages",
    router,
    ("/", "This Gemini capsule's homepage", "index"),
    ("/donate", "Many ways to donate to Fuwn", "donate"),
    (
      "/gemini",
      "Information and resources for the Gemini protocol",
      "gemini"
    ),
    (
      "/gopher",
      "Information and resources for the Gopher protocol",
      "gopher"
    ),
    (
      "/licensing",
      "The licensing terms of this Gemini capsule",
      "licensing"
    ),
  );
}
