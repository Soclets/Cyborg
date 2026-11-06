// A word on the next line isn't this number's unit, so this is a missing `;`.
// trace: D33, D99
fn f [] -> void:
    return void;
end fn f

fn.main [] -> void:
    let $a: i32 = 1
    f();
    return void;
end fn.main
