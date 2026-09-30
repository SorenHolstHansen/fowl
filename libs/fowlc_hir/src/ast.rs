use fowlc_interner::InternedStr;

pub struct FunctionDefinition {
    pub name: InternedStr,
}

pub enum Node {
    FunctionDefinition(FunctionDefinition),
}
