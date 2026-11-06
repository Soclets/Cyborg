// The second test hasn't run when the first block runs.
// trace: D126, D2
import std;

fn eat [$s: Given str] -> bool:
    return 1 < 2;
end fn eat

fn.main [] -> void:
    let $s: str = "a";
    if 1 < 2:
        std::println(Lend $s);
    else if eat(Give $s):
        return void;
    end if
    return void;
end fn.main
