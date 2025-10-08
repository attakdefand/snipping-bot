use crate::errors::SniperError;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct InMemoryBus {
    tx: broadcast::Sender<Vec<u8>>,
}

impl InMemoryBus {
    pub fn new(buffer: usize) -> Self {
        let (tx, _rx) = broadcast::channel(buffer);
        Self { tx }
    }
    pub async fn publish<T: serde::Serialize>(
        &self,
        _subject: &str,
        msg: &T,
    ) -> Result<(), SniperError> {
        let bytes = serde_json::to_vec(msg).map_err(|e| SniperError::Bus(e.to_string()))?;
        let _ = self.tx.send(bytes);
        Ok(())
    }
    pub fn subscribe(&self, _subject: &str) -> broadcast::Receiver<Vec<u8>> {
        self.tx.subscribe()
    }
}
