use uuid::Uuid;

pub fn gen_uuid() -> String {
    Uuid::new_v4()
        .to_hyphenated()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}

pub fn or<T: Sized>(cond: bool, truth_val: T, false_val: T) -> T {
    if cond {
        truth_val
    } else {
        false_val
    }
}
