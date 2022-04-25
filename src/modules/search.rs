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

use crate::search::{INDEX, SCHEMA};

pub fn module(router: &mut windmark::Router) {
  crate::route::track_mount(
    router,
    "/search",
    "A search engine for this Gemini capsule",
    Box::new(|context| {
      let mut response = String::from(
        "# SEARCH\n\n=> /search?action=go Search!\n=> /random I'm Feeling \
         Lucky",
      );

      if let Some(query) = context.url.query_pairs().next() {
        if query.0 == "action" && query.1 == "go" {
          return windmark::Response::Input(
            "What would you like to search for?".to_string(),
          );
        }

        {
          let path = (*SCHEMA.lock().unwrap()).get_field("path").unwrap();
          let description =
            (*SCHEMA.lock().unwrap()).get_field("description").unwrap();
          let content = (*SCHEMA.lock().unwrap()).get_field("content").unwrap();
          let mut results = String::new();

          let searcher = (*INDEX.lock().unwrap())
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommit)
            .try_into()
            .unwrap()
            .searcher();
          let top_docs = searcher
            .search(
              &tantivy::query::QueryParser::for_index(
                &(*INDEX.lock().unwrap()),
                vec![path, description, content],
              )
              .parse_query(&query.0.to_string())
              .unwrap(),
              &tantivy::collector::TopDocs::with_limit(crate::SEARCH_SIZE),
            )
            .unwrap();

          for (_score, document_address) in top_docs {
            let retrieved_document = searcher.doc(document_address).unwrap();

            macro_rules! text {
              ($field:ident) => {{
                retrieved_document
                  .get_first($field)
                  .unwrap()
                  .as_text()
                  .unwrap()
              }};
              ($document:ident, $field:ident) => {{
                $document.get_first($field).unwrap().as_text().unwrap()
              }};
            }

            results +=
              &format!("=> {} {}{}\n", text!(path), text!(description), {
                let mut lines = retrieved_document
                  .get_first(content)
                  .unwrap()
                  .as_text()
                  .unwrap()
                  .lines()
                  .skip(2);

                lines.next().map_or_else(
                  || "".to_string(),
                  |first_line| {
                    let mut context_lines = lines.skip_while(|l| {
                      !l.to_lowercase().contains(&query.0.to_string())
                    });

                    format!(
                      "\n> ... {}\n> {}\n> {} ...",
                      first_line,
                      context_lines.next().unwrap_or(""),
                      context_lines.next().unwrap_or("")
                    )
                  },
                )
              });
          }

          response += &format!(
            "\n\nYou searched for \"{}\"!\n\n## RESULTS\n\n{}\n\nIn need of \
             more results? This search engine populates its index with route \
             paths and route descriptions on startup. However, route content \
             isn't populated until the route is first visited. After a \
             route's first visit, it is updated after every five minutes, at \
             time of visit.",
            query.0,
            if results.is_empty() {
              "There are no results for your query...".to_string()
            } else {
              results.trim_end().to_string()
            },
          );
        }
      }

      crate::success!(response, context)
    }),
  );
}
