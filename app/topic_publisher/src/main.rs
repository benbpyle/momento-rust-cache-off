mod models;

use aws_config::{from_env, meta::region::RegionProviderChain};
use models::MomentoModel;
use momento::{CredentialProvider, TopicClient};

#[tokio::main]
async fn main() {
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

    let mut i = 0;

    while i < 100 {
        let m = MomentoModel::new(String::from("KeyOne"), String::from("KeyTwo"), i);
        let t = serde_json::to_string(&m).unwrap();

        match topic_client.publish("cache-off", "cache-off", t).await {
            Ok(_) => {
                println!("Published message");
            }
            Err(e) => {
                println!("(Error)={e:?}");
            }
        }

        i += 1;
    }
}
