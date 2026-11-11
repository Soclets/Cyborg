// The test runs again each time round.
// trace: D128, D2

fn eat [$s: Given str] returns bool:
    return false;
end fn eat

fn.main [none] returns void:
    let $s: str = "a";
    while eat(Give $s):
        let $n: i32 = 1;
    end while
    return void;
end fn.main
