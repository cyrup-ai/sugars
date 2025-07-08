//────────────────────────────────────────────────────────────────────────────
// examples/ai_agent_builder.rs – JSON Object Syntax Demo
//────────────────────────────────────────────────────────────────────────────

use std::marker::PhantomData;

// Provider enum
#[derive(Debug, Clone)]
pub enum Provider {
    OpenAI,
    Anthropic, 
    Mistral,
}

// Tool struct
#[derive(Debug, Clone)]
pub struct Tool {
    name: String,
    config: Vec<(String, String)>,
}

// Agent struct - what we're building
pub struct Agent {
    name: String,
    providers: Vec<Provider>,
    tools: Vec<Tool>,
    config: Vec<(String, String)>,
    metadata: Vec<(String, String)>,
}

// Typestate markers
pub struct NoName;
pub struct HasName;

pub struct AgentBuilder<N> {
    name: String,
    providers: Vec<Provider>,
    tools: Vec<Tool>,
    config: Vec<(String, String)>,
    metadata: Vec<(String, String)>,
    _phantom: PhantomData<N>,
}

impl AgentBuilder<NoName> {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            providers: Vec::new(),
            tools: Vec::new(),
            config: Vec::new(),
            metadata: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<N> AgentBuilder<N> {
    pub fn name(self, name: impl Into<String>) -> AgentBuilder<HasName> {
        AgentBuilder {
            name: name.into(),
            providers: self.providers,
            tools: self.tools,
            config: self.config,
            metadata: self.metadata,
            _phantom: PhantomData,
        }
    }

    // Clean variadic API
    pub fn providers(mut self, first: Provider, rest: Provider) -> Self {
        self.providers.push(first);
        self.providers.push(rest);
        self
    }

    pub fn provider(mut self, provider: Provider) -> Self {
        self.providers.push(provider);
        self
    }
}

impl AgentBuilder<HasName> {
    pub fn build(self) -> Agent {
        Agent {
            name: self.name,
            providers: self.providers,
            tools: self.tools,
            config: self.config,
            metadata: self.metadata,
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Builder Pattern Example\n");

    // Clean builder usage
    let agent = AgentBuilder::new()
        .name("research-assistant")
        .providers(Provider::OpenAI, Provider::Anthropic)
        .provider(Provider::Mistral)
        .build();

    println!("Built agent: {}", agent.name);
    println!("Providers: {}", agent.providers.len());

    println!("\n✅ Example completed");
}