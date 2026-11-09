// A float literal's example names `f64`, since `: i32 = 1.5` would be refused too.
// trace: D216, D50, D59
@config(default);

fn.main [] -> void:
    let $x := 1.5;
    return void;
end fn.main
