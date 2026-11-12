// Building or reading a C union is shield code too.
// trace: D398, D38
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
    let $n: union.@csup::untagged number = number[i = 1];
    return void;
end fn.main
