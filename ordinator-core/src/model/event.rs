#[derive(Debug)]
pub enum Event {
    Branched,
    Exited,
    NotFound,
    Reset,
    Unbranched,
}
