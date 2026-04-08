# ADR F-008 — Futuro: Elasticsearch (Búsqueda Full-Text Avanzada)

| Campo | Valor |
|-------|-------|
| **Estado** | 🔮 Futuro — activar cuando SQLite LIKE no sea suficiente |
| **Fecha** | 2026 |
| **Autores** | Milton Hipamo / Laboratorio 3030 |
| **Relacionado con** | ADR 0004 (SQLite), ADR F-002 (PostgreSQL), ADR 0031 (Nivel 2+) |

---

## Contexto

Este ADR documenta la implementación de **Elasticsearch** (o OpenSearch) cuando las
capacidades de búsqueda de SQLite (LIKE, FTS5) ya no sean suficientes para las necesidades
de búsqueda full-text avanzada del producto.

**Activar cuando:**
- Búsquedas complejas: fuzzy matching, autocompletado en tiempo real, búsqueda semántica
- Volumen de documentos: >1M documentos indexados
- Requisitos de búsqueda: facetas, agregaciones, highlight de resultados
- Necesidad de análisis de texto: stemming, sinónimos, stop words

**NO activar en MVP.** SQLite FTS5 es suficiente hasta ~100k documentos.

---

## Decisión futura

Usar **Elasticsearch** o **OpenSearch** como motor de búsqueda dedicado.

### Arquitectura

```
                    Usuario busca "machine learning"
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                      Elasticsearch Cluster                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │  Node 1     │  │  Node 2     │  │  Node 3     │          │
│  │  (master)   │  │  (data)     │  │  (data)     │          │
│  │             │  │             │  │             │          │
│  │  Index:     │  │  Index:     │  │  Index:     │          │
│  │  documents  │  │  documents  │  │  documents  │          │
│  │  ├ shard 0  │  │  ├ shard 1  │  │  ├ shard 2  │          │
│  │  └ replica  │  │  └ replica  │  │  └ replica  │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└──────────────────────────────┬────────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│                      API (Axum)                             │
│                                                              │
│  /api/v1/search?q=machine+learning&f=category&sort=relevance │
│                                                              │
│  1. Parsear query (query DSL)                                │
│  2. Enviar a Elasticsearch                                    │
│  3. Enriquecer resultados (DB para metadatos)                │
│  4. Retornar JSON con highlights, facets, suggestions        │
└──────────────────────────────────────────────────────────────┘
```

### Sync DB → Elasticsearch

```
SQLite/PostgreSQL (source of truth)
       │
       ├──► Cambio en documents table
       │
       ▼
┌──────────────┐
│  CDC /       │  Change Data Capture
│  Debezium    │  (o polling periódico)
└──────┬───────┘
       │
       ▼
┌──────────────┐
│  Indexer     │  Transformar a documento ES
│  (Rust)      │  y enviar bulk index
└──────┬───────┘
       │
       ▼
Elasticsearch (search index)
```

---

## Cuándo activar

| Criterio | Umbral |
|----------|--------|
| Volumen | >1M documentos para indexar |
| Complejidad | Necesidad de fuzzy search, autocompletado, facets |
| Performance | Queries >500ms en SQLite FTS5 |
| Features | Highlight, aggregations, geo-search, ML ranking |
| Presupuesto | $100+/mes para ES managed justificable |

---

## Implementación

### 1. Elasticsearch Cluster (Docker)

```yaml
# infra/docker/compose.elasticsearch.yml
version: '3.8'
services:
  es-master:
    image: elasticsearch:8.12.0
    environment:
      - cluster.name=boilerplate-search
      - node.name=es-master
      - node.roles=master
      - discovery.seed_hosts=es-data1,es-data2
      - cluster.initial_master_nodes=es-master
      - bootstrap.memory_lock=true
      - "ES_JAVA_OPTS=-Xms512m -Xmx512m"
      - xpack.security.enabled=false
    ulimits:
      memlock:
        soft: -1
        hard: -1
    volumes:
      - es-master-data:/usr/share/elasticsearch/data
    ports:
      - "9200:9200"
    restart: unless-stopped

  es-data1:
    image: elasticsearch:8.12.0
    environment:
      - cluster.name=boilerplate-search
      - node.name=es-data1
      - node.roles=data,ingest
      - discovery.seed_hosts=es-master
      - bootstrap.memory_lock=true
      - "ES_JAVA_OPTS=-Xms1g -Xmx1g"
      - xpack.security.enabled=false
    ulimits:
      memlock:
        soft: -1
        hard: -1
    volumes:
      - es-data1-data:/usr/share/elasticsearch/data
    restart: unless-stopped

  es-data2:
    image: elasticsearch:8.12.0
    environment:
      - cluster.name=boilerplate-search
      - node.name=es-data2
      - node.roles=data,ingest
      - discovery.seed_hosts=es-master
      - bootstrap.memory_lock=true
      - "ES_JAVA_OPTS=-Xms1g -Xmx1g"
      - xpack.security.enabled=false
    ulimits:
      memlock:
        soft: -1
        hard: -1
    volumes:
      - es-data2-data:/usr/share/elasticsearch/data
    restart: unless-stopped

  kibana:
    image: kibana:8.12.0
    environment:
      - ELASTICSEARCH_HOSTS=http://es-master:9200
    ports:
      - "5601:5601"
    depends_on:
      - es-master
    restart: unless-stopped

volumes:
  es-master-data:
  es-data1-data:
  es-data2-data:
```

