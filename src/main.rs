mod parameters;
fn main() {
    let parser = parameters::ParamsParser::new();
    let params = parser.parse_params();
    println!("{:?}", params);
}
