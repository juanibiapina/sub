pub struct SubCommand {
    pub name: String,
    pub summary: String,
    pub help: String,
}

impl SubCommand {
    pub fn new(name: String, summary: String, help: String) -> SubCommand {
        SubCommand {
            name,
            summary,
            help,
        }
    }
}
