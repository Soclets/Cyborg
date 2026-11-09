// A `///` attaches to what's below it, and nothing says what one after code attaches to.
// trace: D205, D118
@config(default);

fn.main [] -> void:
    let $x: i32 = 1; /// a doc
    return void;
end fn.main
