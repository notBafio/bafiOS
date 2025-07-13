use core::arch::asm;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Rx {
    pub status: u16,
    pub len: u16,
}

use crate::net::arp::Arp;
use crate::pci::PciDevice;
use libk::mmio::{read_8, read_16, read_32, write_8, write_16, write_32};
use libk::mutex::Mutex;

const RX_BUFFER_SIZE: u32 = 8192 + 16 + 1500;
const TX_BUFFER_SIZE: u32 = 2048;

const RTL_RESET: u8 = 0x10;
const RTL_RE: u8 = 0x08;
const RTL_TE: u8 = 0x04;

const RTL_TSD_TOK: u32 = 1 << 15;
const RTL_TSD_EOR: u32 = 1 << 14;
const RTL_TSD_FS: u32 = 1 << 16;
const RTL_TSD_LS: u32 = 1 << 17;

const RTL_ROK: u16 = 0x01;
const RTL_TOK: u16 = 0x04;

static mut MMIO: u32 = 0;
static mut RX_BUFFER: u32 = 0;
static mut RX_OFFSET: u32 = 0;
static mut TX_BUFFERS: [u32; 4] = [0; 4];
static mut NEXT_TX_BUFFER: usize = 0;

const TX_TSAD: [u32; 4] = [0x20, 0x24, 0x28, 0x2C];
const TX_TSD: [u32; 4] = [0x10, 0x14, 0x18, 0x1C];

const RTL_CR: u32 = 0x37;
const RTL_RBSTART: u32 = 0x30;
const RTL_IMR: u32 = 0x3C;
const RTL_ISR: u32 = 0x3E;
const RTL_RCR: u32 = 0x44;
const RTL_TCR: u32 = 0x40;
const RTL_CONFIG1: u32 = 0x52;
const RTL_CAPR: u32 = 0x38;

const RTL_TSD_OWN: u32 = 1 << 13;
const RTL_TSD_SIZE_MASK: u32 = 0xFFFF0000;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Ethernet {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub eth_type: u16,
}

#[derive(Debug)]
pub struct Rtl8139Driver {
    mmio: u32,
    pub mac_address: [u8; 6],
    pub ip: [u8; 4],
    rx_buffer: u32,
    rx_offset: u32,
    tx_buffers: [u32; 4],
    tx_index: usize,
    pub subnet: [u8; 4],
    pub gateway: [u8; 4],
    pub dns: [u8; 4],
}

pub static mut RTL8139: Rtl8139Driver = Rtl8139Driver {
    mmio: 0,
    mac_address: [0; 6],
    ip: [0; 4],
    rx_buffer: 0,
    rx_offset: 0,
    tx_buffers: [0; 4],
    tx_index: 0,
    subnet: [0; 4],
    gateway: [0; 4],
    dns: [0; 4],
};

impl Rtl8139Driver {
    pub fn init(&mut self) {
        unsafe {
            let pci_dev = PciDevice::new(0x10EC, 0x8139).unwrap();
            self.mmio = pci_dev.get_bar(1).unwrap();
            MMIO = self.mmio;

            pci_dev.enable_bus_mastering();

            write_8(self.mmio + RTL_CR, RTL_RESET);
            while (read_8(self.mmio + RTL_CR) & RTL_RESET) != 0 {}

            self.rx_buffer = (*(&raw mut crate::pmm::PADDR))
                .malloc(RX_BUFFER_SIZE)
                .unwrap();
            RX_BUFFER = self.rx_buffer;
            write_32(self.mmio + RTL_RBSTART, self.rx_buffer);
            self.rx_offset = 0;

            for i in 0..4 {
                self.tx_buffers[i] = (*(&raw mut crate::pmm::PADDR))
                    .malloc(TX_BUFFER_SIZE)
                    .unwrap();
                write_32(self.mmio + TX_TSAD[i], self.tx_buffers[i]);
            }

            write_16(self.mmio + RTL_IMR, RTL_ROK);
            write_8(self.mmio + RTL_CR, RTL_RE | RTL_TE);

            write_32(self.mmio + RTL_RCR, 0xF | (1 << 7));

            for i in 0..6 {
                self.mac_address[i] = read_8(self.mmio + i as u32);
            }

            libk::println!("rtl8139 inited");
        }
    }

