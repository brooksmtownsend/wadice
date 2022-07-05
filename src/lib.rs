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

        Ok(HttpResponse {
            body: format!("Your die roll is: {}", total).as_bytes().to_vec(),
            ..Default::default()
        })
    }
}
