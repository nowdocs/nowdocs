//! # Store sync facade
//!
//! `Store` wraps LanceDB's async API behind a synchronous interface using
//! `tokio::runtime::Builder::new_current_thread().enable_all()`. This is
//! suitable for CLI and stdio MCP server consumers.
//!
//! **Limitation:** `Store` cannot be used inside an existing tokio runtime
//! (detected via `catch_unwind(block_on)` probe; `spawn_blocking` callers
//! are supported). Future async MCP transport or library embedding should
//! introduce an `AsyncStore` variant (deferred to v2).

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use arrow_array::{Array, FixedSizeListArray, Float16Array, RecordBatch, StringArray, UInt32Array};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use half::f16;
use lance_arrow::FixedSizeListArrayExt;
use lancedb::index::scalar::{FtsIndexBuilder, FullTextSearchQuery};
use lancedb::index::Index;
use lancedb::query::{ExecutableQuery, QueryBase, QueryExecutionOptions};
use lancedb::{table::NewColumnTransform, Session};

use crate::cache;
use crate::chunker::{Chunk, ChunkType};

pub struct SearchHit {
    pub score: f32,
    pub chunk_idx: u32,
    pub heading_path: String,
    pub source_url: String,
    pub api_version: Option<String>,
    pub chunk_type: ChunkType,
    pub text: String,
}

pub struct Store {
    #[allow(dead_code)]
    docset: String,
    conn: lancedb::Connection,
    runtime: tokio::runtime::Runtime,
    table_name: String,
}

const TABLE_NAME: &str = "chunks";
const VECTOR_DIM: usize = 512;

