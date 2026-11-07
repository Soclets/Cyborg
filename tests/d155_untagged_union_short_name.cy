// trace: D155, D38
import @csup;

use @csup::union.untagged as untagged;

shield c_number:
    reason "matches the union in libfoo's header"
    untagged number:
        i: i32
        f: f32
    end untagged number
end shield c_number

fn.main [] -> void:
    return void;
end fn.main
