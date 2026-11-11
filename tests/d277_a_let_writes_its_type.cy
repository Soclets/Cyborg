// Every `let` writes its type, so one with none is refused, asking for one.
// trace: D277
fn.main [none] returns void:
    let $count = 0;
    return void;
end fn.main