impl Store {
    pub fn open(docset: &str) -> Result<Self> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            // Detect if we are in an actively running async context where nested block_on would panic.
            // In tokio::task::spawn_blocking, block_on is permitted and does not panic.
            let is_nested_async = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handle.block_on(async {});
            }))
            .is_err();

            if is_nested_async {
                bail!(
                    "Store::open cannot run inside an existing Tokio runtime; \
                     nowdocs Store is synchronous over LanceDB async APIs, so call \
                     it from a blocking thread or refactor the caller to use an async store boundary"
                );
            }
        }

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to build tokio runtime")?;

        let db_path = cache::db_path(docset);
        let db_path_str = db_path.to_string_lossy().to_string();

        // Shared session for LRU + metadata cache (spec §6.5 C1).
        let session = Arc::new(Session::new(256, 256, Default::default()));

        let conn = runtime
            .block_on(lancedb::connect(&db_path_str).session(session).execute())
            .context("failed to connect to lancedb")?;

        let table_name = TABLE_NAME.to_string();

        // Create only when the table is genuinely absent. Do not treat an
        // arbitrary open error (corruption, permissions, incompatible metadata)
        // as "missing", because creating over that state can mask a broken
        // cache and make recovery harder.
        let table_names = runtime
            .block_on(conn.table_names().execute())
            .context("failed to list lancedb tables")?;
        if !table_names.iter().any(|name| name == &table_name) {
            let schema = table_schema();
            runtime
                .block_on(conn.create_empty_table(&table_name, schema).execute())
                .context("failed to create table")?;

            // Build FTS index on "text" column.
            let table = runtime
                .block_on(conn.open_table(&table_name).execute())
                .context("failed to open newly created table")?;
            runtime
                .block_on(
                    table
                        .create_index(&["text"], Index::FTS(FtsIndexBuilder::default()))
                        .execute(),
                )
                .context("failed to create FTS index")?;
        } else {
            let table = runtime
                .block_on(conn.open_table(&table_name).execute())
                .context("failed to open existing table")?;
            runtime
                .block_on(ensure_table_schema(&table))
                .context("failed to migrate table schema")?;
        }

        Ok(Self {
            docset: docset.to_string(),
            conn,
            runtime,
            table_name,
        })
    }

    pub fn insert(&self, chunks: &[Chunk], vectors: &[Vec<f32>]) -> Result<()> {
        if chunks.len() != vectors.len() {
            anyhow::bail!(
                "chunks ({}) and vectors ({}) length mismatch",
                chunks.len(),
                vectors.len()
            );
        }

        for (i, vector) in vectors.iter().enumerate() {
            if vector.len() != VECTOR_DIM {
                anyhow::bail!(
                    "vector[{i}] has dimension {}, expected {}",
                    vector.len(),
                    VECTOR_DIM
                );
            }
        }

        if chunks.is_empty() {
            return Ok(());
        }

        let batch = chunks_to_batch(chunks, vectors)?;
        let table = self
            .runtime
            .block_on(self.conn.open_table(&self.table_name).execute())
            .context("failed to open table for insert")?;

        self.runtime
            .block_on(table.add(vec![batch]).execute())
            .context("failed to insert rows")?;

        Ok(())
    }

    pub fn fetch_by_idx(&self, ids: &[u32]) -> Result<Vec<SearchHit>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let filter = ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let filter = format!("chunk_idx IN ({filter})");

        let table = self
            .runtime
            .block_on(self.conn.open_table(&self.table_name).execute())
            .context("failed to open table for fetch_by_idx")?;

        let batches: Vec<RecordBatch> = self.runtime.block_on(async {
            let stream = table
                .query()
                .only_if(&filter)
                .execute()
                .await
                .context("fetch_by_idx query failed")?;
            stream
                .try_collect::<Vec<RecordBatch>>()
                .await
                .context("failed to collect fetch_by_idx results")
        })?;

        parse_search_hits(&batches, 0.0)
    }

    /// Fetch raw f32 vectors for the given chunk indices (N1). Used by MMR to
    /// compute inter-chunk cosine similarity for diversity reranking. Additive:
    /// it does not modify `fetch_by_idx` or `hybrid_search`. Indices not present
    /// in the table are simply absent from the returned map; an empty input
    /// short-circuits to an empty map. Vectors are stored as f16 and converted
    /// back to f32 here (the small round-trip error is immaterial for cosine).
    pub fn fetch_vectors(&self, chunk_ids: &[u32]) -> Result<HashMap<u32, Vec<f32>>> {
        if chunk_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let filter = chunk_ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let filter = format!("chunk_idx IN ({filter})");

        let table = self
            .runtime
            .block_on(self.conn.open_table(&self.table_name).execute())
            .context("failed to open table for fetch_vectors")?;

        let batches: Vec<RecordBatch> = self.runtime.block_on(async {
            let stream = table
                .query()
                .only_if(&filter)
                .execute()
                .await
                .context("fetch_vectors query failed")?;
            stream
                .try_collect::<Vec<RecordBatch>>()
                .await
                .context("failed to collect fetch_vectors results")
        })?;

        parse_vectors(&batches)
    }

    /// Number of rows in the docset's table. Used by install's promote step to
    /// verify a freshly installed `.lance` table actually contains chunks
    /// without loading every row into memory.
    pub fn row_count(&self) -> Result<u64> {
        let table = self
            .runtime
            .block_on(self.conn.open_table(&self.table_name).execute())
            .context("failed to open table for row_count")?;
        let n = self
            .runtime
            .block_on(table.count_rows(None))
            .context("failed to count rows")?;
        Ok(n as u64)
    }

    /// Dump all chunks (text + metadata only, no vectors) from the store.
    pub fn dump_chunks(&self) -> Result<Vec<Chunk>> {
        let table = self
            .runtime
            .block_on(self.conn.open_table(&self.table_name).execute())
            .context("failed to open table for dump_chunks")?;

        let batches: Vec<RecordBatch> = self.runtime.block_on(async {
            let stream = table
                .query()
                .execute()
                .await
                .context("dump_chunks query failed")?;
            stream
                .try_collect::<Vec<RecordBatch>>()
                .await
                .context("failed to collect dump_chunks results")
        })?;

        let hits = parse_search_hits(&batches, 0.0)?;
        Ok(hits
            .into_iter()
            .map(|h| Chunk {
                idx: h.chunk_idx,
                heading_path: h.heading_path,
                source_url: h.source_url,
                api_version: h.api_version,
                chunk_type: h.chunk_type,
                text: h.text,
            })
            .collect())
    }

    pub fn hybrid_search(
        &self,
        query_vector: &[f32],
        query_text: &str,
        top_k: usize,
    ) -> Result<Vec<SearchHit>> {
        self.hybrid_search_k(query_vector, query_text, top_k, 60.0)
    }

    /// Hybrid search with a configurable RRF fusion constant. `hybrid_search`
    /// delegates with the default k=60 (paper-recommended, near-optimal per
    /// Cormack et al. 2009); this variant lets diagnostics probe how the fusion
    /// constant shifts recall. k is the constant in `score = 1/(rank + k)`:
    /// smaller k makes the top ranks dominate, larger k flattens scores.
    pub fn hybrid_search_k(
        &self,
        query_vector: &[f32],
        query_text: &str,
        top_k: usize,
        rrf_k: f32,
    ) -> Result<Vec<SearchHit>> {
        let table = self
            .runtime
            .block_on(self.conn.open_table(&self.table_name).execute())
            .context("failed to open table for search")?;

        let qv_f16: Vec<f16> = query_vector.iter().map(|&v| f16::from_f32(v)).collect();
        let fts_query = FullTextSearchQuery::new(query_text.to_string());

        let batches: Vec<RecordBatch> = self.runtime.block_on(async {
            let stream = table
                .query()
                .full_text_search(fts_query)
                .nearest_to(&*qv_f16)?
                .limit(top_k)
                .rerank(Arc::new(lancedb::rerankers::rrf::RRFReranker::new(rrf_k)))
                .execute_hybrid(QueryExecutionOptions::default())
                .await
                .context("hybrid search execution failed")?;
            stream
                .try_collect::<Vec<RecordBatch>>()
                .await
                .context("failed to collect hybrid search results")
        })?;

        let mut hits = parse_search_hits_with_score(&batches)?;
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        Ok(hits)
    }

    /// Pure FTS (BM25) search, no vector, no reranker. For diagnostics: isolates
    /// the FTS channel's recall/ranking from the vector channel and RRF fusion.
    /// Hits are ordered by descending BM25 `_score`.
    pub fn fts_search(&self, query_text: &str, top_k: usize) -> Result<Vec<SearchHit>> {
        let table = self
            .runtime
            .block_on(self.conn.open_table(&self.table_name).execute())
            .context("failed to open table for FTS search")?;
        let fts_query = FullTextSearchQuery::new(query_text.to_string());
        let batches: Vec<RecordBatch> = self.runtime.block_on(async {
            let stream = table
                .query()
                .full_text_search(fts_query)
                .limit(top_k)
                .execute()
                .await
                .context("FTS search execution failed")?;
            stream
                .try_collect::<Vec<RecordBatch>>()
                .await
                .context("failed to collect FTS search results")
        })?;
        let mut hits = parse_search_hits_with_score(&batches)?;
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        Ok(hits)
    }

    /// Pure vector (k-NN) search, no FTS, no reranker. For diagnostics: isolates
    /// the vector channel's recall/ranking. Hits are ordered by ascending
    /// `_distance` (smaller = more similar), surfaced as descending score via
    /// the negation in `parse_search_hits_with_score`.
    pub fn vector_search(&self, query_vector: &[f32], top_k: usize) -> Result<Vec<SearchHit>> {
        let table = self
            .runtime
            .block_on(self.conn.open_table(&self.table_name).execute())
            .context("failed to open table for vector search")?;
        let qv_f16: Vec<f16> = query_vector.iter().map(|&v| f16::from_f32(v)).collect();
        let batches: Vec<RecordBatch> = self.runtime.block_on(async {
            let stream = table
                .query()
                .nearest_to(&*qv_f16)?
                .limit(top_k)
                .execute()
                .await
                .context("vector search execution failed")?;
            stream
                .try_collect::<Vec<RecordBatch>>()
                .await
                .context("failed to collect vector search results")
        })?;
        let mut hits = parse_search_hits_with_score(&batches)?;
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        Ok(hits)
    }
}

