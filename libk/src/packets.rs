/*#[repr(C, packed)]

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ArpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub arp_frame: Arp,
}


#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TcpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub ip_frame: Ip,
    pub tcp_frame: Tcp,
}

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

pub static mut ARP_CACHE: [ArpCache; 16] = [ArpCache { ip: [0; 4], mac: [0; 6], taken: false }; 16];

pub fn find_mac(ip: [u8; 4]) -> Option<[u8;6]> {
    unsafe {
        ARP_CACHE.iter().find(|entry| entry.ip == ip).map(|entry| entry.mac)
    }
}

pub fn send_arp_request(target_ip: [u8; 4]) {
    let broadcast_mac = [0xFF; 6];
    let eth_type =0x0806;

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
        core::slice::from_raw_parts(
            &arp_packet as *const _ as *const u8,
            frame_size,
        )
    };

    send_packet(frame, broadcast_mac, eth_type);
}

pub fn send_arp_response(target_ip: [u8; 4], target_mac: [u8; 6]) {
    let broadcast_mac = [0xFF; 6];
    let eth_type =0x0806;

    let arp_packet = Arp {
        hardware_type: 0x0001u16.to_be(),
        protocol_type: 0x0800u16.to_be(),
        hardware_size: 6,
        protocol_size: 4,
        operation: 0x0002u16.to_be(),
        sender_mac: mac_address,
        sender_ip: ip,
        target_mac: target_mac,
        target_ip: target_ip,
    };

    let frame_size = core::mem::size_of::<Arp>();
    let frame = unsafe {
        core::slice::from_raw_parts(
            &arp_packet as *const _ as *const u8,
            frame_size,
        )
    };
}

pub fn handle_arp( packet: ArpPacket) {
    crate::println!("{:?}", packet.arp_frame.sender_mac);
    crate::println!("{:?}", packet.arp_frame.sender_ip);

    unsafe {
        ARP_CACHE.iter_mut().for_each(|entry| {
            if !entry.taken {
                entry.ip = packet.arp_frame.sender_ip;
                entry.mac = packet.arp_frame.sender_mac;
                return;
            }
        });
    }
}

pub fn send_dhcp_discover() {

    let mut dhcp_options = [0u8; 312];
    let mut offset = 0;

    dhcp_options[0..4].copy_from_slice(&[0x63, 0x82, 0x53, 0x63]);
    offset += 4;

    dhcp_options[offset] = 53;
    dhcp_options[offset+1] = 1;
    dhcp_options[offset+2] = 1;
    offset += 3;

    dhcp_options[offset] = 55;
    dhcp_options[offset+1] = 4;
    dhcp_options[offset+2] = 1;
    dhcp_options[offset+3] = 3;
    dhcp_options[offset+4] = 6;
    dhcp_options[offset+5] = 15;
    offset += 6;

    dhcp_options[offset] = 61;
    dhcp_options[offset+1] = 7;
    dhcp_options[offset+2] = 1;
    dhcp_options[offset+3..offset+9].copy_from_slice(&self.mac_address);
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
        core::slice::from_raw_parts(
            &dhcp_packet as *const _ as *const u8,
            dhcp_size,
        )
    };

    self.send_udp_packet( 68, dhcp, [0xff; 6], [255, 255, 255, 255], 67);
}

use crate::net::rtl8139::Rtl8139Driver;
use core::mem;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Icmp {
    pub icmp_type: u8,
    pub code: u8,
    pub checksum: u16,
    pub identifier: u16,
    pub sequence: u16,
}


impl Rtl8139Driver {
    pub fn ping(&mut self, dest_ip: [u8; 4], identifier: u16) {

        let mut dest_mac = None;

        if dest_ip[0] == 10 || dest_ip[0] == 127 || dest_ip[0] == 192 || dest_ip[0] == 172 {
            dest_mac = crate::net::arp::find_mac(dest_ip);

        } else {
            dest_mac = crate::net::arp::find_mac(self.gateway);
        }

        if dest_mac == None { return; }


        let icmp_header = Icmp {
            icmp_type: 8,
            code: 0,
            checksum: 0,
            identifier: identifier.to_be(),
            sequence: 0u16.to_be(),
        };

        let payload: [u8; 8] = [0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68];
        let total_icmp_length = mem::size_of::<Icmp>() + payload.len();

        let mut icmp_packet = alloc::vec![0u8; total_icmp_length];
        unsafe {
            core::ptr::copy(
                &icmp_header as *const Icmp as *const u8,
                icmp_packet.as_mut_ptr(),
                mem::size_of::<Icmp>(),
            );

            core::ptr::copy(
                payload.as_ptr(),
                icmp_packet.as_mut_ptr().add(mem::size_of::<Icmp>()),
                payload.len(),
            );
        }

        let checksum = self.checksum(icmp_packet.as_ptr(), total_icmp_length);
        icmp_packet[2] = (checksum >> 8) as u8;
        icmp_packet[3] = (checksum & 0xFF) as u8;

        let total_length = (mem::size_of::<Ip>() + total_icmp_length) as u16;
        let mut ip_header = Ip {
            version_ihl: 0x45,
            dscp_ecn: 0,
            total_length: total_length.to_be(),
            identification: 0,
            flags_fragment_offset: 0,
            ttl: 64,
            protocol: 1,
            checksum: 0,
            src_ip: self.ip,
            dest_ip,
        };

        ip_header.checksum = self.checksum_icmp(&ip_header as *const _ as *const u8, core::mem::size_of::<Ip>()).to_be();

        let mut packet = alloc::vec![0u8; total_length as usize];

        unsafe {
            core::ptr::copy(
                &ip_header as *const Ip as *const u8,
                packet.as_mut_ptr(),
                mem::size_of::<Ip>(),
            );
            core::ptr::copy(
                icmp_packet.as_ptr(),
                packet.as_mut_ptr().add(mem::size_of::<Ip>()),
                total_icmp_length,
            );
        }

        self.send_packet(&packet, dest_mac.unwrap(), 0x0800);
    }

    pub fn checksum_icmp(&self, data: *const u8, length: usize) -> u16 {
        let mut sum: u32 = 0;
        let mut i = 0;

        while i < length {
            let word = if i + 1 < length {
                unsafe { (*(data.add(i) as *const u16)).to_be() }
            } else {
                unsafe { (*(data.add(i) as *const u8) as u16) << 8 }
            };
            sum = sum.wrapping_add(u32::from(word));
            i += 2;
        }
        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }
        !(sum as u16)
    }
}

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct Tcp {
    pub src_port: u16,
    pub dest_port: u16,
    pub sequence: u32,
    pub acknowledgment: u32,
    pub data_offset_reserved_flags: u16,
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
}

pub fn send_tcp_syn(src_port: u16, dest_mac: [u8; 6], dest_ip: [u8; 4], dest_port: u16, seq_number: u32) {
    let tcp_header_length = core::mem::size_of::<Tcp>();
    let ip_header_length = core::mem::size_of::<Ip>();
    let total_length = (ip_header_length + tcp_header_length) as u16;

    let mut tcp_header = Tcp {
        src_port: src_port.to_be(),
        dest_port: dest_port.to_be(),
        sequence: seq_number.to_be(),
        acknowledgment: 0,
        data_offset_reserved_flags: ((5 << 12) as u16 | 0x02u16).to_be(),
        window_size: 1024u16.to_be(),
        checksum: 0,
        urgent_pointer: 0,
    };

    let tcp_length = tcp_header_length as u16;
    let src_ip_addr = if dest_port == 67 { [0, 0, 0, 0] } else { self.ip };

    let mut ip_header = Ip {
        version_ihl: 0x45,
        dscp_ecn: 0,
        total_length: total_length.to_be(),
        identification: 0,
        flags_fragment_offset: 0,
        ttl: 255,
        protocol: 0x06,
        checksum: 0,
        src_ip: src_ip_addr,
        dest_ip,
    };
    ip_header.checksum = self.checksum(&ip_header as *const _ as *const u8, ip_header_length).to_be();
    let pseudo_header = [
        ip_header.src_ip[0], ip_header.src_ip[1], ip_header.src_ip[2], ip_header.src_ip[3],
        ip_header.dest_ip[0], ip_header.dest_ip[1], ip_header.dest_ip[2], ip_header.dest_ip[3],
        0, ip_header.protocol,
        (tcp_length).to_be_bytes()[0], (tcp_length).to_be_bytes()[1],
    ];
    let mut checksum_buffer = alloc::vec::Vec::with_capacity(pseudo_header.len() + tcp_header_length);
    checksum_buffer.extend_from_slice(&pseudo_header);
    checksum_buffer.extend_from_slice(unsafe {
        core::slice::from_raw_parts(&tcp_header as *const _ as *const u8, tcp_header_length)
    });
    let checksum = self.checksum(checksum_buffer.as_ptr(), checksum_buffer.len());
    tcp_header.checksum = checksum.to_be();
    let mut packet = alloc::vec![0u8; total_length as usize];
    unsafe {
        core::ptr::copy(
            &ip_header as *const Ip as *const u8,
            packet.as_mut_ptr(),
            ip_header_length,
        );
        core::ptr::copy(
            &tcp_header as *const Tcp as *const u8,
            packet.as_mut_ptr().add(ip_header_length),
            tcp_header_length,
        );
    }

    self.send_packet(&packet, dest_mac, 0x0800);
}

pub fn send_tcp_ack(&mut self, src_port: u16, dest_mac: [u8; 6], dest_ip: [u8; 4], dest_port: u16, seq_number: u32, ack_number: u32) {
    let tcp_header_length = core::mem::size_of::<Tcp>();
    let ip_header_length = core::mem::size_of::<Ip>();
    let total_length = (ip_header_length + tcp_header_length) as u16;

    let mut tcp_header = Tcp {
        src_port: src_port.to_be(),
        dest_port: dest_port.to_be(),
        sequence: seq_number.to_be(),
        acknowledgment: ack_number.to_be(),
        data_offset_reserved_flags: ((5 << 12) as u16 | 0x10u16).to_be(),
        window_size: 1024u16.to_be(),
        checksum: 0,
        urgent_pointer: 0,
    };

    let src_ip_addr = if dest_port == 67 { [0, 0, 0, 0] } else { self.ip };
    let mut ip_header = Ip {
        version_ihl: 0x45,
        dscp_ecn: 0,
        total_length: total_length.to_be(),
        identification: 0,
        flags_fragment_offset: 0,
        ttl: 255,
        protocol: 0x06,
        checksum: 0,
        src_ip: src_ip_addr,
        dest_ip,
    };

    ip_header.checksum = self.checksum(&ip_header as *const _ as *const u8, ip_header_length)
        .to_be();

    let pseudo_header = [
        ip_header.src_ip[0], ip_header.src_ip[1], ip_header.src_ip[2], ip_header.src_ip[3],
        ip_header.dest_ip[0], ip_header.dest_ip[1], ip_header.dest_ip[2], ip_header.dest_ip[3],
        0, ip_header.protocol,
        (tcp_header_length as u16).to_be_bytes()[0], (tcp_header_length as u16).to_be_bytes()[1],
    ];

    let mut checksum_buffer = alloc::vec::Vec::with_capacity(pseudo_header.len() + tcp_header_length);
    checksum_buffer.extend_from_slice(&pseudo_header);
    checksum_buffer.extend_from_slice(unsafe {
        core::slice::from_raw_parts(&tcp_header as *const _ as *const u8, tcp_header_length)
    });

    tcp_header.checksum = self.checksum(checksum_buffer.as_ptr(), checksum_buffer.len()).to_be();

    let mut packet = alloc::vec![0u8; total_length as usize];
    unsafe {
        core::ptr::copy(
            &ip_header as *const Ip as *const u8,
            packet.as_mut_ptr(),
            ip_header_length,
        );
        core::ptr::copy(
            &tcp_header as *const Tcp as *const u8,
            packet.as_mut_ptr().add(ip_header_length),
            tcp_header_length,
        );
    }

    self.send_packet(&packet, dest_mac, 0x0800);
}

pub fn send_http_get_request(
    &mut self,
    src_port: u16,
    dest_mac: [u8; 6],
    dest_ip: [u8; 4],
    dest_port: u16,
    seq_number: u32,
    ack_number: u32,
) {
    let http_request = b"GET / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: BaremetalOS/1.0\r\nConnection: close\r\n\r\n";
    let tcp_header_length = core::mem::size_of::<Tcp>();
    let ip_header_length = core::mem::size_of::<Ip>();
    let total_length = (ip_header_length + tcp_header_length + http_request.len()) as u16;
    let mut tcp_header = Tcp {
        src_port: src_port.to_be(),
        dest_port: dest_port.to_be(),
        sequence: seq_number.to_be(),
        acknowledgment: ack_number.to_be(),
        data_offset_reserved_flags: ((5 << 12) as u16 | 0x18u16).to_be(),
        window_size: 1024u16.to_be(),
        checksum: 0,
        urgent_pointer: 0,
    };
    let tcp_length = (tcp_header_length + http_request.len()) as u16;
    let src_ip_addr = if dest_port == 67 { [0, 0, 0, 0] } else { self.ip };
    let mut ip_header = Ip {
        version_ihl: 0x45,
        dscp_ecn: 0,
        total_length: total_length.to_be(),
        identification: 0,
        flags_fragment_offset: 0,
        ttl: 255,
        protocol: 0x06,
        checksum: 0,
        src_ip: src_ip_addr,
        dest_ip,
    };
    let ip_header_length = core::mem::size_of::<Ip>();
    ip_header.checksum = self
        .checksum(&ip_header as *const _ as *const u8, ip_header_length)
        .to_be();
    let pseudo_header = [
        ip_header.src_ip[0], ip_header.src_ip[1], ip_header.src_ip[2], ip_header.src_ip[3],
        ip_header.dest_ip[0], ip_header.dest_ip[1], ip_header.dest_ip[2], ip_header.dest_ip[3],
        0, ip_header.protocol,
        (tcp_length).to_be_bytes()[0], (tcp_length).to_be_bytes()[1],
    ];
    let mut checksum_buffer = alloc::vec::Vec::with_capacity(pseudo_header.len() + tcp_header_length + http_request.len());
    checksum_buffer.extend_from_slice(&pseudo_header);
    checksum_buffer.extend_from_slice(unsafe {
        core::slice::from_raw_parts(&tcp_header as *const Tcp as *const u8, tcp_header_length)
    });
    checksum_buffer.extend_from_slice(http_request);
    let checksum = self.checksum(checksum_buffer.as_ptr(), checksum_buffer.len());
    tcp_header.checksum = checksum.to_be();
    let mut packet = alloc::vec![0u8; total_length as usize];
    unsafe {
        core::ptr::copy(
            &ip_header as *const Ip as *const u8,
            packet.as_mut_ptr(),
            ip_header_length,
        );
        core::ptr::copy(
            &tcp_header as *const Tcp as *const u8,
            packet.as_mut_ptr().add(ip_header_length),
            tcp_header_length,
        );
        core::ptr::copy(
            http_request.as_ptr(),
            packet.as_mut_ptr().add(ip_header_length + tcp_header_length),
            http_request.len(),
        );
    }
    self.send_packet(&packet, dest_mac, 0x0800);
}


pub fn send_udp_packet(&mut self, src_port: u16, data: &[u8], dest_mac: [u8; 6], dest_ip: [u8; 4], dest_port: u16) {

    let udp_length = (core::mem::size_of::<Udp>() + data.len()) as u16;
    let udp_header = Udp {
        src_port: src_port.to_be(),
        dest_port: dest_port.to_be(),
        length: udp_length.to_be(),
        checksum: 0,
    };

    let total_length = (core::mem::size_of::<Ip>() + udp_length as usize) as u16;
    let src_ip_addr = if dest_port == 67 { [0, 0, 0, 0] } else { self.ip};

    let mut ip_header = Ip {
        version_ihl: 0x45,
        dscp_ecn: 0,
        total_length: total_length.to_be(),
        identification: 0,
        flags_fragment_offset: 0,
        ttl: 255,
        protocol: 0x11,
        checksum: 0,
        src_ip: src_ip_addr,
        dest_ip,
    };

    ip_header.checksum = self.checksum(&ip_header as *const _ as *const u8, core::mem::size_of::<Ip>()).to_be();

    let header_size = core::mem::size_of::<Ip>() + core::mem::size_of::<Udp>();
    let mut packet = alloc::vec![0u8; total_length as usize];

    unsafe {
        core::ptr::copy(
            &ip_header as *const Ip as *const u8,
            packet.as_mut_ptr(),
            core::mem::size_of::<Ip>(),
        );
        core::ptr::copy(
            &udp_header as *const Udp as *const u8,
            packet.as_mut_ptr().add(core::mem::size_of::<Ip>()),
            core::mem::size_of::<Udp>(),
        );
        core::ptr::copy(
            data.as_ptr(),
            packet.as_mut_ptr().add(header_size),
            data.len(),
        );
    }

    self.send_packet(&packet, dest_mac, 0x0800);
}

pub fn checksum(&self, data: *const u8, length: usize) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;

    while i < length {
        let word = if i + 1 < length {
            unsafe { u16::from_be(core::ptr::read_unaligned(data.add(i) as *const u16)) }
        } else {
            unsafe { (core::ptr::read_unaligned(data.add(i)) as u16) << 8 }
        };
        sum += u32::from(word);
        i += 2;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}*/

