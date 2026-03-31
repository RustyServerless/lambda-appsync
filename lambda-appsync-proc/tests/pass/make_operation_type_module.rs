mod types {
    lambda_appsync::make_types!("../../../../schema.graphql");
}

// Don't use types::* — instead use type_module
lambda_appsync::make_operation!("../../../../schema.graphql", type_module = types,);

fn main() {}
