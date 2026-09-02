use std::collections::BTreeSet;
use std::error::Error;

use serde::Deserialize;
use wow_core::E0_OPERATION_IDS;

#[derive(Debug, Deserialize)]
struct Contract {
    e0_operations: Vec<Operation>,
}

#[derive(Debug, Deserialize)]
struct Operation {
    id: String,
    status: String,
}

#[test]
fn public_operation_registry_matches_machine_contract() -> Result<(), Box<dyn Error>> {
    let contract: Contract = serde_json::from_str(include_str!("../CONTRACT.json"))?;
    let required = contract
        .e0_operations
        .into_iter()
        .filter(|operation| operation.status == "required")
        .map(|operation| operation.id)
        .collect::<Vec<_>>();
    let implemented = E0_OPERATION_IDS
        .iter()
        .map(|operation| (*operation).to_owned())
        .collect::<Vec<_>>();

    assert_eq!(required, implemented);
    assert_eq!(
        implemented.len(),
        implemented.iter().collect::<BTreeSet<_>>().len()
    );
    Ok(())
}