/// Parse record batches into SearchHit items with a fixed score (for fetch_by_idx).
fn parse_search_hits(batches: &[RecordBatch], default_score: f32) -> Result<Vec<SearchHit>> {
    let mut hits = Vec::new();
    for batch in batches {
        if batch.column_by_name("chunk_idx").is_none() {
            continue;
        }
        let idx_col = batch
            .column_by_name("chunk_idx")
            .and_then(|c| c.as_any().downcast_ref::<UInt32Array>())
            .context("missing chunk_idx column")?;
        let heading_col = batch
            .column_by_name("heading_path")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("missing heading_path column")?;
        let url_col = batch
            .column_by_name("source_url")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("missing source_url column")?;
        let api_col = batch
            .column_by_name("api_version")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>());
        let ctype_col = batch
            .column_by_name("chunk_type")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("missing chunk_type column")?;
        let text_col = batch
            .column_by_name("text")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("missing text column")?;

        for row in 0..batch.num_rows() {
            let chunk_type = match ctype_col.value(row) {
                "Code" => ChunkType::Code,
                _ => ChunkType::Info,
            };
            hits.push(SearchHit {
                score: default_score,
                chunk_idx: idx_col.value(row),
                heading_path: heading_col.value(row).to_string(),
                source_url: url_col.value(row).to_string(),
                api_version: string_value(api_col, row),
                chunk_type,
                text: text_col.value(row).to_string(),
            });
        }
    }
    hits.sort_by_key(|h| h.chunk_idx);
    Ok(hits)
}

