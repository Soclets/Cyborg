// What can convert to what isn't decided, and a `bool` isn't a number.
// trace: D127, D122

fn.main [none] returns void:
    let $s: str = true as str;
    return void;
end fn.main
