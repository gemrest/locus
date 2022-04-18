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

use std::{lazy::SyncLazy, sync::Mutex};

use tokio::time::Instant;

use crate::ROUTES;

pub const CACHE_RATE: u64 = 60 * 5;

#[derive(Debug)]
pub struct Route {
  pub description: String,
  pub text_cache:  String,
}
impl Route {
  pub fn new(description: &str) -> Self {
    Self {
      description: description.to_string(),
      text_cache:  "".to_string(),
    }
  }
}

pub fn hits_from(route: &str) -> i32 {
  if let Ok(database) = crate::DATABASE.lock() {
    (*database)
      .get::<i32>(if route.is_empty() { "/" } else { route })
      .unwrap()
  } else {
    0
  }
}

pub fn track_mount(
  router: &mut windmark::Router,
  route: &str,
  description: &str,
  handler: windmark::handler::RouteResponse,
) {
  (*crate::ROUTES.lock().unwrap())
    .insert(route.to_string(), Route::new(description));
  router.mount(route, handler);
}

pub fn cache(context: &windmark::returnable::RouteContext<'_>, response: &str) {
  static LAST_CACHED: SyncLazy<Mutex<Instant>> =
    SyncLazy::new(|| Mutex::new(Instant::now()));

  if (*LAST_CACHED.lock().unwrap()).elapsed()
    >= std::time::Duration::from_secs(CACHE_RATE)
    || (*ROUTES.lock().unwrap())
      .get(context.url.path())
      .is_some_and(|&r| r.text_cache.is_empty())
  {
    (*LAST_CACHED.lock().unwrap()) = Instant::now();

    if let Some(route) = (*ROUTES.lock().unwrap()).get_mut(context.url.path()) {
      route.text_cache = response.to_string();
      info!("cache set for {}", context.url.path());
    } else {
      warn!(
        "cache could not be set for {} as it is not being tracked",
        context.url.path()
      );
    }

    trace!("recache for {}", context.url.path());
  } else {
    trace!(
      "no cache, with last: {:?}",
      (*ROUTES.lock().unwrap()).get(context.url.path())
    );
  }
}
