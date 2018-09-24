use codec::{Msg, MsgCodec};
use futures::sync::mpsc::{self, UnboundedSender};
use futures::{Future, Sink, Stream};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::codec::{Decoder, Encoder};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

type Sender = UnboundedSender<Msg>;

pub struct Node {
    inner: Arc<Mutex<NodeInner>>,
    addr: SocketAddr,
}

impl Node {
    pub fn new(addr: SocketAddr) -> Self {
        let inner = NodeInner::new(&addr);

        Node {
            inner: Arc::new(Mutex::new(inner)),
            addr,
        }
    }

    pub fn run(
        &self,
        addrs: Vec<SocketAddr>,
    ) -> impl Future<Item = (), Error = io::Error> {
        let inner = self.inner.clone();

        for addr in addrs {
            let client = Node::connect(inner.clone(), addr);
            tokio::spawn(client.then(|_| Ok(())));
        }

        self.serve()
    }

    fn connect(
        inner: Arc<Mutex<NodeInner>>,
        addr: SocketAddr,
    ) -> impl Future<Item = (), Error = io::Error> {
        let inner = inner.clone();
        TcpStream::connect(&addr).and_then(move |stream| {
            let (sink, stream) = MsgCodec.framed(stream).split();
            let (tx, rx) = mpsc::unbounded();

            let inner = inner.clone();
            let read = stream.for_each(move |msg| Node::process(inner.clone(), msg, tx.clone()));
            tokio::spawn(read.then(|_| Ok(())));

            let write =
                sink.send_all(rx.map_err(|_| {
                    io::Error::new(io::ErrorKind::Other, "rx shouldn't have an error")
                }));
            tokio::spawn(write.then(|_| Ok(())));

            Ok(())
        })
    }

    fn serve(&self) -> impl Future<Item = (), Error = io::Error> {
        let socket = TcpListener::bind(&self.addr).unwrap();

        let inner = self.inner.clone();
        socket.incoming().for_each(move |stream| {
            let (sink, stream) = MsgCodec.framed(stream).split();
            let (tx, rx) = mpsc::unbounded();

            let inner_clone = inner.clone();
            let read =
                stream.for_each(move |msg| Node::process(inner_clone.clone(), msg, tx.clone()));
            tokio::spawn(read.map_err(|_| ()));

            let write =
                sink.send_all(rx.map_err(|()| {
                    io::Error::new(io::ErrorKind::Other, "rx shouldn't have an error")
                }));
            tokio::spawn(write.map(|_| ()).map_err(|_| ()));

            Ok(())
        })
    }

    fn process(inner: Arc<Mutex<NodeInner>>, msg: Msg, tx: Sender) -> Result<(), io::Error> {
        match msg {
            Msg::Connect => Ok(()),
        }
    }
}

struct NodeInner {
    id: Uuid,
    addr: SocketAddr,
    peers: HashMap<Uuid, (SocketAddr, Sender)>,
}

impl NodeInner {
    pub fn new(addr: &SocketAddr) -> Self {
        NodeInner {
            id: Uuid::new_v4(),
            addr: addr.clone(),
            peers: HashMap::new(),
        }
    }
}
