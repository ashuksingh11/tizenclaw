#include <gtest/gtest.h>

#include "embedding_store.hh"

#include <cstdio>
#include <cmath>

using namespace tizenclaw;

class EmbeddingStoreTest : public ::testing::Test {
protected:
  void SetUp() override {
    db_path_ = "/tmp/test_embeddings.db";
    std::remove(db_path_.c_str());
  }

  void TearDown() override {
    store_.Close();
    std::remove(db_path_.c_str());
  }

  EmbeddingStore store_;
  std::string db_path_;
};

TEST_F(EmbeddingStoreTest, InitializeAndClose) {
  EXPECT_TRUE(store_.Initialize(db_path_));
  EXPECT_EQ(store_.GetChunkCount(), 0);
  store_.Close();
}

TEST_F(EmbeddingStoreTest, StoreAndCount) {
  ASSERT_TRUE(store_.Initialize(db_path_));

  std::vector<float> emb = {
      0.1f, 0.2f, 0.3f, 0.4f};
  EXPECT_TRUE(
      store_.StoreChunk("test", "hello", emb));
  EXPECT_EQ(store_.GetChunkCount(), 1);

  EXPECT_TRUE(
      store_.StoreChunk("test", "world", emb));
  EXPECT_EQ(store_.GetChunkCount(), 2);
}

TEST_F(EmbeddingStoreTest, SearchTopK) {
  ASSERT_TRUE(store_.Initialize(db_path_));

  // Store 3 chunks with different embeddings
  std::vector<float> emb1 = {1, 0, 0, 0};
  std::vector<float> emb2 = {0, 1, 0, 0};
  std::vector<float> emb3 = {0.9f, 0.1f, 0, 0};

  store_.StoreChunk("doc1", "chunk1", emb1);
  store_.StoreChunk("doc2", "chunk2", emb2);
  store_.StoreChunk("doc3", "chunk3", emb3);

  // Search with query similar to emb1
  std::vector<float> query = {1, 0, 0, 0};
  auto results = store_.Search(query, 2);

  ASSERT_EQ(results.size(), 2u);
  // chunk1 should be first (exact match)
  EXPECT_EQ(results[0].chunk_text, "chunk1");
  EXPECT_NEAR(results[0].score, 1.0f, 0.01f);
  // chunk3 should be second (most similar)
  EXPECT_EQ(results[1].chunk_text, "chunk3");
}

TEST_F(EmbeddingStoreTest, DeleteSource) {
  ASSERT_TRUE(store_.Initialize(db_path_));

  std::vector<float> emb = {1, 0, 0};
  store_.StoreChunk("src1", "a", emb);
  store_.StoreChunk("src1", "b", emb);
  store_.StoreChunk("src2", "c", emb);
  EXPECT_EQ(store_.GetChunkCount(), 3);

  EXPECT_TRUE(store_.DeleteSource("src1"));
  EXPECT_EQ(store_.GetChunkCount(), 1);
}

TEST_F(EmbeddingStoreTest,
       CosineSimilarityIdentical) {
  std::vector<float> a = {1, 2, 3};
  float sim =
      EmbeddingStore::CosineSimilarity(a, a);
  EXPECT_NEAR(sim, 1.0f, 0.001f);
}

TEST_F(EmbeddingStoreTest,
       CosineSimilarityOrthogonal) {
  std::vector<float> a = {1, 0, 0};
  std::vector<float> b = {0, 1, 0};
  float sim =
      EmbeddingStore::CosineSimilarity(a, b);
  EXPECT_NEAR(sim, 0.0f, 0.001f);
}

TEST_F(EmbeddingStoreTest,
       CosineSimilarityDifferentSize) {
  std::vector<float> a = {1, 2};
  std::vector<float> b = {1, 2, 3};
  // Different sizes → 0
  EXPECT_NEAR(
      EmbeddingStore::CosineSimilarity(a, b),
      0.0f, 0.001f);
}

TEST_F(EmbeddingStoreTest, ChunkTextBasic) {
  std::string text =
      "Hello world. This is a test. End.";
  auto chunks =
      EmbeddingStore::ChunkText(text, 20, 5);
  EXPECT_GT(chunks.size(), 0u);
  // All text should be covered
  for (const auto& c : chunks) {
    EXPECT_FALSE(c.empty());
  }
}

TEST_F(EmbeddingStoreTest, ChunkTextEmpty) {
  auto chunks =
      EmbeddingStore::ChunkText("", 500, 50);
  EXPECT_EQ(chunks.size(), 0u);
}
