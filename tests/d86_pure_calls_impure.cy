// trace: D86

fn g [none] -> i32:
    return 1;
end fn g

fn.pure f [none] -> i32:
    return g();
end fn.pure f

fn.main [none] -> void:
    return void;
end fn.main
