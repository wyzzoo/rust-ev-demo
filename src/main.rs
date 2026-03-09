use std::vec;
use std::collections::HashMap;
//use std::os::unix::io::AsRawFd;
//use libc::{epoll_createl, epoll_ctl, epoll_wait, EPOLLIN, EPOLL_CTL_ADD};
use polling::{Event, Events, Poller};
use std::io;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

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
    events: HashMap<u32, EventCallback>,
    timers: Vec<Timer>
}

impl EventLoop {
   fn new() -> io::Result<EventLoop> { 
       Ok(EventLoop {
           poller: Poller::new()?,
           events: HashMap::new(),
           timers: Vec::new(),
       })
   }

   fn run() {
   }
}

fn main() {
    println!("=== start ===");
    let ev = EventLoop::new();
    ev?.run();
    println!("=== end ===");
}
