use uuid::Uuid;

pub fn gen_uuid() -> String {
    Uuid::new_v4()
        .to_hyphenated()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}
