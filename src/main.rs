use std::{
    thread::sleep,
    time::Duration,
    vec::Vec,
};

use hyper::{
    Client,
    Request,
    Body,
    body::HttpBody as _,
    header::ACCEPT,
Method,
};
use hyper_tls::HttpsConnector;

use serde::{Deserialize, Serialize};

use  flate2::bufread::GzDecoder;

#[tokio::main]
async fn main() {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    loop {
        // problems
        get_problems(&client).await;
        get_moja(&client).await;

        sleep(Duration::from_secs(60));
    }
}

async fn get_problems(client: &Client<HttpsConnector<hyper::client::HttpConnector>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    // let client = hyper::Client::new();

    // Parse an `http::Uri`...
    // let uri = "https://kenkoooo.com/atcoder/internal-api/contest/recent".parse()?;
    // let uri = "https://google.com/".parse()?;
    // let uri = "https://kenkoooo.com/atcoder/internal-api/contest/recent".parse()?;
    // let req = Request::builder().method("GET").uri(uri).header("accept", "gzip").body(()).unwrap();
    let req = Request::builder()
        .method(Method::GET)
        .uri("https://kenkoooo.com/atcoder/internal-api/contest/recent")
        .header("Accept-Encoding", "gzip")
        // IF YOU DON'T INCLUDE THIS HEADER, ONLY THE FIRST PROPERTY OF THE STRUCT GETS RETURNED???
        .body(Body::from("")).unwrap();
    // let mut resp = client.get(uri).header("allow", "gzip").build().await?;
    // let mut resp = client.get(uri).header(ACCEPT ("gzip")).build().await?;
    let mut resp_gzip = client.request(req).await?;
    // let mut resp = send(req).await?;
    let data = hyper::body::to_bytes(resp_gzip.into_body())
        .await?
        .to_vec();
    // println!("{}", type_of(&resp_gzip));
    let resp = GzDecoder::new(&data[..]);
    // println!("status={}", resp_gzip.status());

    let mut result = std::io::read_to_string(resp).unwrap();
    /*
    while let Some(chunk) = resp_gzip.body_mut().data().await {
        // stdout().write_all(&chunk?).await?;
        result += &String::from_utf8((&chunk?).to_vec()).unwrap();
    }
    */
    println!("{}", result);

#[derive(Serialize, Deserialize, Debug)]
    struct Problem {
        id: String,
        title: String,
        start_epoch_second: i64,
    }

    let deserialized_map: Vec<Problem> = serde_json::from_str(&result).unwrap();
    println!("{:?}", deserialized_map);
    fn type_of<T>(_: T) -> String{
        let a = std::any::type_name::<T>();
        return a.to_string();
    }
    println!("{}", type_of(&client));


    Ok(())
}


async fn get_moja(client: &Client<HttpsConnector<hyper::client::HttpConnector>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let uri = "https://mojacoder.app/_next/data/gFS1O7T42djs1wRKmTHvP/ja/problems.json".parse()?;
    let mut resp = client.get(uri).await?;
    let mut result = String::from("");
    while let Some(chunk) = resp.body_mut().data().await {
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
    fn type_of<T>(_: T) -> String{
        let a = std::any::type_name::<T>();
        return a.to_string();
    }


    Ok(())
}
