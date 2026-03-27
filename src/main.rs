use std::vec::Vec;
use std::collections::HashMap;

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

use polling::{Event, Events, Poller};

use clap::Parser;

struct TcpStreamSource {
    key: usize,
    stream: TcpStream,
    // write_buf: Vec<u8>,
    read_buf: Vec<u8>,
}

impl TcpStreamSource {
    fn new(key: usize, stream: TcpStream) -> TcpStreamSource {
        TcpStreamSource {
            key,
            stream,
            // write_buf: Vec::new(),
            read_buf: Vec::new(),
        }
    }
}


enum EventSource {
    Listener(TcpListener),
    Stream(TcpStreamSource),
}

struct EventCallback {
    on_read: Option<Box<dyn Fn()>>,
    on_write: Option<Box<dyn Fn()>>,
}

enum Timer {
    OnceTimer(Box<dyn FnOnce()>),
    RepeatTimer(Box<dyn Fn()>),
}

struct EventLoop {
    poller: Poller,
    events: HashMap<usize, EventCallback>,
    timers: Vec<Timer>,
    stop: bool,
    fds: HashMap<usize, EventSource>,
    next_key: usize,
}

impl EventLoop {
    fn new() -> io::Result<EventLoop> {
        Ok(EventLoop {
            poller: Poller::new()?,
            events: HashMap::new(),
            timers: Vec::new(),
            stop: false,
            fds: HashMap::new(),
            next_key: 1,
        })
    }

    fn listen(&mut self, addr: &str) -> io::Result<()> {
        let socket = TcpListener::bind(addr)?;
        socket.set_nonblocking(true)?;
        let key = self.get_key();
        unsafe{ self.poller.add(&socket, Event::all(key))?};
        self.fds
            .insert(key, EventSource::Listener(socket));
        Ok(())
    }

    fn get_key(&mut self) -> usize {
        let key = self.next_key;
        self.next_key += 1;
        key
    }

    fn run(&mut self) -> io::Result<()> {
        let mut events = Events::new();

        while !self.stop {
            events.clear();
            println!("poll");
            self.poller.wait(&mut events, None)?;
            println!("poll results: {}", events.len());

            for ev in events.iter() {
                println!("event: {}", ev.key);
                if let Err(msg) = self.process_event(&ev) {
                    println!("io error: {}, ignored", msg);
                }
            }
        }

        Ok(())
    }

    fn process_event(&mut self, ev: &Event) -> io::Result<()> {
              if let Some(EventSource::Listener(socket)) = self.fds.get(&ev.key) {
                  let socket = socket.try_clone()?;
          self.on_conn(&socket)?;
      }
      if let Some(EventSource::Stream(stream_source)) = self.fds.get(&ev.key) {
          if ev.writable {
              self.on_write(ev.key)?;
          }

          if ev.readable {
              self.on_read(ev.key)?;
          }
      }
      Ok(())
    }

    fn on_write(&mut self, key: usize) -> io::Result<()> {
        if let EventSource::Stream(stream_source) = self.fds.get_mut(&key).unwrap() {
            let mut socket = &stream_source.stream;
            socket.write_all(&stream_source.read_buf)?;
        }
        Ok(())
    }

    fn on_read(&mut self, key: usize) -> io::Result<()> {
        let mut buf = [0; 1024];
        if let EventSource::Stream(stream_source) = self.fds.get_mut(&key).unwrap() {
        loop {
            let mut socket = &stream_source.stream;
            let n = socket.read(&mut buf)?;
            println!("read {} bytes", n);
            unsafe { self.poller.modify(&socket, Event::readable(stream_source.key)) };
            if n == 0 {
                break;
            }
            stream_source.read_buf.extend_from_slice(&buf[0..n]);
            unsafe { self.poller.modify(&socket, Event::writable(stream_source.key)) };
            println!("pending write {} bytes", n);
        }
        Ok(())
    }
        else {
        panic!("on_read: invalid key");
    }
    }

    fn stop(&mut self) {
        self.stop = true
    }
    fn on_conn(&mut self, socket: &TcpListener) -> io::Result<()> {
        println!("on_conn");
        let (stream, _) = socket.accept()?;
        stream.set_nonblocking(true)?;
        let key = self.get_key();
        unsafe { self.poller.add(&stream, Event::readable(key))? };
        self.fds
            .insert(key, EventSource::Stream(TcpStreamSource::new(key, stream)));
        Ok(())
    }
}

#[derive(Parser)]
struct Command {
    #[clap(short, long, default_value = "8999")]
    port: u16,
}

fn main() -> io::Result<()> {
    println!("=== start ===");
    let mut ev = EventLoop::new()?;
    let args = Command::parse();
    println!("port: {}", args.port);
    ev.listen(&format!("127.0.0.1:{}", args.port))?;
    ev.run()?;
    println!("=== end ===");
    Ok(())
}

// test basic polling usage
// fn main() -> io::Result<()> {
//     println!("=== start ===");
//     let poller = Poller::new().unwrap();
//     let mut events = Events::new();
//     let socket = TcpListener::bind("127.0.0.1:8999").unwrap();
//     socket.set_nonblocking(true).unwrap();
//     unsafe { poller.add(&socket, Event::all(1)).unwrap()};
//     poller.wait(&mut events, None).unwrap();
//     println!("=== end ===");
//     Ok(())
// }
