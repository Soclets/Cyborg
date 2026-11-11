// A value given on one path can't be used after, on any path.
// trace: D2

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] returns void:
    return void;
end fn eat

fn read [$b: box] returns i32:
    return $b.n;
end fn read

fn.main [none] returns void:
    let $b: box = box[n = 1];
    if 1 below 2:
        eat(Give $b);
    end if
    let $n: i32 = read(Lend $b);
    return void;
end fn.main