    pub fn send_packet(&mut self, data: &[u8], dst_mac: [u8; 6], eth_type: u16) {
        unsafe {
            let eth_frame = Ethernet {
                dst_mac,
                src_mac: self.mac_address,
                eth_type: eth_type.to_be(),
            };

            let tsd_addr = self.mmio + TX_TSD[self.tx_index as usize];
            while read_32(tsd_addr) & (1 << 31) != 0 {}

            core::ptr::copy(
                &eth_frame as *const _ as *const u8,
                self.tx_buffers[self.tx_index as usize] as *mut u8,
                core::mem::size_of::<Ethernet>(),
            );
            core::ptr::copy(
                data.as_ptr(),
                (self.tx_buffers[self.tx_index as usize] as *mut u8)
                    .offset(core::mem::size_of::<Ethernet>() as isize),
                data.len(),
            );

            write_32(
                self.mmio + TX_TSAD[self.tx_index as usize] as u32,
                self.tx_buffers[self.tx_index as usize],
            );

            let mut tx_status = core::mem::size_of::<Ethernet>() as u32 + data.len() as u32;
            libk::println!("pkg len: {}", tx_status);
            tx_status |= 1 << 31;
            tx_status &= !(1 << 13);

            let prev = self.tx_index;
            self.tx_index = (self.tx_index + 1) % 4;

            write_32(self.mmio + TX_TSD[prev as usize] as u32, tx_status);
        }
    }

    pub fn send_clean_packet(&mut self, data: &[u8]) {
        unsafe {
            let tsd_addr = self.mmio + TX_TSD[self.tx_index as usize];
            while read_32(tsd_addr) & (1 << 31) != 0 {}

            core::ptr::copy(
                data.as_ptr(),
                self.tx_buffers[self.tx_index as usize] as *mut u8,
                data.len(),
            );

            write_32(
                self.mmio + TX_TSAD[self.tx_index as usize] as u32,
                self.tx_buffers[self.tx_index as usize],
            );

            let mut tx_status = data.len() as u32;
            libk::println!("pkg len: {}", tx_status);
            tx_status |= 1 << 31;
            tx_status &= !(1 << 13);

            let prev = self.tx_index;
            self.tx_index = (self.tx_index + 1) % 4;

            write_32(self.mmio + TX_TSD[prev as usize] as u32, tx_status);
        }
    }
}

const ARP: u16 = (0x0806 as u16).to_be();
const IP: u16 = (0x0800 as u16).to_be();
const ICMP: u8 = 1;
const UDP: u8 = 17;
const TCP: u8 = 6;

