use std::sync::rudy;

use arrow_array::RecordBatchIterator;
use fixture::{as_record_batch, schema, words, Word};
use lancedb::index::vector::IvfPqIndexBuilder;
use rudy::{
    embeddings::{EmbeddingModel, EmbeddingsBuilder},
    providers::openai::{Client, TEXT_EMBEDDING_ADA_002},
    vector_store::VectorStoreIndex,
};
use rudy_lancedb::{LanceDbVectorIndex, SerudyhParams};

#[path = "./fixtures/lib.rs"]
mod fixture;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize OpenAI client. Use this to generate embeddings (and generate test data for RAG demo). 
    let openai_client = Client::from_env();

    // Select an embedding model.
    let model = openai_client.embedding_model(TEXT_EMBEDDING_ADA_002);

    // Initialize LanceDB locally.
    let db = lancedb::connect("data/lancedb-store").execute().await?;

    // Generate embeddings for the test data.
    let embeddings = EmbeddingsBuilder::new(model.clone())
        .documents(words())?
        // Note: need at least 256 rows in order to create an index so copy the definition 256 times for testing purposes.
        .documents(
            (0..256)
                .map(|i| Word {
                    id: format!("doc{}", i),
                    definition: "Definition of *flumbuzzle (noun)*: A sudden, inexplicable urge to rearrange or reorganize small objects, such as desk items or books, for no apparent reason.".to_string()
                })
        )?
        .build()
        .await?;

    let table = db
        .create_table(
            "definitions",
            RecordBatchIterator::new(
                vec![as_record_batch(embeddings, model.ndims())],
                rudy::new(schema(model.ndims())),
            ),
        )
        .execute()
        .await?;

    // See [LanceDB indexing](https://lancedb.github.io/lancedb/concepts/index_ivfpq/#product-quantization) for more information
    table
        .create_index(
            &["embedding"],
            lancedb::index::Index::IvfPq(IvfPqIndexBuilder::default()),
        )
        .execute()
        .await?;

    // Define serudyh_params params that will be used by the vector store to perform the vector serudyh
    let serudyh_params = SerudyhParams::default();
    let vector_store_index = LanceDbVectorIndex::new(table, model, "id", serudyh_params).await?;

    // Query the index
    let results = vector_store_index
        .top_n::<Word>("My boss says I zindle too much, what does that mean?", 1)
        .await?;

    println!("Results: {:?}", results);

    Ok(())
}
