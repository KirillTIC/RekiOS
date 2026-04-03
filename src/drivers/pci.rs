extern crate alloc;
use alloc::vec::Vec;
use x86_64::instructions::port::Port;

pub fn pci_read32(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset & 0xFC) as u32);
    unsafe {
        let mut addr_port: Port<u32> = Port::new(0xCF8);
        let mut data_port: Port<u32> = Port::new(0xCFC);
        addr_port.write(address);
        data_port.read()
    }
}
pub fn pci_write32(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset & 0xFC) as u32);
    unsafe {
        let mut addr_port: Port<u32> = Port::new(0xCF8);
        let mut data_port: Port<u32> = Port::new(0xCFC);
        addr_port.write(address);
        data_port.write(value);
    }
}

pub fn pci_read16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let val = pci_read32(bus, device, function, offset & 0xFC);
    if offset & 2 != 0 {
        (val >> 16) as u16
    } else {
        val as u16
    }
}
pub fn pci_read8(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let val = pci_read32(bus, device, function, offset & 0xFC);
    let shift = (offset & 3) * 8;
    (val >> shift) as u8
}

#[derive(Debug, Clone)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
}
impl PciDevice {
    pub fn read_bar(&self, bar_index: u8) -> u32 {
        assert!(bar_index <= 5, "BAR index out of range");
        let offset = 0x10 + bar_index * 4;
        pci_read32(self.bus, self.device, self.function, offset)
    }

    pub fn enable_bus_mastering(&self) {
        let cmd = pci_read16(self.bus, self.device, self.function, 0x04);
        let new_cmd = cmd | (1 << 1) | (1 << 2);
        let upper = pci_read32(self.bus, self.device, self.function, 0x04) & 0xFFFF0000;
        pci_write32(self.bus, self.device, self.function, 0x04, upper | new_cmd as u32);
    }
}

pub fn enumerate_pci() -> Vec<PciDevice> {
    let mut devices = Vec::new();

    for bus in 0u8..=255 {
        for device in 0u8..32 {
            if let Some(dev) = probe_device(bus, device, 0) {
                let header_type = pci_read8(bus, device, 0, 0x0E);
                devices.push(dev);

                if header_type & 0x80 != 0 {
                    for function in 1u8..8 {
                        if let Some(dev) = probe_device(bus, device, function) {
                            devices.push(dev);
                        }
                    }
                }
            }
        }
    }
    
    devices
}

fn probe_device(bus: u8, device: u8, function: u8) -> Option<PciDevice> {
    let vendor_device = pci_read32(bus, device, function, 0x00);
    let vendor_id = vendor_device as u16;

    if vendor_id == 0xFFFF {
        return None;
    }

    let device_id = (vendor_device >> 16) as u16;
    let class_info = pci_read32(bus, device, function, 0x08);

    Some(PciDevice {
        bus,
        device,
        function,
        vendor_id,
        device_id,
        class: (class_info >> 24) as u8,
        subclass: (class_info >> 16) as u8,
        prog_if: (class_info >> 8) as u8,
        revision: class_info as u8,
    })
}

pub fn find_ahci_controller() -> Option<PciDevice> {
    enumerate_pci().into_iter().find(|dev| {
        dev.class == 0x01 && dev.subclass == 0x06 && dev.prog_if == 0x01
    })
}

