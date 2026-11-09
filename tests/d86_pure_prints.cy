// trace: D86
import std;
@config(default);

fn.pure f [] -> i32:
    std::println("hi");
    return 1;
end fn.pure f

fn.main [] -> void:
    return void;
end fn.main
