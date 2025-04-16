use middleware::extractor::ExtractionMiddleware;

pub mod middleware {
    pub mod extractor;
}

pub fn middleware() -> ExtractionMiddleware {
    ExtractionMiddleware::new()
}
