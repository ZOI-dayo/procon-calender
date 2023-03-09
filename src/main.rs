use std::{
    thread::sleep,
    time::Duration,
    collections::HashMap,
    vec::Vec,
};

use hyper::body::HttpBody as _;
use tokio::io::{stdout, AsyncWriteExt as _};
use hyper_tls::HttpsConnector;
use hyper::Client;

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    loop {
        let client = reqwest::Client::new();
        // problems
        let problems = get_problems(client).await;
        println!("aaaa");
        // dbg!(problems);

        sleep(Duration::from_secs(60));
    }
}

async fn get_problems(client: reqwest::Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("aaa");
    /*

    let json = client
        // .get("https://kenkoooo.com/atcoder/internal-api/contest/recent")
        .get("https://mojacoder.app/_next/data/gFS1O7T42djs1wRKmTHvP/ja/problems.json")
        .header(reqwest::header::ACCEPT_ENCODING, "gzip, deflate, br")
        .send()
        .await
        .unwrap()
        .json::<HashMap<String, String>>()
        .await
        .unwrap();
    return format!("{:?}", json);
    */
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    // let client = hyper::Client::new();

    // Parse an `http::Uri`...
    // let uri = "https://kenkoooo.com/atcoder/internal-api/contest/recent".parse()?;
    // let uri = "https://google.com/".parse()?;
    let uri = "https://mojacoder.app/_next/data/gFS1O7T42djs1wRKmTHvP/ja/problems.json".parse()?;
    let mut resp = client.get(uri).await?;
    println!("status={}", resp.status());

    let mut result = String::from("");
    while let Some(chunk) = resp.body_mut().data().await {
        // stdout().write_all(&chunk?).await?;
        result += &String::from_utf8((&chunk?).to_vec()).unwrap();
    }
    println!("{}", result);

#[derive(Serialize, Deserialize, Debug)]
    struct Problem {
        id: String,
        title: String,
        datetime: String
    }
#[derive(Serialize, Deserialize, Debug)]
    struct PageProps {
        newProblems: Vec<Problem>
    }
#[derive(Serialize, Deserialize, Debug)]
    struct Data {
        pageProps: PageProps
    }

    let deserialized_map: Data = serde_json::from_str(&result).unwrap();
    println!("{:?}", deserialized_map);


    Ok(())
}
