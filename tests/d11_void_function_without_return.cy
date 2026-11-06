// Whether every `void` function says `return void;` isn't decided.
// trace: D11
fn f [] -> void:
end fn f

fn.main [] -> void:
    f();
    return void;
end fn.main
