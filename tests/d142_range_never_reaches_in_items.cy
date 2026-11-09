// trace: D142, D128
@config(default);

fn.main [] -> void:
    for $i in range[from: 5 items, to: 1 items]:
        return void;
    end for
    return void;
end fn.main
