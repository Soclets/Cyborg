// Using a call for its effect and dropping its value isn't in the standard yet.
// trace: D11
fn f [] -> i32:
    return 1;
end fn f

fn.main [] -> void:
    f();
    return void;
end fn.main
