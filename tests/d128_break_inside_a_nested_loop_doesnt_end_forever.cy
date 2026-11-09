// That `break` stops the `for`, so the `forever` around it still has no way out.
// trace: D128, D11
import std;
@config(default);

fn f [] -> i32:
    forever:
        for $i in range[from: 1, to: 2]:
            break;
        end for
        return 1;
    end forever
end fn f

fn.main [] -> void:
    std::println(f() as str);
    return void;
end fn.main
