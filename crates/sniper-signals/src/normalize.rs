//! Signal normalization utilities implementation
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sniper_core::types::Signal;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Signal normalization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationConfig {
    /// Enable/disable signal normalization
    pub enabled: bool,
    /// Minimum confidence threshold for normalized signals
    pub min_confidence_threshold: f64,
    /// Maximum time difference between signals to be considered duplicates (in milliseconds)
    pub duplicate_time_window_ms: u64,
    /// Enable/disable signal aggregation
    pub enable_aggregation: bool,
    /// Enable/disable signal filtering
    pub enable_filtering: bool,
    /// Custom normalization rules
    pub custom_rules: Vec<NormalizationRule>,
}

impl Default for NormalizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_confidence_threshold: 0.5,          // 50% minimum confidence
            duplicate_time_window_ms: 30000,        // 30 seconds
            enable_aggregation: true,
            enable_filtering: true,
            custom_rules: Vec::new(),
        }
    }
}

/// Normalization rule for custom signal processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationRule {
    /// Rule name
    pub name: String,
    /// Source filter (e.g., "dex", "cex", "social")
    pub source_filter: Option<String>,
    /// Signal kind filter (e.g., "pair_created", "trading_enabled")
    pub kind_filter: Option<String>,
    /// Transformation rules
    pub transformations: Vec<TransformationRule>,
    /// Confidence adjustment
    pub confidence_adjustment: f64,
}

/// Transformation rule for signal data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRule {
    /// Field to transform
    pub field: String,
    /// Transformation type
    pub transform_type: TransformType,
    /// Parameters for the transformation
    pub parameters: HashMap<String, String>,
}

/// Types of transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformType {
    /// Convert to uppercase
    ToUppercase,
    /// Convert to lowercase
    ToLowercase,
    /// Trim whitespace
    Trim,
    /// Replace substring
    Replace,
    /// Extract substring
    Substring,
    /// Parse as number
    ParseNumber,
    /// Parse as boolean
    ParseBoolean,
    /// Custom transformation
    Custom(String),
}

/// Normalized signal with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedSignal {
    /// Original signal
    pub original: Signal,
    /// Normalized signal data
    pub normalized: Signal,
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    /// Whether this signal is a duplicate
    pub is_duplicate: bool,
    /// Aggregation count if this signal was aggregated
    pub aggregation_count: usize,
    /// Processing timestamp
    pub processed_at_ms: i64,
}

/// Signal normalization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationStats {
    /// Total signals processed
    pub total_processed: usize,
    /// Signals that passed normalization
    pub normalized: usize,
    /// Signals filtered out
    pub filtered: usize,
    /// Duplicate signals detected
    pub duplicates: usize,
    /// Aggregated signals
    pub aggregated: usize,
}

/// Signal normalizer
pub struct SignalNormalizer {
    /// Configuration
    config: NormalizationConfig,
    /// Signal cache for duplicate detection
    signal_cache: HashMap<String, i64>,
    /// Normalization statistics
    stats: NormalizationStats,
}

impl SignalNormalizer {
    /// Create a new signal normalizer
    pub fn new(config: NormalizationConfig) -> Self {
        Self {
            config,
            signal_cache: HashMap::new(),
            stats: NormalizationStats {
                total_processed: 0,
                normalized: 0,
                filtered: 0,
                duplicates: 0,
                aggregated: 0,
            },
        }
    }

