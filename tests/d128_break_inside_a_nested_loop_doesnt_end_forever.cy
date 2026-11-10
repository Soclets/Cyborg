// That `break` stops the `for`, so the `forever` around it still has no way out.
// trace: D128, D11
import std;

fn f [none] -> i32:
    forever:
        for $i in range[from: 1, to: 2]:
            break;
        end for
        return 1;
    end forever
end fn f

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", f());
    return void;
end fn.main
