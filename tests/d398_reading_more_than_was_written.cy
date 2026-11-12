// Reading more bytes than the field written holds isn't said.
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
    shield bits:
        reason "reading past what was written"
        let $n: union.@csup::untagged number = number[b = 1];
        let $w: i64 = $n.w;
    end shield bits
    return void;
end fn.main
