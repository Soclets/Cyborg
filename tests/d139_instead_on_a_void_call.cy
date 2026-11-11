// trace: D139

fn f [none] -> void fails [a]:
    return void;
end fn f

fn.main [none] -> void:
    f() otherwise:
        case a:
            instead 1;
    end otherwise
    return void;
end fn.main
