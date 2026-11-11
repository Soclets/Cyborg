// trace: D128, D2

fn eat [$s: Given str] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $s: str set_to "a";
    let mut $n: i32 set_to 2;
    while $n above 0:
        eat(Give $s);
        change $n set_to $n - 1;
    end while
    return void;
end fn.main
