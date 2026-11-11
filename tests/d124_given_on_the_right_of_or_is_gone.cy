// trace: D124, D2

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] returns bool:
    return 1 below 2;
end fn eat

fn.main [none] returns void:
    let $b: box set_to box[n set_to 1];
    if 1 below 2 or eat(Give $b):
        let $c: box set_to Give $b;
    end if
    return void;
end fn.main
