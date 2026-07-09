# In-Process LLM Embedding
## VectorVolt + EKOS — Native AI Inference Without External APIs

**Status:** Architecture Capture (Phase 5 — after core engine is stable)
**Captured:** 2026-07-09

---

## 1. The Problem with External LLM APIs

Today's standard approach for AI in databases:

```
Database → ETL pipeline → External LLM API → Embedding stored back → Vector index
```

This has fundamental problems for a sub-microsecond OLTP engine:
- **Network round-trip:** 50-500 ms per embedding call — catastrophic for write latency
- **External dependency:** OpenAI / Anthropic outage = no embeddings
- **Data privacy:** Source data leaves the deployment perimeter
- **Cost unpredictability:** Per-token billing adds variable overhead to every INSERT

VectorVolt solves this by embedding a **lightweight inference runtime directly inside the
process** — no network, no API key, no data leaving the node.

---

## 2. What "In-Process" Actually Means

The inference engine runs as a **separate thread pool** on **dedicated CPU cores**
(pinned, never competing with OLTP transaction threads):

```
CPU Cores 0-11:    OLTP transaction processing (VectorVolt)
CPU Cores 12-15:   In-process LLM inference (EKOS embedding engine)
CPU Cores 16-17:   Background tasks (pre-fetch, WAL shipping, ARP)
```

The LLM threads and the OLTP threads share the same process address space.
Embedding generation is a function call — no IPC, no serialization overhead.

```rust
// From the OLTP write path:
let embedding: Vector768 = embedding_engine.generate(text_value)?;
// ^ this is a local function call, not a network request
```

---

## 3. Model Selection — Small is Correct

A 70B parameter model does not belong inside a database engine. We use
**quantized small models** that fit in a few GB of RAM and run on CPU without GPU:

| Model | Parameters | Q4 RAM | Inference Latency | Use Case |
|---|---|---|---|---|
| Phi-3-mini | 3.8B | ~2 GB | ~100 ms / chunk | General text embeddings |
| DeepSeek-Coder-1.3B | 1.3B | ~1 GB | ~50 ms / chunk | Code/schema understanding |
| Qwen-2.5-1.5B | 1.5B | ~1 GB | ~60 ms / chunk | Multilingual support |
| Custom fine-tuned | ~1B | < 1 GB | ~40 ms / chunk | Domain-specific (Phase 6) |

**Default:** Phi-3-mini or Qwen-2.5-1.5B quantized to Q4_K_M via llama.cpp.

For a server with 256 GB RAM, reserving 2-4 GB for the inference engine is acceptable.
OLTP buffer pool retains > 250 GB.

**Inference framework:** `llama.cpp` via Rust bindings, or `ort` (ONNX Runtime) for
ONNX-format models. Both run fully on CPU with no GPU dependency.

---

## 4. VECTOR as a Native Column Type

VectorVolt introduces `VECTOR(n)` as a first-class column type alongside `INT`, `TEXT`, `JSON`.

```sql
-- Schema definition
CREATE TABLE products (
    id          BIGINT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    embedding   VECTOR(768)  -- 768-dimension embedding, stored natively
);
```

**Storage:** A `VECTOR(768)` column stores 768 × 4-byte floats = 3,072 bytes per row.
Stored in a dedicated column file within the Micro-Partition, separate from scalar columns.
This enables SIMD-accelerated cosine similarity without touching scalar data.

---

## 5. generate_embedding() — Built-in SQL Function

```sql
-- Auto-generate embedding during INSERT:
INSERT INTO products (id, name, description, embedding)
VALUES (1, 'Laptop', 'Thin and light', generate_embedding(description));

-- Auto-generate on UPDATE:
UPDATE products
SET embedding = generate_embedding(description)
WHERE id = 1;

-- The embedding is generated synchronously during the write path.
-- The transaction does not commit until the embedding is computed.
```

**Implementation:**

```rust
// Built-in SQL function registered in VectorVolt's function registry
pub fn generate_embedding(
    text: &str,
    engine: &EmbeddingEngine,
) -> Result<Vector768, EmbeddingError> {
    // Blocking call to in-process inference thread pool
    // No network, no serialization
    engine.embed(text)
}
```

**Write latency impact:** Embedding generation adds 50-100 ms per INSERT/UPDATE that
includes `generate_embedding()`. This is acceptable because:
1. Embedding generation is not expected on every row — only on semantically rich columns
2. The trade-off is explicit: the caller chose to generate an embedding
3. Future optimization: async embedding with deferred commit (Phase 6+)

**Async variant (future):**

```sql
-- INSERT commits immediately, embedding fills in asynchronously
INSERT INTO products (id, name, description)
VALUES (1, 'Laptop', 'Thin and light')
WITH DEFERRED_EMBEDDING (description → embedding);
```

---

## 6. Native Vector Index — HNSW

To support similarity search (`ORDER BY cosine_similarity(embedding, query_vec) DESC`),
VectorVolt maintains an **HNSW (Hierarchical Navigable Small World) index** for each
`VECTOR` column:

```sql
-- Similarity search:
SELECT id, name
FROM products
ORDER BY cosine_similarity(embedding, generate_embedding('portable gaming machine'))
LIMIT 10;
```

**HNSW properties:**
- Approximate nearest neighbor: sub-linear time, O(log n) per query
- Construction: incremental, each INSERT adds the new vector to the graph
- RAM: ~100 bytes per vector overhead for the graph structure

