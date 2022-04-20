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

use tokio::time::Instant;

static UPTIME: SyncLazy<Instant> = SyncLazy::new(Instant::now);

pub fn module(router: &mut windmark::Router) {
  crate::route::track_mount(
    router,
    "/uptime",
    "The uptime of Locus (A.K.A., The Locus Epoch). (\\?[s|ms|mu|ns]?)?",
    Box::new(move |context| {
      windmark::Response::Success(context.url.query().map_or_else(
        || UPTIME.elapsed().as_nanos().to_string(),
        |query| {
          match query {
            "secs" | "seconds" | "s" => UPTIME.elapsed().as_secs().to_string(),
            "milli" | "milliseconds" | "ms" =>
              UPTIME.elapsed().as_millis().to_string(),
            "micro" | "microseconds" | "mu" =>
              UPTIME.elapsed().as_micros().to_string(),
            _ => UPTIME.elapsed().as_nanos().to_string(),
          }
        },
      ))
    }),
  );
}
