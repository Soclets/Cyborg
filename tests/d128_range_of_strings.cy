// trace: D128
@config(default);

fn.main [] -> void:
    for $i in range[from: "a", to: "b"]:
        return void;
    end for
    return void;
end fn.main
