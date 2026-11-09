// trace: D128
@config(default);
import std;

fn.pure again [] -> bool:
    return false;
end fn.pure again

fn.main [] -> void:
    while again():
        return void;
    end while
    std::io::fmt_print("done{newln}");
    return void;
end fn.main