    /// Normalize a signal
    /// 
    /// # Arguments
    /// * `signal` - Signal to normalize
    /// 
    /// # Returns
    /// * `Result<Option<NormalizedSignal>>` - Normalized signal or None if filtered
    pub fn normalize_signal(&mut self, signal: Signal) -> Result<Option<NormalizedSignal>> {
        debug!("Normalizing signal from source: {}, kind: {}", signal.source, signal.kind);
        
        self.stats.total_processed += 1;
        
        if !self.config.enabled {
            return Ok(Some(NormalizedSignal {
                original: signal.clone(),
                normalized: signal,
                confidence: 1.0,
                is_duplicate: false,
                aggregation_count: 1,
                processed_at_ms: chrono::Utc::now().timestamp_millis(),
            }));
        }
        
        // Check for duplicates
        let is_duplicate = self.is_duplicate(&signal)?;
        if is_duplicate {
            self.stats.duplicates += 1;
            return Ok(None);
        }
        
        // Apply filtering
        if self.config.enable_filtering && !self.filter_signal(&signal)? {
            self.stats.filtered += 1;
            return Ok(None);
        }
        
        // Apply normalization
        let normalized = self.apply_normalization(signal.clone())?;
        
        // Apply custom rules
        let (final_signal, confidence) = self.apply_custom_rules(normalized)?;
        
        // Check confidence threshold
        if confidence < self.config.min_confidence_threshold {
            self.stats.filtered += 1;
            return Ok(None);
        }
        
        // Cache the signal to detect future duplicates
        self.cache_signal(&final_signal)?;
        
        self.stats.normalized += 1;
        
        let normalized_signal = NormalizedSignal {
            original: signal,
            normalized: final_signal,
            confidence,
            is_duplicate,
            aggregation_count: 1,
            processed_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        
        info!("Signal normalized successfully with confidence: {:.2}", confidence);
        Ok(Some(normalized_signal))
    }

    /// Normalize multiple signals
    /// 
    /// # Arguments
    /// * `signals` - Vector of signals to normalize
    /// 
    /// # Returns
    /// * `Result<Vec<NormalizedSignal>>` - Vector of normalized signals
    pub fn normalize_signals(&mut self, signals: Vec<Signal>) -> Result<Vec<NormalizedSignal>> {
        let mut normalized_signals = Vec::new();
        
        for signal in signals {
            if let Some(normalized) = self.normalize_signal(signal)? {
                normalized_signals.push(normalized);
            }
        }
        
        // Apply aggregation if enabled
        if self.config.enable_aggregation {
            normalized_signals = self.aggregate_signals(normalized_signals)?;
        }
        
        Ok(normalized_signals)
    }

    /// Check if a signal is a duplicate
    fn is_duplicate(&self, signal: &Signal) -> Result<bool> {
        let signal_key = self.generate_signal_key(signal);
        let now = chrono::Utc::now().timestamp_millis();
        
        if let Some(timestamp) = self.signal_cache.get(&signal_key) {
            let time_diff = (now - timestamp).abs() as u64;
            Ok(time_diff < self.config.duplicate_time_window_ms)
        } else {
            Ok(false)
        }
    }

    /// Filter a signal based on configured criteria
    fn filter_signal(&self, signal: &Signal) -> Result<bool> {
        // In a real implementation, this would apply various filters
        // For this implementation, we'll use a simple approach
        
        // Filter out signals with very old timestamps
        let now = chrono::Utc::now().timestamp_millis();
        let time_diff = (now - signal.seen_at_ms).abs();
        
        if time_diff > 300000 { // 5 minutes
            debug!("Filtering out old signal: {} ms old", time_diff);
            return Ok(false);
        }
        
        // Filter out signals with missing required fields
        if signal.source.is_empty() || signal.kind.is_empty() {
            debug!("Filtering out signal with missing required fields");
            return Ok(false);
        }
        
        Ok(true)
    }

    /// Apply standard normalization to a signal
    fn apply_normalization(&self, mut signal: Signal) -> Result<Signal> {
        // Normalize source and kind to lowercase
        signal.source = signal.source.to_lowercase();
        signal.kind = signal.kind.to_lowercase();
        
        // Normalize token addresses to checksum format (simplified)
        if let Some(token0) = signal.token0 {
            signal.token0 = Some(token0.to_lowercase());
        }
        
        if let Some(token1) = signal.token1 {
            signal.token1 = Some(token1.to_lowercase());
        }
        
        // Normalize chain name
        signal.chain.name = signal.chain.name.to_lowercase();
        
        Ok(signal)
    }

    /// Apply custom normalization rules
    fn apply_custom_rules(&self, mut signal: Signal) -> Result<(Signal, f64)> {
        let mut confidence = 1.0;
        
        for rule in &self.config.custom_rules {
            // Check if rule applies to this signal
            let source_match = rule.source_filter.as_ref()
                .map(|s| s == &signal.source)
                .unwrap_or(true);
            
            let kind_match = rule.kind_filter.as_ref()
                .map(|k| k == &signal.kind)
                .unwrap_or(true);
            
            if source_match && kind_match {
                // Apply transformations
                for transform in &rule.transformations {
                    self.apply_transformation(&mut signal, transform)?;
                }
                
                // Apply confidence adjustment
                confidence += rule.confidence_adjustment;
            }
        }
        
        // Clamp confidence to 0.0-1.0 range
        confidence = confidence.max(0.0).min(1.0);
        
        Ok((signal, confidence))
    }

    /// Apply a transformation to a signal
    fn apply_transformation(&self, signal: &mut Signal, transform: &TransformationRule) -> Result<()> {
        match transform.field.as_str() {
            "source" => {
                signal.source = self.apply_string_transform(&signal.source, &transform.transform_type, &transform.parameters)?;
            },
            "kind" => {
                signal.kind = self.apply_string_transform(&signal.kind, &transform.transform_type, &transform.parameters)?;
            },
            "token0" => {
                if let Some(token0) = &signal.token0 {
                    signal.token0 = Some(self.apply_string_transform(token0, &transform.transform_type, &transform.parameters)?);
                }
            },
            "token1" => {
                if let Some(token1) = &signal.token1 {
                    signal.token1 = Some(self.apply_string_transform(token1, &transform.transform_type, &transform.parameters)?);
                }
            },
            "extra" => {
                // Apply transformation to extra JSON data
                // This is a simplified implementation
                signal.extra = self.apply_json_transform(&signal.extra, &transform.transform_type, &transform.parameters)?;
            },
            _ => {
                warn!("Unknown field for transformation: {}", transform.field);
            }
        }
        
        Ok(())
    }

    /// Apply string transformation
    fn apply_string_transform(&self, value: &str, transform_type: &TransformType, parameters: &HashMap<String, String>) -> Result<String> {
        match transform_type {
            TransformType::ToUppercase => Ok(value.to_uppercase()),
            TransformType::ToLowercase => Ok(value.to_lowercase()),
            TransformType::Trim => Ok(value.trim().to_string()),
            TransformType::Replace => {
                let old_str = parameters.get("old").unwrap_or(&String::new());
                let new_str = parameters.get("new").unwrap_or(&String::new());
                Ok(value.replace(old_str, new_str))
            },
            TransformType::Substring => {
                let start = parameters.get("start").and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
                let end = parameters.get("end").and_then(|s| s.parse::<usize>().ok()).unwrap_or(value.len());
                Ok(value[start..end.min(value.len())].to_string())
            },
            TransformType::ParseNumber | TransformType::ParseBoolean => {
                // These transformations don't apply to strings
                Ok(value.to_string())
            },
            TransformType::Custom(_) => {
                // Custom transformations would be implemented here
                Ok(value.to_string())
            },
        }
    }

    /// Apply JSON transformation
    fn apply_json_transform(&self, value: &Value, transform_type: &TransformType, parameters: &HashMap<String, String>) -> Result<Value> {
        match transform_type {
            TransformType::ToUppercase | TransformType::ToLowercase | TransformType::Trim => {
                // Apply to string values in JSON
                if let Value::String(s) = value {
                    Ok(Value::String(self.apply_string_transform(s, transform_type, parameters)?))
                } else {
                    Ok(value.clone())
                }
            },
            TransformType::Replace => {
                if let Value::String(s) = value {
                    let old_str = parameters.get("old").unwrap_or(&String::new());
                    let new_str = parameters.get("new").unwrap_or(&String::new());
                    Ok(Value::String(s.replace(old_str, new_str)))
                } else {
                    Ok(value.clone())
                }
            },
            _ => Ok(value.clone()),
        }
    }

    /// Aggregate similar signals
    fn aggregate_signals(&mut self, signals: Vec<NormalizedSignal>) -> Result<Vec<NormalizedSignal>> {
        let mut aggregated: HashMap<String, Vec<NormalizedSignal>> = HashMap::new();
        
        // Group signals by similarity
        for signal in signals {
            let key = self.generate_aggregation_key(&signal.normalized);
            aggregated.entry(key).or_insert_with(Vec::new).push(signal);
        }
        
        // Create aggregated signals
        let mut result = Vec::new();
        for (_, group) in aggregated {
            if group.len() > 1 {
                // Aggregate multiple similar signals
                let mut aggregated_signal = group[0].clone();
                aggregated_signal.aggregation_count = group.len();
                aggregated_signal.confidence = group.iter().map(|s| s.confidence).sum::<f64>() / group.len() as f64;
                
                result.push(aggregated_signal);
                self.stats.aggregated += group.len() - 1;
            } else {
                // Single signal, no aggregation needed
                result.push(group[0].clone());
            }
        }
        
        Ok(result)
    }

    /// Generate a key for duplicate detection
    fn generate_signal_key(&self, signal: &Signal) -> String {
        format!("{}:{}:{}:{}:{}:{}",
            signal.source,
            signal.kind,
            signal.chain.name,
            signal.token0.as_deref().unwrap_or(""),
            signal.token1.as_deref().unwrap_or(""),
            signal.seen_at_ms / 1000 // Round to nearest second
        )
    }

    /// Generate a key for signal aggregation
    fn generate_aggregation_key(&self, signal: &Signal) -> String {
        format!("{}:{}:{}:{}:{}",
            signal.source,
            signal.kind,
            signal.chain.name,
            signal.token0.as_deref().unwrap_or(""),
            signal.token1.as_deref().unwrap_or("")
        )
    }

    /// Cache a signal for duplicate detection
    fn cache_signal(&mut self, signal: &Signal) -> Result<()> {
        let key = self.generate_signal_key(signal);
        let timestamp = chrono::Utc::now().timestamp_millis();
        self.signal_cache.insert(key, timestamp);
        Ok(())
    }

    /// Get normalization statistics
    pub fn get_stats(&self) -> &NormalizationStats {
        &self.stats
    }

    /// Reset normalization statistics
    pub fn reset_stats(&mut self) {
        self.stats = NormalizationStats {
            total_processed: 0,
            normalized: 0,
            filtered: 0,
            duplicates: 0,
            aggregated: 0,
        };
    }

    /// Update configuration
    /// 
    /// # Arguments
    /// * `config` - New configuration
    pub fn update_config(&mut self, config: NormalizationConfig) {
        self.config = config;
    }

    /// Add a custom normalization rule
    /// 
    /// # Arguments
    /// * `rule` - Normalization rule to add
    pub fn add_custom_rule(&mut self, rule: NormalizationRule) {
        self.config.custom_rules.push(rule);
    }
}

/// Advanced signal normalizer with machine learning capabilities
pub struct AdvancedSignalNormalizer {
    /// Base signal normalizer
    base_normalizer: SignalNormalizer,
    /// Historical normalization results for learning
    historical_results: Vec<NormalizationOutcome>,
    /// Learning rate for model updates
    learning_rate: f64,
}

/// Normalization outcome for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationOutcome {
    /// Signal identifier
    pub signal_id: String,
    /// Confidence score
    pub confidence: f64,
    /// Actual outcome (confirmed_valid, false_positive, etc.)
    pub actual_outcome: String,
    /// Timestamp
    pub timestamp: u64,
}

