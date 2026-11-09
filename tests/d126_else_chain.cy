// trace: D126
import std;
@config(default);

fn size [$n: i32] -> void:
    if $n > 100:
        std::println("huge");
    else if $n > 0:
        std::println("big");
    else:
        std::println("small");
    end if
    return void;
end fn size

fn.main [] -> void:
    size(101);
    size(1);
    size(0);
    return void;
end fn.main
