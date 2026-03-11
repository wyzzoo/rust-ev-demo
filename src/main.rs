use std::vec;
use std::collections::HashMap;
//use std::os::unix::io::AsRawFd;
//use libc::{epoll_createl, epoll_ctl, epoll_wait, EPOLLIN, EPOLL_CTL_ADD};
use polling::{Event, Events, Poller};
use std::io;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::env;

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

   fn listen(self:&mut Self, addr: &str) -> io::Result<()> {
       let socket = TcpListener::bind(addr)?;
       socket.set_nonblocking(true)?;
       let key = self.get_key();
       self.fds.insert(key, EventSource::Listener(socket.try_clone()?));
       unsafe{ self.poller.add(&socket, Event::readable(key))? };
       Ok(())
   }

   fn get_key(self:&mut Self) -> usize {
       let key = self.next_key;
       self.next_key += 1;
       key
   }

   fn run(self:&mut Self) -> io::Result<()> {
       let mut events = Events::new();

       while !self.stop {
           events.clear();
           self.poller.wait(&mut events, None)?;

           for ev in events.iter() {
               match self.fds.get(&ev.key).ok_or(()) {  

                Result::Ok(EventSource::Listener(socket)) => {

                   let (stream, _) = socket.accept()?;
                           stream.set_nonblocking(true).unwrap();
                           unsafe{ self.poller.add(&stream, Event::readable(0))? };
               }
              _ => break,
           }
       }
       }

       Ok(())
   }

   fn stop(self:&mut Self) {
    self.stop = true
   }
}

fn on_conn(stream:&mut TcpStream) {
    println!("on_conn");
}

fn main() -> std::io::Result<()> {
    println!("=== start ===");
    let mut ev = EventLoop::new()?;
    ev.listen("127.0.0.1:8080")?;
    ev.run();
    println!("=== end ===");
    Ok(())
}
