// trace: D29, D84

fn wrong [$n: i32] returns i32:
    ensures result at_least $n;

    return $n - 1;
end fn wrong

fn.main [none] returns void:
    let $a: i32 = wrong(5);
    return void;
end fn.main
