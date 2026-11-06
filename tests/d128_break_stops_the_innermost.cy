// trace: D128
import std;

fn.main [] -> void:
    for $a in range[from: 1, to: 2]:
        for $b in range[from: 1, to: 3]:
            if $b == 2:
                break;
            end if
            std::println($a as str, " ", $b as str);
        end for
    end for
    return void;
end fn.main
