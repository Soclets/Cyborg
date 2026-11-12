// trace: D116

fn.main [none] -> void:
    let $b: bool = 1 below 2;
    let $a: str[len.max: 20 bytes] = not $b as str[len.max: 20 bytes];
    return void;
end fn.main
