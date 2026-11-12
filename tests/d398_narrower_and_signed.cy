// Reading a narrower field takes the first bytes, least significant first, and a signed field
// takes its top bit as the sign.
// trace: D398
import std;
import os;
import @csup;

shield c_number:
    reason "matches the union in libfoo's header"
    union.@csup::untagged number:
        i: i32;
        b: u8;
        f: f32.ieee;
        w: i64;
    end union.@csup::untagged number
end shield c_number


fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    shield bits:
        reason "reading the bytes back as other fields"
        let $n: union.@csup::untagged number = number[i = -2, rest = all 0];
        std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $n.b, $n.i);
        let $m: union.@csup::untagged number = number[f = -0.5, rest = all 0];
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $m.i);
    end shield bits
    return void;
end fn.main
