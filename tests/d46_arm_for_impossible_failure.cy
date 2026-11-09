// trace: D46
@config(default);

fn load [] -> i32 fails [a]:
    return 0;
end fn load

fn.main [] -> void:
    let $n: i32 = load() otherwise:
        case a:
            return void;
        case c:
            return void;
    end otherwise
    return void;
end fn.main
