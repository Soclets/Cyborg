// trace: D3

struct box:
    n: i32;
end struct box

fn read [$b: box] returns i32:
    return $b.n;
end fn read

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    let $n: i32 set_to read($b);
    return void;
end fn.main
