// trace: D128
import std;
@config(default);

fn show [$label: str, $n: i32] -> void:
    std::io::fmt_print("{}{}{newln}", $label, $n);
    return void;
end fn show

fn.main [] -> void:
    for $i in range[from: 1, to: 3]:
        show("to ", $i);
    end for
    for $i in range[from: 1, before: 3]:
        show("before ", $i);
    end for
    for $i in range[from: 0, to: 4, step_by: 2]:
        show("by two ", $i);
    end for
    for $i in range[from: 3, to: 1, step_by: -1]:
        show("down ", $i);
    end for
    for $i in range[from: 3, before: 1, step_by: -1]:
        show("down before ", $i);
    end for
    for $i in range[from: 2, to: 2]:
        show("one item ", $i);
    end for
    for $i in range[from: 2, before: 2]:
        show("no items ", $i);
    end for
    for $i in range[from: 1, to: 6, step_by: 2]:
        show("past the end ", $i);
    end for
    return void;
end fn.main
