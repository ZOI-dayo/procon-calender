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

use serde::{Deserialize, de::DeserializeOwned, Serialize};

use flate2::bufread::GzDecoder;

use chrono::{Utc, DateTime, Duration, TimeZone};

use serde_json::json;

use jsonwebtoken::{encode, Header, Algorithm, EncodingKey};

mod http_helper;
use http_helper::HttpHelper;

mod google_calender;
use google_calender::GoogleCalender;
use google_calender::{CalenderEvent, CalenderTime};

use regex::Regex;

#[tokio::main]
async fn main() {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    let http_helper = HttpHelper::new();
    let google_calender = GoogleCalender::new().await;
    loop {
        let mut contests: Vec<ProconContest> = vec![];

        contests.append(get_problems(&http_helper/*&client*/).await.as_mut());
        contests.append(get_moja(&http_helper/*&client*/).await.as_mut());

        // add_calender(&client, google_login(&client).await).await;
        let events = google_calender.get_events().await;
        let mut new_contests: Vec<ProconContest> = vec![];
        for c in contests {
            let event: CalenderEvent = CalenderEvent {
                summary: c.title.clone(),
                description: None,
                location: c.url.clone(),
                start: CalenderTime{ dateTime:c.begin, timeZone: String::from("Asia/Tokyo") },
                end: CalenderTime{ dateTime:c.end, timeZone: String::from("Asia/Tokyo") },
            };
            if !&events.contains(&event) {
                new_contests.push(c);
            }
        }

        for c in new_contests {
            google_calender.add_event(c.title, String::new(), c.url, c.begin, c.end).await;
        }

        // TODO: 1~2h?
        sleep(std::time::Duration::from_secs(60*60));
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


async fn get_problems(http_helper: &HttpHelper/*client: &Client<HttpsConnector<hyper::client::HttpConnector>>*/) -> Vec<ProconContest> {
    #[derive(Serialize, Deserialize, Debug)]
    struct ProblemsProblem {
        id: String,
        title: String,
        start_epoch_second: i64,
        duration_second: i64,
    }
    let data = http_helper.get_json_gzip::<Vec<ProblemsProblem>>("https://kenkoooo.com/atcoder/internal-api/contest/recent").await;
    let mut contests: Vec<ProconContest> = Vec::new();
    for p in data {
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

    return contests;
}


async fn get_moja(http_helper: &HttpHelper/*client: &Client<HttpsConnector<hyper::client::HttpConnector>>*/) -> Vec<ProconContest> {
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

    // TODO: リンク変わるのどうにかする
    // let data = http_helper.get_json::<MojacoderData>("https://mojacoder.app/_next/data/zQ2R1boaeCCvdUgvPtmUh/ja/contests.json").await;
    let mut data = vec![];
    let html = http_helper.get("https://mojacoder.app/contests").await;
let re = Regex::new("<script id=\"__NEXT_DATA__\" type=\"application/json\">\\{\"props\":\\{\"pageProps\":\\{\"newContests\":(.*)},\"__N_SSG\":true\\},\"page\":\"/contests\",\"query\":\\{\\},\"buildId\":\".*\",\"runtimeConfig\":\\{\\},\"isFallback\":false,\"gsp\":true,\"locale\":\"ja\",\"locales\":\\[\"ja\",\"en\"\\],\"defaultLocale\":\"ja\",\"scriptLoader\":\\[\\]\\}</script>").unwrap();
match re.captures(&html) {
    Some(caps) => {
        println!("data: {}", &caps[1]);
        data = HttpHelper::to_json::<Vec<MojacoderContest>>((&caps[1]).to_string());
    }
    None => println!("Not found"),
}
println!("{:?}", data);
/*
   for m in re.find_iter(&html) {
    println!("Found `{}` at {}-{}", m.as_str(), m.start(), m.end());
}*/

    let mut contests: Vec<ProconContest> = Vec::new();
    for p in data {
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

    return contests;
}
/*
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
            scope: "https://www.googleapis.com/auth/calendar".to_string(),
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

    println!("{}", result);

    let token_response_body: Token = serde_json::from_str(&result).unwrap();

    // let access_token = token_response_body.get("access_token").unwrap().as_str();

    return String::from(token_response_body.access_token);
}
*/
/*
async fn add_calender(client: &Client<HttpsConnector<hyper::client::HttpConnector>>, token: String) {
    #[derive(Debug, Deserialize)]
    struct GoogleCalenderInfo {
        id: String
    }
    let google_calender_info: GoogleCalenderInfo = serde_json::from_reader(std::fs::File::open("secret/google_calender_info.json").unwrap()).unwrap();


    let token_body = json!({
        "start": {
            "dateTime": "2023-03-13T17:00:00+09:00",
            "timeZone": "Asia/Tokyo"
        },
        "end": {
            "dateTime": "2023-03-13T18:50:00+09:00",
            "timeZone": "Asia/Tokyo"
        },
        "summary": "title",
        "description": "https://hoge.com\naaaaiiiiiuuuuu",
        "location": "https://hoge.com"
    });

    #[derive(Debug, Deserialize)]
    struct CalenderData {
        items: Vec<CalenderEvent>,
    }
    #[derive(Debug, Deserialize)]
    struct CalenderEvent {
        id: String,
        summary: String,
        description: String,
        location: String,
        start: CalenderTime,
        end: CalenderTime,
    }
    #[derive(Debug, Deserialize)]
    struct CalenderTime {
        dateTime: DateTime<Utc>,
        timeZone: String,
    }
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("https://www.googleapis.com/calendar/v3/calendars/{}/events", google_calender_info.id))
        .header("Authorization", format!("OAuth {}", token))
        .body(Body::from("")).unwrap();
    let mut resp = client.request(req).await.unwrap();
    let mut result = String::new();
    while let Some(chunk) = resp.body_mut().data().await {
        result += &String::from_utf8((&chunk.unwrap()).to_vec()).unwrap();
    }
    println!("{}", result);
    let calender_data: CalenderData = serde_json::from_str(&result).unwrap();
    println!("{:?}", calender_data);

    // TOOD: for contest in list: if it is not added to calender, add it to list.

    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("https://www.googleapis.com/calendar/v3/calendars/{}/events", google_calender_info.id))
        .header("Authorization", format!("OAuth {}", token))
        .body(Body::from(token_body.to_string())).unwrap();
    let mut resp = client.request(req).await.unwrap();
    let mut result = String::new();
    while let Some(chunk) = resp.body_mut().data().await {
        result += &String::from_utf8((&chunk.unwrap()).to_vec()).unwrap();
    }
    println!("{}", result);
}

*/
// Debug

fn type_of<T>(_: T) -> String{
    let a = std::any::type_name::<T>();
    return a.to_string();
}
