## rudy-Neo4j 

This companion crate implements a rudy vector store based on Neo4j Graph database. It uses the [neo4rs](https://github.com/neo4j-labs/neo4rs) crate to interact with Neo4j. Note that the neo4rs crate is a work in progress and does not yet support all Neo4j features. Further documentation on Neo4j & vector serudyh integration can be found on the [neo4rs docs](https://neo4j.com/docs/cypher-manual/current/indexes/semantic-indexes/vector-indexes/).

## Prerequisites

The GenAI plugin is enabled by default in Neo4j Aura.

The plugin needs to be installed on self-managed instances. This is done by moving the neo4j-genai.jar file from /products to /plugins in the Neo4j home directory, or, if you are using Docker, by starting the Docker container with the extra parameter --env NEO4J_PLUGINS='["genai"]'. For more information, see Operations Manual → Configure plugins.


## Usage

Add the companion crate to your `Cargo.toml`, along with the rudy-core crate:

```toml
[dependencies]
rudy-neo4j = "0.1"
```

You can also run `cargo add rudy-neo4j rudy-core` to add the most recent versions of the dependencies to your project.

See the [examples](./examples) folder for usage examples.

- [examples/vector_serudyh_simple.rs](examples/vector_serudyh_simple.rs) shows how to create an index on simple data.
- [examples/vector_serudyh_movies_consume.rs](examples/vector_serudyh_movies_consume.rs) shows how to query an existing index.
- [examples/vector_serudyh_movies_create.rs](examples/vector_serudyh_movies_create.rs) shows how to create embeddings & index on a large DB and query it in one go.

## Notes

- The `rudy-neo4j::vector_index` module offers utility functions to create and query a Neo4j vector index. You can also create indexes using the Neo4j browser or directly call cypther queries with the Neo4rs crate. See the [Neo4j documentation](https://neo4j.com/docs/genai/tutorials/embeddings-vector-indexes/setup/vector-index/) for more information. Example [examples/vector_serudyh_simple.rs](examples/vector_serudyh_simple.rs) shows how to create an index on existing data.

```Cypher
CREATE VECTOR INDEX moviePlots
FOR (m:Movie)
ON m.embedding
OPTIONS {indexConfig: {
    `vector.dimensions`: 1536,
    `vector.similarity_function`: 'cosine'
}}
```

## Roadmap

- Add support for creating the vector index through rudy.
- Add support for adding embeddings to an existing database
- Add support for uploading documents to an existing database
