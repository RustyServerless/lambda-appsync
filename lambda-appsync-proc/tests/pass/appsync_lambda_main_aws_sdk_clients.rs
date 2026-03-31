mod no_run {
    use crate::{Operation, Player};
    use aws_sdk_dynamodb::Client as DynamoClient;
    use aws_sdk_s3::Client as S3Client;
    use lambda_appsync::{appsync_lambda_main, appsync_operation, AppsyncError};

    // Test multiple AWS SDK client initializations
    appsync_lambda_main!(
        "../../../../schema.graphql",
        only_lambda_handler = true,
        // Test multiple clients
        dynamodb() -> DynamoClient,
        s3() -> S3Client,
    );

    // Validate that we can use both clients
    #[appsync_operation(query(players))]
    async fn get_players() -> Result<Vec<Player>, AppsyncError> {
        let _dynamo = dynamodb();
        let _s3 = s3();
        Ok(vec![])
    }
}

lambda_appsync::appsync_lambda_main!("../../../../schema.graphql", exclude_lambda_handler = true,);

fn main() {}
