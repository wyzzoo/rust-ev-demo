use std::env;

use std::collections::HashMap;
use std::vec;

use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};

use polling::{Event, Events, Poller};

enum EventSource {
    Listener(TcpListener),
    Stream(TcpStream),
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
        self.fds
            .insert(key, EventSource::Listener(socket.try_clone()?));
        unsafe { self.poller.add(&socket, Event::readable(key))? };
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
      if let Some(EventSource::Stream(socket)) = self.fds.get(&ev.key) {
                  let mut socket = socket.try_clone()?;
          self.on_read(&mut socket)?;
      }
      Ok(())
    }

    fn on_read(&mut self, socket: &mut TcpStream) -> io::Result<()> {
        let mut buf = [0; 1024];
        loop {
            let n = socket.read(&mut buf)?;
            println!("read {} bytes", n);
            if n == 0 {
                break;
            }
        }
        Ok(())
    }

    fn stop(&mut self) {
        self.stop = true
    }
    fn on_conn(&mut self, socket: &TcpListener) -> io::Result<()> {
        println!("on_conn");
        let (stream, _) = socket.accept()?;
        stream.set_nonblocking(true)?;
        let key = self.get_key();
        self.fds
            .insert(key, EventSource::Stream(stream.try_clone()?));
        unsafe { self.poller.add(&stream, Event::readable(key))? };
        Ok(())
    }
}

fn main() -> io::Result<()> {
    println!("=== start ===");
    let mut ev = EventLoop::new()?;
    ev.listen("127.0.0.1:8999")?;
    ev.run();
    println!("=== end ===");
    Ok(())
}
