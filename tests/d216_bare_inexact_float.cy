// A float that f64 rounds gets D59's `~` in the example.
// trace: D216, D50, D59
@config(default);

fn.main [] -> void:
    let $x := 0.1;
    return void;
end fn.main
