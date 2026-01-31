// Copyright 2025 Canton Wallet SDK Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Transaction estimator

use canton_ledger_api::proto::Commands;
use tracing::debug;
use crate::error::{TransactionError, TransactionResult};

/// Transaction estimator
#[derive(Debug, Clone)]
pub struct TransactionEstimator {
    base_cost: u64,
    cost_per_command: u64,
    cost_per_byte: u64,
}

impl Default for TransactionEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionEstimator {
    pub fn new() -> Self {
        Self {
            base_cost: 1000,
            cost_per_command: 100,
            cost_per_byte: 1,
        }
    }

    pub fn with_base_cost(mut self, cost: u64) -> Self {
        self.base_cost = cost;
        self
    }

    pub fn with_cost_per_command(mut self, cost: u64) -> Self {
        self.cost_per_command = cost;
        self
    }

    pub fn with_cost_per_byte(mut self, cost: u64) -> Self {
        self.cost_per_byte = cost;
        self
    }

    pub fn estimate(&self, transaction: &Commands) -> TransactionResult<u64> {
        debug!("Estimating transaction cost");

        let size = self.estimate_size(transaction)?;
        let command_count = transaction.commands.len() as u64;

        let total_cost = self.base_cost
            + (command_count * self.cost_per_command)
            + (size as u64 * self.cost_per_byte);

        debug!("Estimated transaction cost: {}", total_cost);
        Ok(total_cost)
    }

    pub fn estimate_size(&self, transaction: &Commands) -> TransactionResult<usize> {
        let size = serde_json::to_vec(transaction)
            .map_err(|e| TransactionError::SerializationError(e.to_string()))?
            .len();

        Ok(size)
    }

    pub fn estimate_gas(&self, transaction: &Commands) -> TransactionResult<u64> {
        self.estimate(transaction)
    }

    pub fn estimate_execution_time(&self, transaction: &Commands) -> TransactionResult<std::time::Duration> {
        let cost = self.estimate(transaction)?;
        let milliseconds = (cost as f64 / 1000.0) as u64;
        Ok(std::time::Duration::from_millis(milliseconds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canton_ledger_api::proto::{Command, CreateCommand, Identifier, DamlRecord};

    #[test]
    fn test_estimate_transaction() {
        let estimator = TransactionEstimator::new();
        let transaction = Commands {
            party: "party".to_string(),
            commands: vec![Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: DamlRecord::new(),
            })],
            act_as: vec!["party".to_string()],
            ..Default::default()
        };

        let result = estimator.estimate(&transaction);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_estimate_size() {
        let estimator = TransactionEstimator::new();
        let transaction = Commands {
            party: "party".to_string(),
            commands: vec![Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: DamlRecord::new(),
            })],
            act_as: vec!["party".to_string()],
            ..Default::default()
        };

        let result = estimator.estimate_size(&transaction);
        assert!(result.is_ok());
        assert!(result.unwrap() > 0);
    }

    #[test]
    fn test_estimate_execution_time() {
        let estimator = TransactionEstimator::new();
        let transaction = Commands {
            party: "party".to_string(),
            commands: vec![Command::Create(CreateCommand {
                template_id: Identifier {
                    package_id: "pkg".to_string(),
                    module_name: "mod".to_string(),
                    entity_name: "tpl".to_string(),
                },
                create_arguments: DamlRecord::new(),
            })],
            act_as: vec!["party".to_string()],
            ..Default::default()
        };

        let result = estimator.estimate_execution_time(&transaction);
        assert!(result.is_ok());
        assert!(result.unwrap().as_millis() > 0);
    }
}