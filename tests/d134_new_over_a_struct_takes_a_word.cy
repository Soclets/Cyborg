// trace: D134, D3

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn f [$w: Given wrapped] -> wrapped:
    return $w;
end fn f

fn.main [none] -> void:
    return void;
end fn.main
