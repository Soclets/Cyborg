// An endless loop says `forever`.
// trace: D128
@config(default);

fn.main [] -> void:
    for $i in range[from: 1]:
        return void;
    end for
    return void;
end fn.main
