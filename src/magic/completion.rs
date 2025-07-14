#[derive(Debug, PartialEq, Clone)]
pub enum CompletionType {
    Script,
    LiteralCommand(String),
}

#[derive(Debug, Clone)]
pub struct CompletionInfo {
    pub provides_completions: bool,
    pub completion_types: std::collections::HashMap<String, CompletionType>,
}

impl CompletionInfo {
    pub fn new() -> Self {
        Self {
            provides_completions: false,
            completion_types: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_completions(completion_types: std::collections::HashMap<String, CompletionType>) -> Self {
        Self {
            provides_completions: !completion_types.is_empty(),
            completion_types,
        }
    }
    
    pub fn get_completion_type(&self, name: &str) -> Option<&CompletionType> {
        self.completion_types.get(name)
    }
}