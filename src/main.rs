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

#[macro_use]
extern crate log;

use std::{collections::HashMap, lazy::SyncLazy, sync::Mutex};

use pickledb::PickleDb;
use tokio::time::Instant;

const SEARCH_SIZE: usize = 10;
const ERROR_HANDLER_RESPONSE: &str =
  "The requested resource could not be found at this time. You can try \
   refreshing the page, if that doesn't change anything; contact Fuwn! \
   (contact@fuwn.me)";

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

#[derive(yarte::Template)]
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

fn time_section(timer: &mut Instant, context: &str) {
  info!(
    "{} took {}ms",
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
  let mut router = windmark::Router::new();

  router.set_private_key_file(".locus/locus_private.pem");
  router.set_certificate_file(".locus/locus_public.pem");
  router.set_error_handler(Box::new(|_| {
    windmark::Response::NotFound(ERROR_HANDLER_RESPONSE.into())
  }));
  router.set_fix_path(true);

  time_section(&mut time_mount, "creating router");

  time_mounts("module", &mut time_mount, || stateless!(router, modules));

  std::thread::spawn(modules::search::index);

  router.run().await
}
