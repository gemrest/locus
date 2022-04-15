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

use std::{
  collections::HashMap,
  fs::{self, read_dir},
  io::Read,
};

use rand::seq::SliceRandom;
use windmark::Response;

use crate::{success, track_mount, Main, QUOTES};

#[allow(clippy::too_many_lines)]
pub fn multi_blog(router: &mut windmark::Router) {
  let paths = read_dir("content/pages/blog").unwrap();
  let mut blogs: HashMap<String, HashMap<_, _>> = HashMap::new();

  for path in paths {
    let blog = path.unwrap().path().display().to_string();
    let blog_paths = read_dir(&blog).unwrap();
    let mut entries: HashMap<_, String> = HashMap::new();

    blog_paths
      .map(|e| e.unwrap().path().display().to_string())
      .for_each(|file| {
        let mut contents = String::new();

        fs::File::open(&file)
          .unwrap()
          .read_to_string(&mut contents)
          .unwrap();

        entries.insert(
          file.replace(&blog, "").replace(".gmi", "").replace(
            {
              #[cfg(windows)]
              {
                '\\'
              }

              #[cfg(unix)]
              {
                '/'
              }
            },
            "",
          ),
          contents,
        );
      });

    blogs.insert(
      blog
        .replace(
          {
            #[cfg(windows)]
            {
              "content/pages/blog\\"
            }

            #[cfg(unix)]
            {
              "content/pages/blog/"
            }
          },
          "",
        )
        .split('_')
        .map(|s| {
          // https://gist.github.com/jpastuszek/2704f3c5a3864b05c48ee688d0fd21d7
          let mut c = s.chars();

          match c.next() {
            None => String::new(),
            Some(f) =>
              f.to_uppercase()
                .chain(c.flat_map(char::to_lowercase))
                .collect(),
          }
        })
        .collect::<Vec<_>>()
        .join(" "),
      entries,
    );
  }

  let blog_clone = blogs.clone();

  track_mount(
    router,
    "/blog",
    "Fuwn's blogs",
    Box::new(move |context| {
      success!(
        &format!(
          "# BLOGS ({})\n\n{}",
          blog_clone.len(),
          blog_clone
            .iter()
            .map(|(title, _)| title.clone())
            .collect::<Vec<_>>()
            .into_iter()
            .map(|i| {
              format!(
                "=> {} {}",
                format_args!("/blog/{}", i.replace(' ', "_").to_lowercase(),),
                i
              )
            })
            .collect::<Vec<_>>()
            .join("\n")
        ),
        context
      )
    }),
  );

  for (blog, entries) in blogs {
    let fixed_blog_name = blog.replace(' ', "_").to_lowercase();
    let entries_clone = entries.clone();
    let fixed_blog_name_clone = fixed_blog_name.clone();
    let blog_clone = blog.clone();

    track_mount(
      router,
      &format!("/blog/{}", fixed_blog_name),
      &format!("{} ― One of Fuwn's blogs", &blog),
      Box::new(move |context| {
        success!(
          &format!(
            "# {} ({})\n\n{}",
            blog.to_uppercase(),
            entries_clone.len(),
            entries_clone
              .iter()
              .map(|(title, _)| title.clone())
              .collect::<Vec<_>>()
              .into_iter()
              .map(|i| {
                format!(
                  "=> {} {}",
                  format_args!(
                    "/blog/{}/{}",
                    fixed_blog_name_clone,
                    i.to_lowercase()
                  ),
                  i
                )
              })
              .collect::<Vec<_>>()
              .join("\n")
          ),
          context
        )
      }),
    );

    for (title, contents) in entries {
      track_mount(
        router,
        &format!("/blog/{}/{}", fixed_blog_name, title.to_lowercase()),
        &format!(
          "{}, {} ― An entry to one of Fuwn's blogs",
          title, blog_clone
        ),
        Box::new(move |context| success!(contents, context)),
      );
    }
  }
}
