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

use tantivy::schema;
use tempfile::TempDir;

const SEARCH_INDEX_SIZE: usize = 10_000_000;
const SEARCH_SIZE: usize = 10;

static INDEX_PATH: SyncLazy<Mutex<TempDir>> =
  SyncLazy::new(|| Mutex::new(TempDir::new().unwrap()));
static SCHEMA: SyncLazy<Mutex<schema::Schema>> = SyncLazy::new(|| {
  Mutex::new({
    let mut schema_builder = schema::Schema::builder();

    schema_builder.add_text_field("path", schema::TEXT | schema::STORED);
    schema_builder.add_text_field("description", schema::TEXT | schema::STORED);
    schema_builder.add_text_field("content", schema::TEXT | schema::STORED);

    schema_builder.build()
  })
});
static INDEX: SyncLazy<Mutex<tantivy::Index>> = SyncLazy::new(|| {
  Mutex::new({
    tantivy::Index::create_in_dir(
      &(*INDEX_PATH.lock().unwrap()),
      (*SCHEMA.lock().unwrap()).clone(),
    )
    .unwrap()
  })
});
static INDEX_WRITER: SyncLazy<Mutex<tantivy::IndexWriter>> =
  SyncLazy::new(|| {
    Mutex::new((*INDEX.lock().unwrap()).writer(SEARCH_INDEX_SIZE).unwrap())
  });

pub(super) fn module(router: &mut windmark::Router) {
  crate::route::track_mount(
    router,
    "/search",
    "A search engine for this Gemini capsule",
    Box::new(|context| {
      let mut response = String::from(
        "# Search\n\n=> /search?action=go Search!\n=> /random I'm Feeling \
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
              &tantivy::collector::TopDocs::with_limit(SEARCH_SIZE),
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

pub fn index() {
  info!("spawned search indexer");

  loop {
    let path = (*SCHEMA.lock().unwrap()).get_field("path").unwrap();
    let description =
      (*SCHEMA.lock().unwrap()).get_field("description").unwrap();
    let content = (*SCHEMA.lock().unwrap()).get_field("content").unwrap();
    let time = tokio::time::Instant::now();
    let mut new = 0;

    for (route_path, information) in &(*crate::route::ROUTES.lock().unwrap()) {
      // Pretty inefficient, but I'll figure this out later.
      (*INDEX_WRITER.lock().unwrap())
        .delete_all_documents()
        .unwrap();

      (*INDEX_WRITER.lock().unwrap())
        .add_document(tantivy::doc!(
          path => route_path.clone(),
          description => information.description.clone(),
          content => information.text_cache.clone()
        ))
        .unwrap();

      new += 1;
    }

    (*INDEX_WRITER.lock().unwrap()).commit().unwrap();

    info!(
      "commit {} new items into search index in {}ms",
      new,
      time.elapsed().as_nanos() as f64 / 1_000_000.0
    );

    std::thread::sleep(std::time::Duration::from_secs(
      crate::route::CACHE_RATE,
    ));
  }
}
