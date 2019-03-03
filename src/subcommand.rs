pub struct SubCommand {
    pub name: String,
    pub summary: String,
}

impl SubCommand {
    pub fn new(name: String, summary: String) -> SubCommand {
        SubCommand {
            name,
            summary,
        }
    }
}
