// trace: D46

fn load [none] returns i32 fails [a, b]:
    return 0;
end fn load

fn.main [none] returns void:
    let $n: i32 = load() otherwise:
        case a:
            return void;
    end otherwise
    return void;
end fn.main
