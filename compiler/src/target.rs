//! What a program is built for, and how (D26, D224, D261, D281, D282).

use lyftar::linking::LinkTarget;

/// A target. Bare-metal ones name the exact board (D281). The names of the macOS and Linux targets aren't decided,
/// so theirs are placeholders, as `--target=` itself is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Placeholder name: `macos_aarch64`.
    MacOsAarch64,
    /// Placeholder name: `linux_aarch64`.
    LinuxAarch64,
    /// Placeholder name: `linux_x86_64`.
    LinuxX86_64,
    /// The owner's board, with a Renesas RA4M1, a Cortex-M4 with hardware floats (D253, D281).
    ArduinoUnoR4Wifi,
}

impl Target {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Target> {
        match name {
            "macos_aarch64" => Some(Target::MacOsAarch64),
            "linux_aarch64" => Some(Target::LinuxAarch64),
            "linux_x86_64" => Some(Target::LinuxX86_64),
            "arduino_uno_r4_wifi" => Some(Target::ArduinoUnoR4Wifi),
            _ => None,
        }
    }

    /// The machine this compiler runs on, which a build is for unless it says otherwise.
    #[must_use]
    pub fn host() -> Target {
        if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            Target::LinuxAarch64
        } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            Target::LinuxX86_64
        } else {
            Target::MacOsAarch64
        }
    }

    #[must_use]
    pub fn triple(self) -> &'static str {
        match self {
            Target::MacOsAarch64 => "arm64-apple-macosx11.0.0",
            Target::LinuxAarch64 => "aarch64-unknown-linux-musl",
            Target::LinuxX86_64 => "x86_64-unknown-linux-musl",
            Target::ArduinoUnoR4Wifi => "thumbv7em-none-eabihf",
        }
    }

    #[must_use]
    pub fn cpu(self) -> &'static str {
        match self {
            Target::MacOsAarch64 => "apple-m1",
            Target::LinuxAarch64 => "generic",
            Target::LinuxX86_64 => "x86-64",
            Target::ArduinoUnoR4Wifi => "cortex-m4",
        }
    }

    #[must_use]
    pub fn link_target(self) -> LinkTarget {
        match self {
            Target::MacOsAarch64 => LinkTarget::MacOsAarch64,
            Target::LinuxAarch64 => LinkTarget::LinuxAarch64,
            Target::LinuxX86_64 => LinkTarget::LinuxX86_64,
            Target::ArduinoUnoR4Wifi => LinkTarget::CortexM4,
        }
    }

    /// Whether it runs on an operating system, which `os` needs (D271).
    #[must_use]
    pub fn has_os(self) -> bool {
        !matches!(self, Target::ArduinoUnoR4Wifi)
    }
}

/// How much `Working` optimizes, with LLVM's own levels (D288).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Optimization {
    O0,
    O1,
    O2,
    O3,
    Os,
    Oz,
}

impl Optimization {
    #[must_use]
    pub fn from_flag(flag: &str) -> Option<Optimization> {
        match flag {
            "--opt=O0" => Some(Optimization::O0),
            "--opt=O1" => Some(Optimization::O1),
            "--opt=O2" => Some(Optimization::O2),
            "--opt=O3" => Some(Optimization::O3),
            "--opt=Os" => Some(Optimization::Os),
            "--opt=Oz" => Some(Optimization::Oz),
            _ => None,
        }
    }
}

/// A build mode, always chosen with its own flag (D26, D224, D282). `Working` says how much it optimizes (D288).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Working(Optimization),
    FinalSafe,
    FinalPerf,
    FinalShit,
    FinalSmall,
    Literal,
}

impl Mode {
    #[must_use]
    pub fn from_flag(flag: &str) -> Option<Mode> {
        match flag {
            "--Working" => Some(Mode::Working(Optimization::O0)),
            "--FinalSafe" => Some(Mode::FinalSafe),
            "--FinalPerf" => Some(Mode::FinalPerf),
            "--FinalShit" => Some(Mode::FinalShit),
            "--FinalSmall" => Some(Mode::FinalSmall),
            "--Literal" => Some(Mode::Literal),
            _ => None,
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Mode::Working(_) => "Working",
            Mode::FinalSafe => "FinalSafe",
            Mode::FinalPerf => "FinalPerf",
            Mode::FinalShit => "FinalShit",
            Mode::FinalSmall => "FinalSmall",
            Mode::Literal => "Literal",
        }
    }

    /// LLVM's pass pipeline. `Working` runs the level it says (D288), and `Literal` runs only what turns stack slots
    /// into registers, rewriting nothing the code says (D282).
    #[must_use]
    pub fn pipeline(self) -> &'static str {
        match self {
            Mode::Working(Optimization::O0) => "default<O0>",
            Mode::Working(Optimization::O1) => "default<O1>",
            Mode::Working(Optimization::O2) => "default<O2>",
            Mode::Working(Optimization::O3) => "default<O3>",
            Mode::Working(Optimization::Os) => "default<Os>",
            Mode::Working(Optimization::Oz) => "default<Oz>",
            Mode::FinalSafe | Mode::FinalPerf | Mode::FinalShit => "default<O3>",
            Mode::FinalSmall => "default<Oz>",
            Mode::Literal => "mem2reg",
        }
    }

    /// Whether runtime checks stay: overflow, bounds and division by zero (standard 10.3, 10.4).
    #[must_use]
    pub fn keeps_checks(self) -> bool {
        matches!(self, Mode::Working(_) | Mode::FinalSafe | Mode::FinalSmall | Mode::Literal)
    }
}
