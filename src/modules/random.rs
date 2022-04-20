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

use rand::seq::SliceRandom;

pub fn module(router: &mut windmark::Router) {
  crate::route::track_mount(
    router,
    "/random",
    "Get redirected to a random route",
    Box::new(|_| {
      windmark::Response::TemporaryRedirect(
        (*crate::ROUTES.lock().unwrap())
          .iter()
          .collect::<Vec<_>>()
          .choose(&mut rand::thread_rng())
          .unwrap()
          .0
          .to_string(),
      )
    }),
  );
}
