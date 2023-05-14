use std::{
    vec::Vec,
    collections::HashMap,
};

use serde::{Deserialize, Serialize};

use chrono::{Utc, DateTime, Duration};

use serde_json::json;

use jsonwebtoken::{encode, Header, Algorithm, EncodingKey};

use crate::http_helper::HttpHelper;

#[derive(Debug, Deserialize)]
pub struct GoogleCalenderInfo {
    id: String,
}
#[derive(Debug, Deserialize)]
pub struct CalenderEvent {
    // id: String,
    pub summary: String,
    pub description: Option<String>,
    pub location: String,
    pub start: CalenderTime,
    pub end: CalenderTime,
}
impl PartialEq for CalenderEvent {
    fn eq(&self, other: &Self) -> bool {
        self.summary == other.summary
            && self.start.date_time == other.start.date_time
            && self.end.date_time == other.end.date_time
    }
}
#[derive(Debug, Deserialize)]
pub struct CalenderTime {
    #[serde(rename = "dateTime")]
    pub date_time: DateTime<Utc>,
    #[serde(rename = "timeZone")]
    pub time_zone: String,
}


pub struct GoogleCalender {
    http_helper: HttpHelper,
    cached_token: Option<String>,
    token_exp: i64,
    info: GoogleCalenderInfo,
}

impl GoogleCalender {
    pub async fn new() -> GoogleCalender {
        let info: GoogleCalenderInfo = serde_json::from_reader(std::fs::File::open("secret/google_calender_info.json").unwrap()).unwrap();
        GoogleCalender {
            http_helper: HttpHelper::new(),
            cached_token: None,
            token_exp: 0,
            info,
        }
    }
    async fn get_token(&mut self) -> String {
        // TODO: 期限切れてたら再生成
        if self.cached_token.is_some() {
            if self.token_exp > Utc::now().timestamp() + 60 {
                return self.cached_token.as_ref().unwrap().to_string();
            }
        }
        #[derive(Debug, Deserialize)]
        struct GoogleCredential {
            // r#type: String,
            // project_id: String,
            // private_key_id: String,
            private_key: String,
            client_email: String,
            // client_id: String,
            // auth_uri: String,
            token_uri: String,
            // auth_provider_x509_cert_url: String,
            // client_x509_cert_url: String,
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

        #[derive(Debug, Deserialize)]
        struct Token {
            access_token: String,
        }

        let token_response_body = HttpHelper::to_json::<Token>(self.http_helper.post(&my_claims.aud, token_body.to_string(), "application/json").await);
        self.cache_token(token_response_body.access_token.clone(), exp);
        return token_response_body.access_token;
    }

    fn cache_token(&mut self, token: String, exp: i64) {
        self.token_exp = exp;
        self.cached_token = Some(token);
    }

    pub async fn get_events(&mut self) -> Vec<CalenderEvent> {
        #[derive(Debug, Deserialize)]
        struct CalenderData {
            items: Vec<CalenderEvent>,
        }

        let token = self.get_token().await;

        let calender_data = self.http_helper.get_json_with_header::<CalenderData>(
            &format!("https://www.googleapis.com/calendar/v3/calendars/{}/events?singleEvents=true&maxResults=2500&orderBy=startTime&timeMin={}", self.info.id, Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
            HashMap::from([(String::from("Authorization"), String::from(format!("OAuth {}", token)))])
            ).await;
        return calender_data.items;
    }

    pub async fn add_event(&mut self, title: String, description: String, location: String, start: DateTime<Utc>, end: DateTime<Utc>) {
        // println!("{}", start.to_string());
        let token_body = json!({
            "start": {
                "dateTime": start.to_rfc3339(),
                "timeZone": "Asia/Tokyo"
            },
            "end": {
                "dateTime": end.to_rfc3339(),
                "timeZone": "Asia/Tokyo"
            },
            "summary": title,
            "description": description,
            "location": location
        });

        // TOOD: for contest in list: if it is not added to calender, add it to list.

        let token = self.get_token().await;

         let response = self.http_helper.post_with_header(
            &format!("https://www.googleapis.com/calendar/v3/calendars/{}/events", self.info.id),
            token_body.to_string(),
            "application/json",
            HashMap::from([(String::from("Authorization"), String::from(format!("OAuth {}", token)))])
            ).await;

        println!("Add Event: {}", response);
    }
}
