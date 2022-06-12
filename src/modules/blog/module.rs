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

use crate::{modules::blog::config::Blog, route::track_mount, success};

#[allow(clippy::too_many_lines)]
pub fn module(router: &mut windmark::Router) {
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
            .map(|(title, entries)| (title.clone(), entries.clone()))
            .collect::<Vec<(_, _)>>()
            .into_iter()
            .map(|(title, entries)| {
              let config: Option<Blog> =
                entries.get("blog.json").and_then(|content| {
                  if let Ok(config) = Blog::from_string(content) {
                    Some(config)
                  } else {
                    None
                  }
                });
            let name = config
              .clone()
              .unwrap_or_default()
              .name()
              .clone()
              .unwrap_or_else(|| title.clone());
            let description = config
              .unwrap_or_default()
              .description()
              .clone()
              .unwrap_or_else(|| "One of Fuwn's blogs".to_string());

              format!(
                "=> {} {} ― {}",
                format_args!(
                  "/blog/{}",
                  name.replace(' ', "_").to_lowercase(),
                ),
                name,
              description
              )
            })
            .collect::<Vec<_>>()
            .join("\n")
        ),
        context
      )
    }),
  );

  for (blog, mut entries) in blogs {
    let fixed_blog_name = blog.replace(' ', "_").to_lowercase();
    let fixed_blog_name_clone = fixed_blog_name.clone();
    let config: Option<Blog> =
      entries.remove_entry("blog.json").and_then(|(_, content)| {
        if let Ok(config) = Blog::from_string(&content) {
          Some(config)
        } else {
          None
        }
      });
    let entries_clone = entries.clone();
    let name = config
      .clone()
      .unwrap_or_default()
      .name()
      .clone()
      .unwrap_or_else(|| blog.clone());
    let description = config
      .clone()
      .unwrap_or_default()
      .description()
      .clone()
      .unwrap_or_else(|| "One of Fuwn's blogs".to_string());
    let config_clone = config.clone();

    track_mount(
      router,
      &format!("/blog/{}", fixed_blog_name),
      &format!("{} ― {}", name, description),
      Box::new(move |context| {
        success!(
          &format!(
            "# {} ({})\n\n{}\n\n{}",
            blog.to_uppercase(),
            entries_clone.len(),
            description,
            entries_clone
              .iter()
              .map(|(title, _)| title.clone())
              .collect::<Vec<_>>()
              .into_iter()
              .map(|title| {
                format!(
                  "=> {} {}{}",
                  format_args!(
                    "/blog/{}/{}",
                    fixed_blog_name_clone,
                    title.to_lowercase()
                  ),
                  title,
                  {
                    let post = config_clone
                      .clone()
                      .unwrap_or_default()
                      .posts()
                      .clone()
                      .and_then(|posts| posts.get(&title).cloned())
                      .unwrap_or_default()
                      .description()
                      .clone()
                      .unwrap_or_else(|| "".to_string());

                    if post.is_empty() {
                      "".to_string()
                    } else {
                      format!(" ― {}", post)
                    }
                  }
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
      let header = if let Ok(header) = construct_header(&config, &title) {
        header
      } else {
        "".to_string()
      };

      track_mount(
        router,
        &format!("/blog/{}/{}", fixed_blog_name, title.to_lowercase()),
        &format!("{}, {} ― An entry to one of Fuwn's blogs", name, title),
        Box::new(move |context| {
          success!(format!("{}\n{}", header, contents), context)
        }),
      );
    }
  }
}

fn construct_header(config: &Option<Blog>, name: &str) -> Result<String, ()> {
  let post =
    if let Some(posts) = config.clone().unwrap_or_default().posts().clone() {
      if let Some(post) = posts.get(name) {
        post.clone()
      } else {
        return Err(());
      }
    } else {
      return Err(());
    };

  Ok(format!(
    "# {}\n\n{}{}{}{}",
    post.name().clone().unwrap_or_else(|| name.to_string()),
    if post.author().is_some() {
      format!("Author: {}\n", post.author().clone().unwrap())
    } else {
      "".to_string()
    },
    if post.created().is_some() {
      format!("Created: {}\n", post.created().clone().unwrap())
    } else {
      "".to_string()
    },
    if post.last_modified().is_some() {
      format!("Last Modified: {}\n", post.last_modified().clone().unwrap())
    } else {
      "".to_string()
    },
    if post.description().is_some() {
      format!("\n{}\n", post.description().clone().unwrap())
    } else {
      "".to_string()
    },
  ))
}
