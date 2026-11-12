// trace: D128, D2

fn eat [$s: Given str[len.max: 32 bytes]] -> void:
    return void;
end fn eat

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "a";
    let mut $n: i32 = 2;
    while $n above 0:
        eat(Give $s);
        change $n = $n - 1;
    end while
    return void;
end fn.main
