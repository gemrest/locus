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

#![feature(once_cell, is_some_with)]
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

mod macros;
mod modules;
mod route;
mod search;

#[macro_use]
extern crate log;

use std::{collections::HashMap, lazy::SyncLazy, sync::Mutex};

use pickledb::PickleDb;
use route::track_mount;
use tokio::time::Instant;
use windmark::{Response, Router};
use yarte::Template;

const SEARCH_SIZE: usize = 10;

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
static ROUTES: SyncLazy<Mutex<HashMap<String, route::Route>>> =
  SyncLazy::new(|| Mutex::new(HashMap::new()));

#[derive(Template)]
#[template(path = "main")]
struct Main<'a> {
  body:        &'a str,
  hits:        &'a i32,
  quote:       &'a str,
  commit:      &'a str,
  mini_commit: &'a str,
}

fn time_mounts<T>(context: &str, timer: &mut Instant, mut mounter: T)
where T: FnMut() {
  mounter();

  info!(
    "{} mounts took {}ms",
    context,
    timer.elapsed().as_nanos() as f64 / 1_000_000.0
  );

  *timer = Instant::now();
}

#[windmark::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  std::env::set_var("RUST_LOG", "windmark,locus=trace");
  pretty_env_logger::init();

  let mut time_mount = Instant::now();
  let mut router = Router::new();

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
  router.set_fix_path(true);

  info!(
    "creating router took {}ms",
    time_mount.elapsed().as_nanos() as f64 / 1_000_000.0
  );
  time_mount = Instant::now();

  time_mounts("module", &mut time_mount, || {
    router.attach_stateless(modules::uptime::module);
    router.attach_stateless(modules::sitemap::module);
    router.attach_stateless(modules::search::module);
    router.attach_stateless(modules::remarks::module);
    router.attach_stateless(modules::multi_blog::module);
    router.attach_stateless(modules::random::module);
  });

  time_mounts("static", &mut time_mount, || {
    batch_mount!(
      "files",
      router,
      (
        "/robots.txt",
        "Crawler traffic manager, for robots, not humans",
        "robots.txt"
      ),
      ("/favicon.txt", "This Gemini capsule's icon", "favicon.txt"),
    );

    batch_mount!(
      "pages",
      router,
      ("/", "This Gemini capsule's homepage", "index"),
      ("/contact", "Many ways to contact Fuwn", "contact"),
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
      ("/interests", "A few interests of Fuwn", "interests"),
      ("/skills", "A few skills of Fuwn", "skills"),
      (
        "/licensing",
        "The licensing terms of this Gemini capsule",
        "licensing"
      ),
    );
  });

  std::thread::spawn(search::index);

  router.run().await
}
