// trace: D116

fn.main [none] -> void:
    let $a: i32 = 1;
    let $b: i32 = 2;
    let $s: str[len.max: 20 bytes] = $a + $b as str[len.max: 20 bytes];
    return void;
end fn.main
