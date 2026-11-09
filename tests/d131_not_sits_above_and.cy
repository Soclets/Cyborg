// trace: D131, D117
import std;
@config(default);

fn.main [] -> void:
    let $t: bool = true;
    let $f: bool = false;
    if not $f and $t:
        std::println("(not f) and t");
    end if
    if not $t and $t:
        std::println("not reached");
    end if
    if not $t or $t:
        std::println("(not t) or t");
    end if
    if $t and not $f:
        std::println("t and (not f)");
    end if
    return void;
end fn.main
