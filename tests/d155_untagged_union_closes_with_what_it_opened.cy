// trace: D155, D121
import @csup;

use @csup::union.untagged as untagged;

shield c_number:
    reason "matches the union in libfoo's header"
    untagged number:
        i: i32;
    end @csup::union.untagged number
end shield c_number

fn.main [none] returns void:
    return void;
end fn.main
