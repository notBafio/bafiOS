use crate::net::rtl8139::Rtl8139Driver;

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

impl Rtl8139Driver {
    pub fn send_udp_packet(
        &mut self,
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
            self.ip
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
}
