// trace: D129, D2, D126
import std;

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn close [$f: Given open_file] -> void:
    finish $f;
    return void;
end fn close

fn end_it [$f: Given open_file, $quick: bool] -> void:
    if $quick:
        finish $f;
    else:
        close(Give $f);
    end if
    std::println("ended");
    return void;
end fn end_it

fn.main [] -> void:
    end_it(open_file[fd = 1], true);
    end_it(open_file[fd = 2], false);
    return void;
end fn.main
