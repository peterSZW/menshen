#[macro_use]
extern crate log;

use log::{Level, LevelFilter, Metadata, Record};

static MY_LOGGER: MyLogger = MyLogger;
struct MyLogger;

impl log::Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
    fn flush(&self) {}
}


 

extern crate env_logger;

//extern crate borderland;
extern crate byteorder;
extern crate bytes;
extern crate mio;
extern crate net2;
extern crate slab;

use std::env;
 

use net2::unix::UnixTcpBuilderExt;
use net2::TcpBuilder;

use mio::net::*;
 

use mio::Poll;
use std::net::SocketAddr;


use std::fmt::Write;
 

use std::thread;

mod server;
use server::*;


mod connection;
use connection::*;




pub fn write_str(dest: &mut String, s: &str) {
    write!(dest, "{}", s)
        .map_err(|e| panic!("error writing to string: {}", e))
        .unwrap();
}

const HTTP_RES: &'static str = "HTTP/1.1 200 OK
Date: Sun, 22 Nov 2015 01:00:44 GMT
Server: miohack
Connection: $Connection$
Content-Length: $Content-Length$

$Content$";

fn mainEntry() {
    
//    env_logger::init();

    let res_size = env::var("RES_SIZE")
        .unwrap_or("0".to_string())
        .parse()
        .unwrap();

    let content = if res_size == 0 {
        String::from("Have a nice day.")
    } else {
        let res_base = String::from("DEADBEEF");
        let res_base_len = res_base.len();
        let mut written = 0;
        let mut content = String::new();
        while written < res_size {
            write_str(&mut content, &res_base);
            written = written + res_base_len
        }
        content
    };

    let cl = content.to_owned().into_bytes().len();
    println!("response body size: {} bytes", cl);

    let res = String::from(HTTP_RES)
        .replace("$Connection$", "Keep-Alive")
        .replace("$Content$", &content)
        .replace("$Content-Length$", &cl.to_string());

    let res_bytes = res.to_owned().into_bytes();

    println!(
        "full response size (including headers): {}",
        res_bytes.len()
    );
    // println!("buffer capacity: {}", READ_WRITE_BUF_CAP);
    // let res_f = res_bytes.len() as f32 / READ_WRITE_BUF_CAP as f32;
    // println!("expected number of writes per request: {}", res_f.ceil());

    // let res = ResponseData { data: res_bytes };

    let threads = env::var("THREADS")
        .unwrap_or("2".to_string())
        .parse()
        .unwrap();
    println!("multi-threaded server starting: {} threads", threads);
    let mut children = Vec::new();

    let addr = "127.0.0.1:8000"
        .parse::<SocketAddr>()
        .expect("Failed to parse host:port string");

    let tcp = TcpBuilder::new_v4().unwrap();
    tcp.reuse_address(true).unwrap();
    tcp.reuse_port(true).unwrap();
    tcp.bind(addr).unwrap();

    println!("LISTEN {:?}", addr);

    let listener = tcp.listen(4096).unwrap();
    let listener = TcpListener::from_std(listener).unwrap();
    //     let sock = TcpListener::bind(&addr).expect("Failed to bind address");

    for i in 0..threads {
        let listener = listener.try_clone().unwrap();
        // let res = res.clone();

        children.push(thread::spawn(move || {
            // let srv = listener;
            // let res = Rc::from(res);
            let mut poll = Poll::new().expect("Failed to create Poll");

            println!("thread {} accepting connections", i);

            // Create our Server object and start polling for events. I am hiding away
            // the details of how registering works inside of the `Server` object. One reason I
            // really like this is to get around having to have `const SERVER = Token(0)` at the top of my
            // file. It also keeps our polling options inside `Server`.
            let mut server = Server::new(listener);
            server.run(&mut poll).expect("Failed to run server");
        }));
    }

    for child in children {
        child.join().unwrap();
    }
    println!("joined");
}

fn main() {
//	env_logger::init();

    log::set_logger(&MY_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
    info!("hello log");
    warn!("warning");
    error!("oops");

    println!("Hello, world!");
    mainEntry();
}
