use http::{HeaderValue, Request, Response};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{ready, Context, Poll},
    time::Instant,
};
use tower::{Layer, Service};

#[allow(dead_code)]
pub type ServerTimingExtension = Arc<Mutex<ServerTiming>>;

#[derive(Debug)]
pub struct ServerTiming {
    app: String,
    description: Option<String>,
    created: Instant,
    data: Vec<ServerTimingData>,
}

impl ServerTiming {
    pub fn record(&mut self, name: String, description: Option<String>) {
        self.data.push(ServerTimingData {
            name,
            description,
            created: Instant::now(),
        });
    }
}

#[derive(Debug)]
pub struct ServerTimingData {
    name: String,
    description: Option<String>,
    created: Instant,
}

#[cfg(test)]
mod test;

#[derive(Debug, Clone)]
pub struct ServerTimingLayer<'a> {
    app: &'a str,
    description: Option<&'a str>,
}

impl<'a> ServerTimingLayer<'a> {
    pub fn new(app: &'a str) -> Self {
        ServerTimingLayer {
            app,
            description: None,
        }
    }

    pub fn with_description(&mut self, description: &'a str) -> Self {
        let mut new_self = self.clone();
        new_self.description = Some(description);
        new_self
    }
}

impl<'a, S> Layer<S> for ServerTimingLayer<'a> {
    type Service = ServerTimingService<'a, S>;

    fn layer(&self, service: S) -> Self::Service {
        ServerTimingService {
            service,
            app: self.app,
            description: self.description,
        }
    }
}

#[derive(Clone)]
pub struct ServerTimingService<'a, S> {
    service: S,
    app: &'a str,
    description: Option<&'a str>,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for ServerTimingService<'_, S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let timings = ServerTiming {
            app: self.app.to_string(),
            created: Instant::now(),
            description: self.description.map(|elem| elem.to_string()),
            data: vec![],
        };
        let x = Arc::new(Mutex::new(timings));
        req.extensions_mut().insert(x.clone());

        let (parts, body) = req.into_parts();

        let req = Request::from_parts(parts, body);
        ResponseFuture {
            inner: self.service.call(req),
            timings: x,
        }
    }
}

pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
        inner: F,
        timings: Arc<Mutex<ServerTiming>>,
    }
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Default,
{
    type Output = Result<Response<B>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let timing = self.timings.clone();
        let mut response: Response<B> = ready!(self.project().inner.poll(cx))?;
        let hdr = response.headers_mut();
        // TODO: Once stable for a while, use `as_millis_f32`
        let timing_after = timing.lock().unwrap();
        let x = timing_after.created.elapsed().as_secs_f32() * 1000.0;
        let app = timing_after.app.clone();
        let mut header_value = match &timing_after.description {
            Some(val) => format!("{app};desc=\"{val}\";dur={x:.2}"),
            None => format!("{app};dur={x:.2}"),
        };
        let mut ts = timing_after.created;
        for data in timing_after.data.iter() {
            let x = (data.created - ts).as_secs_f32() * 1000.0;
            ts = data.created;
            let name = data.name.clone();
            let newval = match &data.description {
                Some(val) => format!("{name};desc=\"{val}\";dur={x:.2}"),
                None => format!("{name};dur={x:.2}"),
            };
            header_value = format!("{header_value}, {newval}");
        }
        match hdr.try_entry("Server-Timing") {
            Ok(entry) => {
                match entry {
                    http::header::Entry::Occupied(mut val) => {
                        //has val
                        let old_val = val.get();
                        let new_val = format!("{header_value}, {}", old_val.to_str().unwrap());
                        val.insert(HeaderValue::from_str(&new_val).unwrap());
                    }
                    http::header::Entry::Vacant(val) => {
                        val.insert(HeaderValue::from_str(&header_value).unwrap());
                    }
                }
            }
            Err(_) => {
                // header name was invalid (it wasn't) or too many headers (just give up).
            }
        }

        Poll::Ready(Ok(response))
    }
}