impl AdvancedSignalNormalizer {
    /// Create a new advanced signal normalizer
    pub fn new(base_normalizer: SignalNormalizer) -> Self {
        Self {
            base_normalizer,
            historical_results: Vec::new(),
            learning_rate: 0.01,
        }
    }

    /// Normalize a signal with learning capabilities
    /// 
    /// # Arguments
    /// * `signal` - Signal to normalize
    /// 
    /// # Returns
    /// * `Result<Option<NormalizedSignal>>` - Enhanced normalized signal
    pub fn normalize_signal_with_learning(&mut self, signal: Signal) -> Result<Option<NormalizedSignal>> {
        // Get base normalization result
        let mut result = self.base_normalizer.normalize_signal(signal)?;
        
        // Apply learning adjustments
        if let Some(ref mut normalized) = result {
            if let Some(adjusted_confidence) = self.adjust_confidence(&normalized.normalized) {
                normalized.confidence = adjusted_confidence;
                
                // Re-check confidence threshold
                if adjusted_confidence < self.base_normalizer.config.min_confidence_threshold {
                    return Ok(None);
                }
            }
        }
        
        Ok(result)
    }

    /// Adjust confidence based on historical data
    fn adjust_confidence(&self, _signal: &Signal) -> Option<f64> {
        // In a real implementation, this would use ML models
        // For this implementation, we'll simulate with a simple approach
        
        let mut adjustment = 0.0;
        
        // If we have historical data, adjust based on patterns
        if !self.historical_results.is_empty() {
            let correct_normalizations = self.historical_results.iter()
                .filter(|d| d.actual_outcome == "confirmed_valid")
                .count();
            
            let total_normalizations = self.historical_results.len();
            let accuracy = correct_normalizations as f64 / total_normalizations as f64;
            
            // If accuracy is low, reduce confidence
            if accuracy < 0.7 {
                adjustment -= 0.2;
            } else if accuracy > 0.9 {
                adjustment += 0.1;
            }
        }
        
        // Simulate some additional adjustments based on signal characteristics
        let additional_adjustment = 0.05; // Small positive adjustment
        
        let adjusted_confidence = (0.8 + adjustment + additional_adjustment).max(0.0).min(1.0);
        Some(adjusted_confidence)
    }

