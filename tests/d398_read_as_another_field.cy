// 9.6's example: an untagged union built with one field and read as another, as the same bytes.
// trace: D398, D399
import std;
import os;
import @csup;

shield c_number:
    reason "matches the union in libfoo's header"
    union.@csup::untagged number:
        i: i32;
        f: f32.ieee;
    end union.@csup::untagged number
end shield c_number

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    shield read_bits:
        reason "reading the bits libfoo wrote as an i32"
        let $n: union.@csup::untagged number = number[f = 1.5];
        let $bits: i32 = $n.i;
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $bits);
    end shield read_bits
    return void;
end fn.main
