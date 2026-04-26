wit_bindgen::generate!({
    path: "../../wit",
    world: "string-processor",
});

struct Component;

impl Guest for Component {
    fn process_string(input: String) -> String {
        input.to_ascii_uppercase()
    }
}

export!(Component);
