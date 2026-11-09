// trace: D139
@config(default);

fn f [] -> void fails [a]:
    return void;
end fn f

fn.main [] -> void:
    f() otherwise:
        case a:
            instead 1;
    end otherwise
    return void;
end fn.main
