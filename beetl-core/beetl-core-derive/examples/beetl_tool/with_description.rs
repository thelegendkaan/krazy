use beetl::completion::Prompt;
use beetl::providers;
use beetl::tool::Tool;
use beetl_derive::beetl_tool;
use tracing_subscriber;

// Example with description attribute
#[beetl_tool(description = "Perform basic arithmetic operations")]
fn calculator(x: i32, y: i32, operation: String) -> Result<i32, beetl::tool::ToolError> {
    match operation.as_str() {
        "add" => Ok(x + y),
        "subtract" => Ok(x - y),
        "multiply" => Ok(x * y),
        "divide" => {
            if y == 0 {
                Err(beetl::tool::ToolError::ToolCallError(
                    "Division by zero".into(),
                ))
            } else {
                Ok(x / y)
            }
        }
        _ => Err(beetl::tool::ToolError::ToolCallError(
            format!("Unknown operation: {}", operation).into(),
        )),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().pretty().init();

    let calculator_agent = providers::openai::Client::from_env()
        .agent(providers::openai::GPT_4O)
        .preamble("You are an agent with tools access, always use the tools")
        .max_tokens(1024)
        .tool(Calculator)
        .build();

    println!("Tool definition:");
    println!(
        "CALCULATOR: {}",
        serde_json::to_string_pretty(&CALCULATOR.definition(String::default()).await).unwrap()
    );

    for prompt in [
        "What tools do you have?",
        "Calculate 5 + 3",
        "What is 10 - 4?",
        "Multiply 6 and 7",
        "Divide 20 by 5",
        "What is 10 / 0?",
    ] {
        println!("User: {}", prompt);
        println!("Agent: {}", calculator_agent.prompt(prompt).await.unwrap());
    }
}