pub extern "x86-interrupt" fn net() {
    unsafe {
        let isr = read_16(MMIO + RTL_ISR);
        write_16(MMIO + RTL_ISR, isr);

        (*(&raw mut crate::pic::PICS)).end_interrupt(43);

        if isr & RTL_ROK != 0 {
            let packet = core::ptr::read((RX_BUFFER + RX_OFFSET) as *const Packet);

            if packet.header.status == 1 {
                match packet.ethernet_frame.eth_type {
                    ARP => {
                        let arp_packet =
                            core::ptr::read((RX_BUFFER + RX_OFFSET) as *const ArpPacket);
                        (*(&raw mut RTL8139)).handle_arp(&arp_packet);
                        libk::println!("ARP");
                    }

                    IP => {
                        libk::println!("IP");

                        let protocol = core::ptr::read((RX_BUFFER + RX_OFFSET) as *const IpPacket)
                            .ip_frame
                            .protocol;

                        match protocol {
                            UDP => {
                                libk::println!("UDP PACKET RECEIVED");

                                let udp_packet =
                                    core::ptr::read((RX_BUFFER + RX_OFFSET) as *const UdpPacket);

                                let addr = (*(&raw mut crate::net::socket::SOCKETS))
                                    .get_socket(udp_packet.udp_frame.dest_port.to_be());

                                if addr.is_some() {
                                    core::ptr::copy(
                                        (RX_BUFFER + RX_OFFSET) as *const u8,
                                        addr.unwrap() as *mut u8,
                                        packet.header.len as usize,
                                    );
                                }

                                /*let dhcp_packet = core::ptr::read((RX_BUFFER + RX_OFFSET) as *const DhcpPacket);

                                if dhcp_packet.udp_frame.dest_port.to_be() == 68 {
                                    libk::println!("DHCP");

                                    (*(&raw mut RTL8139)).handle_dhcp(&dhcp_packet);
                                }*/
                            }

                            TCP => {
                                libk::println!("TCP");

                                let tcp_packet =
                                    core::ptr::read((RX_BUFFER + RX_OFFSET) as *const TcpPacket);

                                let src_ip = tcp_packet.ip_frame.src_ip;
                                let _dest_ip = tcp_packet.ip_frame.dest_ip;
                                let src_port = u16::from_be(tcp_packet.tcp_frame.src_port);
                                let dest_port = u16::from_be(tcp_packet.tcp_frame.dest_port);
                                let seq_number = u32::from_be(tcp_packet.tcp_frame.sequence);
                                let ack_number = u32::from_be(tcp_packet.tcp_frame.acknowledgment);
                                let tcp_flags = tcp_packet.tcp_frame.data_offset_reserved_flags;

                                let addr =
                                    (*(&raw mut crate::net::socket::SOCKETS)).get_socket(dest_port);
                                if addr.is_some() {
                                    core::ptr::copy(
                                        (RX_BUFFER + RX_OFFSET) as *const u8,
                                        addr.unwrap() as *mut u8,
                                        packet.header.len as usize,
                                    );
                                }

                                /*if tcp_flags == 0x1260 {
                                    libk::println!("SYN/ACK received");

                                    let sender_mac = tcp_packet.ethernet_frame.src_mac;

                                    (*(&raw mut crate::net::rtl8139::RTL8139)).send_tcp_ack(
                                        dest_port,
                                        sender_mac,
                                        src_ip,
                                        src_port,
                                        ack_number,
                                        seq_number + 1,
                                    );

                                    (*(&raw mut crate::net::rtl8139::RTL8139)).send_http_get_request(dest_port, sender_mac, src_ip, src_port, ack_number, seq_number + 1);
                                }*/
                            }

                            ICMP => {
                                libk::println!("ICMP");
                            }
                            _ => {
                                libk::println!("UFO");
                            }
                        }
                    }
                    _ => {
                        libk::println!("Generic Packet");
                    }
                }

                let total_size = (packet.header.len as u32 + 4 + 3) & !3;
                RX_OFFSET = (RX_OFFSET + total_size) % RX_BUFFER_SIZE;
            }
        }

        /*if isr & RTL_TOK != 0 {
            libk::println!("TX OK");
        }*/
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Packet {
    header: Rx,
    ethernet_frame: Ethernet,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ArpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub arp_frame: crate::net::arp::Arp,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub ip_frame: crate::net::udp::Ip,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct UdpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub ip_frame: crate::net::udp::Ip,
    pub udp_frame: crate::net::udp::Udp,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct TcpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub ip_frame: crate::net::udp::Ip,
    pub tcp_frame: crate::net::tcp::Tcp,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct DhcpPacket {
    pub header: Rx,
    pub ethernet_frame: Ethernet,
    pub ip_frame: crate::net::udp::Ip,
    pub udp_frame: crate::net::udp::Udp,
    pub dhcp_frame: crate::net::dhcp::Dhcp,
}
