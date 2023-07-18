extern crate futures;
extern crate grpcio;
extern crate protos;

use std::{
    io::{self, Read as _},
    sync::Arc,
    thread,
};

use futures::{channel::oneshot, executor::block_on, prelude::*};

use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};

use protos::{
    diner::{Check, Item, Order},
    diner_grpc::{self, Diner},
};

#[derive(Clone)]
struct DinerService;

impl Diner for DinerService {
    fn eat(&mut self, ctx: RpcContext, order: Order, sink: UnarySink<Check>) {
        println!("Received Order {{ {:?} }}", order);
        let mut check = Check::new();
        check.set_total(order.get_items().iter().fold(0.0, |total, &item| {
            total
                + match item {
                    Item::SPAM => 0.05,
                    Item::EGGS => 0.25,
                    Item::HAM => 1.0,
                }
        }));
        let f = sink
            .success(check.clone())
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err))
            .map(move |_| println!("Responded with Check {{ {:?} }}", check));
        ctx.spawn(f)
    }
}

fn main() {
    let env = Arc::new(Environment::new(1));
    let service = diner_grpc::create_diner(DinerService);
    let mut server = ServerBuilder::new(env)
        .register_service(service)
        .bind("127.0.0.1", 0)
        .build()
        .unwrap();
    server.start();
    for (ref host, port) in server.bind_addrs() {
        println!("listening on {}:{}", host, port);
    }
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        println!("Press ENTER to exit...");
        let _ = io::stdin().read(&mut [0]).unwrap();
        tx.send(())
    });
    let _ = block_on(rx);
    let _ = block_on(server.shutdown());
}
