// A `.stable` float read from bits that make NaN is a failure, as 10.4 says.
// trace: D401, D158
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
        reason "a float from NaN bits"
        let $n: union.@csup::untagged bits = bits[u = 2143289344, rest = all 0];
        let $x: f32.stable = $n.f;
    end shield read
    return void;
end fn.main
