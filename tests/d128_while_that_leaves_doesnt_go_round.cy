// With a body that always leaves, there's no second time for the test to run.
// trace: D128, D2

fn eat [$s: Given str] -> bool:
    return false;
end fn eat

fn.main [none] -> void:
    let $s: str = "a";
    while eat(Give $s):
        return void;
    end while
    return void;
end fn.main
