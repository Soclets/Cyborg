// trace: D128
import std;

fn.pure again [] -> bool:
    return false;
end fn.pure again

fn.main [] -> void:
    while again():
        return void;
    end while
    std::println("done");
    return void;
end fn.main
