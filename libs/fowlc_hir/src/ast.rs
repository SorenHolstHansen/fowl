pub struct FunctionDefinition {
    pub name: String,
}

pub enum Node {
    FunctionDefinition(FunctionDefinition),
}
