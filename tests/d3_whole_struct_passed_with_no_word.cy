// trace: D3

struct box:
    n: i32;
end struct box

fn read [$b: box] -> i32:
    return $b.n;
end fn read

fn.main [none] -> void:
    let $b: box = box[n = 1];
    let $n: i32 = read($b);
    return void;
end fn.main
