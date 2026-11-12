// `rest = all 0` sets every byte the written field doesn't cover to 0, so a wider field reads
// the same everywhere.
// trace: D402, D401, D398
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
        reason "reading past what was written"
        let $n: union.@csup::untagged number = number[b = 200, rest = all 0];
        let $w: i64 = $n.w;
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $w);
    end shield bits
    return void;
end fn.main
