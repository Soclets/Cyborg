//! The check a program tuned with `--machine.cpu=` makes when it starts (D365): every instruction family it was built
//! to use is asked for in the way its system answers, and a missing one stops it with an error naming it, instead of
//! a crash midway. A feature this table doesn't know how to ask about isn't checked.

use crate::target::Target;

/// How a system says whether its processor has a feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Probe {
    /// macOS: `sysctlbyname` on `hw.optional.arm.` and this name, which is 1 when it's there.
    Sysctl(&'static str),
    /// Linux on aarch64: a bit of `getauxval(AT_HWCAP)`, or of `AT_HWCAP2` when `second` is set.
    Hwcap { second: bool, bit: u32 },
    /// x86-64: a bit of a register from `cpuid` with this leaf and subleaf 0, the register as 0 to 3 for `eax` to
    /// `edx`.
    Cpuid { leaf: u32, register: u32, bit: u32 },
}

/// The aarch64 features as LLVM names them, with their names on macOS and their bits on Linux.
const AARCH64: &[(&str, &str, bool, u32)] = &[
    ("aes", "FEAT_AES", false, 3),
    ("sha2", "FEAT_SHA256", false, 6),
    ("crc", "FEAT_CRC32", false, 7),
    ("lse", "FEAT_LSE", false, 8),
    ("fullfp16", "FEAT_FP16", false, 9),
    ("rdm", "FEAT_RDM", false, 12),
    ("jsconv", "FEAT_JSCVT", false, 13),
    ("complxnum", "FEAT_FCMA", false, 14),
    ("rcpc", "FEAT_LRCPC", false, 15),
    ("sha3", "FEAT_SHA3", false, 17),
    ("dotprod", "FEAT_DotProd", false, 20),
    ("sve", "FEAT_SVE", false, 22),
    ("fp16fml", "FEAT_FHM", false, 23),
    ("rcpc-immo", "FEAT_LRCPC2", false, 26),
    ("flagm", "FEAT_FlagM", false, 27),
    ("ssbs", "FEAT_SSBS", false, 28),
    ("sb", "FEAT_SB", false, 29),
    ("pauth", "FEAT_PAuth", false, 30),
    ("sve2", "FEAT_SVE2", true, 1),
    ("frint3264", "FEAT_FRINTTS", true, 8),
    ("i8mm", "FEAT_I8MM", true, 13),
    ("bf16", "FEAT_BF16", true, 14),
    ("bti", "FEAT_BTI", true, 17),
    ("mte", "FEAT_MTE", true, 18),
    ("sme", "FEAT_SME", true, 23),
];

/// The x86-64 features as LLVM names them, with where `cpuid` shows them.
const X86_64: &[(&str, u32, u32, u32)] = &[
    ("sse3", 1, 2, 0),
    ("pclmul", 1, 2, 1),
    ("ssse3", 1, 2, 9),
    ("fma", 1, 2, 12),
    ("cx16", 1, 2, 13),
    ("sse4.1", 1, 2, 19),
    ("sse4.2", 1, 2, 20),
    ("movbe", 1, 2, 22),
    ("popcnt", 1, 2, 23),
    ("aes", 1, 2, 25),
    ("xsave", 1, 2, 26),
    ("avx", 1, 2, 28),
    ("f16c", 1, 2, 29),
    ("rdrnd", 1, 2, 30),
    ("bmi", 7, 1, 3),
    ("avx2", 7, 1, 5),
    ("bmi2", 7, 1, 8),
    ("avx512f", 7, 1, 16),
    ("avx512dq", 7, 1, 17),
    ("rdseed", 7, 1, 18),
    ("adx", 7, 1, 19),
    ("avx512cd", 7, 1, 28),
    ("sha", 7, 1, 29),
    ("avx512bw", 7, 1, 30),
    ("avx512vl", 7, 1, 31),
    ("lzcnt", 0x8000_0001, 2, 5),
];

/// What a program built with these features (LLVM's `+name,-name` list) checks when it starts on this target.
#[must_use]
pub fn probes(target: Target, features: &str) -> Vec<(String, Probe)> {
    // trace: D365
    let wanted = features.split(',').filter_map(|feature| feature.strip_prefix('+'));
    let mut found = Vec::new();
    for feature in wanted {
        let probe = match target {
            Target::MacOsAarch64 => {
                AARCH64.iter().find(|(name, ..)| *name == feature).map(|(_, sysctl, ..)| Probe::Sysctl(sysctl))
            }
            Target::LinuxAarch64 => AARCH64
                .iter()
                .find(|(name, ..)| *name == feature)
                .map(|&(_, _, second, bit)| Probe::Hwcap { second, bit }),
            Target::LinuxX86_64 => X86_64
                .iter()
                .find(|(name, ..)| *name == feature)
                .map(|&(_, leaf, register, bit)| Probe::Cpuid { leaf, register, bit }),
            Target::ArduinoUnoR4Wifi => None,
        };
        if let Some(probe) = probe {
            found.push((feature.to_string(), probe));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::{Probe, probes};
    use crate::target::Target;

    #[test]
    fn d365_each_system_asks_its_own_way() {
        let features = "+aes,-sve,+avx2,+lse,+sve2";
        assert_eq!(
            probes(Target::MacOsAarch64, features),
            vec![
                ("aes".to_string(), Probe::Sysctl("FEAT_AES")),
                ("lse".to_string(), Probe::Sysctl("FEAT_LSE")),
                ("sve2".to_string(), Probe::Sysctl("FEAT_SVE2")),
            ]
        );
        assert_eq!(
            probes(Target::LinuxAarch64, features),
            vec![
                ("aes".to_string(), Probe::Hwcap { second: false, bit: 3 }),
                ("lse".to_string(), Probe::Hwcap { second: false, bit: 8 }),
                ("sve2".to_string(), Probe::Hwcap { second: true, bit: 1 }),
            ]
        );
        assert_eq!(
            probes(Target::LinuxX86_64, features),
            vec![
                ("aes".to_string(), Probe::Cpuid { leaf: 1, register: 2, bit: 25 }),
                ("avx2".to_string(), Probe::Cpuid { leaf: 7, register: 1, bit: 5 }),
            ]
        );
        assert!(probes(Target::ArduinoUnoR4Wifi, features).is_empty());
    }
}
