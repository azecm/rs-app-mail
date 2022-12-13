use warp::{Filter, http, Rejection};
use warp::hyper::body::Bytes;

use crate::NotUtf8;
use crate::utils::get_hash;

pub fn with_hash() -> impl Filter<Extract=(String, ), Error=Rejection> + Clone {
    warp::any()
        .and(warp::header::optional::<String>(
            http::header::USER_AGENT.as_str(),
        ))
        .and(warp::header::optional::<String>("remote-addr"))
        .and(warp::header::optional::<String>(
            http::header::ACCEPT_LANGUAGE.as_str(),
        ))
        .and(warp::header::optional::<String>(
            http::header::ACCEPT_ENCODING.as_str(),
        ))
        .map(
            move |user_agent: Option<String>,
                  ip: Option<String>,
                  accept_language: Option<String>,
                  accept_encoding: Option<String>| {
                let browser = user_agent.unwrap_or_default();
                let ip = ip.unwrap_or_default();
                let accept_language = accept_language.unwrap_or_default();
                let accept_encoding = accept_encoding.unwrap_or_default();

                get_hash(format!("{browser}-{ip}-{accept_language}-{accept_encoding}"))
            },
        )
}

pub fn with_body_filter() -> impl Filter<Extract=(String, ), Error=Rejection> + Clone {
    warp::body::bytes().and_then(|body: Bytes| async move {
        std::str::from_utf8(&body)
            .map(String::from)
            .map_err(|_e| warp::reject::custom(NotUtf8))
    })
}