### 2. Cliente Elasticsearch en Rust

```toml
# Cargo.toml
elasticsearch = "8.12"
serde_json = "1.0"
```

```rust
// crates/search/src/elasticsearch.rs
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::{json, Value};

pub struct SearchEngine {
    client: Elasticsearch,
}

impl SearchEngine {
    pub fn new(base_url: &str) -> Result<Self, SearchError> {
        let transport = elasticsearch::http::transport::TransportBuilder::new(
            elasticsearch::http::transport::SingleNodeConnectionPool::new(
                base_url.parse()?
            )
        ).build()?;
        
        let client = Elasticsearch::new(transport);
        Ok(Self { client })
    }
    
    pub async fn index_document<T: serde::Serialize>(
        &self,
        index: &str,
        id: &str,
        document: &T,
    ) -> Result<(), SearchError> {
        self.client
            .index(elasticsearch::IndexParts::IndexId(index, id))
            .body(document)
            .send()
            .await?;
        
        Ok(())
    }
    
    pub async fn search(
        &self,
        index: &str,
        query: &str,
        filters: Vec<(&str, &str)>,
    ) -> Result<SearchResults, SearchError> {
        let mut must_clauses = vec![
            json!({"multi_match": {
                "query": query,
                "fields": ["title^3", "content", "tags"],
                "fuzziness": "AUTO"
            }})
        ];
        
        // Añadir filtros
        for (field, value) in filters {
            must_clauses.push(json!({"term": {field: value}}));
        }
        
        let search_body = json!({
            "query": {
                "bool": {
                    "must": must_clauses
                }
            },
            "highlight": {
                "fields": {
                    "title": {},
                    "content": {"fragment_size": 150}
                }
            },
            "aggs": {
                "by_category": {
                    "terms": {"field": "category"}
                }
            },
            "size": 20
        });
        
        let response = self.client
            .search(SearchParts::Index(&[index]))
            .body(search_body)
            .send()
            .await?;
        
        let response_body: Value = response.json().await?;
        
        // Parsear resultados
        let hits = response_body["hits"]["hits"]
            .as_array()
            .map(|arr| {
                arr.iter().map(|hit| SearchHit {
                    id: hit["_id"].as_str().unwrap_or("").to_string(),
                    score: hit["_score"].as_f64().unwrap_or(0.0),
                    source: hit["_source"].clone(),
                    highlights: hit["highlight"].clone(),
                }).collect()
            }).unwrap_or_default();
        
        let facets = response_body["aggregations"]["by_category"]["buckets"]
            .as_array()
            .map(|arr| {
                arr.iter().map(|bucket| Facet {
                    key: bucket["key"].as_str().unwrap_or("").to_string(),
                    count: bucket["doc_count"].as_u64().unwrap_or(0),
                }).collect()
            }).unwrap_or_default();
        
        Ok(SearchResults { hits, facets, total: hits.len() })
    }
    
    pub async fn suggest(
        &self,
        index: &str,
        prefix: &str,
    ) -> Result<Vec<String>, SearchError> {
        let suggest_body = json!({
            "suggest": {
                "title-suggest": {
                    "prefix": prefix,
                    "completion": {
                        "field": "suggest"
                    }
                }
            }
        });
        
        let response = self.client
            .search(SearchParts::Index(&[index]))
            .body(suggest_body)
            .send()
            .await?;
        
        let body: Value = response.json().await?;
        let suggestions = body["suggest"]["title-suggest"][0]["options"]
            .as_array()
            .map(|opts| {
                opts.iter()
                    .map(|opt| opt["text"].as_str().unwrap_or("").to_string())
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(suggestions)
    }
}
```

### 3. Index Mapping

