// A C union is built by naming exactly the one field written.
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
        reason "two fields at once"
        let $n: union.@csup::untagged number = number[i = 1, b = 2, rest = all 0];
    end shield bits
    return void;
end fn.main
