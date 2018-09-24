extern crate p2pcluster;
extern crate tokio;
extern crate futures;

use p2pcluster::Node;
use futures::Future;

fn main() {
    let node = Node::new("127.0.0.1:3334".parse().unwrap());
    let peers = vec!["127.0.0.1:3334".parse().unwrap()];
    tokio::run(node.run(peers).then(|_| Ok(())));
}
