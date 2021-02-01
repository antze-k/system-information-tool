use byte_unit::Byte;
use serde::Deserialize;
use std::{
    error::Error,
    fmt::{Display, Formatter},
};
use winreg::{enums::*, RegKey};
use wmi::{COMLibrary, WMIConnection};

#[derive(Default, Deserialize)]
#[serde(rename = "Win32_Processor")]
#[serde(rename_all = "PascalCase")]
struct CPU {
    name: String,
}

#[derive(Default, Deserialize)]
#[serde(rename = "Win32_BaseBoard")]
#[serde(rename_all = "PascalCase")]
struct Baseboard {
    manufacturer: String,
    product: String,
}

#[derive(Default, Deserialize)]
#[serde(rename = "Win32_VideoController")]
#[serde(rename_all = "PascalCase")]
struct GPU {
    caption: String,
}

#[derive(Default, Deserialize)]
#[serde(rename = "Win32_PhysicalMemory")]
#[serde(rename_all = "PascalCase")]
struct RAM {
    capacity: String,
}

impl Display for CPU {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for Baseboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.manufacturer, self.product)
    }
}

impl Display for GPU {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.caption)
    }
}

impl Display for RAM {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.capacity.parse::<u128>() {
            Ok(capacity) => write!(f, "{}", Byte::from(capacity).get_appropriate_unit(true).format(1)),
            _ => write!(f, ""),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let com_con = COMLibrary::new().unwrap();
    let wmi_con = WMIConnection::new(com_con.into()).unwrap();

    let cpu = wmi_con.query::<CPU>()?;
    let baseboard = wmi_con.query::<Baseboard>()?;
    let gpu = wmi_con.query::<GPU>()?;
    let ram = wmi_con.query::<RAM>()?;

    let gpu_ram = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}\\0000")?;
    let gpu_ram: u64 = gpu_ram.get_value("HardwareInformation.qwMemorySize")?;

    println!("CPU:        {}", cpu.first().unwrap_or(&Default::default()));
    println!("Baseboard:  {}", baseboard.first().unwrap_or(&Default::default()));
    println!("RAM:        {}", ram.first().unwrap_or(&Default::default()));
    println!("GPU:        {}", gpu.first().unwrap_or(&Default::default()));
    println!("GPU RAM:    {}", Byte::from(gpu_ram).get_appropriate_unit(true).format(1));

    Ok(())
}
