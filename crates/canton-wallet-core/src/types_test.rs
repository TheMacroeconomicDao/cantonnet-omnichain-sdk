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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_id_creation() {
        let wallet_id = WalletId::new();
        assert!(!wallet_id.to_string().is_empty());
    }

    #[test]
    fn test_party_id_creation() {
        let party_id = PartyId::new_unchecked("test-party");
        assert_eq!(party_id.to_string(), "test-party");
    }

    #[test]
    fn test_contract_id_creation() {
        let contract_id = ContractId::new_unchecked("test-contract");
        assert_eq!(contract_id.to_string(), "test-contract");
    }

    #[test]
    fn test_identifier_creation() {
        let identifier = Identifier::new("pkg", "mod", "tpl");
        assert_eq!(identifier.package_name, "pkg");
        assert_eq!(identifier.module_name, "mod");
        assert_eq!(identifier.entity_name, "tpl");
    }

    #[test]
    fn test_daml_value_unit() {
        let value = DamlValue::unit();
        assert!(matches!(value, DamlValue::Unit));
    }

    #[test]
    fn test_daml_value_bool() {
        let value = DamlValue::bool(true);
        assert!(matches!(value, DamlValue::Bool(true)));
    }

    #[test]
    fn test_daml_value_int64() {
        let value = DamlValue::int64(42);
        assert!(matches!(value, DamlValue::Int64(42)));
    }

    #[test]
    fn test_daml_value_text() {
        let value = DamlValue::text("hello");
        assert!(matches!(value, DamlValue::Text(_)));
    }

    #[test]
    fn test_daml_record_creation() {
        let record = DamlRecord::new();
        assert!(record.fields.is_empty());
    }

    #[test]
    fn test_daml_record_with_field() {
        let mut record = DamlRecord::new();
        record.add_field("field1", DamlValue::text("value1"));
        assert_eq!(record.fields.len(), 1);
    }

    #[test]
    fn test_ledger_offset_begin() {
        let offset = LedgerOffset::Begin;
        assert!(matches!(offset, LedgerOffset::Begin));
    }

    #[test]
    fn test_ledger_offset_absolute() {
        let offset = LedgerOffset::Absolute("test-offset".to_string());
        assert!(matches!(offset, LedgerOffset::Absolute(_)));
    }

    #[test]
    fn test_transaction_filter_for_party() {
        let party_id = PartyId::new_unchecked("test-party");
        let filter = TransactionFilter::for_party(&party_id);
        assert!(matches!(filter, TransactionFilter::FiltersByParty(_)));
    }

    #[test]
    fn test_transaction_filter_for_template() {
        let party_id = PartyId::new_unchecked("test-party");
        let template_id = Identifier::new("pkg", "mod", "tpl");
        let filter = TransactionFilter::for_template(&party_id, template_id);
        assert!(matches!(filter, TransactionFilter::FiltersByTemplate(_)));
    }

    #[test]
    fn test_command_create() {
        let template_id = Identifier::new("pkg", "mod", "tpl");
        let arguments = DamlRecord::new();
        let command = Command::Create(CreateCommand {
            template_id,
            create_arguments: arguments,
        });
        assert!(matches!(command, Command::Create(_)));
    }

    #[test]
    fn test_command_exercise() {
        let command = Command::Exercise(ExerciseCommand {
            template_id: None,
            contract_id: "test-contract".to_string(),
            choice: "Choice".to_string(),
            choice_argument: DamlValue::unit().into(),
        });
        assert!(matches!(command, Command::Exercise(_)));
    }

    #[test]
    fn test_event_created() {
        let created_event = CreatedEvent {
            event_id: "test-event-id".to_string(),
            contract_id: "test-contract".to_string(),
            template_id: Identifier::new("pkg", "mod", "tpl"),
            create_arguments: DamlRecord::new(),
            witness_parties: vec!["party1".to_string()],
            signatories: vec!["party1".to_string()],
            agreement_text: None,
        };
        assert_eq!(created_event.contract_id, "test-contract");
    }

    #[test]
    fn test_event_archived() {
        let archived_event = ArchivedEvent {
            event_id: "test-event-id".to_string(),
            contract_id: "test-contract".to_string(),
            witness_parties: vec!["party1".to_string()],
        };
        assert_eq!(archived_event.contract_id, "test-contract");
    }

    #[test]
    fn test_transaction_creation() {
        let transaction = Transaction {
            transaction_id: "test-tx-id".to_string(),
            command_id: "test-cmd-id".to_string(),
            workflow_id: "test-workflow".to_string(),
            effective_at: None,
            ledger_offset: LedgerOffset::Begin,
            transaction_filter: TransactionFilter::default(),
            events: vec![],
            root_event_ids: vec![],
            offset: "test-offset".to_string(),
        };
        assert_eq!(transaction.transaction_id, "test-tx-id");
    }

    #[test]
    fn test_completion_success() {
        let completion = Completion {
            status: CompletionStatus::Success,
            transaction_id: "test-tx-id".to_string(),
            updates: vec![],
        };
        assert!(matches!(completion.status, CompletionStatus::Success));
    }

    #[test]
    fn test_completion_failure() {
        let completion = Completion {
            status: CompletionStatus::Failure,
            transaction_id: "test-tx-id".to_string(),
            updates: vec![],
        };
        assert!(matches!(completion.status, CompletionStatus::Failure));
    }
}
