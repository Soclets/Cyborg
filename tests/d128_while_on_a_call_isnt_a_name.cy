// trace: D128
import std;

fn.pure again [none] -> bool:
    return false;
end fn.pure again

fn.main [none] -> void:
    while again():
        return void;
    end while
    std::io::fmt_print("done{newln}");
    return void;
end fn.main
