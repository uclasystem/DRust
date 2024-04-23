#[derive(Debug, Clone, Copy)]
pub struct GlobalEntry {
    pub key: usize,
    pub value: [u8; 32],
}