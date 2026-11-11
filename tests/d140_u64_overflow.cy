// trace: D140, D42

fn.main [none] returns void:
    let $a: u64 set_to 18446744073709551615;
    let $b: u64 set_to $a + 1;
    return void;
end fn.main
