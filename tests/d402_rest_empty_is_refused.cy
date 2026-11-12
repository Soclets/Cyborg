// `empty` means a thing holds nothing, and a union's bytes always exist.
// trace: D402, D103
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
        reason "rest = empty"
        let $n: union.@csup::untagged bits = bits[b = 1, rest = empty];
    end shield read
    return void;
end fn.main
