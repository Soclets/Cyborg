// Whether you can add a method to a type that isn't yours isn't stated.
// trace: D130

fn i32.%double [$n: i32] returns i32:
    return $n x 2;
end fn i32.%double

fn.main [none] returns void:
    return void;
end fn.main
