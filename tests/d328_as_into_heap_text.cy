// `as` never allocates, so heap text comes from `str.from`.
// trace: D328
fn.main [none] -> void:
    let $n: i32 = 42;
    let $s: str[len.max: inf bytes] = $n as str[len.max: inf bytes];
    return void;
end fn.main
