// trace: D128

fn.main [none] returns void:
    for rows $i in range[from: 1, to: 2]:
        return void;
    end for rows
    for $j in range[from: 1, to: 2]:
        break rows;
    end for
    return void;
end fn.main
