// A block's own names mustn't throw off what's known about names declared after it.
// trace: D2

struct box:
    n: i32;
end struct box

fn eat [$b: Given box] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    if 1 below 2:
        let $inner: i32 set_to 1;
    end if
    let $b: box set_to box[n set_to 1];
    eat(Give $b);
    eat(Give $b);
    return void;
end fn.main
