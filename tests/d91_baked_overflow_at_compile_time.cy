// A failure while baking stops the build, not the run.
// trace: D91, D42

const $a: i32 set_to 2147483647 + 1;

fn.main [none] returns void:
    return void;
end fn.main
