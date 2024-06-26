mod models;

use aws_config::{from_env, meta::region::RegionProviderChain};
use futures::StreamExt;
use momento::{topics::Subscription, CredentialProvider, MomentoError, TopicClient};
use std::error::Error;
use tracing::{error, info};

use crate::models::MomentoModel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    let region_provider = RegionProviderChain::default_provider();
    let config = from_env().region(region_provider).load().await;

    // create the ssm and ddb client
    // ssm is used to fetch Momento's Key
    let client = aws_sdk_ssm::Client::new(&config);
    let parameter = match client
        .get_parameter()
        .name("/keys/momento-pct-key")
        .send()
        .await
    {
        Ok(p) => p,
        Err(_) => panic!("error with aws sdk client"),
    };

    // if no key, panic and don't start
    let api_key = match parameter.parameter {
        Some(p) => p.value.unwrap(),
        None => panic!("Error with parameter"),
    };

    // create a new Momento client
    let topic_client = match TopicClient::builder()
        .configuration(momento::topics::configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_string(api_key).unwrap())
        .build()
    {
        Ok(c) => c,
        Err(_) => panic!("error with momento client"),
    };

    let mut subscription: Subscription = topic_client
        .subscribe("cache-off", "cache-off")
        .await
        .expect("subscribe rpc failed");

    // Consume the subscription
    while let Some(item) = subscription.next().await {
        info!("Received subscription item: {item:?}");
        let value: Result<String, MomentoError> = item.try_into();
        match value {
            Ok(v) => {
                let o: MomentoModel = serde_json::from_str(v.as_str()).unwrap();
                info!(
                    "(Value)={}|(MoModel)={o:?}|(TimeBetween)={}",
                    v,
                    o.time_between_publish_and_received()
                );
            }
            Err(e) => {
                error!("(Error Momento)={}", e);
            }
        }
    }

    Ok(())
}
