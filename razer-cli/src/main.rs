use librazer::command;
use librazer::device;
use librazer::types::{
    CpuBoost, FanMode, FanZone, GpuBoost, LightsAlwaysOn, LogoMode, MaxFanSpeedMode, PerfMode,
};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use clap_num::maybe_hex;

fn create_device(pid: Option<u16>) -> Result<device::Device> {
    const RAZER_BLADE_16_2023_PID: u16 = 0x029f;
    device::Device::new(pid.unwrap_or(RAZER_BLADE_16_2023_PID))
}

pub fn get_info(device: &device::Device) -> Result<String> {
    use std::fmt::Write;
    let mut info = String::new();

    let (perf_mode, fan_mode) = command::get_perf_mode(device)?;
    writeln!(&mut info, "Performance: {:?}", perf_mode)?;

    if perf_mode == PerfMode::Balanced {
        match fan_mode {
            FanMode::Auto => writeln!(&mut info, "Fan: {:?}", fan_mode)?,
            FanMode::Manual => writeln!(
                &mut info,
                "Fan: {} RPM",
                command::get_fan_rpm(device, FanZone::Zone1)?
            )?,
        }
    }

    if perf_mode == PerfMode::Custom {
        let cpu_boost = command::get_cpu_boost(device)?;
        let gpu_boost = command::get_gpu_boost(device)?;
        writeln!(&mut info, "CPU: {:?}", cpu_boost)?;
        writeln!(&mut info, "GPU: {:?}", gpu_boost)?;

        if (cpu_boost == CpuBoost::Boost || cpu_boost == CpuBoost::Overclock)
            && (gpu_boost == GpuBoost::High)
        {
            // TODO: getter for max fan speed mode
        }
    }

    writeln!(&mut info, "Logo: {:?}", command::get_logo_mode(device)?)?;
    writeln!(
        &mut info,
        "Brightness: {}",
        command::get_keyboard_brightness(device)?
    )?;
    write!(
        &mut info,
        "Lights always on: {:?}",
        command::get_lights_always_on(device)?
    )?;

    Ok(info)
}

#[derive(Parser)]
#[command(name = "razerctl", version, about)]
struct Razerctl {
    #[command(subcommand)]
    pub command: RazerCtlCommand,

    /// PID of the Razer device to use
    #[clap(short, long, value_parser=maybe_hex::<u16>)]
    pub pid: Option<u16>,
}

#[derive(Subcommand)]
enum RazerCtlCommand {
    /// List discovered Razer devices
    Enumerate,
    /// Get device info
    Info,
    /// Control performance modes
    Perf(PerfModeCommand),
    /// Control fan
    Fan(FanCommand),
    /// Run Custom Command
    Cmd {
        #[clap(value_parser=maybe_hex::<u16>)]
        command: u16,
        #[clap(value_parser=maybe_hex::<u8>)]
        args: Vec<u8>,
    },
    /// Control Logo
    Logo { logo_mode: LogoMode },
    /// Keyboard backlight
    Backlight { brightness: u8 },
    /// Lights always on
    LightOn { always_on: LightsAlwaysOn },
}

#[derive(Args)]
struct PerfModeCommand {
    #[command(subcommand)]
    pub action: PerfModeActionCommand,
}

#[derive(Subcommand)]
enum PerfModeActionCommand {
    /// Set performance mode
    Mode { perf_mode: PerfMode },
    /// Set CPU boost
    Cpu { cpu_boost: CpuBoost },
    /// Set GPU boost
    Gpu { gpu_boost: GpuBoost },
}

#[derive(Args)]
struct FanCommand {
    #[command(subcommand)]
    pub subcommand: FanSubcommand,
}

#[derive(Subcommand)]
enum FanSubcommand {
    /// Set fan mode to auto
    Auto,
    /// Set fan mode to manual
    Manual,
    /// Set fan rpm
    Rpm {
        #[arg(value_parser = clap::value_parser!(u16).range(2000..=5000))]
        rpm: u16,
    },
    /// Control Max Fan Speed Mode
    Max { max_fan_speed_mode: MaxFanSpeedMode },
}

fn main() -> Result<()> {
    let parser = Razerctl::parse();

    if let RazerCtlCommand::Enumerate = parser.command {
        return device::Device::enumerate();
    }

    let device = create_device(parser.pid)?;

    match parser.command {
        RazerCtlCommand::Enumerate => {
            unreachable!("Enumerate handled above")
        }
        RazerCtlCommand::Info => Ok(println!("{}", get_info(&device)?)),
        RazerCtlCommand::Cmd { command, args } => command::custom_command(&device, command, &args),
        RazerCtlCommand::Perf(command) => match command.action {
            PerfModeActionCommand::Mode { perf_mode } => command::set_perf_mode(&device, perf_mode),
            PerfModeActionCommand::Cpu { cpu_boost } => command::set_cpu_boost(&device, cpu_boost),
            PerfModeActionCommand::Gpu { gpu_boost } => command::set_gpu_boost(&device, gpu_boost),
        },
        RazerCtlCommand::Fan(command) => match command.subcommand {
            FanSubcommand::Auto => command::set_fan_mode(&device, FanMode::Auto),
            FanSubcommand::Manual => command::set_fan_mode(&device, FanMode::Manual),
            FanSubcommand::Rpm { rpm } => command::set_fan_rpm(&device, rpm),
            FanSubcommand::Max { max_fan_speed_mode } => {
                command::set_max_fan_speed_mode(&device, max_fan_speed_mode)
            }
        },
        RazerCtlCommand::Logo { logo_mode } => command::set_logo_mode(&device, logo_mode),
        RazerCtlCommand::Backlight { brightness } => {
            command::set_keyboard_brightness(&device, brightness)
        }
        RazerCtlCommand::LightOn {
            always_on: lights_always_on,
        } => command::set_lights_always_on(&device, lights_always_on),
    }
}
