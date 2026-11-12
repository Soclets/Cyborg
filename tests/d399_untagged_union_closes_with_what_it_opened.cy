// The closer repeats all of what it opens, the limb's narrowing included.
// trace: D121, D399
import std;
import @csup;

shield c_number:
    reason "matches the union in libfoo's header"
    union.@csup::untagged number:
        i: i32;
    end union.untagged number
end shield c_number

fn.main [none] -> void:
    return void;
end fn.main
