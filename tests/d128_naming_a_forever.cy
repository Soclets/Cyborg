// 4.8 says a loop can be named, and only shows it on `for`.
// trace: D128
fn.main [] -> void:
    forever rows:
        break rows;
    end forever rows
    return void;
end fn.main
