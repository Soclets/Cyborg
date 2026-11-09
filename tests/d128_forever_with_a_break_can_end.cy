// trace: D128, D11
@config(default);

fn f [] -> i32:
    forever:
        break;
    end forever
end fn f

fn.main [] -> void:
    return void;
end fn.main
