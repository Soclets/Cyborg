// trace: D86

fn g [none] returns i32:
    return 1;
end fn g

fn.pure f [none] returns i32:
    return g();
end fn.pure f

fn.main [none] returns void:
    return void;
end fn.main
