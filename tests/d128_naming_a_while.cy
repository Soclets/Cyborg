// trace: D128
@config(default);

fn.main [] -> void:
    while rows 1 < 2:
        break rows;
    end while rows
    return void;
end fn.main
