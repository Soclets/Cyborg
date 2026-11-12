// The bytes the written field doesn't cover are a choice, so the build says it.
// trace: D401, D402
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
        reason "leaving out the rest"
        let $n: union.@csup::untagged bits = bits[b = 1];
    end shield read
    return void;
end fn.main
