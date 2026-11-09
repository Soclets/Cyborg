// trace: D128
import std;
@config(default);

fn.main [] -> void:
    for rows $a in range[from: 1, to: 3]:
        for $b in range[from: 1, to: 3]:
            if $a == 2 and $b == 2:
                break rows;
            end if
            std::io::fmt_print("{} {}{newln}", $a, $b);
        end for
    end for rows
    std::io::fmt_print("done{newln}");
    return void;
end fn.main
