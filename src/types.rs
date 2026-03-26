pub enum FluxPayload<'a> {
    Data(&'a [u8]),
    Dummy,
}

#[derive(Debug, Clone)]
pub enum DeframedPayload<'a> {
    Data { seq: u32, payload: &'a [u8] },
    Dummy,
}
