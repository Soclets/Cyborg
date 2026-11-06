// trace: D82
fn.main [] -> void:
    shield c_call:
        reason "calls libfoo"
        @csup::call();
    end shield c_call
    return void;
end fn.main
