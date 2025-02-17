use std::{collections::HashMap, sync::Arc};

use governor::DefaultDirectRateLimiter;
use hyper::server::{conn::AddrIncoming, Builder};

use casper_json_rpc::{ConfigLimit, CorsOrigin, RequestHandlersBuilder};

use crate::{
    node_client::NodeClient,
    rpcs::{
        speculative_exec::{SpeculativeExec, SpeculativeExecTxn, SpeculativeRpcDiscover},
        RpcWithParams, RpcWithoutParams,
    },
};

/// The URL path for all JSON-RPC requests.
pub const SPECULATIVE_EXEC_API_PATH: &str = "rpc";

pub const SPECULATIVE_EXEC_SERVER_NAME: &str = "speculative execution";

/// Run the speculative execution server.
pub async fn run(
    node: Arc<dyn NodeClient>,
    builder: Builder<AddrIncoming>,
    limits: HashMap<String, ConfigLimit>,
    max_body_bytes: u64,
    cors_origin: String,
) {
    let mut handlers = RequestHandlersBuilder::new();
    let mut limiters = HashMap::new();

    macro_rules! register {
        ($rpc:ident) => {
            $rpc::register_as_handler(node.clone(), &mut handlers);
            if let Some(config_limit) = limits.get($rpc::METHOD) {
                let limiter = DefaultDirectRateLimiter::direct(config_limit.quota());
                limiters.insert($rpc::METHOD, limiter);
            }
        };
    }

    register!(SpeculativeExecTxn);
    register!(SpeculativeExec);
    register!(SpeculativeRpcDiscover);

    let handlers = handlers.build();

    match cors_origin.as_str() {
        "" => {
            super::rpcs::run(
                builder,
                handlers,
                limiters,
                max_body_bytes,
                SPECULATIVE_EXEC_API_PATH,
                SPECULATIVE_EXEC_SERVER_NAME,
            )
            .await;
        }
        "*" => {
            super::rpcs::run_with_cors(
                builder,
                handlers,
                limiters,
                max_body_bytes,
                SPECULATIVE_EXEC_API_PATH,
                SPECULATIVE_EXEC_SERVER_NAME,
                CorsOrigin::Any,
            )
            .await;
        }
        _ => {
            super::rpcs::run_with_cors(
                builder,
                handlers,
                limiters,
                max_body_bytes,
                SPECULATIVE_EXEC_API_PATH,
                SPECULATIVE_EXEC_SERVER_NAME,
                CorsOrigin::Specified(cors_origin),
            )
            .await;
        }
    }
}
