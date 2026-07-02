#![no_main]

use btpc_core::bencode::{ByteString, OwnedValue, Value, ValueKind, parse, validate_canonical};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(parsed) = parse(data) else {
        return;
    };
    let Some(owned) = to_owned(&parsed) else {
        return;
    };
    let encoded = owned.to_vec().expect("Vec writes cannot fail");
    validate_canonical(&encoded).expect("owned encoding is canonical");
    parse(&encoded).expect("owned encoding reparses");
});

fn to_owned(value: &Value<'_>) -> Option<OwnedValue> {
    match value.kind() {
        ValueKind::Integer(value) => value.to_i64().map(OwnedValue::integer),
        ValueKind::Bytes(bytes) => Some(OwnedValue::bytes(bytes.to_vec())),
        ValueKind::List(values) => values
            .iter()
            .map(to_owned)
            .collect::<Option<Vec<_>>>()
            .map(OwnedValue::list),
        ValueKind::Dictionary(entries) => {
            let entries = entries
                .iter()
                .map(|(key, value): &(ByteString<'_>, Value<'_>)| {
                    to_owned(value).map(|value| (key.bytes().to_vec(), value))
                })
                .collect::<Option<Vec<_>>>()?;
            OwnedValue::dictionary(entries).ok()
        }
    }
}
