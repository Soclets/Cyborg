// trace: D126, D11
import std;
@config(default);

fn.main [] -> void:
    if 1 < 2:
        std::println("yes");
        return void;
    else:
        return void;
    end if
end fn.main
