#[derive(Debug)]
pub struct Context {
    pub types: (),
    pub funcs: (),
    pub tables: (),
    pub mems: (),
    pub globals: (),
    pub locals: (),
    pub labels: (),
    pub r#return: (),
}

#[derive(Debug, Clone)]
pub enum ValidateError {}
