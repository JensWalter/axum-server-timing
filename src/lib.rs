use axum::http::{HeaderValue, Request, Response};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{ready, Context, Poll},
    time::Instant,
};
use tower::{Layer, Service};

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

impl<'a, S, ReqBody, ResBody> Service<Request<ReqBody>> for ServerTimingService<'a, S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<'a, S::Future>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let (parts, body) = req.into_parts();

        let req = Request::from_parts(parts, body);
        ResponseFuture {
            inner: self.service.call(req),
            request_time: Instant::now(),
            app: self.app,
            description: self.description,
        }
    }
}

pin_project! {
    pub struct ResponseFuture<'a, F> {
        #[pin]
        inner: F,
        request_time: Instant,
        app: &'a str,
        description: Option<&'a str>,
    }
}

impl<F, B, E> Future for ResponseFuture<'_, F>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Default,
{
    type Output = Result<Response<B>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let time = self.request_time;
        let app = self.app;
        let description = self.description;
        let mut response: Response<B> = ready!(self.project().inner.poll(cx))?;
        let hdr = response.headers_mut();
        let x = time.elapsed().as_millis();
        let header_value = match description {
            Some(val) => format!("{app};desc=\"{val}\";dur={x}"),
            None => format!("{app};dur={x}"),
        };
        match hdr.try_entry("Server-Timing") {
            Ok(entry) => {
                match entry {
                    axum::http::header::Entry::Occupied(mut val) => {
                        //has val
                        let old_val = val.get();
                        let new_val = format!("{header_value}, {}", old_val.to_str().unwrap());
                        val.insert(HeaderValue::from_str(&new_val).unwrap());
                    }
                    axum::http::header::Entry::Vacant(val) => {
                        val.insert(HeaderValue::from_str(&header_value).unwrap());
                    }
                }
            }
            Err(_) => {
                hdr.append(
                    "Server-Timing",
                    HeaderValue::from_str(&header_value).unwrap(),
                );
            }
        }

        Poll::Ready(Ok(response))
    }
}
