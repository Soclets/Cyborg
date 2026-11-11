// trace: D140, D42

fn.main [none] -> void:
    let $a: u64 = 18446744073709551615;
    let $b: u64 = $a + 1;
    return void;
end fn.main