fn string_value(col: Option<&StringArray>, row: usize) -> Option<String> {
    let col = col?;
    if col.is_null(row) {
        None
    } else {
        Some(col.value(row).to_string())
    }
}

/// Parse record batches into SearchHit items with _score column (for hybrid_search).
fn parse_search_hits_with_score(batches: &[RecordBatch]) -> Result<Vec<SearchHit>> {
    let mut hits = Vec::new();
    for batch in batches {
        if batch.column_by_name("chunk_idx").is_none() {
            continue;
        }
        // Score: prefer the RRF fused _relevance_score (hybrid), then raw FTS
        // BM25 _score (pure FTS), then negated vector _distance (pure vector
        // k-NN). Negating distance lets every caller sort by score descending
        // uniformly — hybrid/FTS rank high-is-better, vector ranks
        // low-distance-is-better, so -distance sorts the most-similar first.
        let rel_col = batch
            .column_by_name("_relevance_score")
            .and_then(|c| c.as_any().downcast_ref::<arrow_array::Float32Array>());
        let fts_col = batch
            .column_by_name("_score")
            .and_then(|c| c.as_any().downcast_ref::<arrow_array::Float32Array>());
        let dist_col = batch
            .column_by_name("_distance")
            .and_then(|c| c.as_any().downcast_ref::<arrow_array::Float32Array>());
        let idx_col = batch
            .column_by_name("chunk_idx")
            .and_then(|c| c.as_any().downcast_ref::<UInt32Array>())
            .context("missing chunk_idx column")?;
        let heading_col = batch
            .column_by_name("heading_path")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("missing heading_path column")?;
        let url_col = batch
            .column_by_name("source_url")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("missing source_url column")?;
        let api_col = batch
            .column_by_name("api_version")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>());
        let ctype_col = batch
            .column_by_name("chunk_type")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("missing chunk_type column")?;
        let text_col = batch
            .column_by_name("text")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .context("missing text column")?;

        for row in 0..batch.num_rows() {
            let chunk_type = match ctype_col.value(row) {
                "Code" => ChunkType::Code,
                _ => ChunkType::Info,
            };
            hits.push(SearchHit {
                score: match (rel_col, fts_col, dist_col) {
                    (Some(r), _, _) => r.value(row),
                    (None, Some(f), _) => f.value(row),
                    (None, None, Some(d)) => -d.value(row),
                    (None, None, None) => {
                        bail!("missing _relevance_score/_score/_distance column")
                    }
                },
                chunk_idx: idx_col.value(row),
                heading_path: heading_col.value(row).to_string(),
                source_url: url_col.value(row).to_string(),
                api_version: string_value(api_col, row),
                chunk_type,
                text: text_col.value(row).to_string(),
            });
        }
    }
    Ok(hits)
}

/// Parse record batches into a `{chunk_idx -> f32 vector}` map (for `fetch_vectors`).
/// The `vector` column is a `FixedSizeList<Float16, 512>`; each row's list is
/// converted back to f32 for cosine-similarity math in MMR.
fn parse_vectors(batches: &[RecordBatch]) -> Result<HashMap<u32, Vec<f32>>> {
    let mut out = HashMap::new();
    for batch in batches {
        if batch.column_by_name("chunk_idx").is_none() || batch.column_by_name("vector").is_none() {
            continue;
        }
        let idx_col = batch
            .column_by_name("chunk_idx")
            .and_then(|c| c.as_any().downcast_ref::<UInt32Array>())
            .context("missing chunk_idx column")?;
        let vec_col = batch
            .column_by_name("vector")
            .and_then(|c| c.as_any().downcast_ref::<FixedSizeListArray>())
            .context("vector column is not FixedSizeList")?;
        for row in 0..batch.num_rows() {
            let list = vec_col.value(row);
            let f16s = list
                .as_any()
                .downcast_ref::<Float16Array>()
                .context("vector values are not Float16")?;
            let mut v = Vec::with_capacity(f16s.len());
            for i in 0..f16s.len() {
                v.push(f16s.value(i).to_f32());
            }
            out.insert(idx_col.value(row), v);
        }
    }
    Ok(out)
}

