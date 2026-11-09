// trace: D146
import std;
@config(default);

fn.main [] -> void:
    let $by: u8 = 3;
    for $i in range[from: -4, to: 4, step_by: $by]:
        std::println($i as str);
    end for
    return void;
end fn.main
