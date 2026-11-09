// trace: D127, D75
import std;
@config(default);

fn true [] -> i32:
    return 1;
end fn true

fn.main [] -> void:
    let $a: i32 = my::true();
    let $b: bool = true;
    std::println($a as str);
    if $b:
        std::println("still a bool");
    end if
    return void;
end fn.main
