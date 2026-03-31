mod no_run {
    fn custom_log_init() {
        // Here would go custom initialization code
    }
    use lambda_appsync::appsync_lambda_main;
    appsync_lambda_main!("../../../../schema.graphql", log_init = custom_log_init);
}

fn main() {}
