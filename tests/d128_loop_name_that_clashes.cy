// trace: D128, D75

fn.main [none] returns void:
    for x $i in range[from: 1, to: 2]:
        break x;
    end for x
    return void;
end fn.main
