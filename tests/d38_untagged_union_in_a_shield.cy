// trace: D35, D38, D257
import @csup;

shield c_number:
    reason "matches the union in libfoo's header"
    union.@csup::untagged number:
        i: i32;
        f: f32.ieee;
    end union.@csup::untagged number
end shield c_number

fn.main [none] -> void:
    return void;
end fn.main
