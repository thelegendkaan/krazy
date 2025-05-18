use beetl::completion::Prompt;
use beetl::providers;
use beetl_derive::beetl_tool;
use tracing_subscriber;

// Simple example with no attributes
#[beetl_tool]
fn add(a: i32, b: i32) -> Result<i32, beetl::tool::ToolError> {
    Ok(a + b)
}

#[beetl_tool]
fn subtract(a: i32, b: i32) -> Result<i32, beetl::tool::ToolError> {
    Ok(a - b)
}

#[beetl_tool]
fn multiply(a: i32, b: i32) -> Result<i32, beetl::tool::ToolError> {
    Ok(a * b)
}

#[beetl_tool]
fn divide(a: i32, b: i32) -> Result<i32, beetl::tool::ToolError> {
    if b == 0 {
        Err(beetl::tool::ToolError::ToolCallError(
            "Division by zero".into(),
        ))
    } else {
        Ok(a / b)
    }
}

#[beetl_tool]
fn answer_secret_question() -> Result<(bool, bool, bool, bool, bool), beetl::tool::ToolError> {
    Ok((false, false, true, false, false))
}

#[beetl_tool]
fn how_many_rs(s: String) -> Result<usize, beetl::tool::ToolError> {
    Ok(s.chars()
        .filter(|c| *c == 'r' || *c == 'R')
        .collect::<Vec<_>>()
        .len())
}

#[beetl_tool]
fn sum_numbers(numbers: Vec<i64>) -> Result<i64, beetl::tool::ToolError> {
    Ok(numbers.iter().sum())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().pretty().init();

    let calculator_agent = providers::openai::Client::from_env()
        .agent(providers::openai::GPT_4O)
        .preamble("You are an agent with tools access, always use the tools")
        .max_tokens(1024)
        .tool(Add)
        .build();

    for prompt in [
        "What tools do you have?",
        "Calculate 5 + 3",
        "What is 10 + 20?",
        "Add 100 and 200",
    ] {
        println!("User: {}", prompt);
        println!("Agent: {}", calculator_agent.prompt(prompt).await.unwrap());
    }
}
