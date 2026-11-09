// A note for whoever reads this.
// trace: D118, D119
import std;
@config(default);

/// Says hello.
fn hello [] -> void:
    // a note inside
    std::println("hello"); // and one after code
    //
    return void;
end fn hello

comment why_this_exists:
    This text isn't code: let $x: i32 = "nothing";
    comment nested_one:
        Nor is this.
    end comment nested_one
    comment:is not an opener, because it isn't alone like that
end comment why_this_exists

fn.main [] -> void:
    hello();
    return void;
end fn.main
