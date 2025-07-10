//! AI Agent Builder Example
//!
//! This example demonstrates the exact JSON object syntax shown in the
//! cyrup_sugars README.md file. All syntax works exactly as documented.

use sugars_llm::*;
use sugars_macros::hash_map_fn;
use tracing::info;

// Helper trait for the example
trait ExecToText {
    fn exec_to_text(&self) -> String;
}

impl ExecToText for &str {
    fn exec_to_text(&self) -> String {
        format!("Output of: {}", self)
    }
}



fn process_turn() -> String {
    "Processed turn".to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("🤖 AI Agent Builder Example");
    println!("Demonstrating the exact JSON object syntax from README.md");
    println!();

    let stream = FluentAi::agent_role("rusty-squire")
    .completion_provider(Mistral::MAGISTRAL_SMALL)
    .temperature(1.0)
    .max_tokens(8000)
    .system_prompt("Act as a Rust developers 'right hand man'.
        You possess deep expertise in using tools to research rust, cargo doc and github libraries.
        You are a patient and thoughtful software artisan; a master of sequential thinking and step-by-step reasoning.
        You excel in compilation triage ...

        ...
        ...

        Today is {{ date }}

        ~ Be Useful, Not Thorough")
    .context(( // trait Context
        Context::<File>::of("/home/kloudsamurai/ai_docs/mistral_agents.pdf"),
        Context::<Files>::glob("/home/kloudsamurai/cyrup-ai/**/*.{md,txt}"),
        Context::<Directory>::of("/home/kloudsamurai/cyrup-ai/agent-role/ambient-rust"),
        Context::<Github>::glob("/home/kloudsamurai/cyrup-ai/**/*.{rs,md}")
    ))
    .mcp_server::<Stdio>().bin("/user/local/bin/sweetmcp").init("cargo run -- --stdio")
    .tools(( // trait Tool
        Tool::<Perplexity>::new(hash_map_fn!("citations" => "true")),
        Tool::named("cargo").bin("~/.cargo/bin").description("cargo --help".exec_to_text())
    )) // ZeroOneOrMany `Tool` || `McpTool` || NamedTool (WASM)

    .additional_params(hash_map_fn!("beta" => "true"))
    .memory(Library::named("obsidian_vault"))
    .metadata(hash_map_fn!("key" => "val", "foo" => "bar"))
    .on_tool_result(|results| {
        // do stuff
    })
    .on_conversation_turn(|conversation, agent| {
        info!("Agent: Last conversation turn");
        // your custom logic - return a processed message
        process_turn()
    })
    .on_chunk(|chunk| {          // unwrap chunk closure :: NOTE: THIS MUST PRECEDE .chat()
        match chunk {            // `.chat()` returns AsyncStream<MessageChunk> vs. AsyncStream<Result<MessageChunk>>
            Ok(chunk) => {
                println!("{}", chunk);   // stream response here or from the AsyncStream .chat() returns
                Ok(chunk)
            },
            Err(e) => Err(e)
        }
    })
    .into_agent() // Agent Now
    .conversation_history(MessageRole::User, "What time is it in Paris, France")
    .conversation_history(MessageRole::System, "The USER is inquiring about the time in Paris, France. Based on their IP address, I see they are currently in Las Vegas, Nevada, USA. The current local time is 16:45")
    .conversation_history(MessageRole::Assistant, "It's 1:45 AM CEST on July 7, 2025, in Paris, France. That's 9 hours ahead of your current time in Las Vegas.")
    .chat("Hello")?; // AsyncStream<MessageChunk
    
    println!("Chat stream initiated successfully!");

    Ok(())
}