```rust
// Setup del índice con mappings
pub async fn setup_index(&self) -> Result<(), SearchError> {
    let mapping = json!({
        "mappings": {
            "properties": {
                "title": {
                    "type": "text",
                    "analyzer": "standard",
                    "fields": {
                        "keyword": {"type": "keyword"},
                        "suggest": {
                            "type": "completion"
                        }
                    }
                },
                "content": {
                    "type": "text",
                    "analyzer": "standard"
                },
                "tags": {
                    "type": "keyword"
                },
                "category": {
                    "type": "keyword"
                },
                "created_at": {
                    "type": "date"
                },
                "suggest": {
                    "type": "completion"
                }
            }
        },
        "settings": {
            "number_of_shards": 3,
            "number_of_replicas": 1,
            "analysis": {
                "analyzer": {
                    "custom_analyzer": {
                        "type": "custom",
                        "tokenizer": "standard",
                        "filter": ["lowercase", "asciifolding", "synonym_filter"]
                    }
                },
                "filter": {
                    "synonym_filter": {
                        "type": "synonym",
                        "synonyms": [
                            "machine learning, ml, ai",
                            "database, db, datastore"
                        ]
                    }
                }
            }
        }
    });
    
    self.client
        .indices()
        .create(elasticsearch::indices::IndicesCreateParts::Index("documents"))
        .body(mapping)
        .send()
        .await?;
    
    Ok(())
}
```

### 4. Indexer desde SQLite

```rust
// crates/search/src/indexer.rs
pub async fn sync_documents_from_db(
    db_pool: &SqlitePool,
    search: &SearchEngine,
) -> Result<(), IndexerError> {
    let documents = sqlx::query_as::<_, Document>(
        "SELECT id, title, content, tags, category, created_at FROM documents WHERE updated_at > $1"
    )
    .bind(last_sync)
    .fetch_all(db_pool)
    .await?;
    
    // Bulk index
    let mut body: Vec<Value> = Vec::new();
    
    for doc in documents {
        // Action metadata
        body.push(json!({"index": {"_index": "documents", "_id": doc.id}}));
        
        // Document
        body.push(json!({
            "title": doc.title,
            "content": doc.content,
            "tags": doc.tags,
            "category": doc.category,
            "created_at": doc.created_at,
            "suggest": {
                "input": [doc.title.clone()],
                "weight": 1
            }
        }));
    }
    
    // Enviar bulk
    search.client
        .bulk(elasticsearch::BulkParts::None)
        .body(body)
        .send()
        .await?;
    
    Ok(())
}
```

---

## Alternativas

| Motor | Pros | Contras | Cuándo usar |
|-------|------|---------|-------------|
| **SQLite FTS5** | Cero setup, en proceso | Limitado, no distribuido | MVP, <100k docs |
| **PostgreSQL tsvector** | SQL nativo, transaccional | Menos features que ES | <1M docs, pre joins |
| **Meilisearch** | Simple, rápido, Rust | Menos maduro que ES | <10M docs, self-hosted |
| **Typesense** | Open source, rápido | Menos features | <10M docs, faceted search |
| **Elasticsearch** | Full features, maduro | Complejo, JVM, costoso | >1M docs, ML ranking |
| **OpenSearch** | Fork libre de ES | Comunidad menor | Quieres evitar licencia Elastic |

---

## Consecuencias

### ✅ Positivas

- **Búsqueda avanzada:** Fuzzy matching, autocompletado, facets, aggregations
- **Escalabilidad:** Sharding automático, réplicas, distribución
- **Análisis de texto:** Stemming, sinónimos, stop words, n-grams
- **Geo-search:** Búsqueda por proximidad geográfica
- **ML ranking:** Learn to Rank para relevancia personalizada
- **Realtime:** Indexación casi instantánea (refresh interval)

### ⚠️ Negativas / Trade-offs

- **Complejidad:** 3+ nodos (master + data), clustering, shards, replicas
- **Recursos:** JVM con 1-2GB heap mínimo por nodo
- **Costo:** $100-500/mes para cluster managed (Elastic Cloud)
- **Sincronización:** Necesidad de sync DB → ES (eventual consistency)
- **Dual write:** Escrituras deben ir a DB y ES (o CDC)
- **Backup:** Snapshots de ES además de backups de DB

### Decisiones derivadas

- Mantener SQLite/PostgreSQL como source of truth
- ES es solo índice de búsqueda — nunca storage primario
- Sync con CDC (Debezium) o polling periódico
- Bulk indexing para no saturar ES con writes individuales

---

## Estado actual

**No implementar.** Mantener SQLite FTS5 o PostgreSQL tsvector hasta que:
1. Se necesite fuzzy search, autocompletado, o facets
2. Volumen exceda 1M documentos
3. Queries de búsqueda superen 500ms consistentemente

**Path recomendado:**
1. Fase 1-2: SQLite FTS5 (suficiente)
2. Fase 3: Meilisearch (simple, Rust, self-hosted)
3. Fase 4+: Elasticsearch (features avanzadas, ML)

Ver ADR 0004 para SQLite y ADR F-002 para PostgreSQL.
