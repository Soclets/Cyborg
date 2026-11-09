// trace: D208, D35
@config(default);
import @csup;

shield c_number:
    reason "matches the union in libfoo's header"
    @csup::union.untagged number:
        i: i32
    end @csup::union.untagged number
end shield c_number

fn.main [] -> void:
    return void;
end fn.main
