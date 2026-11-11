// trace: D128

fn.main [none] returns void:
    for $i in range[from: 1, to: 2]:
        change $i set_to 5;
    end for
    return void;
end fn.main
