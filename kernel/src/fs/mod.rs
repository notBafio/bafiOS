pub mod fat16;

pub trait  Fs {
    fn check_disk(disk: u8) -> bool;
}