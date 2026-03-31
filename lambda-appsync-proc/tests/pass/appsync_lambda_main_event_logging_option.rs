mod no_run {
    // Test with event_logging enabled
    lambda_appsync::appsync_lambda_main!("../../../../schema.graphql", event_logging = true);
}

fn main() {}
