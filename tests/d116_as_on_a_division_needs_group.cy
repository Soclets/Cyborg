// trace: D116
@config(default);

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = 2;
    let $s: str = $a + $b as str;
    return void;
end fn.main
