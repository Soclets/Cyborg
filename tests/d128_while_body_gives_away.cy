// trace: D128, D2

fn eat [$s: Given str] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $s: str = "a";
    let mut $n: i32 = 2;
    while $n above 0:
        eat(Give $s);
        change $n = $n - 1;
    end while
    return void;
end fn.main
