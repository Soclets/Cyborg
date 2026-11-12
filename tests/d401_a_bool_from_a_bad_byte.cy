// A read is checked like any value of its type, so a `bool` from a byte that's neither 0 nor
// 1 is a failure, as 10.4 says.
// trace: D401
import std;
import os;
import @csup;

shield c_bits:
    reason "matches a union in libfoo's header"
    union.@csup::untagged bits:
        u: u32;
        b: u8;
        flag: bool;
        f: f32.stable;
    end union.@csup::untagged bits
end shield c_bits


fn.main [none] -> void:
    shield read:
        reason "a bool from a byte"
        let $n: union.@csup::untagged bits = bits[b = 2, rest = all 0];
        let $yes: bool = $n.flag;
    end shield read
    return void;
end fn.main
