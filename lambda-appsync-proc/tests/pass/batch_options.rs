// Test with batch processing disabled
mod appsync_lambda_main {
    lambda_appsync::appsync_lambda_main!(
        "../../../../schema.graphql",
        exclude_lambda_handler = true,
        batch = false
    );
}

mod make_appsync {
    lambda_appsync::make_appsync!("../../../../schema.graphql", batch = false);
}

mod make_handlers {
    lambda_appsync::make_types!("../../../../schema.graphql");
    lambda_appsync::make_operation!("../../../../schema.graphql");
    lambda_appsync::make_handlers!(batch = false);
}

fn main() {}
