// trace: D126, D2
import std;

fn eat [$s: Given str] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $xs: std::array<i32>[len: 2 items] = [1, 2];
    let $s: str = "a";
    for $x in $xs:
        if $x == 1:
            std::println("one");
        else:
            eat(Give $s);
        end if
    end for
    return void;
end fn.main
