// Only one block runs, so each may give it away, and it's dead after.
// trace: D126, D2
import std;

fn eat [$s: Given str] -> void:
    std::io::fmt_print("{}{newln}", $s);
    return void;
end fn eat

fn pick [$s: Given str, $first: bool] -> void:
    if $first:
        eat(Give $s);
    else:
        eat(Give $s);
    end if
    return void;
end fn pick

fn.main [none] -> void:
    pick("one", 1 < 2);
    pick("two", 2 < 1);
    return void;
end fn.main