#[derive(Debug, Copy, Clone)]
pub struct Rtl8139Driver {
    _mmio: u32,
    pub mac_address: [u8; 6],
    pub ip: [u8; 4],
    _rx_buffer: u32,
    _rx_offset: u32,
    _tx_buffers: [u32; 4],
    _tx_index: usize,
    pub subnet: [u8; 4],
    pub gateway: [u8; 4],
    pub dns: [u8; 4],
}

pub struct Packet {
    header: Rx,
    ethernet_frame: Ethernet,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Ethernet {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub eth_type: u16,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Rx {
    pub status: u16,
    pub len: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub ip_frame: Ip,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct UdpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub ip_frame: Ip,
    pub udp_frame: Udp,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DhcpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub ip_frame: Ip,
    pub udp_frame: Udp,
    pub dhcp_frame: Dhcp,
}

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

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Udp {
    pub src_port: u16,
    pub dest_port: u16,
    pub length: u16,
    pub checksum: u16,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Ip {
    pub version_ihl: u8,
    pub dscp_ecn: u8,
    pub total_length: u16,
    pub identification: u16,
    pub flags_fragment_offset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub src_ip: [u8; 4],
    pub dest_ip: [u8; 4],
}


impl crate::net::Socket {
    pub fn send_dhcp_discover(&self) {
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
        dhcp_options[offset + 3..offset + 9]
            .copy_from_slice(unsafe { &(*(&raw mut crate::net::NET)).mac_address });
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
            dhcp_packet.chaddr[i] = unsafe { crate::net::NET.mac_address[i] };
        }

        let dhcp_size = core::mem::size_of::<Dhcp>() - 312 + offset;

        let dhcp = unsafe {
            core::slice::from_raw_parts(&dhcp_packet as *const _ as *const u8, dhcp_size)
        };

        self.send_udp_packet(68, dhcp, [0xff; 6], [255, 255, 255, 255], 67);
    }

    pub fn send_udp_packet(
        &self,
        src_port: u16,
        data: &[u8],
        dest_mac: [u8; 6],
        dest_ip: [u8; 4],
        dest_port: u16,
    ) {
        let udp_length = (core::mem::size_of::<Udp>() + data.len()) as u16;
        let udp_header = Udp {
            src_port: src_port.to_be(),
            dest_port: dest_port.to_be(),
            length: udp_length.to_be(),
            checksum: 0,
        };

        let total_length = (core::mem::size_of::<Ip>() + udp_length as usize) as u16;
        let src_ip_addr = if dest_port == 67 {
            [0, 0, 0, 0]
        } else {
            unsafe { crate::net::NET.ip }
        };

        let mut ip_header = Ip {
            version_ihl: 0x45,
            dscp_ecn: 0,
            total_length: total_length.to_be(),
            identification: 0,
            flags_fragment_offset: 0,
            ttl: 255,
            protocol: 0x11,
            checksum: 0,
            src_ip: src_ip_addr,
            dest_ip,
        };

        ip_header.checksum = self
            .checksum(
                &ip_header as *const _ as *const u8,
                core::mem::size_of::<Ip>(),
            )
            .to_be();

        let header_size = core::mem::size_of::<Ip>() + core::mem::size_of::<Udp>();
        let mut packet = alloc::vec![0u8; total_length as usize];

        unsafe {
            core::ptr::copy(
                &ip_header as *const Ip as *const u8,
                packet.as_mut_ptr(),
                core::mem::size_of::<Ip>(),
            );
            core::ptr::copy(
                &udp_header as *const Udp as *const u8,
                packet.as_mut_ptr().add(core::mem::size_of::<Ip>()),
                core::mem::size_of::<Udp>(),
            );
            core::ptr::copy(
                data.as_ptr(),
                packet.as_mut_ptr().add(header_size),
                data.len(),
            );
        }

        self.send_packet(&packet, dest_mac, 0x0800);
    }

    pub fn checksum(&self, data: *const u8, length: usize) -> u16 {
        let mut sum = 0u32;
        let mut i = 0;

        while i < length {
            let word = if i + 1 < length {
                unsafe { u16::from_be(core::ptr::read_unaligned(data.add(i) as *const u16)) }
            } else {
                unsafe { (core::ptr::read_unaligned(data.add(i)) as u16) << 8 }
            };
            sum += u32::from(word);
            i += 2;
        }

        while sum >> 16 != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !(sum as u16)
    }

    pub fn send_packet(&self, data: &[u8], dst_mac: [u8; 6], eth_type: u16) {
        unsafe {
            let eth_frame = Ethernet {
                dst_mac,
                src_mac: crate::net::NET.mac_address,
                eth_type: eth_type.to_be(),
            };

            let header_bytes: &[u8] = core::slice::from_raw_parts(
                &eth_frame as *const Ethernet as *const u8,
                core::mem::size_of::<Ethernet>(),
            );

            self.send(&[header_bytes, data].concat());
        }
    }

    pub fn handle_dhcp<'a>(&self, packet: &'a DhcpPacket) {
        if self.search_option(53, packet).unwrap()[0] == 2 {
            self.send_dhcp_request(packet.dhcp_frame.yiaddr, packet.dhcp_frame.siaddr);
        } else if self.search_option(53, packet).unwrap()[0] == 5 {
            unsafe {
                crate::net::NET.subnet = self.unslice(self.search_option(1, packet).unwrap());
                crate::net::NET.ip = packet.dhcp_frame.yiaddr;
                crate::net::NET.gateway = self.unslice(self.search_option(3, packet).unwrap());
                crate::net::NET.dns = self.unslice(self.search_option(6, packet).unwrap());

                crate::syscall::syscall(34, (*(&raw mut crate::net::NET)).ip.as_ptr() as u32, 0, 0);
                crate::syscall::syscall(
                    35,
                    (*(&raw mut crate::net::NET)).dns.as_ptr() as u32,
                    0,
                    0,
                );
                crate::syscall::syscall(
                    36,
                    (*(&raw mut crate::net::NET)).gateway.as_ptr() as u32,
                    0,
                    0,
                );
                crate::syscall::syscall(
                    37,
                    (*(&raw mut crate::net::NET)).subnet.as_ptr() as u32,
                    0,
                    0,
                );
            }
        }
    }

    pub fn search_option<'a>(&self, target_tag: u8, packet: &'a DhcpPacket) -> Option<&'a [u8]> {
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

    fn unslice(&self, slice: &[u8]) -> [u8; 4] {
        let mut a = [0u8; 4];

        for i in 0..4 {
            a[i] = slice[i];
        }

        return a;
    }

    pub fn send_dhcp_request(&self, new_ip: [u8; 4], server_ip: [u8; 4]) {
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
            dhcp_packet.chaddr[i] = unsafe { crate::net::NET.mac_address[i] };
        }

        let dhcp_size = core::mem::size_of::<Dhcp>() - 312 + offset;

        let dhcp = unsafe {
            core::slice::from_raw_parts(&dhcp_packet as *const _ as *const u8, dhcp_size)
        };

        self.send_udp_packet(68, dhcp, [0xff; 6], [255, 255, 255, 255], 67);
    }
}
