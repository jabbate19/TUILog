use cursive::{Cursive, views::{Dialog, LinearLayout}};

pub fn safe_unwrap<V>(s: &mut Cursive, val: Option<V>) -> V {
    match val {
        Some(v) => v,
        None => {
            s.add_layer(Dialog::around(
                LinearLayout::vertical().child(
                    Dialog::text("Error: None value returned from Option")
                ).child(Dialog::button("Ok", |s| s.pop_layer()))).title("Error"));
        }
    }
}