For 100 million rows: HNSW graph overhead ~10 GB. Acceptable on a 256 GB RAM node.

---

## 7. LoRA Fine-Tuning from WAL — The Self-Improving Embedding

This is the most distinctive capability: the model improves automatically from production
usage without any manual data pipeline.

**How it works:**

Every time a VectorVolt DBA or operator **manually fixes a slow/incorrect query result**
(e.g., by adding a missing index, correcting a query plan, or approving an ARP patch),
EKOS records the event as a training pair:

```rust
pub struct TrainingPair {
    pub input_context:  String,   // the query / operation description
    pub old_embedding:  Vector768, // what the model produced before
    pub correct_output: String,   // what the correct behavior was
    pub timestamp:      u64,
}
```

These pairs accumulate in a training queue. At a configurable schedule (e.g., 2 AM nightly,
when OLTP load is low), EKOS runs **LoRA fine-tuning** on the current embedding model:

```
Training Pairs (last 30 days)
         ↓
LoRA adapter computation (running on inference cores)
         ↓
New adapter weights (.safetensors, ~50 MB)
         ↓
Shadowbox("qa"): test embedding quality on held-out validation set
         ↓
Confidence score ≥ 95%?
         ↓ YES
New adapter merged into active model
         ↓
Old adapter archived as KO (rollback available)
```

**Why LoRA and not full fine-tuning:**
- LoRA updates only a small rank-decomposition of the weight matrices (~1% of parameters)
- Training on CPU takes hours, not days
- Adapter is tiny (~50 MB) — swappable without reloading the full model
- Previous adapter preserved for instant rollback if quality degrades

After 6 months of production, the embedding model understands your domain-specific
vocabulary (e.g., part numbers, medical codes, legal terms) significantly better than the
base model.

---

## 8. Resource Isolation — The Critical Rule

The inference engine **must never compete with OLTP transaction threads**.

```toml
# ekosd.toml
[embedding]
enabled          = true
model_path       = "/opt/ekos/models/phi3-mini-q4.gguf"
cpu_cores        = [12, 13, 14, 15]      # dedicated core group
ram_budget_gb    = 3                      # hard ceiling
thread_pool_size = 4
max_queue_depth  = 100                   # back-pressure if queue fills
timeout_ms       = 500                   # fail fast if overloaded

[embedding.fine_tune]
enabled          = true
schedule         = "0 2 * * *"           # 2 AM nightly (cron)
training_window_days = 30
min_training_pairs   = 50                # don't fine-tune on < 50 examples
lora_rank        = 16                    # adapter rank
```

**cgroups enforcement:** The embedding crate registers its threads with a Linux cgroup
that limits CPU and memory. Even if inference runs slow, OLTP transactions are unaffected.

---

## 9. Use Cases

| Use Case | Column Type | Query Pattern |
|---|---|---|
| Product catalog search | VECTOR(768) on description | Similarity search by natural language |
| Parts catalog (OEM/collision) | VECTOR(768) on part description | "Find parts similar to X" |
| Knowledge Object retrieval | VECTOR(1024) on KO payload | EKOS internal: find relevant KOs |
| Error classification (ARP) | VECTOR(768) on error message | Map new errors to known root causes |
| Log anomaly detection | VECTOR(512) on log line | Cluster unexpected log patterns |
| Document similarity | VECTOR(1536) on document chunks | Legal / compliance document matching |

---

## 10. The "AI-Native OLTP" Positioning

Most AI-augmented databases separate concerns:

```
OLTP Database    ←→    Vector Database    ←→    LLM API
(Postgres)             (pgvector / Pinecone)    (OpenAI)
```

VectorVolt collapses this into one:

```
VectorVolt
  ├── OLTP transactions (< 200 ns)
  ├── Native VECTOR columns (HNSW index)
  ├── generate_embedding() built-in function
  └── In-process LLM (no API, no network, no data leaves)
```

This is not "AI features added to a database." This is an OLTP engine where AI inference
is a primitive, as fundamental as `SUM()` or `COUNT()`.

---

## 11. Crate Addition

```
ekos/crates/ekos-embedding/
  src/
    engine.rs      ← EmbeddingEngine: llama.cpp or ONNX Runtime wrapper
    vector.rs      ← Vector768 type, HNSW index integration
    fine_tune.rs   ← LoRA adapter computation, training pair accumulation
    resource.rs    ← cgroup registration, CPU affinity, memory budget
  Cargo.toml

vectorvolt/crates/vectorvolt-vector/
  src/
    column.rs      ← VECTOR(n) column type, storage layout
    index.rs       ← HNSW index construction and query
    functions.rs   ← generate_embedding() SQL function registration
  Cargo.toml
```

---

## 12. Phase Placement

```
Phase 2:   VECTOR(n) column type in storage engine
           HNSW index construction and cosine_similarity() function
           (No embedding generation yet — caller provides pre-computed vectors)

Phase 3:   ekos-embedding crate: llama.cpp integration
           generate_embedding() SQL function (synchronous)
           CPU affinity + cgroup resource isolation

Phase 4:   Async embedding with deferred commit
           Training pair accumulation from production

Phase 5:   LoRA fine-tuning pipeline
           Shadowbox validation of new adapter
           Admin panel: embedding quality metrics, fine-tune history

Phase 6:   Custom domain model training (from scratch, Phase 7+)
           Multi-model support (different models per table/use case)
```

---

*The model learns from your data, runs in your process, never leaves your server.*
*AI inference is not an integration. It is a primitive.*
