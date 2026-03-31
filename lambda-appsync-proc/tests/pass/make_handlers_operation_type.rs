mod ops {
    lambda_appsync::make_types!("../../../../schema.graphql");
    lambda_appsync::make_operation!("../../../../schema.graphql");
}

lambda_appsync::make_handlers!(operation_type = ops::Operation);

fn main() {}
