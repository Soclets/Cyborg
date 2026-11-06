// trace: D128
import std;

fn.main [] -> void:
    for rows $i in range[from: 1, to: 2]:
        break rows;
    end for rows
    for rows $j in range[from: 1, to: 2]:
        std::println("second");
        break rows;
    end for rows
    return void;
end fn.main
