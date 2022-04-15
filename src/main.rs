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

use std::{collections::HashMap, lazy::SyncLazy, sync::Mutex};

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
static ROUTES: SyncLazy<Mutex<HashMap<String, String>>> =
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

fn hits_from_route(route: &str) -> i32 {
  if let Ok(database) = DATABASE.lock() {
    (*database)
      .get::<i32>(if route.is_empty() { "/" } else { route })
      .unwrap()
  } else {
    0
  }
}

fn track_mount(
  router: &mut Router,
  route: &str,
  description: &str,
  handler: windmark::handler::RouteResponse,
) {
  (*ROUTES.lock().unwrap()).insert(route.to_string(), description.to_string());
  router.mount(route, handler);
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
  router.set_fix_path(true);
  track_mount(
    &mut router,
    "/uptime",
    "The uptime of Locus (A.K.A., The Locus Epoch). (\\?[s|ms|mu|ns]?)?",
    Box::new(move |context| {
      Response::Success(context.url.query().map_or_else(
        || uptime.elapsed().as_nanos().to_string(),
        |query| {
          match query {
            "secs" | "seconds" | "s" => uptime.elapsed().as_secs().to_string(),
            "milli" | "milliseconds" | "ms" =>
              uptime.elapsed().as_millis().to_string(),
            "micro" | "microseconds" | "mu" =>
              uptime.elapsed().as_micros().to_string(),
            _ => uptime.elapsed().as_nanos().to_string(),
          }
        },
      ))
    }),
  );

  info!(
    "preliminary mounts took {}ms",
    time_mount.elapsed().as_nanos() as f64 / 1_000_000.0
  );
  time_mount = Instant::now();

  router.attach_stateless(modules::multi_blog);

  info!(
    "blog mounts took {}ms",
    time_mount.elapsed().as_nanos() as f64 / 1_000_000.0
  );
  time_mount = Instant::now();

  mount_file!(
    router,
    "/robots.txt",
    "Crawler traffic manager, for robots, not humans",
    "robots.txt"
  );
  mount_file!(
    router,
    "/favicon.txt",
    "This Gemini capsule's icon",
    "favicon.txt"
  );
  mount_page!(router, "/", "This Gemini capsule's homepage", "index");
  mount_page!(router, "/contact", "Many ways to contact Fuwn", "contact");
  mount_page!(router, "/donate", "Many ways to donate to Fuwn", "donate");
  mount_page!(
    router,
    "/gemini",
    "Information and resources for the Gemini protocol",
    "gemini"
  );
  mount_page!(
    router,
    "/gopher",
    "Information and resources for the Gopher protocol",
    "gopher"
  );
  mount_page!(router, "/interests", "A few interests of Fuwn", "interests");
  mount_page!(router, "/skills", "A few skills of Fuwn", "skills");

  info!(
    "static mounts took {}ms",
    time_mount.elapsed().as_nanos() as f64 / 1_000_000.0
  );

  track_mount(
    &mut router,
    "/sitemap",
    "A map of all publicly available routes on this Gemini capsule",
    Box::new(|context| {
      success!(
        format!(
          "# SITEMAP\n\n{}",
          (*ROUTES.lock().unwrap())
            .iter()
            .map(|(r, d)| format!("=> {} {}", r, d))
            .collect::<Vec<_>>()
            .join("\n")
        ),
        context
      )
    }),
  );

  track_mount(
    &mut router,
    "/search",
    "A search engine for this Gemini capsule",
    Box::new(|context| {
      let mut query_pairs = context.url.query_pairs();
      let mut response =
        String::from("# SEARCH\n\n=> /search?action=go Search!");

      if let Some(query) = query_pairs.next() {
        if query.0 == "action" && query.1 == "go" {
          return Response::Input(
            "What would you like to search for?".to_string(),
          );
        }

        let results = (*ROUTES.lock().unwrap())
          .iter()
          .map(|(r, d)| format!("=> {} {}", r, d))
          .filter(|r| r.to_lowercase().contains(&query.0.to_string()))
          .collect::<Vec<_>>()
          .join("\n");

        response += &format!(
          "\n\nYou searched for \"{}\"!\n\n## RESULTS\n\n{}",
          query.0,
          if results.is_empty() {
            "There are no results for your query...".to_string()
          } else {
            results
          },
        );
      }

      success!(response, context)
    }),
  );

  track_mount(
    &mut router,
    "/remarks",
    "Fuwn's remarks",
    Box::new(|context| {
      let remarks: Vec<String> =
        serde_json::from_str(include_str!("../content/json/remarks.json"))
          .unwrap();

      success!(
        format!(
          "# REMARKS\n\n{}",
          remarks
            .into_iter()
            .map(|r| format!("* {}", r))
            .collect::<Vec<String>>()
            .join("\n")
        ),
        context
      )
    }),
  );

  router.run().await
}
