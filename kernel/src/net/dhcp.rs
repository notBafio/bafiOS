use crate::net::rtl8139::Rtl8139Driver;
use crate::net::udp::Ip;
use crate::net::udp::Udp;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Dhcp {
    pub op: u8,
    pub htype: u8,
    pub hlen: u8,
    pub hops: u8,
    pub xid: u32,
    pub secs: u16,
    pub flags: u16,
    pub ciaddr: [u8; 4],
    pub yiaddr: [u8; 4],
    pub siaddr: [u8; 4],
    pub giaddr: [u8; 4],
    pub chaddr: [u8; 16],
    pub sname: [u8; 64],
    pub file: [u8; 128],

    pub options: [u8; 312],
}

impl Rtl8139Driver {
    pub fn send_dhcp_discover(&mut self) {
        let mut dhcp_options = [0u8; 312];
        let mut offset = 0;

        dhcp_options[0..4].copy_from_slice(&[0x63, 0x82, 0x53, 0x63]);
        offset += 4;

        dhcp_options[offset] = 53;
        dhcp_options[offset + 1] = 1;
        dhcp_options[offset + 2] = 1;
        offset += 3;

        dhcp_options[offset] = 55;
        dhcp_options[offset + 1] = 4;
        dhcp_options[offset + 2] = 1;
        dhcp_options[offset + 3] = 3;
        dhcp_options[offset + 4] = 6;
        dhcp_options[offset + 5] = 15;
        offset += 6;

        dhcp_options[offset] = 61;
        dhcp_options[offset + 1] = 7;
        dhcp_options[offset + 2] = 1;
        dhcp_options[offset + 3..offset + 9].copy_from_slice(&self.mac_address);
        offset += 9;

        dhcp_options[offset] = 255;
        offset += 1;

        let xid = 0xfe55a as u32;

        let mut dhcp_packet = Dhcp {
            op: 1,
            htype: 1,
            hlen: 6,
            hops: 0,
            xid: xid.to_be(),
            secs: 0,
            flags: 0x8000u16.to_be(),
            ciaddr: [0; 4],
            yiaddr: [0; 4],
            siaddr: [0; 4],
            giaddr: [0; 4],
            chaddr: [0; 16],
            sname: [0; 64],
            file: [0; 128],
            options: dhcp_options,
        };

        for i in 0..6 {
            dhcp_packet.chaddr[i] = self.mac_address[i];
        }

        let dhcp_size = core::mem::size_of::<Dhcp>() - 312 + offset;

        let dhcp = unsafe {
            core::slice::from_raw_parts(&dhcp_packet as *const _ as *const u8, dhcp_size)
        };

        self.send_udp_packet(68, dhcp, [0xff; 6], [255, 255, 255, 255], 67);
    }

    pub fn send_dhcp_request(&mut self, new_ip: [u8; 4], server_ip: [u8; 4]) {
        let mut dhcp_options = [0u8; 312];
        let mut offset = 0;

        dhcp_options[0..4].copy_from_slice(&[0x63, 0x82, 0x53, 0x63]);
        offset += 4;

        dhcp_options[offset] = 53;
        dhcp_options[offset + 1] = 1;
        dhcp_options[offset + 2] = 3;
        offset += 3;

        dhcp_options[offset] = 50;
        dhcp_options[offset + 1] = 4;
        dhcp_options[offset + 2] = new_ip[0];
        dhcp_options[offset + 3] = new_ip[1];
        dhcp_options[offset + 4] = new_ip[2];
        dhcp_options[offset + 5] = new_ip[3];
        offset += 6;

        dhcp_options[offset] = 54;
        dhcp_options[offset + 1] = 4;
        dhcp_options[offset + 2] = server_ip[0];
        dhcp_options[offset + 3] = server_ip[1];
        dhcp_options[offset + 4] = server_ip[2];
        dhcp_options[offset + 5] = server_ip[3];
        offset += 6;

        dhcp_options[offset] = 255;
        offset += 1;

        let xid = 0xfe55a as u32;

        let mut dhcp_packet = Dhcp {
            op: 1,
            htype: 1,
            hlen: 6,
            hops: 0,
            xid: xid.to_be(),
            secs: 0,
            flags: 0x8000u16.to_be(),
            ciaddr: [0; 4],
            yiaddr: [0; 4],
            siaddr: [0; 4],
            giaddr: [0; 4],
            chaddr: [0; 16],
            sname: [0; 64],
            file: [0; 128],
            options: dhcp_options,
        };

        for i in 0..6 {
            dhcp_packet.chaddr[i] = self.mac_address[i];
        }

        let dhcp_size = core::mem::size_of::<Dhcp>() - 312 + offset;

        let dhcp = unsafe {
            core::slice::from_raw_parts(&dhcp_packet as *const _ as *const u8, dhcp_size)
        };

        self.send_udp_packet(68, dhcp, [0xff; 6], [255, 255, 255, 255], 67);
    }

    pub fn handle_dhcp<'a>(&mut self, packet: &'a crate::net::rtl8139::DhcpPacket) {
        if self.search_option(53, packet).unwrap()[0] == 2 {
            self.send_dhcp_request(packet.dhcp_frame.yiaddr, packet.dhcp_frame.siaddr);
        } else if self.search_option(53, packet).unwrap()[0] == 5 {
            self.subnet = unslice(self.search_option(1, packet).unwrap());
            self.ip = packet.dhcp_frame.yiaddr;
            self.gateway = unslice(self.search_option(3, packet).unwrap());
            self.dns = unslice(self.search_option(6, packet).unwrap());
        }
    }

    pub fn search_option<'a>(
        &self,
        target_tag: u8,
        packet: &'a crate::net::rtl8139::DhcpPacket,
    ) -> Option<&'a [u8]> {
        let options = &packet.dhcp_frame.options;
        let mut idx = 4;

        while idx < options.len() {
            match options[idx] {
                255 => return None,

                0 => {
                    idx += 1;
                    continue;
                }

                current_tag => {
                    if idx + 1 >= options.len() {
                        return None;
                    }

                    let length = options[idx + 1] as usize;
                    let data_start = idx + 2;
                    let data_end = data_start + length;

                    if data_end > options.len() {
                        return None;
                    }

                    if current_tag == target_tag {
                        return Some(&options[data_start..data_end]);
                    }

                    idx = data_end;
                }
            }
        }

        None
    }
}

fn unslice(slice: &[u8]) -> [u8; 4] {
    let mut a = [0u8; 4];

    for i in 0..4 {
        a[i] = slice[i];
    }

    return a;
}
