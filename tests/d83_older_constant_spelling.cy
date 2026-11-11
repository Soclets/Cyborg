// `:=` used to make a constant; D216 now gives it a new meaning, and a written type uses `=`.
// trace: D83, D58, D216

fn.main [none] returns void:
    let $a: i32 := 1;
    return void;
end fn.main
