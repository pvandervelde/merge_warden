// Merge Warden server binary entry point.
//
// See docs/spec/interfaces/server-config.md    — startup configuration
// See docs/spec/interfaces/server-ingress.md   — event ingress abstraction
// See docs/spec/design/containerisation.md     — deployment spec

mod config;
mod errors;
mod ingress;
mod telemetry;
mod webhook;

use errors::ServerError;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    todo!("See docs/spec/interfaces/server-config.md and docs/spec/interfaces/server-ingress.md")
}
