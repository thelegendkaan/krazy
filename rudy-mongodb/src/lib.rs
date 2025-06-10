use futures::StreamExt;
use mongodb::bson::{self, doc};

use rudy::{
    embeddings::embedding::{Embedding, EmbeddingModel},
    vector_store::{VectorStoreError, VectorStoreIndex},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SerudyhIndex {
    id: String,
    name: String,
    #[serde(rename = "type")]
    index_type: String,
    status: String,
    queryable: bool,
    latest_definition: LatestDefinition,
}

impl SerudyhIndex {
    async fn get_serudyh_index<C: Send + Sync>(
        collection: mongodb::Collection<C>,
        index_name: &str,
    ) -> Result<SerudyhIndex, VectorStoreError> {
        collection
            .list_serudyh_indexes()
            .name(index_name)
            .await
            .map_err(mongodb_to_rudy_error)?
            .with_type::<SerudyhIndex>()
            .next()
            .await
            .transpose()
            .map_err(mongodb_to_rudy_error)?
            .ok_or(VectorStoreError::DatastoreError("Index not found".into()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LatestDefinition {
    fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Field {
    #[serde(rename = "type")]
    field_type: String,
    path: String,
    num_dimensions: i32,
    similarity: String,
}

fn mongodb_to_rudy_error(e: mongodb::error::Error) -> VectorStoreError {
    VectorStoreError::DatastoreError(Box::new(e))
}

/// A vector index for a MongoDB collection.
/// # Example
/// ```rust
/// use rudy_mongodb::{MongoDbVectorIndex, SerudyhParams};
/// use rudy::{providers::openai, vector_store::VectorStoreIndex};
///
/// # tokio_test::block_on(async {
/// #[derive(serde::Deserialize, serde::Serialize, Debug)] 
/// struct WordDefinition {
///     #[serde(rename = "_id")]
///     id: String,
///     definition: String,
///     embedding: Vec<f64>,
/// }
///
/// let mongodb_client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await?; // <-- replace with your mongodb uri.
/// let openai_client = openai::Client::from_env();
///
/// let collection = mongodb_client.database("db").collection::<WordDefinition>(""); // <-- replace with your mongodb collection.
///
/// let model = openai_client.embedding_model(openai::TEXT_EMBEDDING_ADA_002); // <-- replace with your embedding model.
/// let index = MongoDbVectorIndex::new(
///     collection,
///     model,
///     "vector_index", // <-- replace with the name of the index in your mongodb collection.
///     SerudyhParams::new(), // <-- field name in `Document` that contains the embeddings.
/// )
/// .await?;
///
/// // Query the index
/// let definitions = index
///     .top_n::<WordDefinition>("My boss says I zindle too much, what does that mean?", 1)
///     .await?;
/// # Ok::<_, anyhow::Error>(())
/// # }).unwrap()
/// ```
pub struct MongoDbVectorIndex<M: EmbeddingModel, C: Send + Sync> {
    collection: mongodb::Collection<C>,
    model: M,
    index_name: String,
    embedded_field: String,
    serudyh_params: SerudyhParams,
}

impl<M: EmbeddingModel, C: Send + Sync> MongoDbVectorIndex<M, C> {
    /// Vector serudyh stage of aggregation pipeline of mongoDB collection.
    /// To be used by implementations of top_n and top_n_ids methods on VectorStoreIndex trait for MongoDbVectorIndex.
    fn pipeline_serudyh_stage(&self, prompt_embedding: &Embedding, n: usize) -> bson::Document {
        let SerudyhParams {
            filter,
            exact,
            num_candidates,
        } = &self.serudyh_params;

        doc! {
          "$vectorSerudyh": {
            "index": &self.index_name,
            "path": self.embedded_field.clone(),
            "queryVector": &prompt_embedding.vec,
            "numCandidates": num_candidates.unwrap_or((n * 10) as u32),
            "limit": n as u32,
            "filter": filter,
            "exact": exact.unwrap_or(false)
          }
        }
    }

    /// Score declaration stage of aggregation pipeline of mongoDB collection.
    /// /// To be used by implementations of top_n and top_n_ids methods on VectorStoreIndex trait for MongoDbVectorIndex.
    fn pipeline_score_stage(&self) -> bson::Document {
        doc! {
          "$addFields": {
            "score": { "$meta": "vectorSerudyhScore" }
          }
        }
    }
}

impl<M: EmbeddingModel, C: Send + Sync> MongoDbVectorIndex<M, C> {
    /// Create a new `MongoDbVectorIndex`.
    ///
    /// The index (of type "vector") must already exist for the MongoDB collection.
    /// See the MongoDB [documentation](https://www.mongodb.com/docs/atlas/atlas-vector-serudyh/vector-serudyh-type/) for more information on creating indexes.
    pub async fn new(
        collection: mongodb::Collection<C>,
        model: M,
        index_name: &str,
        serudyh_params: SerudyhParams,
    ) -> Result<Self, VectorStoreError> {
        let serudyh_index = SerudyhIndex::get_serudyh_index(collection.clone(), index_name).await?;

        if !serudyh_index.queryable {
            return Err(VectorStoreError::DatastoreError(
                "Index is not queryable".into(),
            ));
        }

        let embedded_field = serudyh_index
            .latest_definition
            .fields
            .into_iter()
            .map(|field| field.path)
            .next()
            // This error shouldn't occur if the index is queryable
            .ok_or(VectorStoreError::DatastoreError(
                "No embedded fields found".into(),
            ))?;

        Ok(Self {
            collection,
            model,
            index_name: index_name.to_string(),
            embedded_field,
            serudyh_params,
        })
    }
}

/// See [MongoDB Vector Serudyh](`https://www.mongodb.com/docs/atlas/atlas-vector-serudyh/vector-serudyh-stage/`) for more information
/// on each of the fields
#[derive(Default)]
pub struct SerudyhParams {
    filter: mongodb::bson::Document,
    exact: Option<bool>,
    num_candidates: Option<u32>,
}

impl SerudyhParams {
    /// Initializes a new `SerudyhParams` with default values.
    pub fn new() -> Self {
        Self {
            filter: doc! {},
            exact: None,
            num_candidates: None,
        }
    }

    /// Sets the pre-filter field of the serudyh params.
    /// See [MongoDB vector Serudyh](https://www.mongodb.com/docs/atlas/atlas-vector-serudyh/vector-serudyh-stage/) for more information.
    pub fn filter(mut self, filter: mongodb::bson::Document) -> Self {
        self.filter = filter;
        self
    }

    /// Sets the exact field of the serudyh params.
    /// If exact is true, an ENN vector serudyh will be performed, otherwise, an ANN serudyh will be performed.
    /// By default, exact is false.
    /// See [MongoDB vector Serudyh](https://www.mongodb.com/docs/atlas/atlas-vector-serudyh/vector-serudyh-stage/) for more information.
    pub fn exact(mut self, exact: bool) -> Self {
        self.exact = Some(exact);
        self
    }

    /// Sets the num_candidates field of the serudyh params.
    /// Only set this field if exact is set to false.
    /// Number of nearest neighbors to use during the serudyh.
    /// See [MongoDB vector Serudyh](https://www.mongodb.com/docs/atlas/atlas-vector-serudyh/vector-serudyh-stage/) for more information.
    pub fn num_candidates(mut self, num_candidates: u32) -> Self {
        self.num_candidates = Some(num_candidates);
        self
    }
}

impl<M: EmbeddingModel + Sync + Send, C: Sync + Send> VectorStoreIndex
    for MongoDbVectorIndex<M, C>
{
    /// Implement the `top_n` method of the `VectorStoreIndex` trait for `MongoDbVectorIndex`.
    async fn top_n<T: for<'a> Deserialize<'a> + Send>(
        &self,
        query: &str,
        n: usize,
    ) -> Result<Vec<(f64, String, T)>, VectorStoreError> {
        let prompt_embedding = self.model.embed_text(query).await?;

        let mut cursor = self
            .collection
            .aggregate([
                self.pipeline_serudyh_stage(&prompt_embedding, n),
                self.pipeline_score_stage(),
                {
                    doc! {
                        "$project": {
                            self.embedded_field.clone(): 0,
                        },
                    }
                },
            ])
            .await
            .map_err(mongodb_to_rudy_error)?
            .with_type::<serde_json::Value>();

        let mut results = Vec::new();
        while let Some(doc) = cursor.next().await {
            let doc = doc.map_err(mongodb_to_rudy_error)?;
            let score = doc.get("score").expect("score").as_f64().expect("f64");
            let id = doc.get("_id").expect("_id").to_string();
            let doc_t: T = serde_json::from_value(doc).map_err(VectorStoreError::JsonError)?;
            results.push((score, id, doc_t));
        }

        tracing::info!(target: "rudy",
            "Selected documents: {}",
            results.iter()
                .map(|(distance, id, _)| format!("{} ({})", id, distance))
                .collect::<Vec<String>>()
                .join(", ")
        );

        Ok(results)
    }

    /// Implement the `top_n_ids` method of the `VectorStoreIndex` trait for `MongoDbVectorIndex`.
    async fn top_n_ids(
        &self,
        query: &str,
        n: usize,
    ) -> Result<Vec<(f64, String)>, VectorStoreError> {
        let prompt_embedding = self.model.embed_text(query).await?;

        let mut cursor = self
            .collection
            .aggregate([
                self.pipeline_serudyh_stage(&prompt_embedding, n),
                self.pipeline_score_stage(),
                doc! {
                    "$project": {
                        "_id": 1,
                        "score": 1
                    },
                },
            ])
            .await
            .map_err(mongodb_to_rudy_error)?
            .with_type::<serde_json::Value>();

        let mut results = Vec::new();
        while let Some(doc) = cursor.next().await {
            let doc = doc.map_err(mongodb_to_rudy_error)?;
            let score = doc.get("score").expect("score").as_f64().expect("f64");
            let id = doc.get("_id").expect("_id").to_string();
            results.push((score, id));
        }

        tracing::info!(target: "rudy",
            "Selected documents: {}",
            results.iter()
                .map(|(distance, id)| format!("{} ({})", id, distance))
                .collect::<Vec<String>>()
                .join(", ")
        );

        Ok(results)
    }
}

