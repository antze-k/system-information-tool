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
            Ok(capacity) => write!(
                f,
                "{}",
                Byte::from(capacity).get_appropriate_unit(true).format(1)
            ),
            _ => write!(f, ""),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let com_con = COMLibrary::new().unwrap();
    let wmi_con = WMIConnection::new(com_con.into()).unwrap();

    let cpu = wmi_con.query::<CPU>()?;
    let baseboard = wmi_con.query::<Baseboard>()?;
    let ram = wmi_con.query::<RAM>()?;
    let total_ram = ram
        .into_iter()
        .map(|v| v.capacity.parse::<u128>())
        .filter_map(Result::ok)
        .sum::<u128>();

    println!("CPU:        {}", cpu.first().unwrap_or(&Default::default()));
    println!("Baseboard:  {}", baseboard.first().unwrap_or(&Default::default()));
    println!("RAM:        {}", Byte::from(total_ram).get_appropriate_unit(true).format(1));

    let gpus = "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}";
    let gpus = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(gpus)?;
    for key in gpus
        .enum_keys()
        .filter_map(Result::ok)
        .filter(|x| x.len() == 4 && x.chars().all(|x| x.is_digit(10)))
    {
        let gpu = gpus.open_subkey(key)?;
        let name: String = gpu.get_value("DriverDesc")?;
        let ram: u64 = gpu.get_value("HardwareInformation.qwMemorySize")?;
        let ram = Byte::from(ram).get_appropriate_unit(true).format(1);
        println!("GPU:        {}", name);
        println!("GPU RAM:    {}", ram);
    }

    Ok(())
}
