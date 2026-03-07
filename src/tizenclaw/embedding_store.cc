#include "embedding_store.hh"
#include "../common/logging.hh"

#include <cmath>
#include <cstring>
#include <algorithm>
#include <numeric>
#include <sstream>

namespace tizenclaw {

EmbeddingStore::~EmbeddingStore() {
  Close();
}

bool EmbeddingStore::Initialize(
    const std::string& db_path) {
  if (db_) {
    Close();
  }

  int rc = sqlite3_open(db_path.c_str(), &db_);
  if (rc != SQLITE_OK) {
    LOG(ERROR) << "Failed to open SQLite DB: "
               << db_path << " — "
               << sqlite3_errmsg(db_);
    db_ = nullptr;
    return false;
  }

  // Enable WAL mode for concurrent readers
  sqlite3_exec(db_, "PRAGMA journal_mode=WAL;",
               nullptr, nullptr, nullptr);

  if (!CreateTable()) {
    Close();
    return false;
  }

  LOG(INFO) << "EmbeddingStore initialized: "
            << db_path;
  return true;
}

void EmbeddingStore::Close() {
  if (db_) {
    sqlite3_close(db_);
    db_ = nullptr;
  }
}

bool EmbeddingStore::CreateTable() {
  const char* sql =
      "CREATE TABLE IF NOT EXISTS documents ("
      "  id INTEGER PRIMARY KEY AUTOINCREMENT,"
      "  source TEXT NOT NULL,"
      "  chunk_text TEXT NOT NULL,"
      "  embedding BLOB NOT NULL,"
      "  created_at TEXT DEFAULT "
      "    (datetime('now'))"
      ");";

  char* err = nullptr;
  int rc = sqlite3_exec(
      db_, sql, nullptr, nullptr, &err);
  if (rc != SQLITE_OK) {
    LOG(ERROR) << "Failed to create table: "
               << (err ? err : "unknown");
    sqlite3_free(err);
    return false;
  }
  return true;
}

bool EmbeddingStore::StoreChunk(
    const std::string& source,
    const std::string& chunk_text,
    const std::vector<float>& embedding) {
  if (!db_) return false;

  const char* sql =
      "INSERT INTO documents "
      "(source, chunk_text, embedding) "
      "VALUES (?, ?, ?);";

  sqlite3_stmt* stmt = nullptr;
  int rc = sqlite3_prepare_v2(
      db_, sql, -1, &stmt, nullptr);
  if (rc != SQLITE_OK) {
    LOG(ERROR) << "Prepare failed: "
               << sqlite3_errmsg(db_);
    return false;
  }

  sqlite3_bind_text(
      stmt, 1, source.c_str(),
      static_cast<int>(source.size()),
      SQLITE_TRANSIENT);
  sqlite3_bind_text(
      stmt, 2, chunk_text.c_str(),
      static_cast<int>(chunk_text.size()),
      SQLITE_TRANSIENT);

  auto blob = FloatsToBlob(embedding);
  sqlite3_bind_blob(
      stmt, 3, blob.data(),
      static_cast<int>(blob.size()),
      SQLITE_TRANSIENT);

  rc = sqlite3_step(stmt);
  sqlite3_finalize(stmt);

  if (rc != SQLITE_DONE) {
    LOG(ERROR) << "Insert failed: "
               << sqlite3_errmsg(db_);
    return false;
  }
  return true;
}

std::vector<EmbeddingStore::SearchResult>
EmbeddingStore::Search(
    const std::vector<float>& query_embedding,
    int top_k) const {
  std::vector<SearchResult> results;
  if (!db_ || query_embedding.empty()) {
    return results;
  }

  const char* sql =
      "SELECT source, chunk_text, embedding "
      "FROM documents;";

  sqlite3_stmt* stmt = nullptr;
  int rc = sqlite3_prepare_v2(
      db_, sql, -1, &stmt, nullptr);
  if (rc != SQLITE_OK) {
    return results;
  }

  // Collect all results with scores
  std::vector<SearchResult> all;
  while (sqlite3_step(stmt) == SQLITE_ROW) {
    SearchResult r;
    const char* src =
        reinterpret_cast<const char*>(
            sqlite3_column_text(stmt, 0));
    const char* txt =
        reinterpret_cast<const char*>(
            sqlite3_column_text(stmt, 1));
    r.source = src ? src : "";
    r.chunk_text = txt ? txt : "";

    const void* blob_data =
        sqlite3_column_blob(stmt, 2);
    int blob_size =
        sqlite3_column_bytes(stmt, 2);
    auto emb = BlobToFloats(
        blob_data, blob_size);

    r.score = CosineSimilarity(
        query_embedding, emb);
    all.push_back(std::move(r));
  }
  sqlite3_finalize(stmt);

  // Sort by descending score
  std::sort(all.begin(), all.end(),
      [](const SearchResult& a,
         const SearchResult& b) {
        return a.score > b.score;
      });

  // Return top_k
  int count = std::min(
      top_k, static_cast<int>(all.size()));
  results.assign(
      all.begin(), all.begin() + count);
  return results;
}

bool EmbeddingStore::DeleteSource(
    const std::string& source) {
  if (!db_) return false;

  const char* sql =
      "DELETE FROM documents WHERE source = ?;";

  sqlite3_stmt* stmt = nullptr;
  int rc = sqlite3_prepare_v2(
      db_, sql, -1, &stmt, nullptr);
  if (rc != SQLITE_OK) return false;

  sqlite3_bind_text(
      stmt, 1, source.c_str(),
      static_cast<int>(source.size()),
      SQLITE_TRANSIENT);

  rc = sqlite3_step(stmt);
  sqlite3_finalize(stmt);
  return rc == SQLITE_DONE;
}

int EmbeddingStore::GetChunkCount() const {
  if (!db_) return 0;

  const char* sql =
      "SELECT COUNT(*) FROM documents;";
  sqlite3_stmt* stmt = nullptr;
  int rc = sqlite3_prepare_v2(
      db_, sql, -1, &stmt, nullptr);
  if (rc != SQLITE_OK) return 0;

  int count = 0;
  if (sqlite3_step(stmt) == SQLITE_ROW) {
    count = sqlite3_column_int(stmt, 0);
  }
  sqlite3_finalize(stmt);
  return count;
}

// --- Text chunking ---

std::vector<std::string>
EmbeddingStore::ChunkText(
    const std::string& text,
    size_t chunk_size,
    size_t overlap) {
  std::vector<std::string> chunks;
  if (text.empty() || chunk_size == 0) {
    return chunks;
  }

  size_t pos = 0;
  while (pos < text.size()) {
    size_t end = std::min(
        pos + chunk_size, text.size());

    // Try to break at a sentence boundary
    if (end < text.size()) {
      size_t last_period =
          text.rfind('.', end);
      if (last_period != std::string::npos &&
          last_period > pos + chunk_size / 2) {
        end = last_period + 1;
      }
    }

    chunks.push_back(
        text.substr(pos, end - pos));

    if (end >= text.size()) break;

    // Next chunk starts with overlap
    pos = (end > overlap) ?
        end - overlap : end;
  }
  return chunks;
}

// --- Cosine similarity ---

float EmbeddingStore::CosineSimilarity(
    const std::vector<float>& a,
    const std::vector<float>& b) {
  if (a.size() != b.size() || a.empty()) {
    return 0.0f;
  }

  float dot = 0.0f;
  float norm_a = 0.0f;
  float norm_b = 0.0f;

  for (size_t i = 0; i < a.size(); ++i) {
    dot += a[i] * b[i];
    norm_a += a[i] * a[i];
    norm_b += b[i] * b[i];
  }

  float denom = std::sqrt(norm_a) *
                std::sqrt(norm_b);
  if (denom < 1e-10f) return 0.0f;

  return dot / denom;
}

// --- BLOB <-> float conversion ---

std::vector<uint8_t>
EmbeddingStore::FloatsToBlob(
    const std::vector<float>& v) {
  std::vector<uint8_t> blob(
      v.size() * sizeof(float));
  std::memcpy(blob.data(), v.data(),
              blob.size());
  return blob;
}

std::vector<float>
EmbeddingStore::BlobToFloats(
    const void* data, int size) {
  if (!data || size <= 0) return {};
  size_t count =
      static_cast<size_t>(size) / sizeof(float);
  std::vector<float> v(count);
  std::memcpy(v.data(), data,
              count * sizeof(float));
  return v;
}

}  // namespace tizenclaw
