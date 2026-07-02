use btpc_core::ErrorCategory;
use btpc_core::create::{
    PIECE_LENGTH_POLICY_ID, PieceLengthMode, automatic_piece_length, validate_piece_length,
};

#[test]
fn automatic_policy_is_locked_at_every_boundary() {
    let bands = [
        (16 * 1024 * 1024, 16 * 1024),
        (32 * 1024 * 1024, 32 * 1024),
        (64 * 1024 * 1024, 64 * 1024),
        (128 * 1024 * 1024, 128 * 1024),
        (256 * 1024 * 1024, 256 * 1024),
        (512 * 1024 * 1024, 512 * 1024),
        (1024 * 1024 * 1024, 1024 * 1024),
        (2 * 1024 * 1024 * 1024, 2 * 1024 * 1024),
        (4 * 1024 * 1024 * 1024, 4 * 1024 * 1024),
        (8 * 1024 * 1024 * 1024, 8 * 1024 * 1024),
        (16 * 1024 * 1024 * 1024, 16 * 1024 * 1024),
    ];

    assert_eq!(automatic_piece_length(0), 16 * 1024);
    for (index, (maximum_payload, piece_length)) in bands.into_iter().enumerate() {
        assert_eq!(automatic_piece_length(maximum_payload), piece_length);
        if maximum_payload > 0 {
            assert_eq!(automatic_piece_length(maximum_payload - 1), piece_length);
        }
        let expected_above = bands
            .get(index + 1)
            .map_or(16 * 1024 * 1024, |(_, next)| *next);
        assert_eq!(automatic_piece_length(maximum_payload + 1), expected_above);
    }
    assert_eq!(automatic_piece_length(u64::MAX), 16 * 1024 * 1024);
    assert_eq!(PIECE_LENGTH_POLICY_ID, "btpc-piece-v1");
}

#[test]
fn explicit_validation_is_power_of_two_bounded_and_mode_specific() {
    assert_eq!(
        validate_piece_length(8 * 1024, PieceLengthMode::V1).unwrap(),
        8 * 1024
    );
    assert_eq!(
        validate_piece_length(16 * 1024, PieceLengthMode::V2).unwrap(),
        16 * 1024
    );
    assert_eq!(
        validate_piece_length(16 * 1024, PieceLengthMode::Hybrid).unwrap(),
        16 * 1024
    );

    for (value, mode) in [
        (0, PieceLengthMode::V1),
        (3, PieceLengthMode::V1),
        (8 * 1024, PieceLengthMode::V2),
        (8 * 1024, PieceLengthMode::Hybrid),
        (32 * 1024 * 1024, PieceLengthMode::V1),
    ] {
        assert_eq!(
            validate_piece_length(value, mode).unwrap_err().category(),
            ErrorCategory::Metainfo
        );
    }
}

#[test]
fn target_piece_policy_respects_piece_count_and_maximum() {
    use btpc_core::create::{CreateOptions, Creator, NoProgress, PieceLength};
    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    std::fs::write(&payload, vec![1_u8; 200_000]).unwrap();
    let options = CreateOptions::builder()
        .piece_length(PieceLength::Target {
            pieces: 4,
            maximum: 64 * 1024,
        })
        .build()
        .unwrap();
    let result = Creator::new(&payload)
        .options(options)
        .create(&NoProgress)
        .unwrap();
    assert!(result.piece_count() <= 4);
    assert!(result.piece_length() <= 64 * 1024);
}
