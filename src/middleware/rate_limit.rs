use std::collections::HashMap;
use std::time::{Duration, Instant};

use rocket::http::{Header, Method, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::Responder;
use rocket::{Response, response};
use rocket_okapi::r#gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiResponse, Responses};
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use tokio::sync::Mutex;
use tracing::warn;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RateLimitBucket {
    Read,
    Mutation,
}

impl RateLimitBucket {
    fn from_method(method: Method) -> Self {
        if method.supports_payload() {
            RateLimitBucket::Mutation
        } else {
            RateLimitBucket::Read
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum RateLimitIdentity {
    Ip(String),
    User(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RateLimitKey {
    identity: RateLimitIdentity,
    bucket: RateLimitBucket,
}

#[derive(Debug, Clone)]
struct Counter {
    window_start: Instant,
    count: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct RateLimitConfig {
    pub read_limit: u32,
    pub mutation_limit: u32,
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            read_limit: 300,
            mutation_limit: 60,
            window: Duration::from_secs(60),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RateLimiter {
    config: RateLimitConfig,
    counters: Mutex<HashMap<RateLimitKey, Counter>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            counters: Mutex::new(HashMap::new()),
        }
    }

    async fn check(&self, identities: &[RateLimitIdentity], bucket: RateLimitBucket) -> RateLimitDecision {
        let limit = self.limit_for_bucket(bucket);
        let now = Instant::now();
        let mut counters = self.counters.lock().await;
        let mut retry_after: Option<Duration> = None;

        for identity in identities {
            let key = RateLimitKey {
                identity: identity.clone(),
                bucket,
            };
            let counter = counters.entry(key).or_insert_with(|| Counter {
                window_start: now,
                count: 0,
            });

            if now.duration_since(counter.window_start) >= self.config.window {
                counter.window_start = now;
                counter.count = 0;
            }

            if counter.count >= limit {
                let elapsed = now.duration_since(counter.window_start);
                let remaining = self.config.window.saturating_sub(elapsed);
                retry_after = Some(retry_after.map_or(remaining, |current| current.max(remaining)));
            } else {
                counter.count += 1;
            }
        }

        if let Some(retry_after) = retry_after {
            RateLimitDecision::Limited { retry_after }
        } else {
            RateLimitDecision::Allow
        }
    }

    fn limit_for_bucket(&self, bucket: RateLimitBucket) -> u32 {
        match bucket {
            RateLimitBucket::Read => self.config.read_limit,
            RateLimitBucket::Mutation => self.config.mutation_limit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RateLimitDecision {
    Allow,
    Limited { retry_after: Duration },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RateLimit;

#[derive(Debug)]
pub(crate) struct RateLimitError {
    retry_after_secs: u64,
}

impl RateLimitError {
    fn new(retry_after: Duration) -> Self {
        let seconds = retry_after.as_secs().max(1);
        Self {
            retry_after_secs: seconds,
        }
    }
}

impl<'r> Responder<'r, 'static> for RateLimitError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let body = "Too Many Requests";
        Response::build()
            .status(Status::TooManyRequests)
            .header(Header::new("Retry-After", self.retry_after_secs.to_string()))
            .sized_body(body.len(), std::io::Cursor::new(body))
            .ok()
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimit {
    type Error = RateLimitError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let limiter = match request.rocket().state::<RateLimiter>() {
            Some(limiter) => limiter,
            None => return Outcome::Success(RateLimit),
        };

        let bucket = RateLimitBucket::from_method(request.method());
        let ip = request
            .client_ip()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut identities = vec![RateLimitIdentity::Ip(ip.clone())];
        if let Some(user_id) = extract_user_id(request) {
            identities.push(RateLimitIdentity::User(user_id));
        }

        match limiter.check(&identities, bucket).await {
            RateLimitDecision::Allow => Outcome::Success(RateLimit),
            RateLimitDecision::Limited { retry_after } => {
                warn!(
                    method = %request.method(),
                    uri = %request.uri(),
                    ip = %ip,
                    retry_after_secs = %retry_after.as_secs(),
                    "rate limit exceeded"
                );
                Outcome::Error((Status::TooManyRequests, RateLimitError::new(retry_after)))
            }
        }
    }
}

impl<'a> OpenApiFromRequest<'a> for RateLimit {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }

    fn get_responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        responses.responses.insert(
            "429".to_string(),
            RefOr::Object(OpenApiResponse {
                description: "Too Many Requests".to_string(),
                ..Default::default()
            }),
        );
        Ok(responses)
    }
}

fn extract_user_id(request: &Request<'_>) -> Option<String> {
    let cookie = request.cookies().get_private("user")?;
    let (id_str, _) = cookie.value().split_once(':')?;
    let id = Uuid::parse_str(id_str).ok()?;
    Some(id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rocket::async_test]
    async fn rate_limiter_blocks_after_limit() {
        let limiter = RateLimiter::new(RateLimitConfig {
            read_limit: 2,
            mutation_limit: 1,
            window: Duration::from_secs(60),
        });

        let identities = vec![RateLimitIdentity::Ip("127.0.0.1".to_string())];

        assert!(matches!(
            limiter.check(&identities, RateLimitBucket::Read).await,
            RateLimitDecision::Allow
        ));
        assert!(matches!(
            limiter.check(&identities, RateLimitBucket::Read).await,
            RateLimitDecision::Allow
        ));
        assert!(matches!(
            limiter.check(&identities, RateLimitBucket::Read).await,
            RateLimitDecision::Limited { .. }
        ));
    }
}
