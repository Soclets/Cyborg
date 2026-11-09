// The oracle has no C in it.
// trace: D38, D34
import @csup;
@config(default);

fn.main [] -> void:
    shield c_call:
        reason "calls libfoo"
        @csup::call();
    end shield c_call
    return void;
end fn.main
