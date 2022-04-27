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

pub static INDEX_PATH: SyncLazy<Mutex<TempDir>> =
  SyncLazy::new(|| Mutex::new(TempDir::new().unwrap()));
pub static SCHEMA: SyncLazy<Mutex<schema::Schema>> =
  SyncLazy::new(|| {
    Mutex::new({
      let mut schema_builder = schema::Schema::builder();

      schema_builder.add_text_field("path", schema::TEXT | schema::STORED);
      schema_builder
        .add_text_field("description", schema::TEXT | schema::STORED);
      schema_builder.add_text_field("content", schema::TEXT | schema::STORED);

      schema_builder.build()
    })
  });
pub static INDEX: SyncLazy<Mutex<tantivy::Index>> = SyncLazy::new(|| {
  Mutex::new({
    tantivy::Index::create_in_dir(
      &(*INDEX_PATH.lock().unwrap()),
      (*SCHEMA.lock().unwrap()).clone(),
    )
    .unwrap()
  })
});
pub static INDEX_WRITER: SyncLazy<Mutex<tantivy::IndexWriter>> =
  SyncLazy::new(|| {
    Mutex::new((*INDEX.lock().unwrap()).writer(SEARCH_INDEX_SIZE).unwrap())
  });

pub fn index() {
  info!("spawned search indexer");

  loop {
    let path = (*SCHEMA.lock().unwrap()).get_field("path").unwrap();
    let description =
      (*SCHEMA.lock().unwrap()).get_field("description").unwrap();
    let content = (*SCHEMA.lock().unwrap()).get_field("content").unwrap();
    let time = tokio::time::Instant::now();
    let mut new = 0;

    for (route_path, information) in &(*crate::ROUTES.lock().unwrap()) {
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
