use std::{
    thread::sleep,
    vec::Vec,
};

use serde::{Deserialize, Serialize};

use chrono::{Utc, DateTime, Duration, TimeZone};

mod http_helper;
use http_helper::HttpHelper;

mod google_calender;
use google_calender::GoogleCalender;
use google_calender::{CalenderEvent, CalenderTime};

use regex::Regex;

#[tokio::main]
async fn main() {
    let http_helper = HttpHelper::new();
    let mut google_calender = GoogleCalender::new().await;

    let mut contests: Vec<ProconContest> = vec![];

    contests.append(get_problems(&http_helper).await.as_mut());
    contests.append(get_moja(&http_helper).await.as_mut());

    let events = google_calender.get_events().await.iter().clone().map(|e| {e.location.clone()}).collect::<Vec<_>>();
    let mut new_contests: Vec<ProconContest> = vec![];
    // println!("{:?}", events.len());
    for c in contests {
        if !events.iter().any(|e| {e == &c.url.clone()}) {
            // println!("{:?}", c.url.clone());
            new_contests.push(c);
        }
    }

    for c in new_contests {
        google_calender.add_event(c.title, String::new(), c.url, c.begin, c.end).await;
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


async fn get_problems(http_helper: &HttpHelper) -> Vec<ProconContest> {
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
        let begin = Utc.timestamp_opt(p.start_epoch_second, 0).unwrap();
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
        if (contest.end - Utc::now()).num_days() > 30 {
            continue;
        }
        if (contest.end - contest.begin).num_days() > 1 {
            continue;
        }
        println!("adding contest from Problems : {}", &contest.title);
        contests.push(contest);
    }

    return contests;
}


async fn get_moja(http_helper: &HttpHelper) -> Vec<ProconContest> {
    #[derive(Serialize, Deserialize, Debug)]
    struct MojacoderUserDetail {
        #[serde(rename = "screenName")]
        screen_name: String,
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
        #[serde(rename = "startDatetime")]
        start_datetime: String,
        user: MojacoderUser,
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct MojacoderPageProps {
        #[serde(rename = "newContests")]
        new_contests: Vec<MojacoderContest>
    }
    #[derive(Serialize, Deserialize, Debug)]
    struct MojacoderData {
        #[serde(rename = "pageProps")]
        page_props: MojacoderPageProps
    }

    let mut data = vec![];
    let html = http_helper.get("https://mojacoder.app/contests").await;
let re = Regex::new("<script id=\"__NEXT_DATA__\" type=\"application/json\">\\{\"props\":\\{\"pageProps\":\\{\"newContests\":(.*)},\"__N_SSG\":true\\},\"page\":\"/contests\",\"query\":\\{\\},\"buildId\":\".*\",\"runtimeConfig\":\\{\\},\"isFallback\":false,\"gsp\":true,\"locale\":\"ja\",\"locales\":\\[\"ja\",\"en\"\\],\"defaultLocale\":\"ja\",\"scriptLoader\":\\[\\]\\}</script>").unwrap();
match re.captures(&html) {
    Some(caps) => {
        // println!("data: {}", &caps[1]);
        data = HttpHelper::to_json::<Vec<MojacoderContest>>((&caps[1]).to_string());
    }
    None => println!("Not found"),
}
// println!("{:?}", data);
    let mut contests: Vec<ProconContest> = Vec::new();
    for p in data {
        let begin = DateTime::parse_from_rfc3339(&p.start_datetime).unwrap().with_timezone(&Utc);
        let contest = ProconContest {
            id: format!("mojacoder_{}", p.id),
            title: p.name,
            begin,
            end: begin + Duration::seconds(p.duration),
            url: format!("https://mojacoder.app/users/{}/contests/{}", p.user.detail.screen_name, p.slug),
        };
        if (contest.end - Utc::now()).num_seconds() < 0 {
            continue;
        }
        if (contest.end - Utc::now()).num_days() > 30 {
            continue;
        }
        if (contest.end - contest.begin).num_days() > 1 {
            continue;
        }
        println!("adding contest from Moja : {}", &contest.title);
        contests.push(contest);
    }

    return contests;
}

// Debug
/*
fn type_of<T>(_: T) -> String{
    let a = std::any::type_name::<T>();
    return a.to_string();
}
*/