/// Ensure an existing table can accept the current RecordBatch schema.
///
/// Compatible additive migrations are applied in-place: any newly introduced
/// nullable field is appended as an all-null Arrow column. Required-field or
/// type changes are intentionally rejected with a rebuild hint because they need
/// re-materializing vectors/chunks rather than blind schema surgery.
async fn ensure_table_schema(table: &lancedb::Table) -> Result<()> {
    let existing = table.schema().await?;
    let expected = table_schema();
    let mut missing_nullable = Vec::new();

    for field in expected.fields() {
        match existing.field_with_name(field.name()) {
            Ok(current) if current.data_type() == field.data_type() => {}
            Ok(current) => bail!(
                "incompatible LanceDB column `{}`: found {:?}, expected {:?}; run `nowdocs rebuild <docset>`",
                field.name(),
                current.data_type(),
                field.data_type()
            ),
            Err(_) if field.is_nullable() => missing_nullable.push(field.clone()),
            Err(_) => bail!(
                "missing required LanceDB column `{}`; run `nowdocs rebuild <docset>`",
                field.name()
            ),
        }
    }

    if !missing_nullable.is_empty() {
        let schema = Arc::new(Schema::new(missing_nullable));
        table
            .add_columns(NewColumnTransform::AllNulls(schema), None)
            .await?;
    }

    Ok(())
}

/// Build the table schema: id, vector(FixedSizeList<f16,512>), heading_path,
/// source_url, api_version, chunk_type, chunk_idx, text.
fn table_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        // dead field, do not rely on
        Field::new("id", DataType::UInt32, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float16, true)),
                VECTOR_DIM as i32,
            ),
            false,
        ),
        Field::new("heading_path", DataType::Utf8, false),
        Field::new("source_url", DataType::Utf8, false),
        Field::new("api_version", DataType::Utf8, true),
        Field::new("chunk_type", DataType::Utf8, false),
        Field::new("chunk_idx", DataType::UInt32, false),
        Field::new("text", DataType::Utf8, false),
    ]))
}

/// Convert chunks + f32 vectors into an arrow RecordBatch (vector column as f16).
fn chunks_to_batch(chunks: &[Chunk], vectors: &[Vec<f32>]) -> Result<RecordBatch> {
    let n = chunks.len();

    // dead field, do not rely on
    let ids: UInt32Array = (0..n as u32).collect();
    let heading_paths = StringArray::from(
        chunks
            .iter()
            .map(|c| c.heading_path.as_str())
            .collect::<Vec<_>>(),
    );
    let source_urls = StringArray::from(
        chunks
            .iter()
            .map(|c| c.source_url.as_str())
            .collect::<Vec<_>>(),
    );
    let api_versions = StringArray::from(
        chunks
            .iter()
            .map(|c| c.api_version.as_deref())
            .collect::<Vec<_>>(),
    );
    let chunk_types = StringArray::from(
        chunks
            .iter()
            .map(|c| match c.chunk_type {
                ChunkType::Code => "Code",
                ChunkType::Info => "Info",
            })
            .collect::<Vec<_>>(),
    );
    let chunk_idxs: UInt32Array = chunks.iter().map(|c| c.idx).collect();
    let texts = StringArray::from(chunks.iter().map(|c| c.text.as_str()).collect::<Vec<_>>());

    // Build FixedSizeList<f16, 512> from f32 vectors.
    let flat_f16: Vec<f16> = vectors
        .iter()
        .flat_map(|v| v.iter().map(|&x| f16::from_f32(x)))
        .collect();
    let values = Float16Array::from(flat_f16);
    let vector_array = FixedSizeListArray::try_new_from_values(values, VECTOR_DIM as i32)?;

    let batch = RecordBatch::try_new(
        table_schema(),
        vec![
            Arc::new(ids),
            Arc::new(vector_array),
            Arc::new(heading_paths),
            Arc::new(source_urls),
            Arc::new(api_versions),
            Arc::new(chunk_types),
            Arc::new(chunk_idxs),
            Arc::new(texts),
        ],
    )?;
    Ok(batch)
}
