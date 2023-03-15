use std::{
    vec::Vec,
    collections::HashMap,
};

use hyper::{
    Client,
    Request,
    Body,
    Method,
    header::HeaderValue,
};
use hyper_tls::HttpsConnector;

use serde::de::DeserializeOwned;

use flate2::bufread::GzDecoder;


pub struct HttpHelper {
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
}

impl HttpHelper {
    pub fn new() -> HttpHelper {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, Body>(https);
        return HttpHelper {client};
    }

    pub async fn request(&self, url: &str, method: Method, body: String, headers: HashMap<String, String>) -> Vec<u8> {
        let mut req = Request::builder()
            .method(method)
            .uri(url.to_string());
        for (key, value) in headers.iter() {
            req = req.header(key.clone().as_str(), HeaderValue::from_str(value.clone().as_str()).unwrap());
        }
        let resp = self.client.request(req.body(Body::from(body)).unwrap()).await.unwrap();
        return hyper::body::to_bytes(resp.into_body()).await.unwrap().to_vec();
    }

    pub fn to_json<T>(data: String) -> T
        where
            T: DeserializeOwned,
        {
            println!("{}", data);
            return serde_json::from_reader(data.as_bytes()).unwrap();
        }

    pub async fn get(&self, url: &str) -> String
        {
            let response: Vec<u8> = self.request(url, Method::GET, String::new(), HashMap::new()).await;
            let text = String::from_utf8(response).unwrap();
            return text;
        }

    pub async fn get_with_header(&self, url: &str, headers: HashMap<String, String>) -> String
        {
            let response: Vec<u8> = self.request(url, Method::GET, String::new(), headers).await;
            let text = String::from_utf8(response).unwrap();
            return text;
        }

/*
    pub async fn get_json<T>(&self, url: &str) -> T
        where
            T: DeserializeOwned,
        {
            return HttpHelper::to_json::<T>(self.get(url).await);
        }
*/

    pub async fn get_json_with_header<T>(&self, url: &str, headers: HashMap<String, String>) -> T
        where
            T: DeserializeOwned,
        {
            return HttpHelper::to_json::<T>(self.get_with_header(url, headers).await);
        }


    pub async fn get_json_gzip<T>(&self, url: &str) -> T
        where
            T: DeserializeOwned,
        {
            let response = self.request(&url.to_string(), Method::GET, String::new(), HashMap::from([(String::from("Accept-Encoding"), String::from("gzip"))])).await;
            let decoder = GzDecoder::new(&response[..]);
            return HttpHelper::to_json::<T>(std::io::read_to_string(decoder).unwrap());
        }

    pub async fn post(&self, url: &str, body: String, content_type: &str) -> String
        {
            let response: Vec<u8> = self.request(url, Method::POST, body, HashMap::from([(String::from("Content-Type"), String::from(content_type))])).await;
            let text = String::from_utf8(response).unwrap();
            return text;
        }

    pub async fn post_with_header(&self, url: &str, body: String, content_type: &str, headers: HashMap<String, String>) -> String
        {
            let mut headers = headers.clone();
            headers.insert(String::from("Content-Type"), String::from(content_type));
            let response: Vec<u8> = self.request(url, Method::POST, body, headers).await;
            let text = String::from_utf8(response).unwrap();
            return text;
        }

/*
    pub async fn post_json_with_header<T>(&self, url: &str, body: String, content_type: &str, headers: HashMap<String, String>) -> T
        where
            T: DeserializeOwned,
        {
            return HttpHelper::to_json::<T>(self.post_with_header(url, body, content_type, headers).await);
        }
*/


}

