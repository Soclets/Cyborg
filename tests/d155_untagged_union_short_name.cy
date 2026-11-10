// trace: D155, D38, D257
import @csup;

use @csup::union.untagged as untagged;

shield c_number:
    reason "matches the union in libfoo's header"
    untagged number:
        i: i32;
        f: f32.ieee;
    end untagged number
end shield c_number

fn.main [none] -> void:
    return void;
end fn.main
