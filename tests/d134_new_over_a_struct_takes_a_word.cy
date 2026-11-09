// trace: D134, D3
@config(default);

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn f [$w: Given wrapped] -> wrapped:
    return $w;
end fn f

fn.main [] -> void:
    return void;
end fn.main
