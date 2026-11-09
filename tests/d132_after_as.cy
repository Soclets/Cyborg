// trace: D132, D7
import std;
@config(default);

fn.main [] -> void:
    let $n: i32 = 5;
    let $s: str = $n as str ?? "none";
    std::println(Lend $s);
    return void;
end fn.main
