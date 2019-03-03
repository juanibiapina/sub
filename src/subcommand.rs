pub struct SubCommand {
    pub name: String,
}

impl SubCommand {
    pub fn new(name: String) -> SubCommand {
        SubCommand {
            name,
        }
    }
}
