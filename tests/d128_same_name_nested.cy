// trace: D128
@config(default);

fn.main [] -> void:
    for rows $i in range[from: 1, to: 2]:
        for rows $j in range[from: 1, to: 2]:
            break rows;
        end for rows
    end for rows
    return void;
end fn.main
