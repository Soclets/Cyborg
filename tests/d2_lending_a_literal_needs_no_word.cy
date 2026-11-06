// trace: D2
import std;

fn f [$s: str] -> void:
    std::println(Lend $s);
    return void;
end fn f

fn g [$s: Given str] -> void:
    std::println(Lend $s);
    return void;
end fn g

fn.main [] -> void:
    f("fresh");
    g("fresh too");
    return void;
end fn.main
