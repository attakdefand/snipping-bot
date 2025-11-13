pub mod contracts;
pub mod mev;
pub mod providers;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Test that modules can be imported
        let _provider_manager = providers::ProviderManager::new();
        let _mev_relay_manager = mev::MevRelayManager::new();
        let _contract_manager = contracts::ContractManager::new();
    }
}
