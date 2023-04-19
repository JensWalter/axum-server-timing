use std::{time::Instant, task::{Poll, Context, ready}, future::{Future}, pin::Pin};

use axum::http::{Request, Response, HeaderValue};
use pin_project_lite::pin_project;
use tower::{Service, Layer};

#[derive(Debug, Clone)]
pub struct ServerTimingLayer {}

impl ServerTimingLayer {
    pub fn new() -> Self {
        ServerTimingLayer {}
    }
}

impl<S> Layer<S> for ServerTimingLayer {
    type Service = ServerTimingService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ServerTimingService {
            service,
        }
    }
}

#[derive(Clone)]
pub struct ServerTimingService<S> {
    service: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for ServerTimingService<S>
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

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let (parts, body) = req.into_parts();

            let req = Request::from_parts(parts, body);
            ResponseFuture {
                // inner: Kind::CorsCall {
                    inner: self.service.call(req),
                // },
                request_time: Instant::now(),
            }
    }
}


pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
        inner: F,
        #[pin]
        request_time: Instant,
    }
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Default,
{
    type Output = Result<Response<B>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let time = self.request_time;
        let mut response: Response<B> = ready!(self.project().inner.poll(cx))?;
        let hdr = response.headers_mut();
        let x = time.elapsed().as_millis();
hdr.append("Server-Timing", HeaderValue::from_str(&format!("{x}")).unwrap());
        Poll::Ready(Ok(response))
    }
}