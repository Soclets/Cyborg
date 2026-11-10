// A negative float that f64 rounds keeps its sign after the `~`.
// trace: D216, D50, D59

fn.main [none] -> void:
    let $x := -0.1;
    return void;
end fn.main
