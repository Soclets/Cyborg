// trace: D86
@config(default);

fn g [] -> i32:
    return 1;
end fn g

fn.pure f [] -> i32:
    return g();
end fn.pure f

fn.main [] -> void:
    return void;
end fn.main
