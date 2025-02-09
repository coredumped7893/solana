use {
    solana_program_runtime::compute_budget_processor::process_compute_budget_instructions,
    solana_sdk::{
        feature_set::FeatureSet,
        instruction::CompiledInstruction,
        pubkey::Pubkey,
        transaction::{SanitizedTransaction, SanitizedVersionedTransaction},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionPriorityDetails {
    pub priority: u64,
    pub compute_unit_limit: u64,
}

pub trait GetTransactionPriorityDetails {
    fn get_transaction_priority_details(
        &self,
        round_compute_unit_price_enabled: bool,
    ) -> Option<TransactionPriorityDetails>;

    fn process_compute_budget_instruction<'a>(
        instructions: impl Iterator<Item = (&'a Pubkey, &'a CompiledInstruction)>,
        _round_compute_unit_price_enabled: bool,
    ) -> Option<TransactionPriorityDetails> {
        let mut feature_set = FeatureSet::default();
        feature_set.activate(
            &solana_sdk::feature_set::add_set_tx_loaded_accounts_data_size_instruction::id(),
            0,
        );

        let compute_budget_limits =
            process_compute_budget_instructions(instructions, &feature_set).ok()?;
        Some(TransactionPriorityDetails {
            priority: compute_budget_limits.compute_unit_price,
            compute_unit_limit: u64::from(compute_budget_limits.compute_unit_limit),
        })
    }
}

impl GetTransactionPriorityDetails for SanitizedVersionedTransaction {
    fn get_transaction_priority_details(
        &self,
        round_compute_unit_price_enabled: bool,
    ) -> Option<TransactionPriorityDetails> {
        Self::process_compute_budget_instruction(
            self.get_message().program_instructions_iter(),
            round_compute_unit_price_enabled,
        )
    }
}

impl GetTransactionPriorityDetails for SanitizedTransaction {
    fn get_transaction_priority_details(
        &self,
        round_compute_unit_price_enabled: bool,
    ) -> Option<TransactionPriorityDetails> {
        Self::process_compute_budget_instruction(
            self.message().program_instructions_iter(),
            round_compute_unit_price_enabled,
        )
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        solana_sdk::{
            compute_budget::ComputeBudgetInstruction,
            message::Message,
            pubkey::Pubkey,
            signature::{Keypair, Signer},
            system_instruction,
            transaction::{Transaction, VersionedTransaction},
        },
    };

    #[test]
    fn test_get_priority_with_valid_request_heap_frame_tx() {
        let keypair = Keypair::new();
        let transaction = Transaction::new_unsigned(Message::new(
            &[
                system_instruction::transfer(&keypair.pubkey(), &Pubkey::new_unique(), 1),
                ComputeBudgetInstruction::request_heap_frame(32 * 1024),
            ],
            Some(&keypair.pubkey()),
        ));

        // assert for SanitizedVersionedTransaction
        let versioned_transaction = VersionedTransaction::from(transaction.clone());
        let sanitized_versioned_transaction =
            SanitizedVersionedTransaction::try_new(versioned_transaction).unwrap();
        assert_eq!(
            sanitized_versioned_transaction.get_transaction_priority_details(false),
            Some(TransactionPriorityDetails {
                priority: 0,
                compute_unit_limit:
                    solana_program_runtime::compute_budget_processor::DEFAULT_INSTRUCTION_COMPUTE_UNIT_LIMIT
                    as u64,
            })
        );

        // assert for SanitizedTransaction
        let sanitized_transaction =
            SanitizedTransaction::try_from_legacy_transaction(transaction).unwrap();
        assert_eq!(
            sanitized_transaction.get_transaction_priority_details(false),
            Some(TransactionPriorityDetails {
                priority: 0,
                compute_unit_limit:
                    solana_program_runtime::compute_budget_processor::DEFAULT_INSTRUCTION_COMPUTE_UNIT_LIMIT
                    as u64,
            })
        );
    }

    #[test]
    fn test_get_priority_with_valid_set_compute_units_limit() {
        let requested_cu = 101u32;
        let keypair = Keypair::new();
        let transaction = Transaction::new_unsigned(Message::new(
            &[
                system_instruction::transfer(&keypair.pubkey(), &Pubkey::new_unique(), 1),
                ComputeBudgetInstruction::set_compute_unit_limit(requested_cu),
            ],
            Some(&keypair.pubkey()),
        ));

        // assert for SanitizedVersionedTransaction
        let versioned_transaction = VersionedTransaction::from(transaction.clone());
        let sanitized_versioned_transaction =
            SanitizedVersionedTransaction::try_new(versioned_transaction).unwrap();
        assert_eq!(
            sanitized_versioned_transaction.get_transaction_priority_details(false),
            Some(TransactionPriorityDetails {
                priority: 0,
                compute_unit_limit: requested_cu as u64,
            })
        );

        // assert for SanitizedTransaction
        let sanitized_transaction =
            SanitizedTransaction::try_from_legacy_transaction(transaction).unwrap();
        assert_eq!(
            sanitized_transaction.get_transaction_priority_details(false),
            Some(TransactionPriorityDetails {
                priority: 0,
                compute_unit_limit: requested_cu as u64,
            })
        );
    }

    #[test]
    fn test_get_priority_with_valid_set_compute_unit_price() {
        let requested_price = 1_000;
        let keypair = Keypair::new();
        let transaction = Transaction::new_unsigned(Message::new(
            &[
                system_instruction::transfer(&keypair.pubkey(), &Pubkey::new_unique(), 1),
                ComputeBudgetInstruction::set_compute_unit_price(requested_price),
            ],
            Some(&keypair.pubkey()),
        ));

        // assert for SanitizedVersionedTransaction
        let versioned_transaction = VersionedTransaction::from(transaction.clone());
        let sanitized_versioned_transaction =
            SanitizedVersionedTransaction::try_new(versioned_transaction).unwrap();
        assert_eq!(
            sanitized_versioned_transaction.get_transaction_priority_details(false),
            Some(TransactionPriorityDetails {
                priority: requested_price,
                compute_unit_limit:
                    solana_program_runtime::compute_budget_processor::DEFAULT_INSTRUCTION_COMPUTE_UNIT_LIMIT
                    as u64,
            })
        );

        // assert for SanitizedTransaction
        let sanitized_transaction =
            SanitizedTransaction::try_from_legacy_transaction(transaction).unwrap();
        assert_eq!(
            sanitized_transaction.get_transaction_priority_details(false),
            Some(TransactionPriorityDetails {
                priority: requested_price,
                compute_unit_limit:
                    solana_program_runtime::compute_budget_processor::DEFAULT_INSTRUCTION_COMPUTE_UNIT_LIMIT
                    as u64,
            })
        );
    }
}
