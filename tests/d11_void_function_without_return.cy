// Whether every `void` function says `return void;` isn't decided.
// trace: D11

fn f [none] -> void:
end fn f

fn.main [none] -> void:
    f();
    return void;
end fn.main
