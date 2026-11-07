// trace: D149
import std;

type.new title = str;

fn.main [] -> void:
    let $s: str = "a";
    let $t: title = Copy $s as title;
    std::println(Lend $s);
    return void;
end fn.main
