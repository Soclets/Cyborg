// A failure while baking stops the build, not the run.
// trace: D91, D42
@config(default);

const $a: i32 = 2147483647 + 1;

fn.main [] -> void:
    return void;
end fn.main
