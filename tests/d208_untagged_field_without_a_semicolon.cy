// trace: D208, D35
import @csup;

shield c_number:
    reason "matches the union in libfoo's header"
    union.@csup::untagged number:
        i: i32
    end union.@csup::untagged number
end shield c_number

fn.main [none] -> void:
    return void;
end fn.main
