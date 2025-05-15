use crate::net::rtl8139::Rtl8139Driver;

use super::rtl8139::ArpPacket;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Arp {
    pub hardware_type: u16,
    pub protocol_type: u16,
    pub hardware_size: u8,
    pub protocol_size: u8,
    pub operation: u16,
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}

#[derive(Copy, Clone, Debug)]
pub struct ArpCache {
    pub ip: [u8; 4],
    pub mac: [u8; 6],
    taken: bool,
}
pub static mut ARP_CACHE: [ArpCache; 16] = [ArpCache {
    ip: [0; 4],
    mac: [0; 6],
    taken: false,
}; 16];

pub fn find_mac(ip: [u8; 4]) -> Option<[u8; 6]> {
    unsafe {
        (*(&raw mut ARP_CACHE))
            .iter()
            .find(|entry| entry.ip == ip)
            .map(|entry| entry.mac)
    }
}

impl Rtl8139Driver {
    pub fn send_arp_request(&mut self, target_ip: [u8; 4]) {
        let broadcast_mac = [0xFF; 6];
        let eth_type = 0x0806;

        let arp_packet = Arp {
            hardware_type: 0x0001u16.to_be(),
            protocol_type: 0x0800u16.to_be(),
            hardware_size: 6,
            protocol_size: 4,
            operation: 0x0001u16.to_be(),
            sender_mac: self.mac_address,
            sender_ip: self.ip,
            target_mac: [0; 6],
            target_ip: target_ip,
        };

        let frame_size = core::mem::size_of::<Arp>();
        let frame = unsafe {
            core::slice::from_raw_parts(&arp_packet as *const _ as *const u8, frame_size)
        };

        self.send_packet(frame, broadcast_mac, eth_type);
    }

    pub fn send_arp_response(&mut self, target_ip: [u8; 4], target_mac: [u8; 6]) {
        let broadcast_mac = [0xFF; 6];
        let eth_type = 0x0806;

        let arp_packet = Arp {
            hardware_type: 0x0001u16.to_be(),
            protocol_type: 0x0800u16.to_be(),
            hardware_size: 6,
            protocol_size: 4,
            operation: 0x0002u16.to_be(),
            sender_mac: self.mac_address,
            sender_ip: self.ip,
            target_mac: target_mac,
            target_ip: target_ip,
        };

        let frame_size = core::mem::size_of::<Arp>();
        let frame = unsafe {
            core::slice::from_raw_parts(&arp_packet as *const _ as *const u8, frame_size)
        };

        self.send_packet(frame, broadcast_mac, eth_type);
    }

    pub fn handle_arp(&self, packet: &crate::net::rtl8139::ArpPacket) {
        libk::println!("{:?}", packet.arp_frame.sender_mac);
        libk::println!("{:?}", packet.arp_frame.sender_ip);

        unsafe {
            (*(&raw mut ARP_CACHE)).iter_mut().for_each(|entry| {
                if !entry.taken {
                    entry.ip = packet.arp_frame.sender_ip;
                    entry.mac = packet.arp_frame.sender_mac;
                    return;
                }
            });
        }
    }
}
