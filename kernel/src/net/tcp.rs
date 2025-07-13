use super::rtl8139::Rtl8139Driver;
use crate::net::udp::Ip;

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

impl Rtl8139Driver {
    pub fn send_tcp_syn(
        &mut self,
        src_port: u16,
        dest_mac: [u8; 6],
        dest_ip: [u8; 4],
        dest_port: u16,
        seq_number: u32,
    ) {
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
        let src_ip_addr = if dest_port == 67 {
            [0, 0, 0, 0]
        } else {
            self.ip
        };

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

        ip_header.checksum = self
            .checksum(&ip_header as *const _ as *const u8, ip_header_length)
            .to_be();

        let pseudo_header = [

            ip_header.src_ip[0],
            ip_header.src_ip[1],
            ip_header.src_ip[2],
            ip_header.src_ip[3],

            ip_header.dest_ip[0],
            ip_header.dest_ip[1],
            ip_header.dest_ip[2],
            ip_header.dest_ip[3],
            0,
            ip_header.protocol,

            (tcp_length).to_be_bytes()[0],
            (tcp_length).to_be_bytes()[1],
        ];

        let mut checksum_buffer =
            alloc::vec::Vec::with_capacity(pseudo_header.len() + tcp_header_length);
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

    pub fn send_tcp_ack(
        &mut self,
        src_port: u16,
        dest_mac: [u8; 6],
        dest_ip: [u8; 4],
        dest_port: u16,
        seq_number: u32,
        ack_number: u32,
    ) {
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

        let src_ip_addr = if dest_port == 67 {
            [0, 0, 0, 0]
        } else {
            self.ip
        };
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

        ip_header.checksum = self
            .checksum(&ip_header as *const _ as *const u8, ip_header_length)
            .to_be();

        let pseudo_header = [
            ip_header.src_ip[0],
            ip_header.src_ip[1],
            ip_header.src_ip[2],
            ip_header.src_ip[3],
            ip_header.dest_ip[0],
            ip_header.dest_ip[1],
            ip_header.dest_ip[2],
            ip_header.dest_ip[3],
            0,
            ip_header.protocol,
            (tcp_header_length as u16).to_be_bytes()[0],
            (tcp_header_length as u16).to_be_bytes()[1],
        ];

        let mut checksum_buffer =
            alloc::vec::Vec::with_capacity(pseudo_header.len() + tcp_header_length);
        checksum_buffer.extend_from_slice(&pseudo_header);
        checksum_buffer.extend_from_slice(unsafe {
            core::slice::from_raw_parts(&tcp_header as *const _ as *const u8, tcp_header_length)
        });

        tcp_header.checksum = self
            .checksum(checksum_buffer.as_ptr(), checksum_buffer.len())
            .to_be();

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
        let src_ip_addr = if dest_port == 67 {
            [0, 0, 0, 0]
        } else {
            self.ip
        };

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
            ip_header.src_ip[0],
            ip_header.src_ip[1],
            ip_header.src_ip[2],
            ip_header.src_ip[3],
            ip_header.dest_ip[0],
            ip_header.dest_ip[1],
            ip_header.dest_ip[2],
            ip_header.dest_ip[3],
            0,
            ip_header.protocol,
            (tcp_length).to_be_bytes()[0],
            (tcp_length).to_be_bytes()[1],
        ];

        let mut checksum_buffer = alloc::vec::Vec::with_capacity(
            pseudo_header.len() + tcp_header_length + http_request.len(),
        );
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
                packet
                    .as_mut_ptr()
                    .add(ip_header_length + tcp_header_length),
                http_request.len(),
            );
        }

        self.send_packet(&packet, dest_mac, 0x0800);
    }
}
