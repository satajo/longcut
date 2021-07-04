#[derive(Debug)]
pub enum Event {
    Branched,
    Canceled,
    Exited,
    NotFound,
    Reset,
    Unbranched,
}