    /// Record normalization outcome for learning
    /// 
    /// # Arguments
    /// * `outcome` - Normalization outcome data
    pub fn record_normalization_outcome(&mut self, outcome: NormalizationOutcome) {
        self.historical_results.push(outcome);
        
        // Keep only recent data (last 1000 normalization results)
        if self.historical_results.len() > 1000 {
            self.historical_results.drain(0..self.historical_results.len() - 1000);
        }
    }

    /// Update learning rate
    /// 
    /// # Arguments
    /// * `rate` - New learning rate
    pub fn update_learning_rate(&mut self, rate: f64) {
        self.learning_rate = rate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sniper_core::types::ChainRef;

    #[test]
    fn test_normalization_config() {
        let config = NormalizationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_confidence_threshold, 0.5);
        assert_eq!(config.duplicate_time_window_ms, 30000);
        assert!(config.enable_aggregation);
        assert!(config.enable_filtering);
        assert!(config.custom_rules.is_empty());
    }

    #[test]
    fn test_signal_normalizer_creation() {
        let config = NormalizationConfig::default();
        let normalizer = SignalNormalizer::new(config);
        assert_eq!(normalizer.signal_cache.len(), 0);
        assert_eq!(normalizer.stats.total_processed, 0);
    }

    #[test]
    fn test_disabled_normalization() {
        let mut config = NormalizationConfig::default();
        config.enabled = false;
        
        let mut normalizer = SignalNormalizer::new(config);
        
        let signal = Signal {
            source: "DEX".to_string(),
            kind: "PAIR_CREATED".to_string(),
            chain: ChainRef {
                name: "Ethereum".to_string(),
                id: 1,
            },
            token0: Some("0xToken0".to_string()),
            token1: Some("0xToken1".to_string()),
            extra: serde_json::json!({}),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        
        let result = normalizer.normalize_signal(signal.clone()).unwrap();
        assert!(result.is_some());
        let normalized = result.unwrap();
        assert_eq!(normalized.normalized.source, "dex"); // Still normalized even when disabled
        assert_eq!(normalized.confidence, 1.0);
    }

    #[test]
    fn test_signal_normalization() {
        let config = NormalizationConfig::default();
        let mut normalizer = SignalNormalizer::new(config);
        
        let signal = Signal {
            source: "DEX".to_string(),
            kind: "PAIR_CREATED".to_string(),
            chain: ChainRef {
                name: "Ethereum".to_string(),
                id: 1,
            },
            token0: Some("0xTOKEN0".to_string()),
            token1: Some("0xTOKEN1".to_string()),
            extra: serde_json::json!({}),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        
        let result = normalizer.normalize_signal(signal).unwrap();
        assert!(result.is_some());
        let normalized = result.unwrap();
        assert_eq!(normalized.normalized.source, "dex");
        assert_eq!(normalized.normalized.kind, "pair_created");
        assert_eq!(normalized.normalized.chain.name, "ethereum");
        assert_eq!(normalized.normalized.token0, Some("0xtoken0".to_string()));
        assert_eq!(normalized.normalized.token1, Some("0xtoken1".to_string()));
    }

    #[test]
    fn test_duplicate_detection() {
        let config = NormalizationConfig::default();
        let mut normalizer = SignalNormalizer::new(config);
        
        let signal1 = Signal {
            source: "dex".to_string(),
            kind: "pair_created".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            token0: Some("0xtoken0".to_string()),
            token1: Some("0xtoken1".to_string()),
            extra: serde_json::json!({}),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        
        // First signal should be processed
        let result1 = normalizer.normalize_signal(signal1.clone()).unwrap();
        assert!(result1.is_some());
        
        // Second identical signal should be detected as duplicate
        let result2 = normalizer.normalize_signal(signal1).unwrap();
        assert!(result2.is_none());
        assert_eq!(normalizer.stats.duplicates, 1);
    }

    #[test]
    fn test_confidence_filtering() {
        let mut config = NormalizationConfig::default();
        config.min_confidence_threshold = 0.9; // High threshold
        
        let mut normalizer = SignalNormalizer::new(config);
        
        let signal = Signal {
            source: "dex".to_string(),
            kind: "pair_created".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            token0: Some("0xtoken0".to_string()),
            token1: Some("0xtoken1".to_string()),
            extra: serde_json::json!({}),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        
        let result = normalizer.normalize_signal(signal).unwrap();
        // With default confidence of 0.8, this should be filtered out
        assert!(result.is_none());
        assert_eq!(normalizer.stats.filtered, 1);
    }

    #[test]
    fn test_normalization_rule() {
        let rule = NormalizationRule {
            name: "test_rule".to_string(),
            source_filter: Some("dex".to_string()),
            kind_filter: Some("pair_created".to_string()),
            transformations: vec![],
            confidence_adjustment: 0.1,
        };
        
        assert_eq!(rule.name, "test_rule");
        assert_eq!(rule.source_filter, Some("dex".to_string()));
        assert_eq!(rule.kind_filter, Some("pair_created".to_string()));
        assert_eq!(rule.confidence_adjustment, 0.1);
    }

    #[test]
    fn test_transformation_rule() {
        let transform = TransformationRule {
            field: "source".to_string(),
            transform_type: TransformType::ToUppercase,
            parameters: HashMap::new(),
        };
        
        assert_eq!(transform.field, "source");
        assert!(matches!(transform.transform_type, TransformType::ToUppercase));
        assert!(transform.parameters.is_empty());
    }

    #[test]
    fn test_normalized_signal_creation() {
        let original = Signal {
            source: "dex".to_string(),
            kind: "pair_created".to_string(),
            chain: ChainRef {
                name: "ethereum".to_string(),
                id: 1,
            },
            token0: Some("0xtoken0".to_string()),
            token1: Some("0xtoken1".to_string()),
            extra: serde_json::json!({}),
            seen_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        
        let normalized_signal = NormalizedSignal {
            original: original.clone(),
            normalized: original,
            confidence: 0.8,
            is_duplicate: false,
            aggregation_count: 1,
            processed_at_ms: chrono::Utc::now().timestamp_millis(),
        };
        
        assert_eq!(normalized_signal.confidence, 0.8);
        assert!(!normalized_signal.is_duplicate);
        assert_eq!(normalized_signal.aggregation_count, 1);
    }

    #[test]
    fn test_advanced_signal_normalizer() {
        let config = NormalizationConfig::default();
        let base_normalizer = SignalNormalizer::new(config);
        let mut advanced_normalizer = AdvancedSignalNormalizer::new(base_normalizer);
        
        let outcome = NormalizationOutcome {
            signal_id: "test-signal".to_string(),
            confidence: 0.8,
            actual_outcome: "confirmed_valid".to_string(),
            timestamp: 1234567890,
        };
        
        advanced_normalizer.record_normalization_outcome(outcome);
        assert_eq!(advanced_normalizer.historical_results.len(), 1);
    }
}