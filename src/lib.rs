use d20::DieRollTerm;
use rust_embed::RustEmbed;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_numbergen::random_in_range;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct WadiceActor {}

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for WadiceActor {
    /// Returns a greeting, "Hello World", in the response body.
    /// If the request contains a query parameter 'name=NAME', the
    /// response is changed to "Hello NAME"
    async fn handle_request(
        &self,
        _ctx: &Context,
        req: &HttpRequest,
    ) -> std::result::Result<HttpResponse, RpcError> {
        if req.path.contains("favicon.ico") {
            if let Some(favicon) = Asset::get("favicon.ico") {
                return Ok(HttpResponse {
                    body: favicon.data.to_vec(),
                    ..Default::default()
                });
            }
        }
        let roll = form_urlencoded::parse(req.query_string.as_bytes())
            .find(|(n, _)| n == "roll")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "1d20".to_string());

        let die_rolls = d20::parse_die_roll_terms(&roll);

        let mut total = 0;
        for roll in die_rolls {
            total = total
                + match roll {
                    DieRollTerm::Modifier(n) => n,
                    DieRollTerm::DieRoll {
                        multiplier: m,
                        sides: s,
                    } => (random_in_range(1, s as u32).await? as i8 * m),
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

        wasmcloud_interface_logging::info!("A user rolled a {}!", total);

        Ok(HttpResponse {
            body: resp_body.into_bytes(),
            ..Default::default()
        })
    }
}
