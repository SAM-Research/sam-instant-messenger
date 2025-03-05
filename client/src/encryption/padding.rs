fn get_padded_message_length(length: usize) -> usize {
    let message_length_with_terminator = length + 1;
    let mut message_part_count = message_length_with_terminator.div_euclid(160);

    if message_length_with_terminator % 160 != 0 {
        message_part_count += 1;
    }

    message_part_count * 160
}

/// Signal's message padding algorithm.
///
/// Pads a message.
///
/// # Arguments
///
/// * `message` - The message to be padded.
///
/// # Returns
///
/// * `Vec<u8>` The padded message.
pub fn pad_message(message: &[u8]) -> Vec<u8> {
    let len = get_padded_message_length(message.len() + 1) - 1;
    let mut plaintext = vec![0u8; len];
    plaintext[..message.len()].copy_from_slice(message);
    plaintext[message.len()] = 0x80;

    plaintext
}

/// Signal's message unpadding algorithm.
///
/// Unpads a message.
///
/// # Arguments
///
/// * `message` - The padded message.
///
/// # Returns
///
/// * `Some(Vec<u8>)` if unpadding succeeds.
/// * `None` if the message was malformed.
pub fn unpad_message(message: &[u8]) -> Option<Vec<u8>> {
    for i in 0..message.len() {
        if message[i] == 0x80 {
            return Some(message[0..i].to_vec());
        }
    }
    None
}

#[cfg(test)]
mod test {
    use crate::encryption::padding::{pad_message, unpad_message};

    #[test]
    fn test_padding() {
        let msg = [5u8; 32];
        let padded = pad_message(&msg);

        assert_eq!(padded.len(), 159);

        assert!(unpad_message(padded.as_ref()).is_some_and(|unpadded| unpadded == msg.to_vec()));
    }
}
