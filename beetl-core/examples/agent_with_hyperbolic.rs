use std::env;

use beetl::{completion::Prompt, providers};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create OpenAI client
    let client = providers::hyperbolic::Client::new(
        &env::var("HYPERBOLIC_API_KEY").expect("HYPERBOLIC_API_KEY not set"),
    );

    // Create agent with a single context prompt
    let comedian_agent = client
        .agent(beetl::providers::hyperbolic::DEEPSEEK_R1)
        .preamble("You are a comedian here to entertain the user using humour and jokes.")
        .build();

    // Prompt the agent and print the response
    let response = comedian_agent.prompt("Entertain me!").await?;
    println!("{}", response);

    Ok(())
}
