// An endless loop says `forever`.
// trace: D128

fn.main [none] returns void:
    for $i in range[from: 1]:
        return void;
    end for
    return void;
end fn.main
