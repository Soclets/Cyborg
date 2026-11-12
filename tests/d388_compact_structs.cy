// `struct.compact` sorts its fields by alignment, largest first, for little padding and the
// same layout everywhere, and its fields are read by name as in any struct.
// trace: D388, D319
import std;
import os;

struct.compact reading:
    flag: u8;
    when: u64;
    level: u32;
end struct.compact reading

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[flag = 1, when = 99, level = 7];
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $r.flag, $r.when, $r.level);
    return void;
end fn.main
