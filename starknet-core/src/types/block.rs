use super::{
    super::serde::unsigned_field_element::{UfeHex, UfeHexOption},
    ConfirmedTransactionReceipt, FieldElement, TransactionType,
};

use serde::Deserialize;
use serde_with::serde_as;

pub enum BlockId {
    Hash(FieldElement),
    Number(u64),
    Pending,
    Latest,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub enum BlockStatus {
    /// Block that is yet to be closed
    Pending,
    /// Block failed in the L2 pipeline
    Aborted,
    /// A reverted block (rejected on L1)
    Reverted,
    /// Block that was created on L2, in contrast to Pending, which is not yet closed
    AcceptedOnL2,
    /// Accepted on L1
    AcceptedOnL1,
}

#[serde_as]
#[derive(Debug, Deserialize)]
#[cfg_attr(test, serde(deny_unknown_fields))]
pub struct Block {
    #[serde(default)]
    #[serde_as(as = "UfeHexOption")]
    pub block_hash: Option<FieldElement>,
    pub block_number: Option<u64>,
    #[serde_as(as = "UfeHex")]
    pub parent_block_hash: FieldElement,
    pub timestamp: u64,
    // Field marked optional as old blocks don't include it yet. Drop optional once resolved.
    #[serde(default)]
    #[serde_as(as = "UfeHexOption")]
    pub sequencer_address: Option<FieldElement>,
    #[serde(default)]
    #[serde_as(as = "UfeHexOption")]
    pub state_root: Option<FieldElement>,
    pub status: BlockStatus,
    #[serde_as(as = "UfeHex")]
    pub gas_price: FieldElement,
    pub transactions: Vec<TransactionType>,
    pub transaction_receipts: Vec<ConfirmedTransactionReceipt>,
}

#[cfg(test)]
mod tests {
    use super::super::transaction::EntryPointType;
    use super::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_block_deser_with_transactions() {
        let raw =
            include_str!("../../test-data/raw_gateway_responses/get_block/1_with_transactions.txt");

        let block: Block = serde_json::from_str(raw).unwrap();

        assert_eq!(block.block_number.unwrap(), 39232);
        assert_eq!(block.status, BlockStatus::AcceptedOnL1);
        assert_eq!(
            block.state_root.unwrap(),
            FieldElement::from_hex_be(
                "06cb132715b8687f1c1d79a7282975986fb0a9c166d64b384cfad965a602fe02"
            )
            .unwrap()
        );
        assert_eq!(block.transactions.len(), 3);
        assert_eq!(block.transaction_receipts.len(), 3);

        if let TransactionType::Deploy(tx) = &block.transactions[0] {
            assert_eq!(tx.constructor_calldata.len(), 2);
        } else {
            panic!("Did not deserialize Transaction::Deploy properly");
        }
        if let TransactionType::InvokeFunction(tx) = &block.transactions[1] {
            assert_eq!(tx.entry_point_type, EntryPointType::External);
            assert_eq!(tx.calldata.len(), 7);
        } else {
            panic!("Did not deserialize Transaction::InvokeFunction properly");
        }
        let receipt = &block.transaction_receipts[0];
        assert_eq!(receipt.execution_resources.n_steps, 68);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_block_deser_with_messages() {
        // has an L2 to L1 message
        let raw =
            include_str!("../../test-data/raw_gateway_responses/get_block/2_with_messages.txt");

        let block: Block = serde_json::from_str(raw).unwrap();

        assert_eq!(block.block_number.unwrap(), 122387);
        assert_eq!(block.transaction_receipts.len(), 49);
        let receipt = &block.transaction_receipts[22];
        assert_eq!(receipt.l2_to_l1_messages.len(), 1);
        assert_eq!(receipt.l2_to_l1_messages[0].payload.len(), 2);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_block_deser_with_events() {
        // has events introduced with StarkNet v0.7.0
        let raw = include_str!("../../test-data/raw_gateway_responses/get_block/3_with_events.txt");

        let block: Block = serde_json::from_str(raw).unwrap();

        assert_eq!(block.block_number.unwrap(), 47543);
        assert_eq!(block.transaction_receipts.len(), 4);
        let receipt = &block.transaction_receipts[3];
        assert_eq!(receipt.events.len(), 1);
        assert_eq!(receipt.events[0].data.len(), 2);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_block_deser_pending() {
        // pending blocks don't have `block_hash`, `block_number`, or `state_root`
        let raw = include_str!("../../test-data/raw_gateway_responses/get_block/4_pending.txt");

        let block: Block = serde_json::from_str(raw).unwrap();

        assert!(block.block_hash.is_none());
        assert!(block.block_number.is_none());
        assert!(block.state_root.is_none());
        assert_eq!(block.status, BlockStatus::Pending);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_block_deser_new_attributes_0_8_1() {
        // This block contains new fields introduced in StarkNet v0.8.1
        let new_block: Block = serde_json::from_str(include_str!(
            "../../test-data/raw_gateway_responses/get_block/5_with_class_hash_and_actual_fee.txt"
        ))
        .unwrap();
        match &new_block.transactions[43] {
            TransactionType::Deploy(transaction) => {
                assert!(transaction.class_hash.is_some());
            }
            _ => panic!("Unexpected transaction type"),
        }
        assert!(&new_block.transaction_receipts[0].actual_fee.is_some());

        let old_block: Block = serde_json::from_str(include_str!(
            "../../test-data/raw_gateway_responses/get_block/2_with_messages.txt"
        ))
        .unwrap();
        match &old_block.transactions[2] {
            TransactionType::Deploy(transaction) => {
                assert!(transaction.class_hash.is_none());
            }
            _ => panic!("Unexpected transaction type"),
        }
        assert!(&old_block.transaction_receipts[0].actual_fee.is_none());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_block_deser_new_attributes_0_8_2() {
        // This block contains new fields introduced in StarkNet v0.8.2
        let new_block: Block = serde_json::from_str(include_str!(
            "../../test-data/raw_gateway_responses/get_block/6_with_sequencer_address.txt"
        ))
        .unwrap();
        assert!(new_block.sequencer_address.is_some());

        let old_block: Block = serde_json::from_str(include_str!(
            "../../test-data/raw_gateway_responses/get_block/2_with_messages.txt"
        ))
        .unwrap();
        assert!(old_block.sequencer_address.is_none());
    }
}
