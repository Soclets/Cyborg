// trace: D124, D2

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] -> bool:
    return 1 below 2;
end fn eat

fn.main [none] -> void:
    let $b: box = box[n = 1];
    if 1 below 2 or eat(Give $b):
        let $c: box = Give $b;
    end if
    return void;
end fn.main
