// `bit_and` and the rest are now the logic words narrowed with `.bit`.
// trace: D361
fn.main [none] -> void:
    let $a: i32 = 6;
    let $b: i32 = $a bit_or 1;
    return void;
end fn.main
