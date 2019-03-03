pub struct SubCommand {
    pub name: String,
    pub summary: String,
    pub usage: String,
    pub help: String,
}

impl SubCommand {
    pub fn new(name: String, summary: String, usage: String, help: String) -> SubCommand {
        SubCommand {
            name,
            summary,
            usage,
            help,
        }
    }
}
