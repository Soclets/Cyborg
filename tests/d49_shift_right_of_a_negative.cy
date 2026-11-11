// Whether `shr` keeps a negative number's sign isn't stated.
// trace: D49

fn.main [none] returns void:
    let $n: i32 set_to -4;
    let $a: i32 set_to $n shr 1;
    return void;
end fn.main
