use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use Acceptor::{Value, Promise};
use anyhow::{Result, Context};
use axum::{routing::post, Json, Extension};
use clap::Parser;
use serde::{Serialize, Deserialize};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    id: usize
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config = Config::new(
        args.id,
        Vec::from([
            (1, "http://0.0.0.0:8000"),
            (2, "http://0.0.0.0:8001"),
            (3, "http://0.0.0.0:8002")
        ])
    );

    let paxos = Paxos::new(config);

    let router = axum::Router::new()
        .route("/", post(client_propose))
        .route("/acceptor/promise", post(acceptor_promise));

    axum::Server::bind(&config.get_current_node().unwrap().addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn client_propose(Extension(paxos): Extension<Arc<Mutex<Paxos>>>, Json(payload): Json<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
    let value = payload.get("value").context("poggers")?.to_string();

    let mut paxos = paxos.try_lock().unwrap();

    paxos.proposer.prepare(value);

    Ok(())
}

#[derive(Deserialize)]
struct PrepareMessage {
    pub id: usize,
    pub value: Value
}

async fn acceptor_promise(Extension(paxos): Extension<Arc<Mutex<Paxos>>>, Json(prepare_message): Json<PrepareMessage>) -> Result<Json<Promise>, String> {
    let paxos = paxos.try_lock().unwrap();
 
    match paxos.acceptor.promise(prepare_message.value) {
        Ok(promise) => Ok(Json(promise)),
        Err(err) => Err(err.to_string())
    }
}

#[derive(Debug)]
struct Node {
    pub id: usize,
    pub addr: SocketAddr
}

impl Node {
    fn new(id: usize, addr: SocketAddr) -> Self {
        Self { id, addr }
    }
}

#[derive(Debug)]
struct Config {
    pub id: usize,
    pub nodes: Vec<Node>
}

impl Config {
    fn get_current_node(&self) -> Result<&Node> {
        let node = self
            .nodes
            .iter()
            .filter(|node| node.id == self.id)
            .next()
            .context(format!("node with id id={} was not found", self.id))?;

        Ok(node)
    }
}

impl Config {
    fn new(id: usize, addresses: Vec<(usize, &'static str)>) -> Self {
        let mut nodes = Vec::new();

        for (id, addr) in addresses {
            nodes.push(Node::new(id, addr.parse().unwrap()));
        }

        Self { id, nodes }
    }
}

struct Paxos {
    pub config: Config,
    pub proposer: Proposer::Proposer,
    pub acceptor: Acceptor::Acceptor,
}

impl Paxos {
    fn new(config: Config) -> Self {
        Self { 
            config, 
            proposer: Proposer::Proposer::new(config), 
            acceptor: Acceptor::Acceptor::new(config)
        }
    }
}

mod Proposer {
    use serde_json::json;

    use crate::Acceptor::{Value, Promise};
    use crate::Config;

    pub struct Proposer {
        pub id: usize,
        pub config: Config,
        pub promises: Vec<Promise>
    }

    impl Proposer {
        pub fn new(config: Config) -> Self {
            Self {
                id: 0,
                config, 
                promises: Vec::new(),
            }
        }
    }

    impl Proposer {
        pub fn prepare(&mut self, value: String) {
            self.id += 1;

            let value = Value { id: self.id, value };

            let client = reqwest::Client::new();

            self.config.nodes.iter().map(|node| {
                client.post(node.addr.to_string())
                    .body(json!(value).to_string())
                    .send()
            });
        }
    }

    #[derive(Clone)]
    pub struct Propose(pub Value);
}

mod Acceptor {
    use anyhow::{Result, anyhow};
    use serde::{Deserialize, Serialize};

    use crate::{Proposer::Propose, Config, PrepareMessage};

    pub struct Acceptor {
        max_id: usize,
        accepted_propose: Option<Propose>,
        config: Config
    }

    impl Acceptor {
        pub fn new(config: Config) -> Self {
            Self { max_id: 0, accepted_propose: None, config }
        }
    }

    impl Acceptor {
        pub fn handle_prepare(&mut self, prepare_message: PrepareMessage) -> Result<Promise> {
            if prepare_message.id < self.max_id {
                return Err(anyhow!("already accepted a propose with a higher id"));
            }

            self.max_id = prepare_message.id;

            Ok(Promise(prepare_message.value))
        }
    }

    pub struct Promise(Value);
    impl Promise {
        pub fn new(value: Value) -> Self {
            Self(value)
        }
    }

    #[derive(Clone, Deserialize, Serialize)]
    pub struct Value {
        pub id: usize,
        pub value: String,
    }
}
