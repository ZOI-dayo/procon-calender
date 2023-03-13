use std::{
    thread::sleep,
    vec::Vec,
    collections::HashMap,
};

use hyper::{
    Client,
    Request,
    Response,
    Body,
    body::HttpBody as _,
    Method,
    header::HeaderValue,
};
use hyper_tls::HttpsConnector;

use serde::{de::DeserializeOwned};

use flate2::bufread::GzDecoder;

use chrono::{Utc, DateTime, Duration, TimeZone};

use serde_json::json;

use jsonwebtoken::{encode, Header, Algorithm, EncodingKey};



pub struct HttpHelper {
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
}

impl HttpHelper {
    pub fn new() -> HttpHelper {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, Body>(https);
        return HttpHelper {client};
    }

    async fn request(&self, url: &str, method: Method, body: String, headers: HashMap<String, String>) -> Vec<u8> {
        let mut req = Request::builder()
            .method(method)
            .uri(url.to_string());
        for (key, value) in headers.iter() {
            req = req.header(key.clone().as_str(), HeaderValue::from_str(value.clone().as_str()).unwrap());
        }
        let resp = self.client.request(req.body(Body::from(body)).unwrap()).await.unwrap();
        return hyper::body::to_bytes(resp.into_body()).await.unwrap().to_vec();
    }

    fn to_json<T>(data: String) -> T
        where
            T: DeserializeOwned,
        {
            return serde_json::from_reader(data.as_bytes()).unwrap();
        }

    pub async fn get<T>(&self, url: &str) -> T
        where
            T: DeserializeOwned,
        {
            let response: Vec<u8> = self.request(url, Method::GET, String::new(), HashMap::new()).await;
            return Self::to_json::<T>(String::from_utf8(response).unwrap());
        }

    pub async fn get_gzip<T>(&self, url: &str) -> T
        where
            T: DeserializeOwned,
        {
            let response = self.request(&url.to_string(), Method::GET, String::new(), HashMap::from([(String::from("Accept-Encoding"), String::from("gzip"))])).await;
            let decoder = GzDecoder::new(&response[..]);
            return Self::to_json::<T>(std::io::read_to_string(decoder).unwrap());
        }
}

