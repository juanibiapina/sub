#[derive(Debug, Clone)]
pub struct Docs {
    pub summary: Option<String>,
    pub usage: Option<String>,
    pub options: Vec<String>,
    pub description: Option<String>,
}

impl Docs {
    pub fn new() -> Self {
        Self {
            summary: None,
            usage: None,
            options: Vec::new(),
            description: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub provides_completions: bool,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            summary: None,
            description: None,
            provides_completions: false,
        }
    }
    
    pub fn from_docs(docs: &Docs, provides_completions: bool) -> Self {
        Self {
            summary: docs.summary.clone(),
            description: docs.description.clone(),
            provides_completions,
        }
    }
}