// trace: D139

fn f [none] returns void fails [a]:
    return void;
end fn f

fn.main [none] returns void:
    f() otherwise:
        case a:
            instead 1;
    end otherwise
    return void;
end fn.main
