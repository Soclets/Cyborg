// trace: D128
@config(default);

fn.main [] -> void:
    for $i in range[from: 1, to: 5, before: 5]:
        return void;
    end for
    return void;
end fn.main
