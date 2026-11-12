// `@csup::union.untagged` dates from before limbs joined base families.
// trace: D399
import std;
import @csup;

shield c_number:
    reason "matches the union in libfoo's header"
    @csup::union.untagged number:
        i: i32;
    end @csup::union.untagged number
end shield c_number

fn.main [none] -> void:
    return void;
end fn.main
