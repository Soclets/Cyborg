// What can convert to what isn't decided, and a `bool` isn't a number.
// trace: D127, D122

fn.main [none] -> void:
    let $s: str[len.max: 20 bytes] = true as str[len.max: 20 bytes];
    return void;
end fn.main
