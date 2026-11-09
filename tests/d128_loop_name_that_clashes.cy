// trace: D128, D75
@config(default);

fn.main [] -> void:
    for x $i in range[from: 1, to: 2]:
        break x;
    end for x
    return void;
end fn.main
