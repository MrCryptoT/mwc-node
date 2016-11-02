// Copyright 2016 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Grin server implementation, accepts incoming connections and connects to
//! other peers in the network.

use std::cell::RefCell;
use std::io;
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use mioco;
use mioco::sync::mpsc::{sync_channel, SyncSender};
use mioco::tcp::{TcpListener, TcpStream};

use core::core;
use core::ser::Error;
use handshake::Handshake;
use peer::Peer;
use types::*;

/// Default address for peer-to-peer connections, placeholder until better
/// configuration is in place.
pub const DEFAULT_LISTEN_ADDR: &'static str = "127.0.0.1:3414";

// replace with some config lookup or something
fn listen_addr() -> SocketAddr {
	FromStr::from_str(DEFAULT_LISTEN_ADDR).unwrap()
}

pub struct DummyAdapter {}
impl NetAdapter for DummyAdapter {
	fn transaction_received(&self, tx: core::Transaction) {}
	fn block_received(&self, b: core::Block) {}
}

/// P2P server implementation, handling bootstrapping to find and connect to
/// peers, receiving connections from other peers and keep track of all of them.
pub struct Server {
	peers: RwLock<Vec<Arc<Peer>>>,
	stop_send: RefCell<Option<SyncSender<u8>>>,
}

unsafe impl Sync for Server {}
unsafe impl Send for Server {}

// TODO TLS
impl Server {
	/// Creates a new idle p2p server with no peers
	pub fn new() -> Server {
		Server {
			peers: RwLock::new(Vec::new()),
			stop_send: RefCell::new(None),
		}
	}
	/// Starts the p2p server. Opens a TCP port to allow incoming
	/// connections and starts the bootstrapping process to find peers.
	pub fn start(&self) -> Result<(), Error> {
		let addr = DEFAULT_LISTEN_ADDR.parse().unwrap();
		let listener = try!(TcpListener::bind(&addr).map_err(&Error::IOErr));
		warn!("P2P server started on {}", addr);

		let hs = Arc::new(Handshake::new());
		let (stop_send, stop_recv) = sync_channel(1);
		{
			let mut stop_mut = self.stop_send.borrow_mut();
			*stop_mut = Some(stop_send);
		}

		loop {
			select!(
        r:listener => {
          let conn = try!(listener.accept().map_err(&Error::IOErr));
          let hs = hs.clone();

          let peer = try!(Peer::connect(conn, &hs));
          let wpeer = Arc::new(peer);
          {
            let mut peers = self.peers.write().unwrap();
            peers.push(wpeer.clone());
            println!("len {}", peers.len())
          }

          mioco::spawn(move || -> io::Result<()> {
            if let Err(err) = wpeer.run(&DummyAdapter{}) {
              error!("{:?}", err);
            }
            Ok(())
          });
        },
        r:stop_recv => {
          stop_recv.recv();
          return Ok(());
        }
      );
		}
	}

	/// Asks all the peers to relay the provided block. A peer may choose to
	/// ignore the relay request if it has knowledge that the remote peer
	/// already knows the block.
	pub fn relay_block(&self, b: &core::Block) -> Result<(), Error> {
		let peers = self.peers.write().unwrap();
		for p in peers.deref() {
			try!(p.send_block(b));
		}
		Ok(())
	}

	/// Asks all the peers to relay the provided transaction. A peer may choose
	/// to ignore the relay request if it has knowledge that the remote peer
	/// already knows the transaction.
	pub fn relay_transaction(&self, tx: &core::Transaction) -> Result<(), Error> {
		let peers = self.peers.write().unwrap();
		for p in peers.deref() {
			try!(p.send_transaction(tx));
		}
		Ok(())
	}

	/// Stops the server. Disconnect from all peers at the same time.
	pub fn stop(&self) {
		let peers = self.peers.write().unwrap();
		for p in peers.deref() {
			p.stop();
		}
		let stop_send = self.stop_send.borrow();
		stop_send.as_ref().unwrap().send(0);
	}

	/// Simulates an unrelated client connecting to our server. Mostly used for
	/// tests.
	pub fn connect_as_client(addr: SocketAddr) -> Result<Peer, Error> {
		let tcp_client = TcpStream::connect(&addr).unwrap();
		Peer::accept(tcp_client, &Handshake::new())
	}

	pub fn peers_count(&self) -> u32 {
		self.peers.read().unwrap().len() as u32
	}
}
