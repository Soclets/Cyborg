// trace: D128
import std;

fn.main [] -> void:
    for rows $a in range[from: 1, to: 3]:
        for $b in range[from: 1, to: 3]:
            if $a == 2 and $b == 2:
                break rows;
            end if
            std::println($a as str, " ", $b as str);
        end for
    end for rows
    std::println("done");
    return void;
end fn.main
