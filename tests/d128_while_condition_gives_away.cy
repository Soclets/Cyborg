// The test runs again each time round.
// trace: D128, D2

fn eat [$s: Given str[len.max: 32 bytes]] -> bool:
    return false;
end fn eat

fn.main [none] -> void:
    let $s: str[len.max: 32 bytes] = "a";
    while eat(Give $s):
        let $n: i32 = 1;
    end while
    return void;
end fn.main
