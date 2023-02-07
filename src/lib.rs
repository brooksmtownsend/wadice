use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_logging::info;
use wasmcloud_interface_numbergen::random_in_range;

/// Embedding favicon.ico for optimal tab experience
#[derive(rust_embed::RustEmbed)]
#[folder = "static/"]
struct Asset;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct WadiceActor {}

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for WadiceActor {
    async fn handle_request(&self, _ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
        if req.path.contains("favicon.ico") {
            if let Some(favicon) = Asset::get("favicon.ico") {
                return Ok(HttpResponse {
                    body: favicon.data.to_vec(),
                    ..Default::default()
                });
            }
        }

        let path = req.path.trim_matches('/');
        let roll = if path.is_empty() {
            "1d20".to_string()
        } else {
            path.to_string()
        };

        let die_rolls = d20::parse_die_roll_terms(&roll);

        let mut total = 0;
        for roll in die_rolls {
            total += match roll {
                d20::DieRollTerm::Modifier(n) => n,
                d20::DieRollTerm::DieRoll {
                    multiplier: m,
                    sides: s,
                } => random_in_range(1, s as u32).await? as i8 * m,
            }
        }

        let resp_body = format!(
            r#"<!DOCTYPE html>
        <html>
        <head>
            <title>Roll: {}</title>
        </head>
        <body>
            <h1>You rolled a {}</h1>
        </body>
        </html>
        "#,
            total, total
        );

        info!("A user rolled a {}!", total);

        Ok(HttpResponse {
            body: resp_body.into_bytes(),
            ..Default::default()
        })
    }
}
