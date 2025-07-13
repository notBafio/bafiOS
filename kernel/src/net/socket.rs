use alloc::vec::Vec;

pub struct Socket {
    buffer: u32,
    port: u16,
}

pub struct Sockets {
    ports: Vec<Socket>,
}

pub static mut SOCKETS: Sockets = Sockets { ports: Vec::new() };

impl Sockets {
    pub fn new(&mut self, port: u16, buffer: u32) {
        for i in self.ports.iter() {
            if i.port == port {
                return;
            }
        }

        if port <= 0 {
            return;
        }

        self.ports.push(Socket { buffer, port });
    }

    pub fn get_socket(&self, port: u16) -> Option<u32> {
        for i in self.ports.iter() {
            if i.port == port {
                return Some(i.buffer);
            }
        }

        None
    }

    pub fn close(&mut self, port: u16) {
        let mut idx: i16 = -1;

        for (i, socket) in self.ports.iter_mut().enumerate() {
            if socket.port == port {
                idx = i as i16;
            }
        }

        if idx > 0 {
            self.ports.remove(idx as usize);
        }
    }
}
