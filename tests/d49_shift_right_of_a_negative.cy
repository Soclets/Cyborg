// Whether `shr` keeps a negative number's sign isn't stated.
// trace: D49

fn.main [none] -> void:
    let $n: i32 = -4;
    let $a: i32 = $n shr 1;
    return void;
end fn.main
