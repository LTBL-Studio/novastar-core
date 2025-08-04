mod types;
use num_enum::TryFromPrimitive;
use types::SenderCardType;

#[test]
fn display_sender_card_type() {
    assert_eq!(
        format!("{}", SenderCardType::try_from_primitive(0x0001).unwrap()),
        "MCTRL300"
    );
}

#[test]
fn display_renamed_sender_card_type() {
    assert_eq!(
        format!("{}", SenderCardType::try_from_primitive(0x1101).unwrap()),
        "MCTRL600/660"
    );
}
