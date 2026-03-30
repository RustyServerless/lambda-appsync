mod types {
    lambda_appsync::make_types!("../../../../schema.graphql");
}

use types::*;

// Test generating only operation enums using make_operation!
lambda_appsync::make_operation!("../../../../schema.graphql");

fn main() {
    // Verify we can use the Operation enum
    let op = Operation::Query(QueryField::Players);
    match op {
        Operation::Query(QueryField::Players) => {}
        _ => panic!("Unexpected operation"),
    }
}
