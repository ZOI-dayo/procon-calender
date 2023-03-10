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

use flate2::bufread::GzDecoder;

use chrono::{Utc, DateTime, Duration, TimeZone};

use serde_json::json;

use jsonwebtoken::{encode, Header, Algorithm, EncodingKey};


#[tokio::main]
async fn main() {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    loop {
        // problems
        get_problems(&client).await;
        get_moja(&client).await;
        println!("{}", google_login(&client).await);

        // TODO: 1~2h?
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

async fn google_login(client: &Client<HttpsConnector<hyper::client::HttpConnector>>) -> String {
    #[derive(Debug, Deserialize)]
    struct GoogleCredential {
        r#type: String,
        project_id: String,
        private_key_id: String,
        private_key: String,
        client_email: String,
        client_id: String,
        auth_uri: String,
        token_uri: String,
        auth_provider_x509_cert_url: String,
        client_x509_cert_url: String,
    }
    let google_credential: GoogleCredential = serde_json::from_reader(std::fs::File::open("secret/google_credential.json").unwrap()).unwrap();

    let now = Utc::now();
    let iat = now.timestamp();
    let exp = (now + Duration::minutes(60)).timestamp();

    #[derive(Debug, Serialize)]
    struct Claims {
        iss: String,
        scope: String,
        aud: String,
        exp: i64,
        iat: i64,
    }
    let mut header = Header::default();
    header.typ = Some("JWT".to_string());
    header.alg = Algorithm::RS256;

    let my_claims =
        Claims {
            iss: google_credential.client_email,
            scope: "https://www.googleapis.com/auth/spreadsheets".to_string(),
            aud: google_credential.token_uri,
            exp,
            iat,
        };

    let jwt = encode(&header, &my_claims, &EncodingKey::from_rsa_pem(google_credential.private_key.as_bytes()).unwrap()).unwrap();

    let token_body = json!({
        "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer",
        "assertion": jwt
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri(my_claims.aud)
        .header("Content-Type", "application/json")
        .body(Body::from(token_body.to_string())).unwrap();
    let mut resp = client.request(req).await.unwrap();

    let mut result = String::new();
    while let Some(chunk) = resp.body_mut().data().await {
        result += &String::from_utf8((&chunk.unwrap()).to_vec()).unwrap();
    }

    #[derive(Debug, Deserialize)]
    struct Token {
        access_token: String,
    }

    let token_response_body: Token = serde_json::from_str(&result).unwrap();

    // let access_token = token_response_body.get("access_token").unwrap().as_str();

    return String::from(token_response_body.access_token);
}

// Debug

fn type_of<T>(_: T) -> String{
    let a = std::any::type_name::<T>();
    return a.to_string();
}
