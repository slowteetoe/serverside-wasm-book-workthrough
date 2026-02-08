use std::time::Duration;

use wasmcloud_component::http;
use wasmcloud_component::wasi::keyvalue::*;

struct Component;

http::export!(Component);

impl http::Server for Component {
    fn handle(
        _request: http::IncomingRequest,
    ) -> http::Result<http::Response<impl http::OutgoingBody>> {
        let bucket = store::open("default").unwrap();
        let count = atomics::increment(&bucket, "counter", 1).unwrap();
        std::thread::sleep(Duration::from_secs(2));
        Ok(http::Response::new(format!(
            "Hello from Rust! I was called {count} times.\n"
        )))
    }
}
