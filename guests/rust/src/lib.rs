wit_bindgen::generate!({
    path: "../../wit",
    world: "string-processor",
});

struct Component;

impl Guest for Component {
    fn process_string(_input: String) -> String {
        unimplemented!()
    }
}

export!(Component);
