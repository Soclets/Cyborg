// `import std::units::time;` gives every unit in the family, and nothing else from `std`.
// trace: D199, D198
@config(default);
import std::units::time;

fn.main [] -> void:
    let $a: i32.std::units::time.seconds = 60 std::units::time.seconds;
    return void;
end fn.main
