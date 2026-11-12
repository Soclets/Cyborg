// A bare parameter takes `Lend`, so that's the one fix.
// trace: D2, D16

struct box:
    n: i32;
end struct box

fn read [$b: Lent box] -> i32:
    return $b.n;
end fn read

fn.main [none] -> void:
    let $b: box = box[n = 1];
    let $n: i32 = read(Give $b);
    return void;
end fn.main
