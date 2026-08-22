#![no_main]

use context_core::{ContextPacket, packet_bytes, validate_packet};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(packet) = serde_json::from_slice::<ContextPacket>(data) else {
        return;
    };

    // Every accepted packet must survive validation and canonical serialization
    // without a panic. Invalid packets are expected and safely rejected.
    if validate_packet(&packet).is_ok() {
        let canonical = packet_bytes(&packet).expect("validated packet must serialize");
        let reparsed: ContextPacket =
            serde_json::from_slice(&canonical).expect("canonical packet must deserialize");
        assert_eq!(packet, reparsed);
        assert!(validate_packet(&reparsed).is_ok());
    }
});
