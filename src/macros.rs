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

pub static QUOTES: SyncLazy<Vec<String>> = SyncLazy::new(|| {
  serde_json::from_str(include_str!("../content/json/quotes.json")).unwrap()
});

#[derive(yarte::Template)]
#[template(path = "main")]
pub struct Main<'a> {
  pub body:        &'a str,
  pub hits:        &'a i32,
  pub quote:       &'a str,
  pub commit:      &'a str,
  pub mini_commit: &'a str,
}

#[macro_export]
macro_rules! success {
  ($body:expr, $context:ident) => {{
    $crate::route::cache(&$context, &$body);

    windmark::Response::Success(
      $crate::macros::Main {
        body:        &$body,
        hits:        &$crate::route::hits_from($context.url.path()),
        quote:       {
          use rand::seq::SliceRandom;

          &$crate::macros::QUOTES
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
        },
        commit:      &format!("/tree/{}", env!("VERGEN_GIT_SHA")),
        mini_commit: env!("VERGEN_GIT_SHA").get(0..5).unwrap_or("UNKNOWN"),
      }
      .to_string(),
    )
  }};
}

#[macro_export]
macro_rules! mount_page {
  ($router:ident, $at:literal, $description:literal, $file:literal) => {
    (*$crate::route::ROUTES.lock().unwrap())
      .insert($at.to_string(), $crate::route::Route::new($description));

    ($router).mount(
      $at,
      Box::new(|context| {
        let content =
          include_str!(concat!("../../content/pages/", $file, ".gmi"));

        $crate::route::cache(&context, &content);
        $crate::success!(content, context)
      }),
    );
  };
}

#[macro_export]
macro_rules! mount_file {
  ($router:ident, $at:literal, $description:literal, $file:literal) => {
    (*$crate::route::ROUTES.lock().unwrap())
      .insert($at.to_string(), $crate::route::Route::new($description));

    ($router).mount(
      $at,
      Box::new(|_| {
        windmark::Response::SuccessFile(include_bytes!(concat!(
          "../../content/meta/",
          $file
        )))
      }),
    );
  };
}

#[macro_export]
macro_rules! batch_mount {
  ("pages", $router:ident, $(($path:literal, $description:literal, $file:literal)),* $(,)?) => {
    $(
      $crate::mount_page!($router, $path, $description, $file);
    )*
  };
  ("files", $router:ident, $(($path:literal, $description:literal, $file:literal)),* $(,)?) => {
    $(
      $crate::mount_file!($router, $path, $description, $file);
    )*
  };
}

#[macro_export]
macro_rules! stateless {
  ($router:ident, $module:tt) => {{
    $router.attach_stateless($module::module);
  }};
}

#[macro_export]
macro_rules! statelesses {
  ($router:ident, $($module:tt),* $(,)?) => {
    $($crate::stateless!($router, $module);)*
  };
}
