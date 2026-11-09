// trace: D129, D75
import std;
@config(default);

fn finish [] -> void:
    std::println("mine");
    return void;
end fn finish

fn.main [] -> void:
    my::finish();
    return void;
end fn.main
