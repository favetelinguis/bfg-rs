use color_eyre::eyre;
use rustls::{ClientConfig, ClientConnection, RootCertStore, Stream, StreamOwned};
use serde::{de, Deserialize, Serialize};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

mod model;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationMessage {
    op: String,
    id: usize,
    app_key: String,
    session: String,
}

impl AuthenticationMessage {
    pub fn new(app_key: &str, session: &str) -> Self {
        Self {
            op: String::from("authentication"),
            id: 0,
            app_key: String::from(app_key),
            session: String::from(session),
        }
    }
}

impl SetId for AuthenticationMessage {
    fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketFilter {
    country_codes: Option<Vec<String>>,
    betting_types: Option<Vec<String>>,
    turn_in_play_enabled: Option<bool>,
    market_types: Option<Vec<String>>,
    venues: Option<Vec<String>>,
    market_ids: Option<Vec<String>>,
    event_type_ids: Option<Vec<String>>,
    event_ids: Option<Vec<String>>,
    bsp_market: Option<bool>,
    race_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketDataFilter {
    ladder_levels: Option<i32>,
    fields: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MarketSubscriptionMessage {
    op: String,
    id: usize,
    segmentation_enabled: Option<bool>,
    conflate_ms: Option<i32>,
    heartbeat_ms: Option<i32>,
    initial_clk: Option<String>,
    clk: Option<String>,
    market_filter: MarketFilter,
    market_data_filter: MarketDataFilter,
}

trait SetId {
    fn set_id(&mut self, id: usize);
}

type MarketCache = Vec<usize>; // Placeholder
type StatusCache = Vec<usize>; // Placeholder

pub struct LinesCodec {
    stream: io::BufReader<StreamOwned<ClientConnection, TcpStream>>,
    num_msg: usize,
}

impl LinesCodec {
    pub fn new() -> eyre::Result<Self> {
        let root_store = RootCertStore {
            // TODO only add the server cert for the endpint i need
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };
        let mut config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let server_name = "stream-api.betfair.com".try_into()?;
        let mut conn = ClientConnection::new(Arc::new(config), server_name)?;
        let mut sock = TcpStream::connect("stream-api.betfair.com:443")?;
        // let mut tls = rustls::Stream::new(&mut conn, &mut sock);
        let mut tls = rustls::StreamOwned::new(conn, sock);
        // Both BufReader and LineWriter need to own a stream
        // We can clone the stream to simulate splitting Tx & Rx
        // let writer = io::LineWriter::new(tls.sock.try_clone()?);
        let stream = io::BufReader::new(tls);
        Ok(Self {
            // tls,
            // reader,
            // writer,
            stream,
            num_msg: 0,
        })
    }

    pub fn send_message<T>(&mut self, mut message: T) -> eyre::Result<()>
    where
        T: Serialize + SetId,
    {
        self.num_msg += 1;
        message.set_id(self.num_msg);

        let json = serde_json::to_string(&message)?;
        self.stream.get_mut().write_all(&json.as_bytes())?;
        self.stream.get_mut().write_all(&[b'\r', b'\n'])?;
        let num = self.stream.get_mut().flush()?;
        Ok(())
    }

    pub fn read_message(&mut self) -> eyre::Result<model::ResponseMessage> {
        let mut line = String::new();
        self.stream.read_line(&mut line)?;
        line.pop(); // Remove \r
        line.pop(); // Remove \n
        let res = serde_json::from_str(line.as_str())?;
        Ok(res)
    }
}
