use std::{
    thread::sleep,
    vec::Vec,
};

use hyper::{
    Client,
    Request,
    Body,
    body::HttpBody as _,
    Method,
};
use hyper_tls::HttpsConnector;

use serde::{Deserialize, Serialize};

use  flate2::bufread::GzDecoder;

use chrono::{Utc, DateTime, Duration, TimeZone};

#[tokio::main]
async fn main() {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    loop {
        // problems
        get_problems(&client).await;
        get_moja(&client).await;

        sleep(std::time::Duration::from_secs(60));
    }
}

#[derive(Serialize, Deserialize, Debug)]
    struct ProconContest {
        id: String,
        title: String,
        begin: DateTime<Utc>,
        end: DateTime<Utc>,
        url: String,
    }


async fn get_problems(client: &Client<HttpsConnector<hyper::client::HttpConnector>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let req = Request::builder()
        .method(Method::GET)
        .uri("https://kenkoooo.com/atcoder/internal-api/contest/recent")
        .header("Accept-Encoding", "gzip")
        .body(Body::from("")).unwrap();
    let resp_gzip = client.request(req).await?;
    let data = hyper::body::to_bytes(resp_gzip.into_body())
        .await?
        .to_vec();
    let decoder = GzDecoder::new(&data[..]);
    let resp = std::io::read_to_string(decoder).unwrap();

#[derive(Serialize, Deserialize, Debug)]
    struct ProblemsProblem {
        id: String,
        title: String,
        start_epoch_second: i64,
duration_second: i64,
    }

    let deserialized_map: Vec<ProblemsProblem> = serde_json::from_str(&resp).unwrap();

    let mut contests: Vec<ProconContest> = Vec::new();
    for p in deserialized_map {
        let begin = Utc.timestamp(p.start_epoch_second, 0);
        let url = format!("https://kenkoooo.com/atcoder#/contest/show/{}", p.id);
        let contest = ProconContest {
            id: format!("problems_{}", p.id),
            title: p.title,
            begin,
            end: begin + Duration::seconds(p.duration_second),
            url,
        };
        if (contest.end - Utc::now()).num_seconds() < 0 {
            continue;
        }
        if (contest.end - Utc::now()).num_days() > 7 {
            continue;
        }
    println!("{:?}", &contest);
        contests.push(contest);
    }

    Ok(())
}


async fn get_moja(client: &Client<HttpsConnector<hyper::client::HttpConnector>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let uri = "https://mojacoder.app/_next/data/gFS1O7T42djs1wRKmTHvP/ja/contests.json".parse()?;
    let mut resp = client.get(uri).await?;
    let mut result = String::from("");
    while let Some(chunk) = resp.body_mut().data().await {
        result += &String::from_utf8((&chunk?).to_vec()).unwrap();
    }

#[derive(Serialize, Deserialize, Debug)]
    struct MojacoderUserDetail {
        screenName: String,
    }
#[derive(Serialize, Deserialize, Debug)]
    struct MojacoderUser {
        detail: MojacoderUserDetail,
    }
#[derive(Serialize, Deserialize, Debug)]
    struct MojacoderContest {
        id: String,
        slug: String,
        name: String,
        duration: i64,
        startDatetime: String,
        user: MojacoderUser,
    }
#[derive(Serialize, Deserialize, Debug)]
    struct MojacoderPageProps {
        newContests: Vec<MojacoderContest>
    }
#[derive(Serialize, Deserialize, Debug)]
    struct MojacoderData {
        pageProps: MojacoderPageProps
    }

    let deserialized_map: MojacoderData = serde_json::from_str(&result).unwrap();

    let mut contests: Vec<ProconContest> = Vec::new();
    for p in deserialized_map.pageProps.newContests {
        let begin = DateTime::parse_from_rfc3339(&p.startDatetime).unwrap().with_timezone(&Utc);
        let contest = ProconContest {
            id: format!("mojacoder_{}", p.id),
            title: p.name,
            begin,
            end: begin + Duration::seconds(p.duration),
            url: format!("https://mojacoder.app/users/{}/contests/{}", p.user.detail.screenName, p.slug),
        };
        if (contest.end - Utc::now()).num_seconds() < 0 {
            continue;
        }
        if (contest.end - Utc::now()).num_days() > 7 {
            continue;
        }
    println!("{:?}", &contest);
        contests.push(contest);
    }


    Ok(())
}

// Debug

fn type_of<T>(_: T) -> String{
    let a = std::any::type_name::<T>();
    return a.to_string();
}
