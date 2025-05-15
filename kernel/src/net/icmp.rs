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

impl Rtl8139Driver {
    pub fn ping(&mut self, dest_ip: [u8; 4], identifier: u16) {
        let mut dest_mac = None;

        if dest_ip[0] == 10 || dest_ip[0] == 127 || dest_ip[0] == 192 || dest_ip[0] == 172 {
            dest_mac = crate::net::arp::find_mac(dest_ip);
        } else {
            dest_mac = crate::net::arp::find_mac(self.gateway);
        }

        if dest_mac == None {
            return;
        }

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

        ip_header.checksum = self
            .checksum_icmp(
                &ip_header as *const _ as *const u8,
                core::mem::size_of::<Ip>(),
            )
            .to_be();

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
