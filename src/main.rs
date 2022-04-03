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

#![feature(once_cell)]
#![deny(
  warnings,
  nonstandard_style,
  unused,
  future_incompatible,
  rust_2018_idioms,
  unsafe_code
)]
#![deny(clippy::all, clippy::nursery, clippy::pedantic)]
#![recursion_limit = "128"]
#![allow(clippy::cast_precision_loss)]

mod constants;
mod macros;
mod modules;

#[macro_use]
extern crate log;

use std::{lazy::SyncLazy, sync::Mutex};

use constants::QUOTES;
use pickledb::PickleDb;
use rand::seq::SliceRandom;
use tokio::time::Instant;
use windmark::{Response, Router};
use yarte::Template;

static DATABASE: SyncLazy<Mutex<PickleDb>> = SyncLazy::new(|| {
  Mutex::new({
    if std::fs::File::open(".locus/locus.db").is_ok() {
      PickleDb::load_json(
        ".locus/locus.db",
        pickledb::PickleDbDumpPolicy::AutoDump,
      )
      .unwrap()
    } else {
      PickleDb::new_json(
        ".locus/locus.db",
        pickledb::PickleDbDumpPolicy::AutoDump,
      )
    }
  })
});

#[derive(Template)]
#[template(path = "main")]
struct Main<'a> {
  body:        &'a str,
  hits:        &'a i32,
  quote:       &'a str,
  commit:      &'a str,
  mini_commit: &'a str,
}

fn hits_from_route(route: &str) -> i32 {
  if let Ok(database) = DATABASE.lock() {
    (*database)
      .get::<i32>(if route.is_empty() { "/" } else { route })
      .unwrap()
  } else {
    0
  }
}

#[windmark::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  std::env::set_var("RUST_LOG", "windmark,locus=trace");
  pretty_env_logger::init();

  let mut time_mount = Instant::now();
  let mut router = Router::new();
  let uptime = Instant::now();

  router.set_private_key_file(".locus/locus_private.pem");
  router.set_certificate_file(".locus/locus_public.pem");
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
  router.set_error_handler(Box::new(|_| {
    Response::NotFound(
      "The requested resource could not be found at this time. You can try \
       refreshing the page, if that doesn't change anything; contact Fuwn! \
       (contact@fuwn.me)"
        .into(),
    )
  }));
  router.mount(
    "/uptime",
    Box::new(move |context| {
      success!(
        &format!("# UPTIME\n\n{} seconds", uptime.elapsed().as_secs()),
        context
      )
    }),
  );

  info!(
    "preliminary mounts took {}ms",
    time_mount.elapsed().as_nanos() as f64 / 1_000_000.0
  );
  time_mount = Instant::now();

  router.attach(modules::multi_blog);

  info!(
    "blog mounts took {}ms",
    time_mount.elapsed().as_nanos() as f64 / 1_000_000.0
  );
  time_mount = Instant::now();

  mount_file!(router, "/robots.txt", "robots.txt");
  mount_file!(router, "/favicon.txt", "favicon.txt");
  mount_page!(router, "/", "index");
  mount_page!(router, "", "index");
  mount_page!(router, "/contact", "contact");
  mount_page!(router, "/donate", "donate");
  mount_page!(router, "/gemini", "gemini");
  mount_page!(router, "/gopher", "gopher");
  mount_page!(router, "/interests", "interests");
  mount_page!(router, "/skills", "skills");

  info!(
    "static mounts took {}ms",
    time_mount.elapsed().as_nanos() as f64 / 1_000_000.0
  );

  router.run().await
}
