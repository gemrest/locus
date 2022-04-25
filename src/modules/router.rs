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

use crate::DATABASE;

pub fn module(router: &mut windmark::Router) {
  router.set_pre_route_callback(Box::new(|stream, url, _| {
    info!(
      "accepted connection from {} to {}",
      stream.peer_addr().unwrap().ip(),
      url.to_string(),
    );

    let url_path = if url.path().is_empty() {
      "/"
    } else {
      url.path()
    };

    let previous_database = (*DATABASE.lock().unwrap()).get::<i32>(url_path);

    match previous_database {
      None => {
        (*DATABASE.lock().unwrap())
          .set::<i32>(url_path, &0)
          .unwrap();
      }
      Some(_) => {}
    }

    let new_database = (*DATABASE.lock().unwrap()).get::<i32>(url_path);

    (*DATABASE.lock().unwrap())
      .set(url_path, &(new_database.unwrap() + 1))
      .unwrap();
  }));
}
