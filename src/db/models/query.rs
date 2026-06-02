#[derive(Clone, Copy, Debug, Default)]
pub struct QueryOptions {
    pub limit: Option<usize>,
    pub offset: usize,
}
