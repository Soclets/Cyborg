// trace: D128
@config(default);

fn.main [] -> void:
    for $i in range[from: 1, to: 2]:
        change $i = 5;
    end for
    return void;
end fn.main
